use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

use bevy_ecs::world::World;
use wgpu::{
    CommandEncoderDescriptor, RenderBundle, RenderBundleDepthStencil, RenderBundleDescriptor,
    RenderBundleEncoderDescriptor, RenderPass, RenderPassDescriptor, TextureViewDescriptor,
};

use crate::color::Color;
use crate::coordinate::elevation::RenderLayer;
use crate::elm::RenderQueueHandle;
use crate::ginkgo::depth::Depth;
use crate::ginkgo::Ginkgo;
use crate::Elm;

#[derive(Default)]
pub struct InstructionSet {
    world: World,
}
pub struct Instructions<R: Render> {
    pub directive_manager: RenderDirectiveManager<R>,
}
impl<R: Render> Instructions<R> {
    pub(crate) fn new() -> Self {
        Self {
            directive_manager: RenderDirectiveManager::new(),
        }
    }
}
#[derive(Default)]
pub(crate) struct Ash {
    pub(crate) renderers: RendererStructure,
    pub(crate) instructions: InstructionSet,
    pub(crate) creation: Vec<fn(&mut RendererStructure, &mut InstructionSet, &Ginkgo)>,
    pub(crate) render_fns: Vec<
        fn(
            &mut RendererStructure,
            &mut InstructionSet,
            &Ginkgo,
            &mut RenderQueueHandle,
        ) -> Option<HashMap<AlphaDrawPointer, AlphaNodes>>,
    >,
    pub(crate) renderer_instructions: Vec<fn(&InstructionSet) -> Vec<&RenderBundle>>,
    pub(crate) drawn: bool,
    pub(crate) alpha_draw_calls: AlphaDrawCalls,
    pub(crate) alpha_draw_fns:
        Vec<for<'a> fn(&'a RendererStructure, AlphaDrawPointer, AlphaRange, &mut RenderPass<'a>)>,
}
#[derive(Copy, Clone, Hash, Ord, PartialOrd, PartialEq, Eq)]
pub(crate) struct AlphaDrawPointer(pub(crate) i32);
pub struct AlphaDraws<R: Render> {
    pub(crate) mapping: HashMap<AlphaDrawPointer, R::DirectiveGroupKey>,
    pub(crate) nodes: HashMap<AlphaDrawPointer, AlphaNodes>,
    changed: bool,
}
impl<R: Render> AlphaDraws<R> {
    pub(crate) fn new() -> Self {
        Self {
            mapping: Default::default(),
            nodes: Default::default(),
            changed: false,
        }
    }
    pub fn set_alpha_nodes(&mut self, ap: i32, nodes: AlphaNodes) {
        self.changed = true;
        let ptr = AlphaDrawPointer(ap);
        self.nodes.insert(ptr, nodes);
    }
}
#[derive(Default)]
pub(crate) struct AlphaDrawCalls {
    pub(crate) calls: Vec<(usize, AlphaDrawPointer, AlphaRange)>,
    pub(crate) unsorted: HashMap<usize, HashMap<AlphaDrawPointer, AlphaNodes>>,
    pub(crate) changed: bool,
}
impl AlphaDrawCalls {
    pub(crate) fn order(&mut self) {
        if self.changed {
            // order
            let mut all = vec![];
            for (renderer_index, an) in self.unsorted.iter() {
                for (ptr, nodes) in an.iter() {
                    for (instance_index, layer) in nodes.0.iter() {
                        all.push((*renderer_index, *ptr, *instance_index, *layer));
                    }
                }
            }
            // TODO add order by layer then group-ptr
            all.sort_by(|lhs, rhs| {
                if lhs.3 > rhs.3 {
                    Ordering::Less
                } else if lhs.3 < rhs.3 {
                    Ordering::Greater
                } else {
                    if lhs.0 < rhs.0 {
                        Ordering::Less
                    } else if lhs.0 > rhs.0 {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                }
            });
            self.calls.clear();
            for (renderer_index, ptr, instance_index, _) in all {
                // TODO optimize => if contiguous renderer-index => keep range going else => end + add call
                self.calls.push((
                    renderer_index,
                    ptr,
                    AlphaRange::new(instance_index as u32, instance_index as u32 + 1),
                ));
            }
            self.changed = false;
        }
    }
}
#[derive(Clone, Debug)]
pub struct AlphaNodes(pub(crate) HashMap<usize, RenderLayer>);
impl AlphaNodes {
    pub fn set_global_layer(mut self, layer: RenderLayer) -> Self {
        for (_, mut l) in self.0.iter_mut() {
            *l = layer;
        }
        self
    }
}
#[derive(Copy, Clone)]
pub(crate) struct AlphaRange {
    pub(crate) start: u32,
    pub(crate) end: u32,
}
impl AlphaRange {
    pub(crate) fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}
#[derive(Default)]
pub(crate) struct RendererStructure {
    pub(crate) renderers: World,
}

pub struct Renderer<R: Render> {
    pub resource_handle: R::Resources,
    pub(crate) alpha_draws: AlphaDraws<R>,
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
            resource_handle: R::create_resources(ginkgo),
            alpha_draws: AlphaDraws::<R>::new(),
        }
    }
    pub fn associate_alpha_pointer(&mut self, ap: i32, key: R::DirectiveGroupKey) {
        self.alpha_draws.mapping.insert(AlphaDrawPointer(ap), key);
    }
    pub fn disassociate_alpha_pointer(&mut self, ap: i32) {
        self.alpha_draws.mapping.remove(&AlphaDrawPointer(ap));
    }
    pub(crate) fn get_key(&self, ap: i32) -> R::DirectiveGroupKey {
        *self.alpha_draws.mapping.get(&AlphaDrawPointer(ap)).unwrap()
    }
}
impl Ash {
    pub(crate) fn initialize(&mut self, ginkgo: &Ginkgo) {
        for c_fn in self.creation.iter() {
            c_fn(&mut self.renderers, &mut self.instructions, ginkgo);
        }
        self.drawn = true;
    }
    pub(crate) fn add_renderer<R: Render>(&mut self) {
        self.creation.push(|r, i, g| {
            let renderer = Renderer::<R>::new(g);
            r.renderers.insert_non_send_resource(renderer);
            i.world.insert_non_send_resource(Instructions::<R>::new());
        });
        self.render_fns.push(
            |r, i, g, rqh| -> Option<HashMap<AlphaDrawPointer, AlphaNodes>> {
                let renderer = &mut *r
                    .renderers
                    .get_non_send_resource_mut::<Renderer<R>>()
                    .unwrap();
                let instructions = &mut *i
                    .world
                    .get_non_send_resource_mut::<Instructions<R>>()
                    .unwrap();
                if R::prepare(renderer, rqh, g) {
                    R::record_opaque(renderer, instructions, g);
                }
                let nodes = if renderer.alpha_draws.changed {
                    Some(renderer.alpha_draws.nodes.clone())
                } else {
                    None
                };
                renderer.alpha_draws.changed = false;
                nodes
            },
        );
        self.renderer_instructions.push(|i| -> Vec<&RenderBundle> {
            i.world
                .get_non_send_resource::<Instructions<R>>()
                .unwrap()
                .directive_manager
                .directives
                .values()
                .map(|d| &d.0)
                .collect::<Vec<&RenderBundle>>()
        });
        self.alpha_draw_fns.push(|r, ap, ar, rpass| {
            let renderer = r.renderers.get_non_send_resource::<Renderer<R>>().unwrap();
            let key = renderer.get_key(ap.0);
            R::draw_alpha_range(renderer, key, ar, rpass);
        });
    }
    pub(crate) fn render(&mut self, ginkgo: &Ginkgo, elm: &mut Elm) {
        let mut handle = RenderQueueHandle::new(elm);
        for (i, r_fn) in self.render_fns.iter().enumerate() {
            if let Some(nodes) = r_fn(
                &mut self.renderers,
                &mut self.instructions,
                ginkgo,
                &mut handle,
            ) {
                self.alpha_draw_calls.unsorted.insert(i, nodes);
                self.alpha_draw_calls.changed = true;
            }
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
            let instructions = r_fn(&self.instructions);
            rpass.execute_bundles(instructions);
        }
        self.alpha_draw_calls.order();
        for (renderer_index, directive_ptr, range) in self.alpha_draw_calls.calls.iter() {
            self.alpha_draw_fns.get(*renderer_index).unwrap()(
                &self.renderers,
                *directive_ptr,
                *range,
                &mut rpass,
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
    type DirectiveGroupKey: Hash + Eq + Copy + Clone;
    type Resources;
    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources;
    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) -> bool;
    fn record_opaque(
        renderer: &mut Renderer<Self>,
        instructions: &mut Instructions<Self>,
        ginkgo: &Ginkgo,
    );
    fn draw_alpha_range<'a>(
        renderer: &'a Renderer<Self>,
        group_key: Self::DirectiveGroupKey,
        alpha_range: AlphaRange,
        render_pass: &mut RenderPass<'a>,
    );
}
