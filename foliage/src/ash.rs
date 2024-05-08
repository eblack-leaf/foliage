use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

use bevy_ecs::world::World;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    CommandEncoderDescriptor, RenderBundle, RenderBundleDepthStencil, RenderBundleDescriptor,
    RenderBundleEncoderDescriptor, RenderPassDescriptor, TextureViewDescriptor,
};

use crate::color::Color;
use crate::ginkgo::{Depth, Ginkgo};
use crate::Elm;

#[derive(Default)]
pub(crate) struct Ash {
    pub(crate) renderers: RendererStructure,
    pub(crate) creation: Vec<Box<fn(&mut RendererStructure, &Ginkgo)>>,
    pub(crate) render_fns: Vec<Box<fn(&mut RendererStructure, &Ginkgo, &Elm)>>,
    pub(crate) renderer_instructions: Vec<Box<fn(&RendererStructure) -> Vec<&RenderBundle>>>,
    pub(crate) drawn: bool,
}

#[derive(Default)]
pub(crate) struct RendererStructure {
    pub(crate) renderers: World,
}

pub(crate) struct Renderer<R: Render> {
    pub(crate) phase: RenderPhase,
    pub directive_manager: RenderDirectiveManager<R>,
    pub resource_handle: R::Resources,
}

pub struct RenderDirectiveManager<R: Render> {
    pub(crate) directives: HashMap<R::DirectiveGroupKey, RenderDirective>,
}

impl<R: Render> RenderDirectiveManager<R> {
    pub(crate) fn new() -> Self {
        Self {
            directives: HashMap::new(),
        }
    }
    pub fn fill(&mut self, key: R::DirectiveGroupKey, render_directive: RenderDirective) {
        self.directives.insert(key, render_directive);
    }
    pub fn remove(&mut self, key: R::DirectiveGroupKey) {
        self.directives.remove(&key);
    }
}

impl<R: Render> Renderer<R> {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        Self {
            phase: R::RENDER_PHASE,
            directive_manager: RenderDirectiveManager::new(),
            resource_handle: R::create_resources(ginkgo),
        }
    }
}

#[derive(Copy, Clone)]
pub enum RenderPhase {
    Opaque,
    Alpha(i32),
}

impl RenderPhase {
    pub const fn value(&self) -> i32 {
        match self {
            RenderPhase::Opaque => 0,
            RenderPhase::Alpha(priority) => *priority,
        }
    }
}

impl PartialEq<Self> for RenderPhase {
    fn eq(&self, other: &Self) -> bool {
        match self {
            RenderPhase::Opaque => match other {
                RenderPhase::Opaque => true,
                RenderPhase::Alpha(_) => false,
            },
            RenderPhase::Alpha(priority_one) => match other {
                RenderPhase::Opaque => false,
                RenderPhase::Alpha(priority_two) => priority_one == priority_two,
            },
        }
    }
}

impl PartialOrd for RenderPhase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            RenderPhase::Opaque => match other {
                RenderPhase::Opaque => Some(Ordering::Equal),
                RenderPhase::Alpha(_) => Some(Ordering::Less),
            },
            RenderPhase::Alpha(priority_one) => match other {
                RenderPhase::Opaque => Some(Ordering::Greater),
                RenderPhase::Alpha(priority_two) => Some(priority_one.cmp(priority_two)),
            },
        }
    }
}

impl Ash {
    pub(crate) fn initialize(&mut self, ginkgo: &Ginkgo) {
        for c_fn in self.creation.iter() {
            (c_fn)(&mut self.renderers, ginkgo);
        }
        // self.drawn = true;
    }
    pub(crate) fn add_renderer<R: Render>(&mut self) {
        self.creation.push(Box::new(|r, g| {
            let renderer = Renderer::<R>::new(g);
            r.renderers.insert_non_send_resource(renderer);
        }));
        self.render_fns.push(Box::new(|r, g, e| {
            let renderer = &mut *r
                .renderers
                .get_non_send_resource_mut::<Renderer<R>>()
                .unwrap();
            let extract = R::extract(e);
            if R::prepare(renderer, extract) {
                R::record(renderer, g);
            }
        }));
        self.renderer_instructions
            .push(Box::new(|r| -> Vec<&RenderBundle> {
                r.renderers
                    .get_non_send_resource::<Renderer<R>>()
                    .unwrap()
                    .directive_manager
                    .directives
                    .iter()
                    .map(|(_, d)| &d.0)
                    .collect::<Vec<&RenderBundle>>()
            }));
    }
    pub(crate) fn render(&mut self, ginkgo: &Ginkgo, elm: &Elm) {
        for r_fn in self.render_fns.iter() {
            (r_fn)(&mut self.renderers, ginkgo, elm);
        }
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
            color_attachments: &ginkgo.color_attachment(&view, Color::BLACK),
            depth_stencil_attachment: ginkgo.depth_stencil_attachment(),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        for r_fn in self.renderer_instructions.iter() {
            let instructions = (r_fn)(&self.renderers);
            rpass.execute_bundles(instructions);
        }
        drop(rpass);
        ginkgo
            .context()
            .queue
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

pub struct RenderDirective(pub(crate) RenderBundle);

pub struct RenderDirectiveRecorder<'a>(pub(crate) wgpu::RenderBundleEncoder<'a>);

impl<'a> RenderDirectiveRecorder<'a> {
    pub fn new(ginkgo: &'a Ginkgo) -> Self {
        Self(
            ginkgo
                .context()
                .device
                .create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
                    label: Some("recorder"),
                    color_formats: &[Some(ginkgo.configuration().config.format)],
                    depth_stencil: Option::from(RenderBundleDepthStencil {
                        format: Depth::FORMAT,
                        depth_read_only: false,
                        stencil_read_only: false,
                    }),
                    sample_count: ginkgo.configuration().msaa.samples(),
                    multiview: None,
                }),
        )
    }
    pub fn finish(self) -> RenderDirective {
        RenderDirective(self.0.finish(&RenderBundleDescriptor {
            label: Some("recorder-finish"),
        }))
    }
}

pub trait Render
where
    Self: Sized + 'static,
{
    type Vertex: Pod + Zeroable;
    type DirectiveGroupKey: Hash + Eq + Copy + Clone;
    const RENDER_PHASE: RenderPhase;
    type Resources;
    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources;
    type Extraction;
    fn extract(elm: &Elm) -> Self::Extraction;
    fn prepare(renderer: &mut Renderer<Self>, extract: Self::Extraction) -> bool;
    fn record(renderer: &mut Renderer<Self>, ginkgo: &Ginkgo);
}
