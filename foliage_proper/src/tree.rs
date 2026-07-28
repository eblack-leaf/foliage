use crate::Trigger;
use crate::anim::runner::AnimationRunner;
use crate::anim::sequence::{AnimationTime, SequenceMarker};
use crate::disable::Disable;
use crate::enable::Enable;
use crate::leaf::Leaf;
use crate::leaf::Stem;
use crate::ops::{Name, StoredKey};
use crate::remove::Remove;
use crate::asset::{AssetLoader, AssetRetrieval, OnRetrieval};
use crate::{Animate, Animation, AssetKey, Sprout, TimeDelta, Timer};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{EntityEvent, Event};
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::message::Message;
use bevy_ecs::observer::IntoEntityObserver;
use bevy_ecs::prelude::{Commands, World};
use bevy_ecs::system::{Query, Res};

/// The handle systems and observers use to change the world: spawn entities, write
/// components, start animations, register handlers.
///
/// A `Commands`, so everything through it is deferred to the end of the current step
/// rather than applied in place. Two consequences worth holding onto: an entity is
/// spawned by the time you get its `Entity` back, but its components are not yet
/// readable through a `Query` in the same system; and a component written here lands
/// before the next system runs. The same vocabulary is available on
/// [`Foliage`](crate::Foliage) before startup -- see [`EcsExtension`].
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
/// the `#[targeted_event]` attribute (injects the field, implements `Event`/`EntityEvent`,
/// generates `new(<payload fields>)` — authors never write `Entity::PLACEHOLDER`); generics
/// like `Resolve<C>`/`Resolved<C>` implement it by hand.
pub trait TargetedEvent: EntityEvent + Clone {
    fn set_target(&mut self, entity: Entity);
}

