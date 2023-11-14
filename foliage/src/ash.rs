use crate::elm::Elm;
use crate::ginkgo::viewport::Viewport;
use crate::ginkgo::Ginkgo;
use anymap::AnyMap;
use std::collections::HashMap;
use std::rc::Rc;
use wgpu::{RenderBundle, RenderPass};

pub(crate) struct AshLeaflet(
    pub(crate) CreateFn,
    pub(crate) ExtractFn,
    pub(crate) InstructionFn,
);
impl AshLeaflet {
    pub(crate) fn leaf_fn<T: Render + 'static>() -> Self {
        Self(
            Box::new(Ash::register::<T>),
            Box::new(T::extract),
            Box::new(Ash::instructions::<T>),
        )
    }
}
pub(crate) type CreateFn = Box<fn(&mut Ash, &Ginkgo)>;
pub(crate) type ExtractFn = Box<fn(&mut Elm, &mut Ash)>;
pub struct Ash {
    pub(crate) renderers: Renderers,
}
pub(crate) struct ExtractionFns(pub(crate) Vec<ExtractFn>);
impl ExtractionFns {
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
            renderers: AnyMap::new(),
        }
    }
    pub(crate) fn establish_renderers(
        &mut self,
        ginkgo: &Ginkgo,
        renderer_queue: Vec<AshLeaflet>,
        extraction_fns: &mut ExtractionFns,
        instruction_fns: &mut InstructionFns,
    ) {
        for leaflet in renderer_queue {
            leaflet.0(self, ginkgo);
            extraction_fns.0.push(leaflet.1);
            instruction_fns.0.push(leaflet.2);
        }
    }
    pub(crate) fn extract(&mut self, elm: &mut Elm, extraction_fns: &ExtractionFns) {
        for extract_fn in extraction_fns.0.iter() {
            extract_fn(elm, self);
        }
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
    fn register<T: Render + 'static>(&mut self, gfx_context: &Ginkgo) {
        self.renderers
            .insert(RenderPackageManager::new(T::create(gfx_context)));
    }
    fn instructions<T: Render + 'static>(&mut self, ginkgo: &Ginkgo) -> Vec<RenderInstructions> {
        self.renderers
            .get_mut::<RenderPackageManager<T>>()
            .unwrap()
            .instructions(ginkgo)
    }
}
pub(crate) type Renderers = AnyMap;
pub trait Render {
    type Key;
    type RenderPackageResources;
    fn create(ginkgo: &Ginkgo) -> Self;
    fn extract(elm: &mut Elm, ash: &mut Ash);

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
    pub data: T,
    pub(crate) instructions: RenderInstructions,
    dirty: bool,
}
pub(crate) struct RenderPackageManager<Renderer: Render> {
    pub(crate) renderer: Renderer,
    pub(crate) packages: HashMap<Renderer::Key, RenderPackage<Renderer::RenderPackageResources>>,
}
impl<Renderer: Render> RenderPackageManager<Renderer> {
    pub(crate) fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
            packages: HashMap::new(),
        }
    }
    pub(crate) fn instructions(&mut self, ginkgo: &Ginkgo) -> Vec<RenderInstructions> {
        let mut instructions = vec![];
        for (_, mut package) in self.packages.iter_mut() {
            if package.dirty {
                package.instructions = self.renderer.record_package(
                    package,
                    RenderInstructionsRecorder::new(ginkgo),
                    ginkgo.viewport.as_ref().unwrap(),
                );
                package.dirty = false;
            }
            instructions.push(package.instructions.clone());
        }
        instructions
    }
}
