use crate::AsTree;
use crate::Differential;
use crate::ash::clip::ClipContext;
use crate::coordinate::points::Points;
use crate::foliage::DiffMarkers;
use crate::ginkgo::ScaleFactor;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::{
    Attachment, Color, Component, Coordinates, Foliage, Author, LeafSprout, Logical, Position,
    ResolvedElevation, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::{Changed, IntoScheduleConfigs, Or, Query};
use bevy_ecs::world::DeferredWorld;
use bytemuck::{Pod, Zeroable};

mod pipeline;
#[derive(Component, Copy, Clone)]
#[require(LineQuad)]
/// A straight segment of fixed weight between two points.
///
/// Positioned in point mode rather than as a box: its `Location` uses
/// [`as_x`](crate::ValueDescriptor)/`as_y` pairs, and its `Section` is the bounding
/// rectangle around the result. Animating the `Location` moves the endpoints, which is
/// how a line draws itself in.
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
        foliage.differential::<LineQuad, ClipContext>();
        foliage.differential::<LineQuad, Color>();
    }
}
/// Thinnest line this renders cleanly. Below it, `distill_descriptor`'s half-weight normal
/// offset falls under a texel and the quad's two long edges land on the same pixel row at
/// some angles and straddle two at others -- so a line visibly thins, thickens and shimmers
/// along its own length, worst at the shallow angles where it is most noticeable.
///
/// Clamped rather than documented as a caveat: a thinner line is never what someone wanted,
/// it is what they asked for before seeing it, and leaving the floor at 1 only moves the
/// discovery to whoever draws the first diagonal. Same reasoning and same value as
/// [`PolylineSprout::weight`](crate::PolylineSprout::weight), which needs the floor for its
/// joints as well.
pub const MIN_LINE_WEIGHT: i32 = 3;

impl Line {
    /// Starts a [`Line`] entity `w` logical pixels thick, clamped to
    /// [`MIN_LINE_WEIGHT`].
    pub fn new(w: i32) -> LineSprout {
        LineSprout {
            leaf: LeafSprout::default(),
            weight: w.max(MIN_LINE_WEIGHT),
            color: None,
        }
    }
    pub(crate) fn new_marker(w: i32) -> Self {
        Self {
            weight: w.max(MIN_LINE_WEIGHT),
        }
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
/// Builder for a [`Line`] entity -- see [`Line::new`].
pub struct LineSprout {
    leaf: LeafSprout,
    weight: i32,
    color: Option<Color>,
}
impl Author for LineSprout {
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
    /// Stroke color.
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
#[require(Differential<LineQuad, ClipContext>)]
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
        let mut tree = world.tree();
        tree.subscribe(this, Remove::push_remove_packet::<Self>);
        tree.subscribe(this, Visibility::push_remove_packet::<Self>);
    }
}