/// Raw ECS-level spawn, underneath the public `Sprout` authoring kit -- `pub(crate)` on
/// purpose, so external consumers can't spawn a bundle bypassing `LeafSprout`'s mandatory
/// `.elevate(...)` (the exact footgun that field was made required to close). Only
/// [`EcsExtension::leaf`]/[`EcsExtension::branch`] (via [`Sow::grow`]) reach this; there is no
/// other way to turn a `Sprout` into a spawned entity from outside this crate.
pub(crate) trait Sow {
    fn sow<B: Bundle>(&mut self, b: B) -> Entity;
    /// Shared machinery behind [`EcsExtension::leaf`]/[`EcsExtension::branch`]: fills `stem` if
    /// a parent was given, folds the seed fields + `spec.root()` into one bundle via
    /// [`Sow::sow`], then runs `Sprout::build`. `pub(crate)` alongside `sow` for the same
    /// reason -- an external `Sprout` impl gets no way to skip the mandatory `.elevate(...)` or
    /// hand-roll a child that forgets its parent.
    fn grow<S: Sprout>(&mut self, mut spec: S, parent: Option<Entity>) -> Entity
    where
        Self: EcsExtension + Sized,
    {
        if let Some(parent) = parent {
            spec.seed().stem = Stem::some(parent);
        }
        let leaf = core::mem::take(spec.seed());
        let this = self.sow((
            leaf.location,
            leaf.stem,
            leaf.elevation
                .expect("elevation not set -- call .elevate(...) before spawning"),
            spec.root(),
        ));
        S::build(this, self);
        this
    }
}
// `Sow` is `pub(crate)` on purpose (see its doc comment) -- `EcsExtension` is only ever
// implemented for `Tree`/`DeferredWorld`/`World` below, all within this crate, so an external
// crate being unable to name the supertrait (and thus unable to implement `EcsExtension` itself)
// is the intended effect, not an oversight.
/// The shared vocabulary for building and changing a tree, implemented for
/// [`Tree`], `World` and `DeferredWorld` so the same calls work from a system, an
/// observer, a component hook, or startup.
///
/// [`leaf`](EcsExtension::leaf) and [`branch`](EcsExtension::branch) spawn;
/// [`write_to`](EcsExtension::write_to) changes a live value;
/// [`animate`](EcsExtension::animate) and [`sequence`](EcsExtension::sequence) move
/// things; [`react`](EcsExtension::react) and [`subscribe`](EcsExtension::subscribe)
/// respond.
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
    /// Runs `afn` on `entity` once the asset `key` names has arrived, with its bytes --
    /// the asset counterpart to [`on_click`](EcsExtension::on_click). Marks the wait and
    /// registers the handler in one call, so the key can't disagree between them.
    ///
    /// On native a `Bytes` asset is already there and this fires almost immediately; on
    /// wasm it fires whenever the fetch resolves. Either way anything that needs the bytes
    /// belongs in `afn` rather than after the spawn.
    fn on_asset<AFN: FnMut(&mut Tree, Entity, Vec<u8>) + Send + Sync + 'static>(
        &mut self,
        entity: Entity,
        key: AssetKey,
        mut afn: AFN,
    ) where
        Self: Sized,
    {
        self.write_to(entity, AssetRetrieval::new(key));
        // written out here rather than built by a helper returning `impl FnMut(..)`: an
        // opaque return type pins one set of lifetimes, and an entity observer has to be
        // higher-ranked over `On<'a, 'b, _>`, so only a closure literal in argument
        // position actually satisfies the bound.
        self.subscribe(
            entity,
            move |trigger: Trigger<OnRetrieval>, mut tree: Tree, loader: Res<AssetLoader>| {
                let asset = loader.retrieve(trigger.event().key).unwrap();
                afn(&mut tree, trigger.event_target(), asset.data);
            },
        );
    }
    /// Re-inserts `entity`'s current value(s) of `C` so any just-registered observer fires with
    /// real data -- the fire-once half of [`EcsExtension::react`]. Internal plumbing; authors
    /// use `react`.
    fn refire<C: Refire>(&mut self, entity: Entity);
    /// Grows `spec` as a top-level entity -- no parent, the [`EcsExtension::branch`] counterpart
    /// for roots (a screen, a one-off widget): `let root = tree.leaf(Icon::new(0).elevate(..));`
    fn leaf<S: Sprout>(&mut self, spec: S) -> Entity
    where
        Self: Sized,
    {
        self.grow(spec, None)
    }
    /// Grows `spec` as a child of `parent` -- `parent` filled by the required argument, so a
    /// child can't be spawned orphaned by forgetting a chained call. THE way to build a
    /// composite's structure, in `Sprout::build` and one-off screens alike:
    /// `let icon = tree.branch(this, Icon::new(0).elevate(..));`
    fn branch<S: Sprout>(&mut self, parent: Entity, spec: S) -> Entity
    where
        Self: Sized,
    {
        self.grow(spec, Some(parent))
    }
    /// Registers `observer` -- a plain bevy entity-observer watching `Trigger<Insert, C>`, full
    /// `SystemParam` freedom -- on `entity`, then re-fires it once with the current value so
    /// initial state and every later `write_to` run the SAME code path. This is the one door
    /// for everything data-dependent in a composite: values AND structure (a dropdown's option
    /// rows, a hand's cards). What the body does -- respawn, pool, patch -- is entirely the
    /// author's policy. The `_` stands in for the observer's inferred marker:
    /// `tree.react::<TextValue, _>(this, ..)`.
    fn react<C: Component + Clone, M>(
        &mut self,
        entity: Entity,
        observer: impl IntoEntityObserver<M>,
    ) {
        self.subscribe(entity, observer);
        self.refire::<(C,)>(entity);
    }
    /// [`EcsExtension::react`] over a component SET: the observer watches
    /// `Trigger<Insert, (A, B)>` and fires when ANY member is written -- one registration, one
    /// body, for state derived from several inputs (Button's style + engagement). The
    /// build-time re-fire inserts only the first present member: one fire is enough because
    /// reaction bodies read the current state of everything they depend on.
    fn react_any<CS: Refire, M>(&mut self, entity: Entity, observer: impl IntoEntityObserver<M>) {
        self.subscribe(entity, observer);
        self.refire::<CS>(entity);
    }
    /// [`EcsExtension::react`] specialized to the pure-copy case: `source`'s `C` is copied to
    /// `target` verbatim, now and on every write. Anything that is NOT a pure copy (a text
    /// whose width also depends on the value) stays an explicit `react` -- the boundary is
    /// visible on purpose.
    fn forward<C: Component + Clone>(&mut self, source: Entity, target: Entity) {
        self.react::<C, _>(
            source,
            move |trigger: Trigger<Insert, C>, values: Query<&C>, mut tree: Tree| {
                tree.entity(target)
                    .insert(values.get(trigger.entity).unwrap().clone());
            },
        );
    }
    /// Grafts further behavior (`on_click`, `animate`, extra components) onto an already-spawned
    /// entity right next to where it was created, instead of in a separate pass elsewhere:
    /// `tree.graft(e).on_click(...).animate(...);`
    fn graft(&mut self, entity: Entity) -> Graft<'_, Self>
    where
        Self: Sized,
    {
        Graft { entity, tree: self }
    }
}

