use crate::ash::clip::{ClipSection, ResolvedClip};
use crate::ash::differential::RenderQueueHandle;
use crate::coordinate::elevation::StackKey;
use crate::foliage::Foliage;
use crate::ginkgo::viewport::ViewportHandle;
use crate::ginkgo::{Ginkgo, ScaleFactor};
use crate::image::Image;
use crate::line::LineQuad;
use crate::polygon::Polygon;
use crate::willow::NearFarDescriptor;
use crate::{Attachment, Color, Icon, Panel, ResolvedElevation, Stem, Text};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::IntoScheduleConfigs;
use bevy_ecs::world::World;
use node::Node;
use render::{ContiguousSpan, PipelineId, Render, Renderer};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use wgpu::{CommandEncoderDescriptor, RenderPassDescriptor, TextureViewDescriptor};

pub(crate) mod clip;
pub(crate) mod differential;
pub(crate) mod instance;
pub(crate) mod node;
pub(crate) mod render;

impl Attachment for Ash {
    fn attach(foliage: &mut Foliage) {
        foliage.differential::<(), ResolvedClip>();
        foliage.define(ClipSection::write_section);
        foliage.define(ClipSection::stem_insert);
        foliage.define(ClipSection::update_inherited);
        foliage.world.insert_resource(ViewportHandle::default());
        foliage.world.insert_resource(ScaleFactor::default());
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
    pub(crate) line: Option<Renderer<LineQuad>>,
    pub(crate) polygon: Option<Renderer<Polygon>>,
    pub(crate) clip: HashMap<Stem, ClipSection>,
    /// entities with a `StackKey`, sorted least-in-front-first by `(StackKey, entity id)`
    /// (`StackKey`'s own `Ord`: greater = more in front; the entity-id tiebreak gives genuine
    /// `StackKey` ties a stable, deterministic order). Maintained incrementally by
    /// `assign_elevations` -- a gapped/fractional-index scheme, so a changed or newly-inserted
    /// entity only ever touches its own `ResolvedElevation` plus its two immediate neighbors',
    /// never the whole list (except the rare renormalize fallback).
    pub(crate) elevation_order: Vec<Entity>,
    /// last-seen `StackKey` per entity, to detect which entities actually need repositioning
    /// this frame without re-deriving everyone's position from scratch.
    pub(crate) stack_key_cache: HashMap<Entity, StackKey>,
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
            line: None,
            polygon: None,
            clip: Default::default(),
            elevation_order: vec![],
            stack_key_cache: HashMap::new(),
        }
    }
    pub(crate) fn initialize(&mut self, ginkgo: &Ginkgo) {
        self.text.replace(Text::renderer(ginkgo));
        self.panel.replace(Panel::renderer(ginkgo));
        self.image.replace(Image::renderer(ginkgo));
        self.icon.replace(Icon::renderer(ginkgo));
        self.line.replace(LineQuad::renderer(ginkgo));
        self.polygon.replace(Polygon::renderer(ginkgo));
    }
    /// Keeps `ResolvedElevation` (the GPU-facing f32 every pipeline already consumes,
    /// unchanged) consistent with `StackKey`'s tree-structured ordering, via a gapped/
    /// fractional-index scheme: a changed or newly-inserted entity is placed between its new
    /// `StackKey`-sorted neighbors' *already-assigned* values, touching only itself (plus,
    /// rarely, a full renormalize if a gap's been bisected too many times to keep meaningful
    /// float precision). Must run *before* `RenderQueueHandle::new(world)` below -- the six
    /// pipelines' own `Render::prepare` calls drain their `ResolvedElevation` differential
    /// through `queues`, which holds `world` mutably for the rest of this function.
    fn assign_elevations(&mut self, world: &mut World) {
        const EPSILON: f32 = 1e-4;
        let nf = NearFarDescriptor::default();
        let initial_gap = (nf.far.value() - nf.near.value()) / 64.0;

        let current: Vec<(Entity, StackKey)> = {
            let mut q = world.query::<(Entity, &StackKey)>();
            q.iter(world).map(|(e, k)| (e, *k)).collect()
        };
        let alive: HashSet<Entity> = current.iter().map(|(e, _)| *e).collect();
        self.elevation_order.retain(|e| alive.contains(e));
        self.stack_key_cache.retain(|e, _| alive.contains(e));

        let dirty: Vec<(Entity, StackKey)> = current
            .into_iter()
            .filter(|(e, k)| self.stack_key_cache.get(e) != Some(k))
            .collect();
        if dirty.is_empty() {
            return;
        }
        for (e, _) in &dirty {
            if let Some(pos) = self.elevation_order.iter().position(|x| x == e) {
                self.elevation_order.remove(pos);
            }
        }
        for (e, k) in &dirty {
            self.stack_key_cache.insert(*e, *k);
        }
        for (e, k) in dirty {
            // Order by `StackKey`, then break genuine `StackKey` ties by entity id -- a stable,
            // total order, so two entities the author gave the *same* elevation (no defined
            // relative order) always resolve the same way run-to-run instead of flickering by
            // whatever draw order the GPU happened to pick.
            let pos = self
                .elevation_order
                .binary_search_by(|probe| {
                    (*world.get::<StackKey>(*probe).unwrap())
                        .cmp(&k)
                        .then_with(|| probe.cmp(&e))
                })
                .unwrap_or_else(|i| i);
            self.elevation_order.insert(pos, e);
            let left = pos.checked_sub(1).and_then(|i| self.elevation_order.get(i)).copied();
            let right = self.elevation_order.get(pos + 1).copied();
            let left_v = left.and_then(|le| world.get::<ResolvedElevation>(le)).map(|r| r.value());
            let right_v = right.and_then(|re| world.get::<ResolvedElevation>(re)).map(|r| r.value());
            // `elevation_order` runs least-in-front-first: `left` (lower index, lower
            // StackKey) must end up with a *larger* raw value than `right` (higher index,
            // higher StackKey), matching the existing "smaller raw = more in front" convention.
            let new_value = match (left_v, right_v) {
                (Some(l), Some(r)) => (l + r) / 2.0,
                (Some(l), None) => l - initial_gap,
                (None, Some(r)) => r + initial_gap,
                (None, None) => (nf.near.value() + nf.far.value()) / 2.0,
            };
            let too_close = left_v.map(|l| (l - new_value).abs() < EPSILON).unwrap_or(false)
                || right_v.map(|r| (r - new_value).abs() < EPSILON).unwrap_or(false);
            // Boundary case: a front/back insertion that runs off the end of `[near, far]` has
            // no room to sit *distinctly* -- clamping it would land it on `near` (or `far`)
            // exactly alongside anything else already pinned there, an equal depth the GPU
            // resolves by draw order (nondeterministic). This is the crowded-front-tier case
            // (overlay panel + its text both pushing toward `near`) that made the popover text
            // "sometimes" vanish behind its panel. Re-space everyone instead, which restores
            // room and keeps every value strictly distinct.
            let hits_boundary =
                new_value <= nf.near.value() || new_value >= nf.far.value();
            if too_close || hits_boundary {
                self.renormalize(world, &nf);
            } else {
                world.entity_mut(e).insert(ResolvedElevation::new(new_value));
            }
        }
    }
    /// Rare fallback: a gap between two `StackKey`-adjacent entities has been bisected so many
    /// times the midpoint would lose meaningful float precision. Re-spaces the *whole* order
    /// evenly across `[near, far]` -- correctness-preserving, not performance-sensitive, since
    /// it only triggers after repeated insertions into the same tiny gap, not on ordinary
    /// changes (which stay local to the changed entity's own two neighbors).
    fn renormalize(&mut self, world: &mut World, nf: &NearFarDescriptor) {
        let n = self.elevation_order.len();
        if n == 0 {
            return;
        }
        let span = nf.far.value() - nf.near.value();
        for (i, e) in self.elevation_order.iter().enumerate() {
            let t = if n == 1 { 0.5 } else { i as f32 / (n - 1) as f32 };
            let value = nf.far.value() - t * span;
            world.entity_mut(*e).insert(ResolvedElevation::new(value));
        }
    }
    pub(crate) fn prepare(&mut self, world: &mut World, ginkgo: &Ginkgo) {
        self.assign_elevations(world);
        let mut queues = RenderQueueHandle::new(world);
        for (entity, clip) in queues.attribute::<(), ResolvedClip>() {
            self.clip.insert(Stem::some(entity), ClipSection(clip.0));
        }
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
        let line_nodes = Render::prepare(self.line.as_mut().unwrap(), &mut queues, ginkgo);
        nodes.extend(line_nodes.updated);
        to_remove.extend(line_nodes.removed);
        let polygon_nodes = Render::prepare(self.polygon.as_mut().unwrap(), &mut queues, ginkgo);
        nodes.extend(polygon_nodes.updated);
        to_remove.extend(polygon_nodes.removed);
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
            lhs.elevation
                .front_to_back(&rhs.elevation)
                .then_with(|| lhs.pipeline.cmp(&rhs.pipeline))
                .then_with(|| lhs.group.cmp(&rhs.group))
                .then_with(|| lhs.clip_context.partial_cmp(&rhs.clip_context).unwrap())
                .then_with(|| lhs.order.cmp(&rhs.order))
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
        if let Some(surface_texture) = ginkgo.surface_texture() {
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
                multiview_mask: None,
                label: Some("render-pass"),
                color_attachments: &ginkgo.color_attachment(&view, Color::gray(900)),
                depth_stencil_attachment: ginkgo.depth_stencil_attachment(),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            for span in self.contiguous.iter() {
                let mut section = ginkgo.viewport().section();
                if let Some(clip) = self.clip.get(&span.clip_context) {
                    section = section
                        .intersection(
                            clip.0
                                .to_physical(ginkgo.configuration().scale_factor.value()),
                        )
                        .unwrap_or_default();
                }
                rpass.set_scissor_rect(
                    section.left() as u32,
                    section.top() as u32,
                    section.width() as u32,
                    section.height() as u32,
                );
                let parameters = span.parameters(section);
                match span.pipeline {
                    PipelineId::Text => {
                        Render::render(self.text.as_mut().unwrap(), &mut rpass, parameters);
                    }
                    PipelineId::Icon => {
                        Render::render(self.icon.as_mut().unwrap(), &mut rpass, parameters);
                    }
                    PipelineId::Line => {
                        Render::render(self.line.as_mut().unwrap(), &mut rpass, parameters);
                    }
                    PipelineId::Panel => {
                        Render::render(self.panel.as_mut().unwrap(), &mut rpass, parameters);
                    }
                    PipelineId::Image => {
                        Render::render(self.image.as_mut().unwrap(), &mut rpass, parameters);
                    }
                    PipelineId::Polygon => {
                        Render::render(self.polygon.as_mut().unwrap(), &mut rpass, parameters);
                    }
                }
            }
            drop(rpass);
            ginkgo
                .context()
                .queue
                .submit(std::iter::once(encoder.finish()));
            ginkgo.context().queue.present(surface_texture);
        }
    }
}

