use crate::ash::clip::prepare_clip_section;
use crate::ginkgo::Ginkgo;
use crate::{Attachment, Color, Component, DiffMarkers, Foliage, Resource};
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::world::World;
use wgpu::{CommandEncoderDescriptor, RenderPassDescriptor, TextureViewDescriptor};

pub(crate) mod clip;
pub(crate) mod differential;
mod node;
pub(crate) mod queue;

impl Attachment for Ash {
    fn attach(foliage: &mut Foliage) {
        foliage
            .diff
            .add_systems(prepare_clip_section.in_set(DiffMarkers::Prepare));
    }
}
pub struct Ash {
    pub(crate) drawn: bool,
}
impl Default for Ash {
    fn default() -> Self {
        Self::new()
    }
}
impl Ash {
    pub fn new() -> Self {
        Self { drawn: false }
    }
    pub(crate) fn initialize(&mut self, ginkgo: &Ginkgo) {
        // TODO create renderers
    }
    pub(crate) fn render(&mut self, ginkgo: &Ginkgo) {
        let surface_texture = ginkgo.surface_texture();
        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder =
            ginkgo
                .context()
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("present-encoder"),
                });
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render-pass"),
            color_attachments: &ginkgo.color_attachment(&view, Color::gray(50)),
            depth_stencil_attachment: ginkgo.depth_stencil_attachment(),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        // draw
        drop(rpass);
        ginkgo
            .context()
            .queue
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

pub trait Render {
    fn extract(frontend: &mut World, backend: &mut World);
    fn prepare();
    fn render();
}
