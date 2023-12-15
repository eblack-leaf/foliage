mod renderer;
mod vertex;

use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::InterfaceContext;
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::ElmConfiguration;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};
#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageData(pub Option<Vec<u8>>, u32, u32);
#[derive(Bundle)]
pub struct Image {
    image_id: DifferentialBundle<ImageId>,
    image_data: DifferentialBundle<ImageData>,
    differentiable: Differentiable,
}
impl Image {
    pub fn new(
        position: Position<InterfaceContext>,
        area: Area<InterfaceContext>,
        layer: Layer,
        image_id: ImageId,
    ) -> Self {
        Self {
            image_id: DifferentialBundle::new(image_id),
            image_data: DifferentialBundle::new(ImageData(None, 1, 1)),
            differentiable: Differentiable::new::<Self>(position, area, layer),
        }
    }
    pub fn image_request(image_id: ImageId, width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            image_id: DifferentialBundle::new(image_id),
            image_data: DifferentialBundle::new(ImageData(Option::from(data), width, height)),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
        }
    }
}
#[derive(Resource, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Component)]
pub struct ImageId(pub i32);
impl Leaf for Image {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        differential_enable!(elm, ImageId, ImageData);
    }
}
