use crate::anim::runner::AnimationRunner;
use crate::anim::sequence::{AnimationTime, SequenceMarker};
use crate::disable::Disable;
use crate::enable::Enable;
use crate::leaf::Leaf;
use crate::ops::{Name, StoredKey};
use crate::remove::Remove;
use crate::time::OnEnd;
use crate::{Animate, Animation, AssetKey, Children, OnClick, Photosynthesis, TimeDelta, Timer};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{EntityEvent, Event};
use bevy_ecs::message::Message;
use bevy_ecs::observer::IntoEntityObserver;
use bevy_ecs::prelude::{Commands, World};

pub type Tree<'w, 's> = Commands<'w, 's>;

/// Replacement for bevy's removed `TriggerTargets`: the things `send_to`/`remove`/`enable`/
/// `disable` accept as targets. Single-`Entity` and array forms iterate without allocating.
pub trait IntoTargets {
    type Iter: Iterator<Item = Entity>;
    fn into_targets(self) -> Self::Iter;
}
impl IntoTargets for Entity {
    type Iter = core::iter::Once<Entity>;
    fn into_targets(self) -> Self::Iter {
        core::iter::once(self)
    }
}
impl IntoTargets for Vec<Entity> {
    type Iter = std::vec::IntoIter<Entity>;
    fn into_targets(self) -> Self::Iter {
        self.into_iter()
    }
}
impl<const N: usize> IntoTargets for [Entity; N] {
    type Iter = core::array::IntoIter<Entity, N>;
    fn into_targets(self) -> Self::Iter {
        self.into_iter()
    }
}

/// An event `send_to`/`trigger_targets` can aim at an entity. bevy 0.19 removed
/// `trigger_targets` — an `EntityEvent` now carries its target inside itself — so foliage's
/// targeted events store an `entity` field the seam below rewrites per target. Implement via
/// [`targeted_event!`] (or by hand for generics); events constructed by users keep their
/// `Default`/`new()` shape with `Entity::PLACEHOLDER` until the seam assigns the real target.
pub trait TargetedEvent: EntityEvent + Clone {
    fn set_target(&mut self, entity: Entity);
}

/// Implements [`TargetedEvent`] for event structs with a named `entity` field.
#[macro_export]
macro_rules! targeted_event {
    ($($t:ty),* $(,)?) => {$(
        impl $crate::TargetedEvent for $t {
            fn set_target(&mut self, entity: $crate::bevy_ecs::entity::Entity) {
                self.entity = entity;
            }
        }
    )*};
}

