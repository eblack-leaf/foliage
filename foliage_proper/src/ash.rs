use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Entity, Query, RemovedComponents};
use bevy_ecs::query::{Added, Changed, Or, With};
use bevy_ecs::system::{Res, ResMut, Resource};
use bevy_ecs::world::World;
use std::cmp::{Ordering, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use wgpu::{CommandEncoderDescriptor, RenderPass, RenderPassDescriptor, TextureViewDescriptor};

use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::elevation::RenderLayer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{DeviceContext, LogicalContext};
use crate::elm::RenderQueueHandle;
use crate::ginkgo::{Ginkgo, ScaleFactor};
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
    pub(crate) prepare_fns: Vec<
        fn(&mut RendererStructure, &Ginkgo, &mut RenderQueueHandle) -> Option<RenderNodeChanges>,
    >,
    pub(crate) drawn: bool,
    pub(crate) draw_calls: DrawCalls,
    pub(crate) draw_fns: Vec<
        for<'a> fn(
            &'a RendererStructure,
            DirectiveGroupPointer,
            DrawRange,
            Section<DeviceContext>,
            &mut RenderPass<'a>,
        ),
    >,
    pub(crate) clipping_sections: HashMap<Entity, ClippingSection>,
}
#[derive(Copy, Clone, Hash, Ord, PartialOrd, PartialEq, Eq, Debug)]
pub(crate) struct DirectiveGroupPointer(pub(crate) i32);
#[derive(Default)]
pub(crate) struct DrawCalls {
    pub(crate) calls: Vec<(usize, DirectiveGroupPointer, DrawRange, ClippingContext)>,
    pub(crate) unsorted: HashMap<usize, HashMap<DirectiveGroupPointer, RenderNodes>>,
    pub(crate) changed: bool,
}
#[derive(Debug, Copy, Clone, Component, PartialEq, Default, PartialOrd)]
pub(crate) enum ClippingContext {
    #[default]
    Screen,
    Entity(Entity),
}
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ClippingSection(pub(crate) Section<DeviceContext>);
#[derive(Component, Copy, Clone, Default)]
pub struct EnableClipping {}
pub(crate) fn pull_clipping_section(
    query: Query<
        (Entity, &Position<LogicalContext>, &Area<LogicalContext>),
        (
            Or<(
                Added<EnableClipping>,
                Changed<Position<LogicalContext>>,
                Changed<Area<LogicalContext>>,
            )>,
            With<EnableClipping>,
        ),
    >,
    mut queue: ResMut<ClippingSectionQueue>,
    scale_factor: Res<ScaleFactor>,
    mut removed: RemovedComponents<EnableClipping>,
) {
    for (entity, pos, area) in query.iter() {
        queue.update.insert(
            entity,
            ClippingSection(Section::logical(*pos, *area).to_device(scale_factor.value())),
        );
    }
    for entity in removed.read() {
        queue.remove.insert(entity);
    }
}
#[derive(Resource, Default)]
pub(crate) struct ClippingSectionQueue {
    update: HashMap<Entity, ClippingSection>,
    remove: HashSet<Entity>,
}
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RenderNode {
    layer: RenderLayer,
    clipping_context: ClippingContext,
}

