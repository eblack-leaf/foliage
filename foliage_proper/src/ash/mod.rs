use crate::ash::clip::prepare_clip_section;
use crate::ash::differential::RenderQueueHandle;
use crate::foliage::{DiffMarkers, Foliage};
use crate::ginkgo::Ginkgo;
use crate::{Attachment, Color, Panel, Text};
use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::world::World;
use node::Node;
use render::{ContiguousSpan, PipelineId, Render, Renderer};
use std::cmp::Ordering;
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
            .add_systems(prepare_clip_section.in_set(DiffMarkers::Finalize));
    }
}
pub(crate) struct Ash {
    pub(crate) drawn: bool,
    pub(crate) nodes: Vec<Node>,
    pub(crate) contiguous: Vec<ContiguousSpan>,
    pub(crate) text: Option<Renderer<Text>>,
    pub(crate) panel: Option<Renderer<Panel>>,
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
        }
    }
    pub(crate) fn initialize(&mut self, ginkgo: &Ginkgo) {
        self.text.replace(Text::renderer(ginkgo));
        self.panel.replace(Panel::renderer(ginkgo));
        // TODO other renderers
    }
    pub(crate) fn prepare(&mut self, world: &mut World, ginkgo: &Ginkgo) {
        let mut queues = RenderQueueHandle::new(world);
        let mut nodes = vec![];
        let mut to_remove = vec![];
        let text_nodes = Render::prepare(self.text.as_mut().unwrap(), &mut queues, ginkgo);
        nodes.extend(text_nodes.updated);
        to_remove.extend(text_nodes.removed);
        let panel_nodes = Render::prepare(self.panel.as_mut().unwrap(), &mut queues, ginkgo);
        nodes.extend(panel_nodes.updated);
        to_remove.extend(panel_nodes.removed);
        // TODO extend other renderers
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
        self.nodes.sort_by(|lhs, rhs| {
            if lhs.elevation < rhs.elevation {
                Ordering::Less
            } else if lhs.elevation > rhs.elevation {
                Ordering::Greater
            } else if lhs.pipeline != rhs.pipeline {
                Ordering::Less
            } else if lhs.group != rhs.group {
                Ordering::Less
            } else if lhs.clip_section != rhs.clip_section {
                Ordering::Less
            } else if lhs.order < rhs.order {
                Ordering::Less
            } else if lhs.order > rhs.order {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        self.contiguous.clear();
        let mut contiguous = 1;
        let mut range_start = None;
        for (index, node) in self.nodes.iter().enumerate() {
            let next = self.nodes.get(index + 1).copied();
            if let Some(next) = next {
                if node.pipeline == next.pipeline
                    && node.group == next.group
                    && node.order + 1 == next.order
                    && node.clip_section == next.clip_section
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
                clip_section: node.clip_section,
            });
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
            let parameters = span.parameters(section, ginkgo.configuration().scale_factor);
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
                PipelineId::Icon => {}
                PipelineId::Shape => {}
                PipelineId::Panel => {
                    Render::render(self.panel.as_mut().unwrap(), &mut rpass, parameters);
                }
                PipelineId::Image => {}
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
