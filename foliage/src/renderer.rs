use std::collections::HashMap;
use std::marker::PhantomData;
use bevy_ecs::prelude::Entity;
use crate::engen::Engen;
use crate::gfx::viewport::Viewport;

pub struct RenderEngen {

}
impl RenderEngen {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
pub(crate) struct RenderLeaflet(pub Box<fn(&mut RenderEngen)>);
impl RenderLeaflet {
    pub(crate) fn leaf_fn<T: RenderLeaf>() -> Self {
        Self(Box::new(T::configure))
    }
}
pub trait RenderLeaf {
    fn configure(render_engen: &mut RenderEngen);
}
pub trait Extract {
    type RenderPacket;
    fn extract(engen: &mut Engen) -> HashMap<Entity, Self::RenderPacket>;
}
pub struct RenderInstructions(pub wgpu::RenderBundle);
pub struct RenderInstructionsRecorder<'a>(pub wgpu::RenderBundleEncoder<'a>);
pub(crate) struct RenderPackage<T> {
    pub(crate) instructions: RenderInstructions,
    data: PhantomData<T>,
}
pub(crate) struct Renderer<RenderResources, Key, RenderPackageData> {
    pub(crate) resources: RenderResources,
    pub(crate) packages: HashMap<Key, RenderPackage<RenderPackageData>>,
}
