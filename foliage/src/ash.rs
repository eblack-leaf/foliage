use std::any::TypeId;
use crate::color::Color;
use crate::ginkgo::Ginkgo;
use crate::Elm;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use wgpu::{CommandEncoderDescriptor, RenderPassDescriptor, TextureViewDescriptor};
#[derive(Default)]
pub(crate) struct Ash {
    pub(crate) renderers: RendererStructure,
    pub(crate) creation: Vec<Box<fn(&mut RendererStructure, &Ginkgo)>>,
    pub(crate) render_fns: Vec<Box<fn(&mut RendererStructure, &Ginkgo, &Elm)>>,
}
pub(crate) type RendererStructure = HashMap<TypeId, Renderer>;
pub(crate) struct Renderer {
    pub(crate) phase: RenderPhase,
    pub(crate) directives: Vec<RenderDirective>,
    pub(crate) resource_handle: Vec<u8>,
}
impl Renderer {
    pub(crate) fn new(render_phase: RenderPhase) -> Self {
        Self {
            phase: render_phase,
            directives: vec![],
            resource_handle: vec![],
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
    pub(crate) fn add_renderer<R: Render>(&mut self) {
        self.creation.push(Box::new(|r, g| {
            let mut renderer = Renderer::new(R::RENDER_PHASE);
            let h = Box::new(R::create_resources(g));
            renderer.resource_handle = ;
            r.push(renderer);
        }));
        self.render_fns.push(
            Box::new(|r, g, e| {
                let renderer = r.get(&TypeId::of::<R>()).expect("renderer-not-present");
                let resources = rmp_serde::from_slice(renderer.resource_handle.as_slice()).expect("incorrect-interpretation");
                let extract = R::extract(e);

            })
        );
    }
    pub(crate) fn present(&self, ginkgo: &Ginkgo) {
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
        for (_, r) in self.renderers.iter() {
            rpass.execute_bundles(
                r.directives
                    .iter()
                    .map(|d| &d.0)
                    .collect::<Vec<&wgpu::RenderBundle>>(),
            );
        }
        drop(rpass);
        ginkgo
            .context()
            .queue
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}
pub struct RenderDirective(pub(crate) wgpu::RenderBundle);
pub struct RenderDirectiveRecorder<'a>(pub(crate) wgpu::RenderBundleEncoder<'a>);
impl<'a> RenderDirectiveRecorder<'a> {
    pub fn new(ginkgo: &'a Ginkgo) -> Self {
        todo!()
    }
    pub fn finish(self) -> RenderDirective {
        todo!()
    }
}
pub trait Render {
    const RENDER_PHASE: RenderPhase;
    fn create_resources(ginkgo: &Ginkgo) -> Self;
    type Extraction;
    fn extract(elm: &Elm) -> Self::Extraction;
    fn prepare(&mut self, extract: Self::Extraction) -> bool;
    fn record(renderer: &mut Renderer, ginkgo: &Ginkgo);
}
