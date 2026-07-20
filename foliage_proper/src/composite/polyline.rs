use crate::{
    Color, Component, EcsExtension, Elevation, Entity, Grid, GridExt, LeafSprout, Line, Location,
    Logical, Polygon, Position, Sprout, Tree,
};
use crate::Trigger;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// A polyline is one entity: [`PolylinePoints`]/[`PolylineStyle`] in, a chain of [`Line`]
/// segments + round [`Polygon`] joints out. Built entirely from the two existing primitives
/// -- no render pipeline of its own -- since a `Line` segment already draws an antialiased
/// quad and a full-`rounding` `Polygon` already draws a true circle regardless of side
/// count. Points are authored in the polyline's own local px space (its resolved box's
/// top-left is `(0, 0)`), same as any other point-mode `Line`.
///
/// Joints exist because `Line`'s end caps are square (independent quads per segment), so
/// consecutive segments meeting at an angle leave a wedge-shaped gap on the outer side of
/// the turn; a circle of diameter == weight centered on the shared vertex exactly covers
/// it, the standard round-join technique (each segment's near corners sit exactly `weight /
/// 2` from the vertex, so the circle's radius reaches precisely to them). This holds
/// whether or not the line is dashed, so joints aren't a plain-line-only option -- see
/// [`dashed_segments`] for how dashing keeps them.
#[derive(Component, Copy, Clone)]
pub struct Polyline {}
impl Polyline {
    pub fn new() -> PolylineSprout {
        PolylineSprout::default()
    }
}

/// Polyline's content: the vertex chain, rewritten as one unit like `ListItems` --
/// `tree.write_to(polyline, PolylinePoints(vec![..]))` re-derives every segment and joint.
#[derive(Component, Clone, Default)]
pub struct PolylinePoints(pub Vec<Position<Logical>>);

/// Polyline's OWN appearance vocabulary, poked as one unit:
/// `tree.write_to(polyline, PolylineStyle { .. })`.
#[derive(Component, Copy, Clone)]
pub struct PolylineStyle {
    pub weight: i32,
    pub color: Color,
    pub dash: Option<DashPattern>,
}
impl Default for PolylineStyle {
    fn default() -> Self {
        Self {
            weight: 3,
            color: Color::default(),
            dash: None,
        }
    }
}

/// A repeating on/off run length along the path, in px, continuous across vertices (the
/// pattern doesn't reset at a bend) -- see [`dashed_segments`].
#[derive(Copy, Clone)]
pub struct DashPattern {
    pub(crate) on: f32,
    pub(crate) off: f32,
}
impl DashPattern {
    /// `on`/`off` clamped to a sane minimum (`on` especially: zero would make the pattern's
    /// cycle length zero, and `traveled % cycle` NaNs).
    pub fn new(on: f32, off: f32) -> Self {
        Self {
            on: on.max(1.0),
            off: off.max(0.0),
        }
    }
}

