use std::collections::HashMap;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Component;
use wgpu::{BindGroup, RenderPipeline, TextureView};
use crate::ash::{Render, Renderer, RenderPhase};
use crate::elm::{Elm, RenderQueueHandle};
use crate::ginkgo::Ginkgo;
use crate::Leaf;

#[derive(Bundle)]
pub struct Image {
    id: ImageId,
}
impl Leaf for Image {
    fn attach(elm: &mut Elm) {
        todo!()
    }
}
#[derive(Copy, Clone, Component, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImageId(pub i32);
pub struct ImageResources {
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: BindGroup,
    images: HashMap<ImageId, ImageGroup>,
}
pub(crate) struct ImageGroup {
    view: TextureView,
    bind_group: BindGroup,

}
impl Render for Image {
    type DirectiveGroupKey = ImageId;
    const RENDER_PHASE: RenderPhase = RenderPhase::Opaque;
    type Resources = ();

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        todo!()
    }

    fn prepare(renderer: &mut Renderer<Self>, queue_handle: &mut RenderQueueHandle, ginkgo: &Ginkgo) -> bool {
        todo!()
    }

    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo) {
        todo!()
    }
}