/// Raw ECS-level spawn, underneath the public `Seed`/`Sprout`/`Photosynthesis` authoring kit --
/// `pub(crate)` on purpose, so external consumers can't spawn a bare `Leaf` bypassing
/// `LeafSprout`'s mandatory `.elevate(...)` (the exact footgun that field was made required to
/// close). Internal composite/primitive code (`author.rs`, `button.rs`, `foliage.rs`) still uses
/// it directly; everyone else goes through `Leaf::sprout()`/`Photosynthesis::photosynthesize`.
pub(crate) trait Sow {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity;
}
// `Sow` is `pub(crate)` on purpose (see its doc comment) -- `EcsExtension` is only ever
// implemented for `Tree`/`DeferredWorld`/`World` below, all within this crate, so an external
// crate being unable to name the supertrait (and thus unable to implement `EcsExtension` itself)
// is the intended effect, not an oversight.
#[allow(private_bounds)]
pub trait EcsExtension: Sow {
    fn send_to<E>(&mut self, e: E, targets: impl IntoTargets)
    where
        E: TargetedEvent,
        for<'a> E::Trigger<'a>: Default;
    /// Same as [`EcsExtension::send_to`] — kept under the pre-0.19 name so the many
    /// composite-internal call sites read the same as they always have.
    fn trigger_targets<E>(&mut self, e: E, targets: impl IntoTargets)
    where
        E: TargetedEvent,
        for<'a> E::Trigger<'a>: Default,
    {
        self.send_to(e, targets);
    }
    fn send<E>(&mut self, e: E)
    where
        E: Event,
        for<'a> E::Trigger<'a>: Default;
    fn queue<E: Message>(&mut self, e: E);
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B);
    fn remove(&mut self, targets: impl IntoTargets);
    fn enable(&mut self, targets: impl IntoTargets);
    fn disable(&mut self, targets: impl IntoTargets);
    fn sequence(&mut self) -> Entity;
    fn animate<A: Animate>(&mut self, anim: Animation<A>) -> Entity;
    fn sequence_end<M>(&mut self, seq: Entity, end: impl IntoEntityObserver<M>);
    fn subscribe<M>(&mut self, e: Entity, sub: impl IntoEntityObserver<M>);
    fn on_click<M>(&mut self, e: Entity, o: impl IntoEntityObserver<M>);
    fn name<S: AsRef<str>>(&mut self, e: Entity, s: S);
    fn store<S: AsRef<str>>(&mut self, k: AssetKey, s: S);
    fn timer<M>(&mut self, t: u64, tf: impl IntoEntityObserver<M>);
    /// Grafts further behavior (`on_click`, `animate`, extra components) onto an already-spawned
    /// entity right next to where it was created, instead of in a separate pass elsewhere:
    /// `tree.graft(e).on_click(...).animate(...);`
    fn graft(&mut self, entity: Entity) -> Graft<'_, Self>
    where
        Self: Sized,
    {
        Graft { entity, tree: self }
    }
    /// Arranges a group of entities under one root without any of `Composite`'s ceremony
    /// (marker component, `Handle` type, `on_insert`/`on_discard` wiring) -- for a one-off
    /// structure, or a custom composite that doesn't need insertion-triggered construction.
    /// `root` spawns first and becomes the parent every `children.spawn(...)` call auto-stems
    /// to; `build` returns whatever "handle" shape the caller wants (a tuple, a `Vec`, a local
    /// struct, nothing). Teardown needs no `Handle`/`on_discard` machinery either: `Remove`
    /// already walks `Branch` (every `Stem`-child) recursively for any entity, so
    /// `tree.remove(root)` tears down everything spawned through `children` here for free.
    fn composite<S: Photosynthesis, R>(
        &mut self,
        root: S,
        build: impl FnOnce(&mut Children<Self>) -> R,
    ) -> (Entity, R)
    where
        Self: Sized,
    {
        let root = root.photosynthesize(self);
        let handle = build(&mut Children::new(root, self));
        (root, handle)
    }
}
/// See [`EcsExtension::graft`].
pub struct Graft<'t, T: EcsExtension + ?Sized> {
    entity: Entity,
    tree: &'t mut T,
}
impl<'t, T: EcsExtension + ?Sized> Graft<'t, T> {
    pub fn id(&self) -> Entity {
        self.entity
    }
    pub fn on_click<M>(self, o: impl IntoEntityObserver<M>) -> Self {
        self.tree.on_click(self.entity, o);
        self
    }
    pub fn animate<A: Animate>(self, anim: Animation<A>) -> Self {
        self.tree.animate(anim.targeting(self.entity));
        self
    }
    pub fn write<B: Bundle>(self, b: B) -> Self {
        self.tree.write_to(self.entity, b);
        self
    }
    pub fn enable(self) -> Self {
        self.tree.enable(self.entity);
        self
    }
    pub fn disable(self) -> Self {
        self.tree.disable(self.entity);
        self
    }
}
impl<'t, T: EcsExtension + ?Sized> From<Graft<'t, T>> for Entity {
    fn from(b: Graft<'t, T>) -> Self {
        b.entity
    }
}
/// Chains multiple `.animate(...)` calls into one sequence without repeating
/// `tree.animate(Animation::new(...)....during(seq))` per line. Each `.animate(anim)` still
/// takes a full `Animation::new(value).targeting(e).start(s).finish(f)` -- animations in a
/// sequence can freely overlap, so this doesn't compute or infer timing, it only removes the
/// per-line `tree.animate(...)`/`.during(seq)` wrapper: `Sequence::new(tree).animate(a1).animate(a2).end(on_finish)`.
pub struct Sequence<'t, T: EcsExtension + ?Sized> {
    id: Entity,
    tree: &'t mut T,
}
impl<'t, T: EcsExtension + ?Sized> Sequence<'t, T> {
    pub fn new(tree: &'t mut T) -> Self {
        let id = tree.sequence();
        Self { id, tree }
    }
    pub fn id(&self) -> Entity {
        self.id
    }
    pub fn animate<A: Animate>(self, anim: Animation<A>) -> Self {
        self.tree.animate(anim.during(self.id));
        self
    }
    pub fn end<M>(self, end: impl IntoEntityObserver<M>) -> Entity {
        self.tree.sequence_end(self.id, end);
        self.id
    }
}
impl Sow for Tree<'_, '_> {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        let entity = self.spawn((Leaf::new(), b)).id();
        entity
    }
}
impl EcsExtension for Tree<'_, '_> {
    fn send_to<E>(&mut self, e: E, targets: impl IntoTargets)
    where
        E: TargetedEvent,
        for<'a> E::Trigger<'a>: Default,
    {
        for target in targets.into_targets() {
            let mut event = e.clone();
            event.set_target(target);
            self.trigger(event);
        }
    }
    fn send<E>(&mut self, e: E)
    where
        E: Event,
        for<'a> E::Trigger<'a>: Default,
    {
        self.trigger(e);
    }
    fn queue<E: Message>(&mut self, e: E) {
        self.write_message(e);
    }
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.entity(entity).insert(b);
    }
    fn remove(&mut self, targets: impl IntoTargets) {
        self.send_to(Remove::new(), targets);
    }
    fn enable(&mut self, targets: impl IntoTargets) {
        self.send_to(Enable::new(), targets);
    }
    fn disable(&mut self, targets: impl IntoTargets) {
        self.send_to(Disable::new(), targets);
    }
    fn sequence(&mut self) -> Entity {
        self.spawn(SequenceMarker::default()).id()
    }
    fn animate<A: Animate>(&mut self, anim: Animation<A>) -> Entity {
        let runner = AnimationRunner::new(
            anim.anim_target.unwrap(),
            anim.a,
            anim.ease,
            anim.seq,
            AnimationTime::from(anim.sequence_time_range),
        );
        self.spawn(runner).id()
    }
    fn sequence_end<M>(&mut self, seq: Entity, end: impl IntoEntityObserver<M>) {
        self.entity(seq).observe(end);
    }

    fn subscribe<M>(&mut self, e: Entity, sub: impl IntoEntityObserver<M>) {
        self.entity(e).observe(sub);
    }
    fn on_click<M>(&mut self, e: Entity, o: impl IntoEntityObserver<M>) {
        self.entity(e).observe(o);
    }
    fn name<S: AsRef<str>>(&mut self, e: Entity, s: S) {
        self.send(Name(s.as_ref().to_string(), e));
    }
    fn store<S: AsRef<str>>(&mut self, k: AssetKey, s: S) {
        self.send(StoredKey(s.as_ref().to_string(), k));
    }
    fn timer<M>(&mut self, t: u64, tf: impl IntoEntityObserver<M>) {
        self.spawn(Timer::new(TimeDelta::from_millis(t)))
            .observe(tf);
    }
}

