use crate::Trigger;
use crate::{AssetKey, Attachment, Foliage, Resource, TargetedEvent};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{EntityEvent, Event};
use bevy_ecs::system::ResMut;
use std::collections::HashMap;

/// "`W` on this entity has settled to a new value" -- the completion half of the resolve
/// protocol, paired with [`Resolve`].
///
/// Fired *by* whatever computed the value, *after* writing it, so an observer of
/// `Resolved<Section<Logical>>` can read the fresh section. This is the event to watch to
/// react to something; [`Resolve`] is the one to fire to request it.
#[derive(EntityEvent)]
pub struct Resolved<W: Send + Sync + 'static> {
    entity: Entity,
    _phantom: std::marker::PhantomData<W>,
}
impl<W: Send + Sync + 'static> Default for Resolved<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Send + Sync + 'static> Resolved<W> {
    /// The event; its target is filled in when it is sent.
    pub fn new() -> Resolved<W> {
        Resolved {
            entity: Entity::PLACEHOLDER,
            _phantom: std::marker::PhantomData,
        }
    }
    /// Inherent, like the `#[targeted_event]`-generated form -- observer bodies need no
    /// `EntityEvent` trait import.
    pub fn event_target(&self) -> Entity {
        self.entity
    }
}
impl<W: Send + Sync + 'static> Clone for Resolved<W> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<W: Send + Sync + 'static> Copy for Resolved<W> {}
impl<W: Send + Sync + 'static> TargetedEvent for Resolved<W> {
    fn set_target(&mut self, entity: Entity) {
        self.entity = entity;
    }
}
/// "Recompute `U` for this entity" -- the request half of the resolve protocol, paired
/// with [`Resolved`].
///
/// Fired *at* whatever owns the computation, typically because an input it depends on
/// changed. Handlers are expected to be idempotent: a redundant request should recompute
/// the same answer rather than misbehave, since several inputs can each ask in one frame.
#[derive(EntityEvent)]
pub struct Resolve<U: Send + Sync + 'static> {
    entity: Entity,
    _phantom: std::marker::PhantomData<U>,
}
impl<U: Send + Sync + 'static> Default for Resolve<U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<U: Send + Sync + 'static> Resolve<U> {
    /// The event; its target is filled in when it is sent.
    pub fn new() -> Resolve<U> {
        Resolve {
            entity: Entity::PLACEHOLDER,
            _phantom: std::marker::PhantomData,
        }
    }
    /// Inherent, like the `#[targeted_event]`-generated form -- observer bodies need no
    /// `EntityEvent` trait import.
    pub fn event_target(&self) -> Entity {
        self.entity
    }
}
impl<U: Send + Sync + 'static> Clone for Resolve<U> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<U: Send + Sync + 'static> Copy for Resolve<U> {}
impl<U: Send + Sync + 'static> TargetedEvent for Resolve<U> {
    fn set_target(&mut self, entity: Entity) {
        self.entity = entity;
    }
}
/// Named entities, so one part of a UI can reach another without the spawning code
/// having to hand the `Entity` down through every layer between them.
#[derive(Resource, Default)]
pub struct Named {
    map: HashMap<String, Entity>,
}
impl Named {
    /// The entity stored under `n`. Panics if nothing was named that -- names are
    /// expected to be a fixed, author-controlled set.
    pub fn get<S: AsRef<str>>(&self, n: S) -> Entity {
        self.map[n.as_ref()]
    }
    /// The entity stored under `n`, or `None`. What the boundary uses -- an app naming
    /// something that was pruned should get an absence, not a panic.
    pub(crate) fn lookup(&self, n: &str) -> Option<Entity> {
        self.map.get(n).copied()
    }
}
impl Attachment for Named {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(Named::default());
        foliage.world.insert_resource(Keyring::default());
        foliage.define(Name::store);
        foliage.define(StoredKey::store);
    }
}
#[derive(Event)]
pub(crate) struct Name(pub(crate) String, pub(crate) Entity);
impl Name {
    pub(crate) fn store(trigger: Trigger<Self>, mut named: ResMut<Named>) {
        let event = trigger.event();
        named.map.insert(event.0.clone(), event.1);
    }
}
/// Named [`AssetKey`]s, so a loaded asset can be reached by the string it was registered
/// under instead of by threading its key through construction.
#[derive(Resource, Default)]
pub struct Keyring {
    map: HashMap<String, AssetKey>,
}
impl Keyring {
    /// The asset key stored under `n`. Panics if nothing was registered under it.
    pub fn get<S: AsRef<str>>(&self, n: S) -> AssetKey {
        self.map[n.as_ref()]
    }
    /// The key stored under `n`, or `None` -- the non-panicking form the boundary uses.
    pub(crate) fn lookup(&self, n: &str) -> Option<AssetKey> {
        self.map.get(n).copied()
    }
}
#[derive(Event)]
pub(crate) struct StoredKey(pub(crate) String, pub(crate) AssetKey);
impl StoredKey {
    fn store(trigger: Trigger<Self>, mut keyring: ResMut<Keyring>) {
        let event = trigger.event();
        keyring.map.insert(event.0.clone(), event.1);
    }
}
