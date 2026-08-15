use crate::Trigger;
use crate::anim::runner::AnimationRunner;
use crate::anim::sequence::{AnimationTime, SequenceMarker};
use crate::asset::{AssetLoader, AssetRetrieval, OnRetrieval};
use crate::disable::Disable;
use crate::enable::Enable;
use crate::node::Node;
use crate::node::{Parent, SpawnedAt};
use crate::ops::{Name, StoredKey};
use crate::remove::Remove;
use crate::{Animate, Animation, AssetKey, Author, TimeDelta, Timer};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{EntityEvent, Event};
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::observer::IntoEntityObserver;
use bevy_ecs::prelude::{Commands, World};
use bevy_ecs::system::{Query, Res};

/// Foliage's own handle on the world: spawn entities, write components, start animations,
/// register handlers. Every engine-internal mutation goes through one of these methods --
/// this is the single door onto raw `bevy_ecs`.
///
/// Wraps a `Commands`, so everything is deferred to the end of the current step rather than
/// applied in place. Two consequences worth holding onto: an entity is spawned by the time
/// you get its `Entity` back, but its components are not yet readable through a `Query` in
/// the same system; and a component written here lands before the next system runs.
///
/// Obtained as a system param (`mut tree: Tree`), or from a `World`/`DeferredWorld` via
/// [`AsTree::tree`] in component hooks and at startup.
#[derive(bevy_ecs::system::SystemParam)]
pub(crate) struct Tree<'w, 's> {
    commands: Commands<'w, 's>,
}

/// Turns the two places that hold the world directly -- a `World` at startup or in a
/// command, and the `DeferredWorld` a component lifecycle hook is handed -- into a [`Tree`].
/// Replaces what used to be a second and third full implementation of every verb.
pub(crate) trait AsTree {
    fn tree(&mut self) -> Tree<'_, '_>;
}
impl AsTree for World {
    fn tree(&mut self) -> Tree<'_, '_> {
        Tree {
            commands: self.commands(),
        }
    }
}
impl AsTree for bevy_ecs::world::DeferredWorld<'_> {
    fn tree(&mut self) -> Tree<'_, '_> {
        Tree {
            commands: self.commands(),
        }
    }
}

