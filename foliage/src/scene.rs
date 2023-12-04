use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::elm::Elm;
use crate::window::ScaleFactor;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use bevy_ecs::system::Commands;
#[derive(Copy, Clone)]
pub enum HorizontalAlignment {
    Center(CoordinateUnit),
    Left(CoordinateUnit),
    Right(CoordinateUnit),
}
#[derive(Copy, Clone)]
pub enum VerticalAlignment {
    Center(CoordinateUnit),
    Top(CoordinateUnit),
    Bottom(CoordinateUnit),
}
impl HorizontalAlignment {
    pub fn calc(&self, section: Section<InterfaceContext>) -> CoordinateUnit {
        match self {
            HorizontalAlignment::Center(offset) => section.center().x + offset,
            HorizontalAlignment::Left(offset) => section.left() + offset,
            HorizontalAlignment::Right(offset) => section.right() + offset,
        }
    }
}
impl VerticalAlignment {
    pub fn calc(&self, section: Section<InterfaceContext>) -> CoordinateUnit {
        match self {
            VerticalAlignment::Center(offset) => section.center().y + offset,
            VerticalAlignment::Top(offset) => section.top() + offset,
            VerticalAlignment::Bottom(offset) => section.bottom() + offset,
        }
    }
}
pub enum LayerAlignment {
    Layer(CoordinateUnit),
}
impl LayerAlignment {
    pub fn layer(&self, layer: Layer) -> Layer {
        match self {
            LayerAlignment::Layer(l) => Layer::new(*l) + layer,
        }
    }
}
#[derive(Resource)]
pub struct SceneHandle<S: Scene + Send> {
    scene: Option<S>,
    nodes: Vec<SceneNode<S>>,
}
pub struct SceneNode<S: Scene> {
    binding: SceneBinding,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
    layer_alignment: LayerAlignment,
    node_fn: SceneNodeFn<S>,
}
impl<S: Scene> SceneNode<S> {
    pub fn new(
        binding: SceneBinding,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
        layer_alignment: LayerAlignment,
        n_fn: SceneNodeFnType<S>,
    ) -> Self {
        Self {
            binding,
            horizontal_alignment,
            vertical_alignment,
            layer_alignment,
            node_fn: SceneNodeFn::new(n_fn),
        }
    }
}
pub type SceneNodeFnType<S> = fn(&S, Area<InterfaceContext>, &mut Commands, &ScaleFactor) -> Entity;
pub struct SceneNodeFn<S: Scene>(Box<SceneNodeFnType<S>>);
impl<S: Scene> SceneNodeFn<S> {
    pub fn new(n_fn: SceneNodeFnType<S>) -> Self {
        Self(Box::new(n_fn))
    }
}
pub type SceneBinding = u32;
impl<S: Scene> SceneHandle<S> {
    pub fn new(scene: S) -> Self {
        Self {
            nodes: scene.nodes(),
            scene: Some(scene),
        }
    }
    pub fn set_scene(&mut self, scene: S) {
        self.scene.replace(scene);
    }
    pub fn spawn_at(
        &self,
        coordinate: Coordinate<InterfaceContext>,
        cmd: &mut Commands,
        scale_factor: &ScaleFactor,
    ) -> Vec<(SceneBinding, Entity)> {
        let mut entities = vec![];
        for node in self.nodes.iter() {
            let entity = (node.node_fn.0)(
                self.scene.as_ref().unwrap(),
                coordinate.section.area,
                cmd,
                scale_factor,
            );
            let position = Position::<InterfaceContext>::new(
                node.horizontal_alignment.calc(coordinate.section),
                node.vertical_alignment.calc(coordinate.section),
            );
            let layer = node.layer_alignment.layer(coordinate.layer);
            cmd.entity(entity).insert(position).insert(layer);
            entities.push((node.binding, entity));
        }
        entities
    }
}

pub trait Scene
where
    Self: Clone + Send,
{
    fn nodes(&self) -> Vec<SceneNode<Self>>;
}

pub trait AddScene {
    fn add_scene<S: Scene + Send + Sync + 'static>(&mut self, scene: S);
}
impl<'a, 'b> AddScene for bevy_ecs::system::Commands<'a, 'b> {
    fn add_scene<S: Scene + Send + Sync + 'static>(&mut self, scene: S) {
        self.insert_resource(SceneHandle::new(scene));
    }
}
impl AddScene for Elm {
    fn add_scene<S: Scene + Send + Sync + 'static>(&mut self, scene: S) {
        self.job.container.insert_resource(SceneHandle::new(scene));
    }
}