impl Sow for bevy_ecs::world::DeferredWorld<'_> {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        self.commands().leaf(b)
    }
}
impl EcsExtension for bevy_ecs::world::DeferredWorld<'_> {
    fn send_to<E>(&mut self, e: E, targets: impl IntoTargets)
    where
        E: TargetedEvent,
        for<'a> E::Trigger<'a>: Default,
    {
        self.commands().send_to(e, targets);
    }
    fn send<E>(&mut self, e: E)
    where
        E: Event,
        for<'a> E::Trigger<'a>: Default,
    {
        self.commands().send(e);
    }
    fn queue<E: Message>(&mut self, e: E) {
        EcsExtension::queue(&mut self.commands(), e);
    }
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.commands().write_to(entity, b);
    }
    fn remove(&mut self, targets: impl IntoTargets) {
        EcsExtension::remove(&mut self.commands(), targets);
    }
    fn enable(&mut self, targets: impl IntoTargets) {
        self.commands().enable(targets);
    }
    fn disable(&mut self, targets: impl IntoTargets) {
        self.commands().disable(targets);
    }
    fn sequence(&mut self) -> Entity {
        self.commands().sequence()
    }
    fn animate<A: Animate>(&mut self, anim: Animation<A>) -> Entity {
        self.commands().animate(anim)
    }
    fn sequence_end<M>(&mut self, seq: Entity, end: impl IntoEntityObserver<M>) {
        self.commands().sequence_end(seq, end);
    }
    fn subscribe<M>(&mut self, e: Entity, sub: impl IntoEntityObserver<M>) {
        self.commands().subscribe(e, sub);
    }
    fn on_click<M>(&mut self, e: Entity, o: impl IntoEntityObserver<M>) {
        self.commands().on_click(e, o);
    }
    fn name<S: AsRef<str>>(&mut self, e: Entity, s: S) {
        self.commands().name(e, s);
    }
    fn store<S: AsRef<str>>(&mut self, k: AssetKey, s: S) {
        self.commands().store(k, s);
    }
    fn timer<M>(&mut self, t: u64, tf: impl IntoEntityObserver<M>) {
        self.commands().timer(t, tf);
    }
}

