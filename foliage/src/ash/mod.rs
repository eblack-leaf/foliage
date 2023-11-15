
use crate::elm::Elm;
use crate::ginkgo::Ginkgo;
use anymap::AnyMap;



use fns::{AshLeaflet, InstructionFns, PrepareFns};
use render::Render;
use render_package::RenderPackageManager;
use render_packet::RenderPacketManager;


use tag::RenderTagged;
use wgpu::RenderBundle;
use render_instructions::RenderInstructions;

pub mod fns;
pub mod render;
pub mod render_package;
pub mod render_packet;
pub mod tag;
mod render_instructions;

pub struct Ash {
    pub(crate) render_packets: RenderPacketManager,
    pub(crate) renderers: Renderers,
}

impl Ash {
    pub(crate) fn new() -> Self {
        Self {
            render_packets: RenderPacketManager::new(),
            renderers: AnyMap::new(),
        }
    }
    pub(crate) fn establish_renderers(
        &mut self,
        ginkgo: &Ginkgo,
        renderer_queue: Vec<AshLeaflet>,
        prepare_fns: &mut PrepareFns,
        instruction_fns: &mut InstructionFns,
    ) {
        for leaflet in renderer_queue {
            leaflet.0(self, ginkgo);
            prepare_fns.0.push(leaflet.1);
            instruction_fns.0.push(leaflet.2);
            self.render_packets.packets.insert(leaflet.3(), None);
        }
    }
    pub(crate) fn extract(&mut self, elm: &mut Elm) {
        for (tag, packets) in elm
            .job
            .container
            .get_resource_mut::<RenderPacketManager>()
            .unwrap()
            .packets
            .iter_mut()
        {
            self.render_packets
                .packets
                .insert(tag.clone(), packets.take());
        }
    }
    pub(crate) fn preparation(&mut self, ginkgo: &Ginkgo, prepare_fns: &PrepareFns) {
        for prepare_fn in prepare_fns.0.iter() {
            prepare_fn(self, ginkgo);
        }
    }
    pub(crate) fn prepare<T: Render + 'static>(&mut self, ginkgo: &Ginkgo) {
        let render_packets = self.render_packets.get(T::tag());
        T::prepare(
            self.renderers.get_mut::<RenderPackageManager<T>>().unwrap(),
            render_packets,
            ginkgo,
        );
    }
    pub(crate) fn render(&mut self, ginkgo: &mut Ginkgo, instruction_fns: &InstructionFns) {
        let mut instructions = vec![];
        for instruction_fn in instruction_fns.0.iter() {
            let group_instructions = instruction_fn(self, ginkgo);
            instructions.extend(group_instructions);
        }
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
                        .map(|i| i.bundle())
                        .collect::<Vec<&RenderBundle>>(),
                );
            ginkgo
                .queue
                .as_ref()
                .unwrap()
                .submit(std::iter::once(encoder.finish()));
            surface_texture.present();
        }
    }
    fn register<T: Render + 'static>(&mut self, ginkgo: &Ginkgo) {
        self.renderers
            .insert(RenderPackageManager::new(T::create(ginkgo)));
    }
    fn instructions<T: Render + 'static>(&mut self, ginkgo: &Ginkgo) -> Vec<RenderInstructions> {
        self.renderers
            .get_mut::<RenderPackageManager<T>>()
            .unwrap()
            .instructions(ginkgo)
    }
}

pub(crate) type Renderers = AnyMap;
