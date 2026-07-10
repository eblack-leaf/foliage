use crate::{AssetKey, Attachment, Foliage, Resource, TargetedEvent};
use bevy_ecs::entity::Entity;
use crate::Trigger;
use bevy_ecs::prelude::{EntityEvent, Event};
use bevy_ecs::system::ResMut;
use std::collections::HashMap;

#[derive(EntityEvent)]
pub struct Write<W: Send + Sync + 'static> {
    entity: Entity,
    _phantom: std::marker::PhantomData<W>,
}
impl<W: Send + Sync + 'static> Default for Write<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Send + Sync + 'static> Write<W> {
    pub fn new() -> Write<W> {
        Write {
            entity: Entity::PLACEHOLDER,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<W: Send + Sync + 'static> Clone for Write<W> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<W: Send + Sync + 'static> Copy for Write<W> {}
impl<W: Send + Sync + 'static> TargetedEvent for Write<W> {
    fn set_target(&mut self, entity: Entity) {
        self.entity = entity;
    }
}
#[derive(EntityEvent)]
pub struct Update<U: Send + Sync + 'static> {
    entity: Entity,
    _phantom: std::marker::PhantomData<U>,
}
impl<U: Send + Sync + 'static> Default for Update<U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<U: Send + Sync + 'static> Update<U> {
    pub fn new() -> Update<U> {
        Update {
            entity: Entity::PLACEHOLDER,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<U: Send + Sync + 'static> Clone for Update<U> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<U: Send + Sync + 'static> Copy for Update<U> {}
impl<U: Send + Sync + 'static> TargetedEvent for Update<U> {
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