/// Replacement for bevy's removed `TriggerTargets`: the things `send_to`/`remove`/`enable`/
/// `disable` accept as targets. Single-`Entity` and array forms iterate without allocating.
pub(crate) trait IntoTargets {
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
pub(crate) trait TargetedEvent: EntityEvent + Clone {
    fn set_target(&mut self, entity: Entity);
}

// `on_click`, `store`, `timer`, `leaf`, `forward` and `on_asset` have no caller since the
// composites were deleted -- they are the vocabulary a widget is built out of, and nothing is
// built out of anything right now. Kept because the primitives still reach for them the moment
// widgets come back, and deleting them would mean writing them again unchanged. Delete for
// real if the widget layer lands without needing them.
#[allow(dead_code)]
impl<'w, 's> Tree<'w, 's> {
    /// Fires `e` at each target, delivered immediately on flush to observers bound to that
    /// entity. The event is cloned once per target and its own target field rewritten.
    pub fn send_to<E>(&mut self, e: E, targets: impl IntoTargets)
    where
        E: TargetedEvent,
        for<'a> E::Trigger<'a>: Default,
    {
        for target in targets.into_targets() {
            let mut event = e.clone();
            event.set_target(target);
            self.commands.trigger(event);
        }
    }
    /// Fires an untargeted event, delivered to global observers.
    pub fn send<E>(&mut self, e: E)
    where
        E: Event,
        for<'a> E::Trigger<'a>: Default,
    {
        self.commands.trigger(e);
    }
    /// Inserts components on an existing entity -- the way a live value is changed after
    /// spawn.
    pub fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.commands.entity(entity).insert(b);
    }
    /// Despawns entities and everything beneath them.
    pub fn remove(&mut self, targets: impl IntoTargets) {
        self.send_to(Remove::new(), targets);
    }
    /// Re-enables interaction on entities and their subtrees.
    pub fn enable(&mut self, targets: impl IntoTargets) {
        self.send_to(Enable::new(), targets);
    }
    /// Disables interaction on entities and their subtrees. They still draw; they stop
    /// competing for input.
    pub fn disable(&mut self, targets: impl IntoTargets) {
        self.send_to(Disable::new(), targets);
    }
    /// Despawns one entity outright, without the [`Remove`] cascade -- for entities that
    /// have no subtree of their own (a timer, an animation runner, a pooled instance).
    pub fn despawn(&mut self, entity: Entity) {
        self.commands.entity(entity).despawn();
    }
    /// Strips a single component off an entity, leaving the rest.
    pub fn strip<C: Component>(&mut self, entity: Entity) {
        self.commands.entity(entity).remove::<C>();
    }
    /// Whether `entity` is still alive -- false once it has been despawned.
    pub fn exists(&mut self, entity: Entity) -> bool {
        self.commands.get_entity(entity).is_ok()
    }
    /// Starts `anim`, returning the runner entity.
    pub fn animate<A: Animate>(&mut self, anim: Animation<A>) -> Entity {
        let runner = AnimationRunner::new(
            anim.anim_target.unwrap(),
            anim.a,
            anim.ease,
            anim.seq,
            AnimationTime::from(anim.sequence_time_range),
            anim.repeat,
            anim.backtrack,
        );
        self.commands.spawn(runner).id()
    }
    /// Registers an entity-observer on `e`.
    pub fn subscribe<M>(&mut self, e: Entity, sub: impl IntoEntityObserver<M>) {
        self.commands.entity(e).observe(sub);
    }
    /// Runs `o` when `e` is clicked.
    pub fn on_click<M>(&mut self, e: Entity, o: impl IntoEntityObserver<M>) {
        self.commands.entity(e).observe(o);
    }
    /// Names `e`, so it can be looked up through [`Named`](crate::Named).
    pub fn name<S: AsRef<str>>(&mut self, e: Entity, s: S) {
        self.send(Name(s.as_ref().to_string(), e));
    }
    /// Stores an asset key under a name, for lookup through [`Keyring`](crate::Keyring).
    pub fn store<S: AsRef<str>>(&mut self, k: AssetKey, s: S) {
        self.send(StoredKey(s.as_ref().to_string(), k));
    }
    /// Runs `tf` once, `t` milliseconds from now.
    pub fn timer<M>(&mut self, t: u64, tf: impl IntoEntityObserver<M>) {
        self.commands
            .spawn(Timer::new(TimeDelta::from_millis(t)))
            .observe(tf);
    }
    /// Re-inserts `entity`'s current value(s) of `C` so any just-registered observer fires
    /// with real data -- the fire-once half of [`react`](Self::react).
    pub fn refire<C: Refire>(&mut self, entity: Entity) {
        self.commands.queue(move |world: &mut World| {
            C::refire(entity, world);
        });
    }
    /// Opens a sequence at an already-allocated id: animations joined to it report their
    /// completion, and it fires [`OnEnd`](crate::OnEnd) once the last one finishes.
    pub(crate) fn sequence_at(&mut self, at: Entity) {
        self.commands.queue(move |world: &mut World| {
            let _ = world.spawn_at(
                at,
                (SequenceMarker::default(), crate::boundary::leaf::Grown),
            );
        });
    }
    /// Opens a sequence the app never named, for an animation that was not joined to one.
    ///
    /// Every animation has to belong to a sequence -- that is what counts it, ends it and
    /// tears its runner down -- so "no sequence" cannot mean no sequence at all. It means one
    /// of these: deliberately not [`Grown`](crate::boundary::leaf::Grown), so its completion
    /// reports nothing, and despawned by the last animation to settle against it.
    pub(crate) fn spawn_sequence(&mut self) -> Entity {
        self.commands.spawn(SequenceMarker::default()).id()
    }
    /// A countdown at an already-allocated id, firing [`OnEnd`](crate::OnEnd) and despawning
    /// itself when it runs out.
    pub(crate) fn timer_at(&mut self, at: Entity, millis: u64) {
        self.commands.queue(move |world: &mut World| {
            let _ = world.spawn_at(
                at,
                (
                    Timer::new(TimeDelta::from_millis(millis)),
                    crate::boundary::leaf::Grown,
                ),
            );
        });
    }
    /// Spawns a scalar tween runner. Not a [`Node`] -- it holds no place in the tree and is
    /// never drawn; it exists to be ticked and to report numbers.
    pub(crate) fn spawn_tween(&mut self, tweening: crate::boundary::tween::Tweening) {
        self.commands.spawn(tweening);
    }
    /// Raw ECS-level spawn, underneath the [`Author`] authoring kit. Every spawned entity
    /// gets a [`Node`] so the registry and the interaction bookkeeping have one hook.
    fn sow<B: Bundle>(&mut self, b: B) -> Entity {
        self.commands.spawn((Node::new(), b)).id()
    }
    /// Shared machinery behind [`leaf`](Self::leaf)/[`branch`](Self::branch): fills `stem` if
    /// a parent was given, folds the seed fields + `spec.root()` into one bundle, then runs
    /// `Author::build`. `#[track_caller]` here and on `leaf`/`branch` is what makes
    /// [`SpawnedAt`] name the author's own call rather than this line.
    #[track_caller]
    fn grow<S: Author>(&mut self, mut spec: S, parent: Option<Entity>) -> Entity {
        if let Some(parent) = parent {
            spec.seed().stem = Parent::some(parent);
        }
        let seed = core::mem::take(spec.seed());
        let elevation = seed
            .elevation
            .expect("elevation not set -- call .elevate(...) before spawning");
        let spawned_at = SpawnedAt(core::panic::Location::caller());
        // A bare `Node` brings in the components everything positioned and drawn needs, but
        // no `Location` -- so the entity exists without yet trying to place itself, which is
        // the room `trimmings` needs.
        let this = self.sow(());
        self.trimmings(this, &seed);
        self.write_to(
            this,
            (seed.location, seed.stem, elevation, spawned_at, spec.root()),
        );
        S::build(this, self);
        this
    }
    /// The optional seed fields, each written only when it was actually asked for -- a
    /// blanket insert of defaults would override what a spec's own `root()` established (an
    /// `Icon` sets its own 1:1 aspect, a `Panel` its own hit shape).
    ///
    /// Applied *before* the `Location` and `root()` that finish the spawn, which is the whole
    /// reason the spawn is split in two. Several of these are read by this entity's own first
    /// resolve or its first text shape, and neither is redone on their account: an anchored
    /// `Location` that resolves without its `Anchor` fails outright and auto-hides with
    /// nothing left to bring it back, and a run that shapes against a default alignment or
    /// font keeps those glyphs at that size. Landing them first is what
    /// `Author`'s "never a follow-up `write_to` after a bare insert" is asking for; queued
    /// commands apply in order, so ordering the two writes is all it takes.
    fn trimmings(&mut self, this: Entity, seed: &crate::LeafSprout) {
        if let Some(grid) = seed.grid {
            self.write_to(this, grid);
        }
        if let Some(anchor) = seed.anchor {
            self.write_to(this, anchor);
        }
        if let Some(opacity) = seed.opacity {
            self.write_to(this, opacity);
        }
        if let Some((h, v)) = seed.alignment {
            self.write_to(this, (h, v));
        }
        if let Some(listener) = seed.listener {
            self.write_to(this, listener);
        }
        if let Some(propagation) = seed.propagation {
            self.write_to(this, propagation);
        }
        if let Some(shape) = seed.shape {
            self.write_to(this, shape);
        }
        if seed.clip_to_viewport {
            self.write_to(this, crate::ClipToViewport);
        }
        if let Some(aspect) = seed.aspect {
            self.write_to(this, aspect);
        }
        if let Some(overscroll) = seed.overscroll {
            self.write_to(this, overscroll);
        }
        if let Some(font) = seed.font {
            self.write_to(this, font);
        }
        if let Some(font_size) = seed.font_size {
            self.write_to(this, font_size);
        }
    }
    /// [`grow`](Self::grow) into an id the caller already allocated, rather than one this
    /// picks. The boundary hands an app its `Leaf` the instant it asks, before the element
    /// exists; this is what makes that name come true.
    pub(crate) fn grow_at<S: Author>(&mut self, at: Entity, mut spec: S, parent: Option<Entity>) {
        if let Some(parent) = parent {
            spec.seed().stem = Parent::some(parent);
        }
        let seed = core::mem::take(spec.seed());
        let elevation = seed
            .elevation
            .expect("elevation not set -- call .elevate(...) before spawning");
        let bundle = (
            seed.location,
            seed.stem,
            elevation,
            SpawnedAt(core::panic::Location::caller()),
            crate::boundary::leaf::Grown,
            spec.root(),
        );
        // Split the same way [`grow`](Self::grow) is, and for the same reason: the bare `Node`
        // claims the id, `trimmings` lands everything the first resolve reads, and only then
        // does the `Location` arrive to be resolved against it.
        self.commands.queue(move |world: &mut World| {
            let _ = world.spawn_at(at, Node::new());
        });
        self.trimmings(at, &seed);
        self.write_to(at, bundle);
        S::build(at, self);
    }
    /// Sprout `spec` as a top-level entity -- no parent, the [`branch`](Self::branch)
    /// counterpart for roots: `let root = tree.leaf(Icon::new(0).elevate(..));`
    #[track_caller]
    pub fn leaf<S: Author>(&mut self, spec: S) -> Entity {
        self.grow(spec, None)
    }
    /// Sprout `spec` as a child of `parent` -- `parent` filled by the required argument, so a
    /// child can't be spawned orphaned by forgetting a chained call:
    /// `let icon = tree.branch(this, Icon::new(0).elevate(..));`
    #[track_caller]
    pub fn branch<S: Author>(&mut self, parent: Entity, spec: S) -> Entity {
        self.grow(spec, Some(parent))
    }
    /// Registers `observer` -- a plain bevy entity-observer watching `Trigger<Insert, C>`,
    /// full `SystemParam` freedom -- on `entity`, then re-fires it once with the current
    /// value so initial state and every later `write_to` run the SAME code path. This is the
    /// one door for everything data-dependent: values AND structure. What the body does --
    /// respawn, pool, patch -- is entirely the author's policy. The `_` stands in for the
    /// observer's inferred marker: `tree.react::<TextValue, _>(this, ..)`.
    pub fn react<C: Component + Clone, M>(
        &mut self,
        entity: Entity,
        observer: impl IntoEntityObserver<M>,
    ) {
        self.subscribe(entity, observer);
        self.refire::<(C,)>(entity);
    }
    /// [`react`](Self::react) over a component SET: the observer watches
    /// `Trigger<Insert, (A, B)>` and fires when ANY member is written -- one registration,
    /// one body, for state derived from several inputs. The build-time re-fire inserts only
    /// the first present member: one fire is enough because reaction bodies read the current
    /// state of everything they depend on.
    pub fn react_any<CS: Refire, M>(
        &mut self,
        entity: Entity,
        observer: impl IntoEntityObserver<M>,
    ) {
        self.subscribe(entity, observer);
        self.refire::<CS>(entity);
    }
    /// [`react`](Self::react) specialized to the pure-copy case: `source`'s `C` is copied to
    /// `target` verbatim, now and on every write. Anything that is NOT a pure copy (a text
    /// whose width also depends on the value) stays an explicit `react` -- the boundary is
    /// visible on purpose.
    pub fn forward<C: Component + Clone>(&mut self, source: Entity, target: Entity) {
        self.react::<C, _>(
            source,
            move |trigger: Trigger<Insert, C>, values: Query<&C>, mut tree: Tree| {
                let value = values.get(trigger.entity).unwrap().clone();
                tree.write_to(target, value);
            },
        );
    }
    /// Runs `afn` on `entity` once the asset `key` names has arrived, with its bytes -- the
    /// asset counterpart to [`on_click`](Self::on_click). Marks the wait and registers the
    /// handler in one call, so the key can't disagree between them.
    ///
    /// On native a `Bytes` asset is already there and this fires almost immediately; on wasm
    /// it fires whenever the fetch resolves. Either way anything that needs the bytes
    /// belongs in `afn` rather than after the spawn.
    pub fn on_asset<AFN: FnMut(&mut Tree, Entity, Vec<u8>) + Send + Sync + 'static>(
        &mut self,
        entity: Entity,
        key: AssetKey,
        mut afn: AFN,
    ) {
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
impl_refire_tuple!(A, B, C, D, E);
impl_refire_tuple!(A, B, C, D, E, F);
