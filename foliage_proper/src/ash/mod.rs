use crate::ash::clip::{prepare_clip_section, ClipQueue, ClipSection};
use crate::ash::differential::RenderQueueHandle;
use crate::foliage::{DiffMarkers, Foliage};
use crate::ginkgo::Ginkgo;
use crate::image::Image;
use crate::shape::Shape;
use crate::{Attachment, ClipContext, Color, Icon, Panel, Text};
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::world::World;
use node::Node;
use render::{ContiguousSpan, PipelineId, Render, Renderer};
use std::cmp::Ordering;
use std::collections::HashMap;
use wgpu::{CommandEncoderDescriptor, RenderPassDescriptor, TextureViewDescriptor};

pub(crate) mod clip;
pub(crate) mod differential;
pub(crate) mod instance;
pub(crate) mod node;
pub(crate) mod render;

impl Attachment for Ash {
    fn attach(foliage: &mut Foliage) {
        foliage
            .diff
            .add_systems(prepare_clip_section.in_set(DiffMarkers::Extract));
        foliage.world.insert_resource(ClipQueue::default());
    }
}
pub(crate) struct Ash {
    pub(crate) drawn: bool,
    pub(crate) nodes: Vec<Node>,
    pub(crate) contiguous: Vec<ContiguousSpan>,
    pub(crate) text: Option<Renderer<Text>>,
    pub(crate) panel: Option<Renderer<Panel>>,
    pub(crate) image: Option<Renderer<Image>>,
    pub(crate) icon: Option<Renderer<Icon>>,
    pub(crate) shape: Option<Renderer<Shape>>,
    pub(crate) clip: HashMap<ClipContext, ClipSection>,
}
impl Default for Ash {
    fn default() -> Self {
        Self::new()
    }
}
impl Ash {
    pub(crate) fn new() -> Self {
        Self {
            drawn: false,
            nodes: vec![],
            contiguous: vec![],
            text: None,
            panel: None,
            image: None,
            icon: None,
            shape: None,
            clip: Default::default(),
        }
    }
    pub(crate) fn initialize(&mut self, ginkgo: &Ginkgo) {
        self.text.replace(Text::renderer(ginkgo));
        self.panel.replace(Panel::renderer(ginkgo));
        self.image.replace(Image::renderer(ginkgo));
        self.icon.replace(Icon::renderer(ginkgo));
        self.shape.replace(Shape::renderer(ginkgo));
    }
    pub(crate) fn prepare(&mut self, world: &mut World, ginkgo: &Ginkgo) {
        for (e, c) in world.get_resource_mut::<ClipQueue>().unwrap().queue.drain() {
            self.clip.insert(ClipContext::Entity(e), c);
        }
        let mut queues = RenderQueueHandle::new(world);
        let mut nodes = vec![];
        let mut to_remove = vec![];
        let text_nodes = Render::prepare(self.text.as_mut().unwrap(), &mut queues, ginkgo);
        nodes.extend(text_nodes.updated);
        to_remove.extend(text_nodes.removed);
        let panel_nodes = Render::prepare(self.panel.as_mut().unwrap(), &mut queues, ginkgo);
        nodes.extend(panel_nodes.updated);
        to_remove.extend(panel_nodes.removed);
        let image_nodes = Render::prepare(self.image.as_mut().unwrap(), &mut queues, ginkgo);
        nodes.extend(image_nodes.updated);
        to_remove.extend(image_nodes.removed);
        let icon_nodes = Render::prepare(self.icon.as_mut().unwrap(), &mut queues, ginkgo);
        nodes.extend(icon_nodes.updated);
        to_remove.extend(icon_nodes.removed);
        let shape_nodes = Render::prepare(self.shape.as_mut().unwrap(), &mut queues, ginkgo);
        nodes.extend(shape_nodes.updated);
        to_remove.extend(shape_nodes.removed);
        if nodes.is_empty() && to_remove.is_empty() {
            return;
        }
        let mut idxs = to_remove
            .iter()
            .filter_map(|rn| {
                self.nodes.iter().position(|n| {
                    n.pipeline == rn.pipeline_id
                        && n.group == rn.group_id
                        && n.instance_id == rn.instance_id
                })
            })
            .collect::<Vec<_>>();
        idxs.sort();
        idxs.reverse();
        for idx in idxs {
            self.nodes.remove(idx);
        }
        let mut to_replace = vec![];
        let mut to_add = vec![];
        for node in nodes {
            if let Some(idx) = self.nodes.iter().position(|n| {
                n.pipeline == node.pipeline
                    && n.group == node.group
                    && n.instance_id == node.instance_id
            }) {
                to_replace.push((node, idx));
            } else {
                to_add.push(node);
            }
        }
        for (node, idx) in to_replace {
            *self.nodes.get_mut(idx).unwrap() = node;
        }
        for node in to_add {
            self.nodes.push(node);
        }
        self.nodes.sort_by(
            |lhs, rhs| match lhs.elevation.0.total_cmp(&rhs.elevation.0) {
                Ordering::Less => Ordering::Greater,
                Ordering::Equal => match lhs.pipeline.cmp(&rhs.pipeline) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => match lhs.group.cmp(&rhs.group) {
                        Ordering::Less => Ordering::Less,
                        Ordering::Equal => {
                            match lhs.clip_context.partial_cmp(&rhs.clip_context).unwrap() {
                                Ordering::Less => Ordering::Less,
                                Ordering::Equal => lhs.order.cmp(&rhs.order),
                                Ordering::Greater => Ordering::Greater,
                            }
                        }
                        Ordering::Greater => Ordering::Greater,
                    },
                    Ordering::Greater => Ordering::Greater,
                },
                Ordering::Greater => Ordering::Less,
            },
        );
        self.contiguous.clear();
        let mut contiguous = 1;
        let mut range_start = None;
        for (index, node) in self.nodes.iter().enumerate() {
            let next = self.nodes.get(index + 1).copied();
            if let Some(next) = next {
                if node.pipeline == next.pipeline
                    && node.group == next.group
                    && node.order + 1 == next.order
                    && node.clip_context == next.clip_context
                {
                    contiguous += 1;
                    if range_start.is_none() {
                        range_start = Some(node.order);
                    }
                    continue;
                }
            }
            let start = range_start.take().unwrap_or(node.order);
            self.contiguous.push(ContiguousSpan {
                pipeline: node.pipeline,
                group: node.group,
                range: start..start + contiguous,
                clip_context: node.clip_context,
            });
            contiguous = 1;
        }
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
            color_attachments: &ginkgo.color_attachment(&view, Color::gray(950)),
            depth_stencil_attachment: ginkgo.depth_stencil_attachment(),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        for span in self.contiguous.iter() {
            let section = ginkgo.viewport().section();
            let clip = self
                .clip
                .get(&span.clip_context)
                .copied()
                .unwrap_or_default();
            let parameters = span.parameters(section, ginkgo.configuration().scale_factor, clip);
            rpass.set_scissor_rect(
                section.left() as u32,
                section.top() as u32,
                section.width() as u32,
                section.height() as u32,
            );
            match span.pipeline {
                PipelineId::Text => {
                    Render::render(self.text.as_mut().unwrap(), &mut rpass, parameters);
                }
                PipelineId::Icon => {
                    Render::render(self.icon.as_mut().unwrap(), &mut rpass, parameters);
                }
                PipelineId::Shape => {
                    Render::render(self.shape.as_mut().unwrap(), &mut rpass, parameters);
                }
                PipelineId::Panel => {
                    Render::render(self.panel.as_mut().unwrap(), &mut rpass, parameters);
                }
                PipelineId::Image => {
                    Render::render(self.image.as_mut().unwrap(), &mut rpass, parameters);
                }
            }
        }
        drop(rpass);
        ginkgo
            .context()
            .queue
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}
