use crate::ginkgo::Ginkgo;
use anymap::AnyMap;
use instruction::InstructionGroups;
use leaflet::RenderLeafletStorage;
use render_packet::RenderPacketPackage;
use renderer::RendererStorage;
use std::collections::HashMap;

pub mod identification;
pub mod instruction;
pub mod leaflet;
pub mod render;
pub mod render_packet;
pub mod renderer;

pub(crate) struct Ash {
    pub(crate) render_packet_package: Option<RenderPacketPackage>,
    pub(crate) renderer_handler: RendererStorage,
    pub(crate) instruction_groups: InstructionGroups,
    pub(crate) render_leaflets: RenderLeafletStorage,
}
impl Ash {
    pub(crate) fn new() -> Self {
        Self {
            render_packet_package: Some(RenderPacketPackage(HashMap::new())),
            renderer_handler: RendererStorage(AnyMap::new()),
            instruction_groups: InstructionGroups::default(),
            render_leaflets: RenderLeafletStorage::new(),
        }
    }
    pub(crate) fn establish(&mut self, ginkgo: &Ginkgo, render_leaflet: RenderLeafletStorage) {
        for (id, leaf) in render_leaflet.iter() {
            (leaf.register_fn)(self, ginkgo);
        }
        self.render_leaflets = render_leaflet
    }
    pub(crate) fn extract(&mut self, package: RenderPacketPackage) {
        self.render_packet_package.replace(package);
    }
    pub(crate) fn prepare(&mut self, ginkgo: &Ginkgo) {
        for (id, leaf) in self.render_leaflets.iter() {
            (leaf.prepare_packages_fn)(
                &mut self.renderer_handler,
                ginkgo,
                self.render_packet_package.as_mut().unwrap(),
            );
            (leaf.prepare_resources_fn)(&mut self.renderer_handler, ginkgo);
        }
    }
    pub(crate) fn record(&mut self, ginkgo: &Ginkgo) {
        for (id, leaf) in self.render_leaflets.iter() {
            (leaf.record_fn)(&mut self.renderer_handler, ginkgo);
            let instructions = (leaf.instruction_fetch_fn)(&mut self.renderer_handler);
            self.instruction_groups.obtain(id).0 = instructions.0.clone();
        }
    }
    pub(crate) fn render(&self, ginkgo: &mut Ginkgo) {
        let instructions = self.instruction_groups.instructions();
        if !instructions.is_empty() {
            let surface_texture = ginkgo.surface_texture();
            let view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = ginkgo.device.as_ref().unwrap().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("command-encoder"),
                },
            );
            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render-pass"),
                    color_attachments: &ginkgo.color_attachment(&view),
                    depth_stencil_attachment: ginkgo.depth_stencil_attachment(),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .execute_bundles(
                    instructions
                        .iter()
                        .map(|i| i.0.as_ref())
                        .collect::<Vec<&wgpu::RenderBundle>>(),
                );
            ginkgo
                .queue
                .as_ref()
                .unwrap()
                .submit(std::iter::once(encoder.finish()));
            surface_texture.present();
        }
    }
}