/// What `react`/`react_any` re-fire at build time: a tuple of `Clone`-able components
/// (arity 1-4; tuple-only so the impls can't collide with a foreign `Component` impl).
/// Re-inserts only the FIRST present member -- one fire is sufficient because reaction
/// bodies read the current state of everything they depend on, that's the pattern.
pub trait Refire: Send + Sync + 'static {
    /// Re-inserts the entity's current value; true if anything was inserted.
    fn refire(entity: Entity, world: &mut World) -> bool;
}
fn refire_one<C: Component + Clone>(entity: Entity, world: &mut World) -> bool {
    if let Some(current) = world.get::<C>(entity).cloned() {
        world.entity_mut(entity).insert(current);
        true
    } else {
        false
    }
}
macro_rules! impl_refire_tuple {
    ($($t:ident),+) => {
        impl<$($t: Component + Clone),+> Refire for ($($t,)+) {
            fn refire(entity: Entity, world: &mut World) -> bool {
                $( if refire_one::<$t>(entity, world) { return true; } )+
                false
            }
        }
    };
}
impl_refire_tuple!(A);
impl_refire_tuple!(A, B);
impl_refire_tuple!(A, B, C);
impl_refire_tuple!(A, B, C, D);
/// See [`EcsExtension::graft`].
pub struct Graft<'t, T: EcsExtension + ?Sized> {
    entity: Entity,
    tree: &'t mut T,
}
impl<'t, T: EcsExtension + ?Sized> Graft<'t, T> {
    /// The entity being configured.
    pub fn id(&self) -> Entity {
        self.entity
    }
    /// Runs `o` when this entity is clicked.
    pub fn on_click<M>(self, o: impl IntoEntityObserver<M>) -> Self {
        self.tree.on_click(self.entity, o);
        self
    }
    /// Starts `anim` against this entity -- no `targeting` needed, it is already known.
    pub fn animate<A: Animate>(self, anim: Animation<A>) -> Self {
        self.tree.animate(anim.targeting(self.entity));
        self
    }
    /// Inserts components on this entity.
    pub fn write<B: Bundle>(self, b: B) -> Self {
        self.tree.write_to(self.entity, b);
        self
    }
    /// Re-enables interaction on this entity and its subtree.
    pub fn enable(self) -> Self {
        self.tree.enable(self.entity);
        self
    }
    /// Disables interaction on this entity and its subtree.
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
    /// Opens a sequence to chain animations onto.
    pub fn new(tree: &'t mut T) -> Self {
        let id = tree.sequence();
        Self { id, tree }
    }
    /// The sequence entity, for anything that needs to name it directly.
    pub fn id(&self) -> Entity {
        self.id
    }
    /// Adds `anim` to this sequence. Its own `start`/`finish` still apply, so entries may
    /// overlap freely -- joining a sequence does not order them.
    pub fn animate<A: Animate>(self, anim: Animation<A>) -> Self {
        self.tree.animate(anim.during(self.id));
        self
    }
    /// Runs `end` once every animation in the sequence has finished, returning the
    /// sequence entity.
    pub fn end<M>(self, end: impl IntoEntityObserver<M>) -> Entity {
        self.tree.sequence_end(self.id, end);
        self.id
    }
}
impl Sow for Tree<'_, '_> {
    fn sow<B: Bundle>(&mut self, b: B) -> Entity {
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
            anim.repeat,
            anim.backtrack,
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
    fn refire<C: Refire>(&mut self, entity: Entity) {
        Commands::queue(self, move |world: &mut World| {
            C::refire(entity, world);
        });
    }
}

impl Sow for bevy_ecs::world::DeferredWorld<'_> {
    fn sow<B: Bundle>(&mut self, b: B) -> Entity {
        self.commands().sow(b)
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
    fn refire<C: Refire>(&mut self, entity: Entity) {
        self.commands().refire::<C>(entity);
    }
}

impl Sow for World {
    fn sow<B: Bundle>(&mut self, b: B) -> Entity {
        self.commands().sow(b)
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
    fn refire<C: Refire>(&mut self, entity: Entity) {
        self.commands().refire::<C>(entity);
    }
}
