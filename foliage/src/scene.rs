use crate::coordinate::layer::Layer;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use std::collections::HashMap;

pub type SceneBinding = u32;
#[derive(Component)]
pub struct Scene {
    pub coordinate: Coordinate<InterfaceContext>,
    pub entities: HashMap<SceneBinding, Entity>,
}
impl Scene {
    pub fn bind(
        &mut self,
        scene_binding: SceneBinding,
        entity: Entity
    ) {

    }
}
pub struct AlignmentTarget(pub Entity);
pub struct Alignment(pub CoordinateUnit);
#[derive(Component)]
pub enum HorizontalAlignment {
    Center(Alignment),
    Left(Alignment),
    Right(Alignment),
}
#[derive(Component)]
pub enum VerticalAlignment {
    Center(Alignment),
    Top(Alignment),
    Right(Alignment),
}
pub struct LayerAlignment(pub Layer);