impl Sow for World {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        self.commands().leaf(b)
    }
}
impl EcsExtension for World {
    fn send_to<E>(&mut self, e: E, targets: impl IntoTargets)
    where
        E: TargetedEvent,
        for<'a> E::Trigger<'a>: Default,
    {
        self.commands().send_to(e, targets);
    }
    fn send<E>(&mut self, e: E)
    where
        E: Event,
        for<'a> E::Trigger<'a>: Default,
    {
        self.commands().send(e);
    }
    fn queue<E: Message>(&mut self, e: E) {
        EcsExtension::queue(&mut self.commands(), e);
    }
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.commands().write_to(entity, b);
    }
    fn remove(&mut self, targets: impl IntoTargets) {
        self.commands().remove(targets);
    }
    fn enable(&mut self, targets: impl IntoTargets) {
        self.commands().enable(targets);
    }
    fn disable(&mut self, targets: impl IntoTargets) {
        self.commands().disable(targets);
    }
    fn sequence(&mut self) -> Entity {
        self.commands().sequence()
    }
    fn animate<A: Animate>(&mut self, anim: Animation<A>) -> Entity {
        self.commands().animate(anim)
    }
    fn sequence_end<M>(&mut self, seq: Entity, end: impl IntoEntityObserver<M>) {
        self.commands().sequence_end(seq, end);
    }

    fn subscribe<M>(&mut self, e: Entity, sub: impl IntoEntityObserver<M>) {
        self.commands().subscribe(e, sub);
    }

    fn on_click<M>(&mut self, e: Entity, o: impl IntoEntityObserver<M>) {
        self.commands().on_click(e, o);
    }

    fn name<S: AsRef<str>>(&mut self, e: Entity, s: S) {
        self.commands().name(e, s);
    }

    fn store<S: AsRef<str>>(&mut self, k: AssetKey, s: S) {
        self.commands().store(k, s);
    }

    fn timer<M>(&mut self, t: u64, tf: impl IntoEntityObserver<M>) {
        self.commands().timer(t, tf);
    }
}
