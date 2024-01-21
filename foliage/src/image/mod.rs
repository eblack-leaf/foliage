mod renderer;
mod vertex;

use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::InterfaceContext;
use crate::differential::{Differentiable, DifferentialBundle};
use crate::differential_enable;
use crate::elm::config::{CoreSet, ElmConfiguration};
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{IntoSystemConfigs, Resource};
use bevy_ecs::query::Added;
use bevy_ecs::system::{Commands, Query};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageData(pub Option<Vec<u8>>);
#[derive(Component, Copy, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ImageView(pub Option<Section<InterfaceContext>>);
#[derive(Bundle, Clone)]
pub struct Image {
    image_id: DifferentialBundle<ImageId>,
    image_data: DifferentialBundle<ImageData>,
    image_view: DifferentialBundle<ImageView>,
    differentiable: Differentiable,
}
impl Image {
    pub fn new(image_id: ImageId) -> Self {
        Self {
            image_id: DifferentialBundle::new(image_id),
            image_data: DifferentialBundle::new(ImageData(None)),
            image_view: DifferentialBundle::new(ImageView::default()),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
        }
    }
    pub fn image_request(image_id: ImageId, data: Vec<u8>) -> Self {
        Self {
            image_id: DifferentialBundle::new(image_id),
            image_data: DifferentialBundle::new(ImageData(Option::from(data))),
            image_view: DifferentialBundle::new(ImageView::default()),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
        }
    }
    pub fn with_view(mut self, view: Option<Section<InterfaceContext>>) -> Self {
        self.image_view = DifferentialBundle::new(ImageView(view));
        self
    }
}
fn clean_requests(mut cmd: Commands, query: Query<(Entity, &ImageData), Added<ImageData>>) {
    for (entity, img_data) in query.iter() {
        if img_data.0.is_some() {
            cmd.entity(entity).despawn();
        }
    }
}
#[derive(Resource, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Component)]
pub struct ImageId(pub i32);
impl Leaf for Image {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main()
            .add_systems(clean_requests.after(CoreSet::RenderPacket));
        differential_enable!(elm, ImageId, ImageData, ImageView);
    }
}
