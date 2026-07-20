use crate::Differential;
use crate::Stem;
use crate::coordinate::points::Points;
use crate::foliage::DiffMarkers;
use crate::ginkgo::ScaleFactor;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::{
    Attachment, Color, Component, Coordinates, Foliage, LeafSprout, Logical, Position,
    ResolvedElevation, Sprout, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::{Changed, IntoScheduleConfigs, Or, Query};
use bevy_ecs::world::DeferredWorld;
use bytemuck::{Pod, Zeroable};

mod pipeline;
#[derive(Component, Copy, Clone)]
#[require(LineQuad)]
pub struct Line {
    pub weight: i32,
}
impl Attachment for Line {
    fn attach(foliage: &mut Foliage) {
        foliage
            .diff
            .add_systems(Line::distill_descriptor.in_set(DiffMarkers::Finalize));
        foliage.remove_queue::<LineQuad>();
        foliage.differential::<LineQuad, LineQuad>();
        foliage.differential::<LineQuad, BlendedOpacity>();
        foliage.differential::<LineQuad, ResolvedElevation>();
        foliage.differential::<LineQuad, Stem>();
        foliage.differential::<LineQuad, Color>();
    }
}
impl Line {
    pub fn new(w: i32) -> LineSprout {
        LineSprout {
            leaf: LeafSprout::default(),
            weight: w.max(1),
            color: None,
        }
    }
    pub(crate) fn new_marker(w: i32) -> Self {
        Self { weight: w.max(1) }
    }
    pub(crate) fn distill_descriptor(
        mut lines: Query<
            (&Points<Logical>, &Line, &mut LineQuad),
            Or<(Changed<Line>, Changed<Points<Logical>>)>,
        >,
        scale_factor: Res<ScaleFactor>,
    ) {
        for (points, line, mut quad) in lines.iter_mut() {
            let pts = (points.data[0].coordinates, points.data[1].coordinates);
            let x_diff = pts.1.a() - pts.0.a();
            let y_diff = pts.1.b() - pts.0.b();
            let slope = y_diff / x_diff;
            let normal_slope = 1.0 / slope;
            let angle = normal_slope.atan();
            let half_weight = line.weight as f32 / 2.0;
            let x_adjust = angle.cos() * half_weight;
            let y_adjust = angle.sin() * half_weight;
            let left_top = Position::logical((pts.0.a() + x_adjust, pts.0.b() - y_adjust));
            let left_bottom = Position::logical((pts.0.a() - x_adjust, pts.0.b() + y_adjust));
            let right_top = Position::logical((pts.1.a() + x_adjust, pts.1.b() - y_adjust));
            let right_bottom = Position::logical((pts.1.a() - x_adjust, pts.1.b() + y_adjust));
            *quad = LineQuad::new(
                EdgePoints::new(
                    left_bottom.to_physical(scale_factor.value()).coordinates,
                    left_top.to_physical(scale_factor.value()).coordinates,
                ),
                EdgePoints::new(
                    right_bottom.to_physical(scale_factor.value()).coordinates,
                    right_top.to_physical(scale_factor.value()).coordinates,
                ),
            );
        }
    }
}
pub struct LineSprout {
    leaf: LeafSprout,
    weight: i32,
    color: Option<Color>,
}
impl Sprout for LineSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Line::new_marker(self.weight),
            self.color.unwrap_or_default(),
        )
    }
}
impl LineSprout {
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct EdgePoints {
    pub(crate) start: Coordinates,
    pub(crate) end: Coordinates,
}
impl EdgePoints {
    pub(crate) fn new(start: Coordinates, end: Coordinates) -> Self {
        Self { start, end }
    }
}
/// The quad `Line` draws -- not a general shape primitive, just `Line`'s own render
/// payload split out so it can carry `#[repr(C)]`/`Pod` without dragging `weight` (a
/// CPU-only authoring value, never uploaded) along into the GPU buffer. `pub(crate)` and
/// unexported: only `Line::distill_descriptor` ever constructs or touches one, so a
/// self-intersecting/degenerate quad can only happen via a real bug in that one system,
/// never from outside code reaching in through `Query<&mut LineQuad>` and hand-editing
/// corners. There's deliberately no path to author one directly -- this framework doesn't
/// support arbitrary/tessellated shapes, only the rectangle `Line` itself needs.
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, Default, PartialEq)]
#[require(Differential<LineQuad, LineQuad>)]
#[require(Differential<LineQuad, Stem>)]
#[require(Color, Differential<LineQuad, Color>)]
#[require(Differential<LineQuad, ResolvedElevation>)]
#[require(Differential<LineQuad, BlendedOpacity>)]
#[require(Points<Logical>)]
#[component(on_add = Self::on_add)]
pub(crate) struct LineQuad {
    pub(crate) left: EdgePoints,
    pub(crate) right: EdgePoints,
}
impl LineQuad {
    pub(crate) fn new(left: EdgePoints, right: EdgePoints) -> Self {
        Self { left, right }
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Remove::push_remove_packet::<Self>)
            .observe(Visibility::push_remove_packet::<Self>);
    }
}