pub struct PolylineSprout {
    leaf: LeafSprout,
    points: Option<PolylinePoints>,
    style: PolylineStyle,
}
impl Default for PolylineSprout {
    fn default() -> Self {
        Self {
            leaf: LeafSprout::default(),
            points: None,
            style: PolylineStyle::default(),
        }
    }
}
impl PolylineSprout {
    /// Vertex chain, in the polyline's own local px space. Required -- same contract as
    /// `List::items`.
    pub fn points<P: Into<Position<Logical>>>(mut self, points: Vec<P>) -> Self {
        self.points = Some(PolylinePoints(points.into_iter().map(Into::into).collect()));
        self
    }
    /// Clamped to >= 3: below that, the joint's rounded-to-a-whole-px diameter (see
    /// `PolylineSprout::build`) can't track `Line`'s perceived width closely enough to look
    /// clean at a bend -- same reasoning as `Polygon::sides`' own floor.
    pub fn weight(mut self, w: i32) -> Self {
        self.style.weight = w.max(3);
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.style.color = c;
        self
    }
    pub fn dash(mut self, d: DashPattern) -> Self {
        self.style.dash = Some(d);
        self
    }
}
impl Sprout for PolylineSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Polyline {},
            self.points.expect("Polyline::points(..) is required"),
            self.style,
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // no static skeleton: segment/joint count is data-dependent, so it all lands via
        // the reaction's first fire -- the List/Dropdown teardown-and-rebuild pattern.
        let mut children: Vec<Entity> = Vec::new();
        tree.react_any::<(PolylinePoints, PolylineStyle), _>(
            this,
            move |trigger: Trigger<Insert, (PolylinePoints, PolylineStyle)>,
                  points_q: Query<&PolylinePoints>,
                  styles: Query<&PolylineStyle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let points = points_q.get(e).unwrap().0.clone();
                let style = *styles.get(e).unwrap();
                for child in children.drain(..) {
                    tree.remove(child);
                }
                if points.len() < 2 {
                    return;
                }
                let (segments, joints) = match style.dash {
                    None => straight_segments(&points),
                    Some(dash) => dashed_segments(&points, dash),
                };
                for (a, b) in segments {
                    let child = tree.branch(
                        e,
                        Line::new(style.weight).color(style.color).at(Location::new().xs(
                            a.left().px().as_x().with(a.top().px().as_y()),
                            b.left().px().as_x().with(b.top().px().as_y()),
                        )).elevate(Elevation::up(1)),
                    );
                    children.push(child);
                }
                // `Line`'s AA fades inward only, from its true edge (0% coverage right at
                // the geometric boundary, ramping to 100% a full `edge_precision` px in --
                // see `line.wgsl`), while `Polygon`'s fades symmetrically, centered on its
                // true boundary -- so a segment's *perceived* edge sits ~half a px inside
                // its true one, but the joint's perceived edge sits exactly on its true one.
                // Shrink the joint to the segment's perceived width (mirrors
                // `edge_precision`'s own `min(1.0, weight / 2.0)` cap) so the two line up.
                // Rounded to a whole px: every renderer's `prepare()` step (this crate's
                // universal, deliberate crisp-pixel-alignment pass, e.g.
                // `polygon/pipeline.rs`) snaps `Section<Logical>` position and area to the
                // nearest physical pixel *independently*, so a `left` derived from a
                // fractional center/width pair can round to a different sub-pixel offset
                // than its width did, visibly shifting the shape off-center. A whole-px
                // diameter sidesteps that entirely, at the cost of `weight == 1` landing
                // back on a plain (slightly-oversized rather than perceived-perfect) circle.
                let diameter =
                    (style.weight as f32 - (style.weight as f32 / 2.0).min(1.0)).round();
                for j in joints {
                    let child = tree.branch(
                        e,
                        Polygon::new().rounding(1.0).color(style.color).at(Location::new().xs(
                            j.left().px().as_center_x().with(diameter.px().as_width()),
                            j.top().px().as_center_y().with(diameter.px().as_height()),
                        )).elevate(Elevation::up(2)),
                    );
                    children.push(child);
                }
            },
        );
    }
}

fn lerp(a: Position<Logical>, b: Position<Logical>, t: f32) -> Position<Logical> {
    Position::logical((
        a.left() + (b.left() - a.left()) * t,
        a.top() + (b.top() - a.top()) * t,
    ))
}

/// One `Line` per edge, one joint `Polygon` per interior vertex -- always, since every
/// vertex is where two segments actually meet.
fn straight_segments(
    points: &[Position<Logical>],
) -> (Vec<(Position<Logical>, Position<Logical>)>, Vec<Position<Logical>>) {
    let segments = points.windows(2).map(|w| (w[0], w[1])).collect();
    let joints = points[1..points.len() - 1].to_vec();
    (segments, joints)
}

/// Walks the path's arc length with the dash pattern's phase carried continuously across
/// vertices (so a dash never resets mid-turn), emitting one `Line` per "on" run -- clipped
/// at each vertex, so a run spanning a bend becomes two `Line`s following the actual path
/// rather than one straight quad cutting the corner. A joint only goes at a vertex the
/// walk's phase is still "on" when it reaches -- i.e. where two emitted `Line`s actually
/// meet at that point. A vertex the phase lands on during "off" gets no joint: nothing is
/// drawn there to seam.
fn dashed_segments(
    points: &[Position<Logical>],
    dash: DashPattern,
) -> (Vec<(Position<Logical>, Position<Logical>)>, Vec<Position<Logical>>) {
    let cycle = dash.on + dash.off;
    let mut segments = Vec::new();
    let mut joints = Vec::new();
    let mut traveled = 0.0f32;
    let last = points.len() - 1;
    for (i, pair) in points.windows(2).enumerate() {
        let (a, b) = (pair[0], pair[1]);
        let len = a.distance(b);
        let mut local = 0.0f32;
        while local < len {
            let phase = traveled % cycle;
            if phase < dash.on {
                let step = (dash.on - phase).min(len - local);
                segments.push((lerp(a, b, local / len), lerp(a, b, (local + step) / len)));
                local += step;
                traveled += step;
            } else {
                let step = (cycle - phase).min(len - local);
                local += step;
                traveled += step;
            }
        }
        let vertex_index = i + 1;
        if vertex_index < last && traveled % cycle < dash.on {
            joints.push(b);
        }
    }
    (segments, joints)
}
