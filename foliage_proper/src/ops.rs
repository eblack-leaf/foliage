use crate::Trigger;
use crate::{AssetKey, Attachment, Foliage, Resource, TargetedEvent};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{EntityEvent, Event};
use bevy_ecs::system::ResMut;
use std::collections::HashMap;

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
#[derive(Resource, Default)]
pub struct Named {
    map: HashMap<String, Entity>,
}
impl Named {
    pub fn get<S: AsRef<str>>(&self, n: S) -> Entity {
        self.map[n.as_ref()]
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
#[derive(Resource, Default)]
pub struct Keyring {
    map: HashMap<String, AssetKey>,
}
impl Keyring {
    pub fn get<S: AsRef<str>>(&self, n: S) -> AssetKey {
        self.map[n.as_ref()]
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
