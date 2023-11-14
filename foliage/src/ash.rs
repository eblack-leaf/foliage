use crate::elm::Elm;
use crate::ginkgo::viewport::Viewport;
use crate::ginkgo::Ginkgo;
use anymap::AnyMap;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Bytes;
use std::rc::Rc;
use wgpu::{RenderBundle, RenderPass};
#[derive(Component, Copy, Clone, Hash, Eq, PartialEq)]
pub struct RenderTag(pub i32);
pub(crate) struct AshLeaflet(
    pub(crate) CreateFn,
    pub(crate) PrepareFn,
    pub(crate) InstructionFn,
);
impl AshLeaflet {
    pub(crate) fn leaf_fn<T: Render + 'static>() -> Self {
        Self(
            Box::new(Ash::register::<T>),
            Box::new(Ash::prepare::<T>),
            Box::new(Ash::instructions::<T>),
        )
    }
}
pub(crate) type CreateFn = Box<fn(&mut Ash, &Ginkgo)>;
pub(crate) type PrepareFn = Box<fn(&mut Ash, &Ginkgo)>;
pub(crate) struct RenderPacketManager {
    pub(crate) packets: HashMap<RenderTag, Option<HashMap<Entity, RenderPacket>>>,
}
pub type RenderPackets = HashMap<Entity, RenderPacket>;
impl RenderPacketManager {
    pub(crate) fn new() -> Self {
        Self {
            packets: HashMap::new(),
        }
    }
    pub(crate) fn get(&mut self, tag: RenderTag) -> Option<RenderPackets> {
        if let Some(h) = self.packets.get_mut(&tag) {
            return h.take();
        }
        None
    }
}
pub struct Ash {
    pub(crate) render_packets: RenderPacketManager,
    pub(crate) renderers: Renderers,
}
pub(crate) struct PrepareFns(pub(crate) Vec<PrepareFn>);
impl PrepareFns {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }
}
pub(crate) struct RenderPassHandle<'a>(pub RenderPass<'a>);
pub(crate) type InstructionFn = Box<fn(&mut Ash, &Ginkgo) -> Vec<RenderInstructions>>;
pub(crate) struct InstructionFns(pub(crate) Vec<InstructionFn>);
impl InstructionFns {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }
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
        let surface_texture = ginkgo.surface_texture();
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ginkgo.device.as_ref().unwrap().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("command-encoder"),
            },
        );
        let mut instructions = vec![];
        for instruction_fn in instruction_fns.0.iter() {
            let group_instructions = instruction_fn(self, &ginkgo);
            instructions.extend(group_instructions);
        }
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
#[derive(Component)]
pub struct RenderPacket(pub HashMap<i32, Vec<u8>>);
pub(crate) type Renderers = AnyMap;
pub trait Render
where
    Self: Sized,
{
    type Key: Hash + Eq + PartialEq;
    type RenderPackageResources;
    fn tag() -> RenderTag;
    fn create(ginkgo: &Ginkgo) -> Self;
    fn prepare(
        pm: &mut RenderPackageManager<Self>,
        render_packets: Option<RenderPackets>,
        ginkgo: &Ginkgo,
    );

    fn record_package(
        &self,
        package: &RenderPackage<Self::RenderPackageResources>,
        recorder: RenderInstructionsRecorder,
        viewport: &Viewport,
    ) -> RenderInstructions;
}
#[derive(Clone)]
pub struct RenderInstructions(pub Rc<RenderBundle>);
impl RenderInstructions {
    pub(crate) fn bundle(&self) -> &wgpu::RenderBundle {
        self.0.as_ref()
    }
}
pub struct RenderInstructionsRecorder<'a>(pub wgpu::RenderBundleEncoder<'a>);
impl<'a> RenderInstructionsRecorder<'a> {
    pub(crate) fn new(ginkgo: &'a Ginkgo) -> Self {
        Self(
            ginkgo
                .device
                .as_ref()
                .unwrap()
                .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: Some("render-bundle"),
                    color_formats: &ginkgo.color_attachment_format(),
                    depth_stencil: ginkgo.render_bundle_depth_stencil(),
                    sample_count: ginkgo.msaa_samples(),
                    multiview: None,
                }),
        )
    }
    pub fn finish(mut self) -> RenderInstructions {
        RenderInstructions(Rc::new(self.0.finish(&wgpu::RenderBundleDescriptor {
            label: Some("render-bundle-desc"),
        })))
    }
}
pub struct RenderPackage<T> {
    pub(crate) data: T,
    pub(crate) instructions: Option<RenderInstructions>,
    dirty: bool,
}
impl<T> RenderPackage<T> {
    pub(crate) fn new(resources: T) -> Self {
        Self {
            data: resources,
            instructions: None,
            dirty: true,
        }
    }
    pub fn flag_dirty(&mut self) {
        self.dirty = true;
    }
}
pub struct RenderPackageManager<Renderer: Render> {
    pub renderer: Renderer,
    pub packages: HashMap<Renderer::Key, RenderPackage<Renderer::RenderPackageResources>>,
}
impl<Renderer: Render> RenderPackageManager<Renderer> {
    pub(crate) fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
            packages: HashMap::new(),
        }
    }
    pub(crate) fn new_package(
        &mut self,
        key: Renderer::Key,
        package_resources: Renderer::RenderPackageResources,
    ) {
        self.packages
            .insert(key, RenderPackage::new(package_resources));
    }
    pub(crate) fn instructions(&mut self, ginkgo: &Ginkgo) -> Vec<RenderInstructions> {
        let mut instructions = vec![];
        for (_, mut package) in self.packages.iter_mut() {
            if package.dirty {
                package.instructions.replace(self.renderer.record_package(
                    package,
                    RenderInstructionsRecorder::new(ginkgo),
                    ginkgo.viewport.as_ref().unwrap(),
                ));
                package.dirty = false;
            }
            instructions.push(package.instructions.clone().unwrap());
        }
        instructions
    }
}
