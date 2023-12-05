use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use bevy_ecs::system::Commands;
use std::collections::HashMap;
pub struct SceneVisibility(pub SceneTag, pub bool);
#[derive(Resource)]
pub struct SnapCompositor {
    // shared resource pool for sharing between scene / bound-entities
    // macro placement tool that can adjust scenes if need be
}
#[derive(Copy, Clone, Hash, Eq, PartialEq, Default)]
pub struct SceneTag(pub(crate) u32);
#[derive(Bundle)]
pub struct SnapAligned {
    alignment: SnapAlignment,
    anchor: AlignmentAnchor,
}
#[derive(Component)]
pub struct SceneLayout {
    pub layout: HashMap<SceneBinding, SnapAlignment>,
}
#[derive(Bundle, Copy, Clone)]
pub struct SnapAlignment {
    pub ha: HorizontalAlignment,
    pub va: VerticalAlignment,
    pub la: LayerAlignment,
}
pub type SceneBinding = u32;
#[derive(Component)]
pub struct Scene {
    pub scene_tag: SceneTag,
    pub coordinate: Coordinate<InterfaceContext>,
    pub entities: HashMap<SceneBinding, Entity>,
    pub layout: SceneLayout,
}
pub trait SceneNode
where
    Self: Bundle,
{
    fn target_area(&self) -> Area<InterfaceContext>; // how to get area out of the given bundle
}
impl Scene {
    pub fn bind<T: SceneNode>(&mut self, scene_binding: SceneBinding, t: T, cmd: &mut Commands) {
        // spawn snap_aligned using binding->layout
        // SceneNode::target_area() to get aligners calc-ed
    }
}
#[derive(Copy, Clone, Component)]
pub struct AlignmentAnchor(pub Coordinate<InterfaceContext>);
#[derive(Copy, Clone)]
pub struct Alignment(pub CoordinateUnit);
#[derive(Component, Copy, Clone)]
pub enum HorizontalAlignment {
    Center(Alignment),
    Left(Alignment),
    Right(Alignment),
}
impl HorizontalAlignment {
    pub fn calc(
        &self,
        scene_section: Section<InterfaceContext>,
        target: Area<InterfaceContext>,
    ) -> CoordinateUnit {
        match self {
            HorizontalAlignment::Center(alignment) => {
                scene_section.center().x - target.width / 2f32 + alignment.0
            }
            HorizontalAlignment::Left(alignment) => scene_section.left() + alignment.0,
            HorizontalAlignment::Right(alignment) => scene_section.right() - alignment.0,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub enum VerticalAlignment {
    Center(Alignment),
    Top(Alignment),
    Bottom(Alignment),
}
impl VerticalAlignment {
    pub fn calc(
        &self,
        scene_section: Section<InterfaceContext>,
        target: Area<InterfaceContext>,
    ) -> CoordinateUnit {
        match self {
            VerticalAlignment::Center(alignment) => {
                scene_section.center().y - target.width / 2f32 + alignment.0
            }
            VerticalAlignment::Top(alignment) => scene_section.top() + alignment.0,
            VerticalAlignment::Bottom(alignment) => scene_section.bottom() - alignment.0,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct LayerAlignment(pub Layer);
impl LayerAlignment {
    pub fn calc(&self, scene: Layer) -> Layer {
        self.0 + scene
    }
}
