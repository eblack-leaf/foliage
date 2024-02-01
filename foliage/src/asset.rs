use crate::elm::config::{CoreSet, ElmConfiguration};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Component, IntoSystemConfigs, Res, Resource};
use bevy_ecs::system::Query;
use std::collections::HashMap;

pub type AssetKey = i32;
#[derive(Resource, Default)]
pub struct AssetContainer {
    pub assets: HashMap<AssetKey, Option<Vec<u8>>>,
}
impl AssetContainer {
    pub fn store(&mut self, id: AssetKey, bytes: Option<Vec<u8>>) {
        self.assets.insert(id, bytes);
    }
}
#[derive(Component, Clone)]
pub struct OnFetch(pub AssetKey, pub Box<AssetFetchFn>);
impl OnFetch {
    pub fn new(key: AssetKey, func: AssetFetchFn) -> Self {
        Self(key, Box::new(func))
    }
}
pub type AssetFetchFn = fn(Vec<u8>, &mut Commands);
fn on_fetch(
    fetch_requests: Query<(Entity, &OnFetch)>,
    mut cmd: Commands,
    assets: Res<AssetContainer>,
) {
    for (entity, on_fetch) in fetch_requests.iter() {
        if let Some(asset) = assets.assets.get(&on_fetch.0) {
            if let Some(asset) = asset {
                on_fetch.1(asset.clone(), &mut cmd);
                cmd.entity(entity).despawn();
            }
        }
    }
}
impl Leaf for AssetContainer {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.container().insert_resource(AssetContainer::default());
        elm.main().add_systems(
            on_fetch.in_set(CoreSet::ProcessEvent), //.run_if(|ac: Res<AssetContainer>| ac.is_changed()),
        );
    }
}

#[macro_export]
macro_rules! load_native_asset {
    ($elm:ident, $id:expr, $p:literal) => {
        #[cfg(not(target_family = "wasm"))]
        $elm.store_local_asset($id, include_bytes!($p).to_vec());
    };
}