#[cfg(test)]
mod elevation_assignment_tests {
    use super::*;
    use crate::{Elevation, EcsExtension, Foliage, Leaf, Location, Sprout};

    fn resolved_of(world: &mut World, e: Entity) -> f32 {
        world.get::<ResolvedElevation>(e).unwrap().value()
    }

    /// `assign_elevations` doesn't need a real `Ginkgo`/GPU/window -- it's pure ECS logic
    /// over `StackKey`/`ResolvedElevation`, so this exercises it directly rather than the
    /// full render `prepare()` path.
    #[test]
    fn resulting_resolved_elevation_order_agrees_with_stack_key_order() {
        let mut foliage = Foliage::new();
        let root = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
        let a = foliage.world.branch(
            root,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        let b = foliage.world.branch(
            root,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(2)),
        );
        let c = foliage
            .world
            .branch(a, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
        foliage.world.flush();

        let mut ash = Ash::new();
        ash.assign_elevations(&mut foliage.world);

        let (ra, rb, rc) = (
            resolved_of(&mut foliage.world, a),
            resolved_of(&mut foliage.world, b),
            resolved_of(&mut foliage.world, c),
        );
        // b (up(2)) is more in front than a (up(1)) -- smaller raw value.
        assert!(rb < ra, "up(2) sibling should resolve more in front (smaller raw) than up(1)");
        // c (up(1) from a) is more in front than a itself.
        assert!(rc < ra, "a's own child should resolve more in front than a itself");
    }

    /// The actual point of the gapped/fractional-index scheme: inserting a new entity between
    /// two already-assigned ones must touch only the new entity's own `ResolvedElevation`,
    /// never its neighbors' -- not a full re-rank of everyone.
    #[test]
    fn inserting_between_two_existing_entities_touches_only_the_new_entity() {
        let mut foliage = Foliage::new();
        let root = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
        let low = foliage.world.branch(
            root,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        let high = foliage.world.branch(
            root,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(5)),
        );
        foliage.world.flush();

        let mut ash = Ash::new();
        ash.assign_elevations(&mut foliage.world);
        let (low_before, high_before) = (
            resolved_of(&mut foliage.world, low),
            resolved_of(&mut foliage.world, high),
        );

        // a second call with nothing changed must be a total no-op.
        ash.assign_elevations(&mut foliage.world);
        assert_eq!(resolved_of(&mut foliage.world, low), low_before);
        assert_eq!(resolved_of(&mut foliage.world, high), high_before);

        // now insert a new sibling that lands strictly between `low` and `high`.
        let middle = foliage.world.branch(
            root,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(3)),
        );
        foliage.world.flush();
        ash.assign_elevations(&mut foliage.world);

        assert_eq!(
            resolved_of(&mut foliage.world, low),
            low_before,
            "low's own resolved elevation must not change just because a new sibling was inserted"
        );
        assert_eq!(
            resolved_of(&mut foliage.world, high),
            high_before,
            "high's own resolved elevation must not change just because a new sibling was inserted"
        );
        let middle_v = resolved_of(&mut foliage.world, middle);
        assert!(
            middle_v < low_before && middle_v > high_before,
            "middle (up(3)) should land strictly between low (up(1)) and high (up(5)) -- \
             low={low_before}, middle={middle_v}, high={high_before}"
        );
    }

