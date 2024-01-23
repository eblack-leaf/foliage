mod renderer;
mod vertex;

use crate::ash::render_packet::RenderPacketStore;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::NumericalContext;
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::{CoreSet, ElmConfiguration};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{IntoSystemConfigs, Resource};
use bevy_ecs::query::{Added, Changed};
use bevy_ecs::system::{Commands, Query};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageData(pub Option<Vec<u8>>);
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct RequestFlag(pub(crate) bool);
#[derive(Bundle, Clone)]
pub struct Image {
    image_id: DifferentialBundle<ImageId>,
    image_data: ImageData,
    was_request: RequestFlag,
    image_storage: DifferentialBundle<ImageStorage>,
    differentiable: Differentiable,
}
impl Image {
    pub fn new(image_id: ImageId) -> Self {
        Self {
            image_id: DifferentialBundle::new(image_id),
            image_data: ImageData(None),
            was_request: RequestFlag::default(),
            image_storage: DifferentialBundle::new(ImageStorage::default()),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
        }
    }
    pub fn fill(image_id: ImageId, data: Vec<u8>) -> Self {
        Self {
            image_id: DifferentialBundle::new(image_id),
            image_data: ImageData(Option::from(data)),
            was_request: RequestFlag(true),
            image_storage: DifferentialBundle::new(ImageStorage::default()),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
        }
    }
    pub fn storage(image_id: ImageId, storage: ImageStorage, data: Vec<u8>) -> Self {
        Self {
            image_id: DifferentialBundle::new(image_id),
            image_data: ImageData(Option::from(data)),
            was_request: RequestFlag(true),
            image_storage: DifferentialBundle::new(storage),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
        }
    }
}
#[derive(Component, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ImageStorage(pub Option<Area<NumericalContext>>);
impl ImageStorage {
    pub fn some(area: Area<NumericalContext>) -> Self {
        Self(Some(area))
    }
}
fn clean_requests(mut cmd: Commands, query: Query<(Entity, &RequestFlag), Added<RequestFlag>>) {
    for (entity, was_request) in query.iter() {
        if was_request.0 {
            cmd.entity(entity).despawn();
        }
    }
}
fn send_image_data(
    mut image_requests: Query<(&mut ImageData, &mut RenderPacketStore), Changed<ImageData>>,
) {
    for (mut data, mut store) in image_requests.iter_mut() {
        if data.0.is_some() {
            store.put(ImageData(Some(data.0.take().unwrap())));
        } else {
            store.put(ImageData(None));
        }
    }
}
#[derive(Resource, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Component)]
pub struct ImageId(pub i32);
impl Leaf for Image {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((
            clean_requests.after(CoreSet::RenderPacket),
            send_image_data.in_set(CoreSet::Differential),
        ));
        differential_enable!(elm, ImageId, ImageStorage, ImageData);
    }
}