use crate::{AssetKey, Attachment, Foliage, Resource};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Event, Trigger};
use bevy_ecs::system::ResMut;
use std::collections::HashMap;

#[derive(Event, Copy, Clone)]
pub struct Write<W> {
    _phantom: std::marker::PhantomData<W>,
}
impl<W> Default for Write<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W> Write<W> {
    pub fn new() -> Write<W> {
        Write {
            _phantom: std::marker::PhantomData,
        }
    }
}
#[derive(Event, Copy, Clone)]
pub struct Update<U> {
    _phantom: std::marker::PhantomData<U>,
}
impl<U> Default for Update<U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<U> Update<U> {
    pub fn new() -> Update<U> {
        Update {
            _phantom: std::marker::PhantomData,
        }
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