    /// The "sometimes invisible" regression: a crowded front tier (many entities all pushing
    /// toward `near`, like an overlay panel + its text plus other forward chrome) must never
    /// leave two entities on the *same* depth. Equal depth is resolved by draw order, which is
    /// nondeterministic -- which is exactly why the popover text sometimes rendered behind its
    /// own panel. Assert every assigned `ResolvedElevation` is distinct.
    #[test]
    fn a_crowded_front_tier_never_assigns_two_entities_the_same_depth() {
        use crate::ClipToViewport;

        let mut foliage = Foliage::new();
        let root = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::abs(0))
                .with(crate::Grid::default()),
        );
        // a stack of forward chrome (abs pushes toward the front), plus an overlay subtree
        // (front tier) with its own nested content -- all clustered at the front where the
        // boundary-clamp collision used to happen.
        for e in [95, 90, 85, 80] {
            foliage
                .world
                .branch(root, Leaf::sprout().at(Location::new()).elevate(Elevation::abs(e)));
        }
        let overlay = foliage.world.branch(
            root,
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(2))
                .with((ClipToViewport, crate::Grid::default())),
        );
        for _ in 0..6 {
            foliage
                .world
                .branch(overlay, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
        }
        foliage.world.flush();

        let mut ash = Ash::new();
        ash.assign_elevations(&mut foliage.world);

        let mut values: Vec<f32> = {
            let mut q = foliage.world.query::<&ResolvedElevation>();
            q.iter(&foliage.world).map(|r| r.value()).collect()
        };
        values.sort_by(|a, b| a.total_cmp(b));
        for pair in values.windows(2) {
            assert_ne!(
                pair[0], pair[1],
                "two entities were assigned the identical depth {} -- a nondeterministic \
                 draw-order tie, all assigned values: {values:?}",
                pair[0]
            );
        }
    }
}
