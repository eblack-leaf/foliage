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
/// Thinnest line this draws: one logical pixel, the thinnest that means anything.
///
/// This was 3 for a long time, and the reason was real -- at 1 or 2 a line visibly thinned,
/// thickened and shimmered along its own length, worst at shallow angles. That was two
/// things, both since fixed: the drawn quad was exactly the line's true width, so with no
/// multisampling (`Ginkgo` runs one sample) the rasterizer simply skipped the pixels whose
/// centers the thin quad missed and the shader never got to feather them -- see `line.wgsl`'s
/// `AA_MARGIN`; and an axis-aligned line's edges were never put on whole pixels the way every
/// other primitive's are -- see [`snap_axis_aligned`](Line::snap_axis_aligned).
///
/// A clamp rather than a documented caveat, still: zero and negative weights have no drawing
/// to do, and clamping is cheaper than every caller checking.
pub const MIN_LINE_WEIGHT: i32 = 1;

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
            let sf = scale_factor.value();
            let mut left = EdgePoints::new(
                left_bottom.to_physical(sf).coordinates,
                left_top.to_physical(sf).coordinates,
            );
            let mut right = EdgePoints::new(
                right_bottom.to_physical(sf).coordinates,
                right_top.to_physical(sf).coordinates,
            );
            if x_diff == 0.0 || y_diff == 0.0 {
                Self::snap_axis_aligned(&mut left, &mut right, y_diff == 0.0);
            }
            *quad = LineQuad::new(left, right);
        }
    }
    /// Puts an axis-aligned segment's four edges on whole device pixels -- the same
    /// crisp-alignment pass every other pipeline runs on its `Section` (`panel/pipeline.rs`'s
    /// `.rounded()`), which this one had never had.
    ///
    /// Only axis-aligned. At any other angle a line has no edge that *can* be snapped: moving
    /// its corners to whole pixels changes both its width and its angle, and the shader's
    /// feather is what makes those read cleanly instead. Axis-aligned is where snapping is
    /// both meaningful and most missed -- a rule or a divider whose centerline lands on a
    /// pixel boundary is split evenly across two rows, so a 1px rule renders as two half-lit
    /// ones: right ink, twice the width, and dimmer than it was asked to be.
    fn snap_axis_aligned(left: &mut EdgePoints, right: &mut EdgePoints, horizontal: bool) {
        if horizontal {
            // `left`/`right` are the two *ends* (the caps), each spanning the weight; which
            // end is which follows the authored direction, so the ends keep their own x.
            let top = left.end.b().min(left.start.b());
            let thickness = (left.start.b() - left.end.b()).abs().round().max(1.0);
            let snapped_top = top.round();
            let snapped_bottom = snapped_top + thickness;
            let (x0, x1) = (left.start.a().round(), right.start.a().round());
            *left = EdgePoints::new((x0, snapped_bottom).into(), (x0, snapped_top).into());
            *right = EdgePoints::new((x1, snapped_bottom).into(), (x1, snapped_top).into());
        } else {
            let near = left.start.a().min(left.end.a());
            let thickness = (left.end.a() - left.start.a()).abs().round().max(1.0);
            let snapped_near = near.round();
            let snapped_far = snapped_near + thickness;
            let (y0, y1) = (left.start.b().round(), right.start.b().round());
            *left = EdgePoints::new((snapped_near, y0).into(), (snapped_far, y0).into());
            *right = EdgePoints::new((snapped_near, y1).into(), (snapped_far, y1).into());
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
