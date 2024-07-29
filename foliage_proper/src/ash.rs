use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

use bevy_ecs::world::World;
use wgpu::{CommandEncoderDescriptor, RenderPass, RenderPassDescriptor, TextureViewDescriptor};

use crate::color::Color;
use crate::coordinate::elevation::RenderLayer;
use crate::elm::RenderQueueHandle;
use crate::ginkgo::Ginkgo;
use crate::Elm;

#[derive(Default)]
pub struct RenderNodeChanges {
    nodes: HashMap<DirectiveGroupPointer, RenderNodes>,
}
pub struct RenderNodeManager<R: Render> {
    nodes: HashMap<DirectiveGroupPointer, RenderNodes>,
    mapping: HashMap<DirectiveGroupPointer, R::DirectiveGroupKey>,
    changed: bool,
}
impl<R: Render> RenderNodeManager<R> {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Default::default(),
            mapping: Default::default(),
            changed: false,
        }
    }
    pub fn set_nodes(&mut self, ptr: i32, nodes: RenderNodes) {
        let ptr = DirectiveGroupPointer(ptr);
        self.nodes.insert(ptr, nodes);
        self.changed = true;
    }
    pub(crate) fn changes(&mut self) -> Option<RenderNodeChanges> {
        if self.changed {
            self.changed = false;
            let mut changes = RenderNodeChanges::default();
            changes.nodes = self.nodes.clone();
            return Some(changes);
        }
        None
    }
}
#[derive(Default)]
pub(crate) struct Ash {
    pub(crate) renderers: RendererStructure,
    pub(crate) creation: Vec<fn(&mut RendererStructure, &Ginkgo)>,
    pub(crate) render_fns: Vec<
        fn(&mut RendererStructure, &Ginkgo, &mut RenderQueueHandle) -> Option<RenderNodeChanges>,
    >,
    pub(crate) drawn: bool,
    pub(crate) draw_calls: DrawCalls,
    pub(crate) alpha_draw_fns: Vec<
        for<'a> fn(&'a RendererStructure, DirectiveGroupPointer, DrawRange, &mut RenderPass<'a>),
    >,
}
#[derive(Copy, Clone, Hash, Ord, PartialOrd, PartialEq, Eq)]
pub(crate) struct DirectiveGroupPointer(pub(crate) i32);
#[derive(Default)]
pub(crate) struct DrawCalls {
    pub(crate) calls: Vec<(usize, DirectiveGroupPointer, DrawRange)>,
    pub(crate) unsorted: HashMap<usize, HashMap<DirectiveGroupPointer, RenderNodes>>,
    pub(crate) changed: bool,
}
impl DrawCalls {
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
                    DrawRange::new(instance_index as u32, instance_index as u32 + 1),
                ));
            }
            self.changed = false;
        }
    }
}
#[derive(Clone, Debug)]
pub struct RenderNodes(pub HashMap<usize, RenderLayer>);
impl RenderNodes {
    pub fn new() -> Self {
        Self {
            0: Default::default(),
        }
    }
}
#[derive(Copy, Clone)]
pub(crate) struct DrawRange {
    pub(crate) start: u32,
    pub(crate) end: u32,
}
impl DrawRange {
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
    pub node_manager: RenderNodeManager<R>,
}

impl<R: Render> Renderer<R> {
    pub(crate) fn new(ginkgo: &Ginkgo) -> Self {
        Self {
            resource_handle: R::create_resources(ginkgo),
            node_manager: RenderNodeManager::<R>::new(),
        }
    }
    pub fn associate_alpha_pointer(&mut self, ap: i32, key: R::DirectiveGroupKey) {
        self.node_manager
            .mapping
            .insert(DirectiveGroupPointer(ap), key);
    }
    pub fn disassociate_alpha_pointer(&mut self, ap: i32) {
        self.node_manager.mapping.remove(&DirectiveGroupPointer(ap));
    }
    pub(crate) fn get_key(&self, ap: i32) -> R::DirectiveGroupKey {
        *self
            .node_manager
            .mapping
            .get(&DirectiveGroupPointer(ap))
            .unwrap()
    }
}
impl Ash {
    pub(crate) fn initialize(&mut self, ginkgo: &Ginkgo) {
        for c_fn in self.creation.iter() {
            c_fn(&mut self.renderers, ginkgo);
        }
        self.drawn = true;
    }
    pub(crate) fn add_renderer<R: Render>(&mut self) {
        self.creation.push(|r, g| {
            let renderer = Renderer::<R>::new(g);
            r.renderers.insert_non_send_resource(renderer);
        });
        self.render_fns
            .push(|r, g, rqh| -> Option<RenderNodeChanges> {
                let renderer = &mut *r
                    .renderers
                    .get_non_send_resource_mut::<Renderer<R>>()
                    .unwrap();
                R::prepare(renderer, rqh, g);
                renderer.node_manager.changes()
            });
        self.alpha_draw_fns.push(|r, ap, ar, rpass| {
            let renderer = r.renderers.get_non_send_resource::<Renderer<R>>().unwrap();
            let key = renderer.get_key(ap.0);
            R::draw(renderer, key, ar, rpass);
        });
    }
    pub(crate) fn render(&mut self, ginkgo: &Ginkgo, elm: &mut Elm) {
        let mut handle = RenderQueueHandle::new(elm);
        for (i, r_fn) in self.render_fns.iter().enumerate() {
            if let Some(changes) = r_fn(&mut self.renderers, ginkgo, &mut handle) {
                self.draw_calls.unsorted.insert(i, changes.nodes);
                self.draw_calls.changed = true;
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
        self.draw_calls.order();
        for (renderer_index, directive_ptr, range) in self.draw_calls.calls.iter() {
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
    );
    fn draw<'a>(
        renderer: &'a Renderer<Self>,
        group_key: Self::DirectiveGroupKey,
        draw_range: DrawRange,
        render_pass: &mut RenderPass<'a>,
    );
}
