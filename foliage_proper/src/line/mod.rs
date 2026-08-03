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
            // The endpoints are snapped to device pixels; the perpendicular offsets that give the
            // quad its weight are not. Rounding the four corners instead would quantize those
            // offsets, and they are fractional for any line that is not axis-aligned -- so a
            // diagonal would come out with its weight and its angle both altered by up to half a
            // pixel per corner. Snapping the ends is what puts the line on a pixel boundary
            // rather than straddling two and rasterizing soft.
            let sf = scale_factor.value();
            let start = Position::logical(pts.0).to_physical(sf).rounded();
            let end = Position::logical(pts.1).to_physical(sf).rounded();
            // Measured between the *snapped* ends, because that is where the line is once it has
            // been snapped, and the perpendicular exists to give that axis its thickness. Taken
            // from the unrounded points instead, the perpendicular belongs to a slightly
            // different angle: both end faces stay parallel to it while the axis has moved, so
            // the quad is a parallelogram rather than a rectangle -- caps slanted, and a true
            // width of `weight * cos` of the disagreement. On a long segment the two angles are
            // indistinguishable; on a short one they are not, which is exactly where the skew
            // would show.
            let mut x_diff = end.left() - start.left();
            let mut y_diff = end.top() - start.top();
            if x_diff == 0.0 && y_diff == 0.0 {
                // Snapping can collapse a segment shorter than a device pixel onto a single
                // point, and `0.0 / 0.0` would take the angle -- and all four corners -- to NaN.
                // The unrounded direction still says which way it was pointing.
                x_diff = pts.1.a() - pts.0.a();
                y_diff = pts.1.b() - pts.0.b();
            }
            let slope = y_diff / x_diff;
            let normal_slope = 1.0 / slope;
            let angle = normal_slope.atan();
            let half_weight = line.weight as f32 / 2.0 * sf;
            let x_adjust = angle.cos() * half_weight;
            let y_adjust = angle.sin() * half_weight;
            let left_top = Position::physical((start.left() + x_adjust, start.top() - y_adjust));
            let left_bottom = Position::physical((start.left() - x_adjust, start.top() + y_adjust));
            let right_top = Position::physical((end.left() + x_adjust, end.top() - y_adjust));
            let right_bottom = Position::physical((end.left() - x_adjust, end.top() + y_adjust));
            *quad = LineQuad::new(
                EdgePoints::new(left_bottom.coordinates, left_top.coordinates),
                EdgePoints::new(right_bottom.coordinates, right_top.coordinates),
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