impl RenderNode {
    pub(crate) fn new(layer: RenderLayer, clip: ClippingContext) -> Self {
        Self {
            layer,
            clipping_context: clip,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderNodes(pub HashMap<usize, RenderNode>);
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
    pub fn associate_directive_group(&mut self, ap: i32, key: R::DirectiveGroupKey) {
        self.node_manager
            .mapping
            .insert(DirectiveGroupPointer(ap), key);
    }
    pub fn disassociate_directive_group(&mut self, ap: i32) {
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
        self.prepare_fns
            .push(|r, g, rqh| -> Option<RenderNodeChanges> {
                let renderer = &mut *r
                    .renderers
                    .get_non_send_resource_mut::<Renderer<R>>()
                    .unwrap();
                R::prepare(renderer, rqh, g);
                renderer.node_manager.changes()
            });
        self.draw_fns.push(|r, ap, ar, cs, rpass| {
            let renderer = r.renderers.get_non_send_resource::<Renderer<R>>().unwrap();
            let key = renderer.get_key(ap.0);
            R::draw(renderer, key, ar, cs, rpass);
        });
    }
    pub(crate) fn order_draw_calls(&mut self) {
        if self.draw_calls.changed {
            // order
            let mut all = vec![];
            for (renderer_index, an) in self.draw_calls.unsorted.iter() {
                for (ptr, nodes) in an.iter() {
                    for (instance_index, node) in nodes.0.iter() {
                        all.push((*renderer_index, *ptr, *instance_index, node.clone()));
                    }
                }
            }
            // TODO add order by layer then group-ptr
            all.sort_by(|lhs, rhs| {
                if lhs.3 < rhs.3 {
                    Ordering::Less
                } else if lhs.3 > rhs.3 {
                    Ordering::Greater
                } else {
                    if lhs.0 < rhs.0 {
                        Ordering::Less
                    } else if lhs.0 > rhs.0 {
                        Ordering::Greater
                    } else {
                        if lhs.1 < rhs.1 {
                            Ordering::Less
                        } else if lhs.1 > rhs.1 {
                            Ordering::Greater
                        } else {
                            if lhs.2 < rhs.2 {
                                Ordering::Less
                            } else if lhs.2 > rhs.2 {
                                Ordering::Greater
                            } else {
                                Ordering::Equal
                            }
                        }
                    }
                }
            });
            self.draw_calls.calls.clear();
            let mut index = 0;
            let mut contiguous = 1u32;
            let mut range_start = None;
            for (renderer_index, ptr, instance_index, node) in all.iter() {
                let this = (
                    *renderer_index,
                    *ptr,
                    *instance_index,
                    node.clipping_context,
                );
                let next = if let Some(n) = all.get(index + 1) {
                    Some((n.0, n.1, n.2, n.3.clipping_context))
                } else {
                    None
                };
                index += 1;
                if let Some(n) = next {
                    if (this.0, this.1, this.2 + 1, this.3) == (n.0, n.1, n.2, n.3) {
                        contiguous += 1;
                        if range_start.is_none() {
                            range_start.replace(this.2);
                        }
                        continue;
                    }
                }
                let start = range_start.take().unwrap_or(this.2);
                self.draw_calls.calls.push((
                    this.0,
                    this.1,
                    DrawRange::new(start as u32, start as u32 + contiguous),
                    node.clipping_context,
                ));
                contiguous = 1;
            }
            self.draw_calls.changed = false;
        }
    }
    pub(crate) fn render(&mut self, ginkgo: &Ginkgo, elm: &mut Elm) {
        for (e, c) in elm
            .ecs
            .world
            .get_resource_mut::<ClippingSectionQueue>()
            .unwrap()
            .update
            .drain()
        {
            self.clipping_sections.insert(e, c);
        }
        for e in elm
            .ecs
            .world
            .get_resource_mut::<ClippingSectionQueue>()
            .unwrap()
            .remove
            .drain()
        {
            self.clipping_sections.remove(&e);
        }
        let mut handle = RenderQueueHandle::new(elm);
        for (i, p_fn) in self.prepare_fns.iter().enumerate() {
            if let Some(changes) = p_fn(&mut self.renderers, ginkgo, &mut handle) {
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
        self.order_draw_calls();
        for (renderer_index, directive_ptr, range, clipping_context_ptr) in
            self.draw_calls.calls.iter()
        {
            let screen_size = ginkgo.viewport().section();
            let clip_section = match clipping_context_ptr {
                ClippingContext::Screen => screen_size,
                ClippingContext::Entity(e) => {
                    self.clipping_sections
                        .get(&e)
                        .cloned()
                        .unwrap_or(ClippingSection(screen_size))
                        .0
                }
            };
            self.draw_fns.get(*renderer_index).as_ref().unwrap()(
                &self.renderers,
                *directive_ptr,
                *range,
                clip_section.intersection(screen_size).unwrap_or_default(),
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
        clipping_section: Section<DeviceContext>,
        render_pass: &mut RenderPass<'a>,
    );
}
