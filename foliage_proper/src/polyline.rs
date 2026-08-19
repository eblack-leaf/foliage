use crate::Sprout;
use crate::Trigger;
use crate::{
    Author, Color, Component, Elevation, Entity, Grid, GridExt, LeafSprout, Line, Location,
    Logical, Opacity, Polygon, Position, Tree,
};
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
/// `dashed_segments` (engine-internal) for how dashing keeps them.
#[derive(Component, Copy, Clone)]
pub struct Polyline {}
impl Polyline {
    pub fn new() -> PolylineSprout {
        PolylineSprout::default()
    }
}

/// Polyline's content: the vertex chain, rewritten as one unit --
/// `canopy.points(polyline, vec![..])` re-derives every segment and joint.
///
/// TODO: points are local px only, so a polyline in a percentage-width box does not stretch
/// with it -- every responsive caller ends up running the same system, reading the resolved
/// `Section` back and rewriting the whole chain on resize (`application`'s `drive_traverses`
/// is exactly this). A fraction-of-the-box vertex would make that the composite's problem
/// instead of every author's. The obvious shape is a per-point unit -- `Position` of px vs. a
/// normalized pair -- resolved in `build` against the polyline's own `Section`, which already
/// has to be read there. The open question is `PolylineDroppedPoints`' sliding window: a
/// window that evicts from the front is authored in real units, so mixed chains need to say
/// what a fraction means when the box has not resolved yet.
#[derive(Component, Clone, Default)]
pub struct PolylinePoints(pub Vec<Position<Logical>>);

/// How much of the path (by arc length, 0.0..=1.0) is currently drawn -- the "draw the
/// line in" effect, distilled into the composite itself rather than left for every author
/// to hand-roll. `canopy.draw_progress(polyline, 0.4)` reveals the first 40% of the path;
/// a caller driving this every frame (their own per-frame write, or eventually a
/// [`Motion`](crate::Motion) variant once one covers it) gets a smooth draw-in with the
/// same zero-churn property `PolylinePoints` writes already have -- see
/// `PolylineSprout::build`: the full
/// point list always determines segment/joint *count* (and therefore the entity pool),
/// completely independent of progress, so animating this from 0 to 1 never spawns or
/// despawns anything after the first frame. Defaults to fully drawn, so ignoring this
/// entirely reproduces today's behavior exactly.
#[derive(Component, Copy, Clone)]
pub struct PolylineDrawProgress(pub f32);
impl Default for PolylineDrawProgress {
    fn default() -> Self {
        Self(1.0)
    }
}

/// The running total of points ever dropped from the *front* of [`PolylinePoints`] over
/// this polyline's lifetime -- a sliding window's own eviction bookkeeping (fixed count,
/// fixed time range, whatever), written alongside the shrunk `PolylinePoints` on the same
/// tick a caller trims its history. Cumulative, not a per-write delta: `Polyline` tracks
/// its own last-seen total internally and diffs against it, so re-sending the same total
/// (or never touching this at all -- it defaults to 0) is always a safe no-op. Only
/// meaningful without `dash` (see `PolylineSprout::build` for why); with a dash pattern
/// active it's read but not acted on, since a dash's segment boundaries don't correspond
/// 1:1 with point count.
#[derive(Component, Copy, Clone, Default)]
pub struct PolylineDroppedPoints(pub usize);

/// Polyline's OWN appearance vocabulary, held as one component rather than field-by-field.
/// Set at spawn through [`PolylineSprout`]'s builder; there is no post-spawn rewrite for it
/// yet, unlike [`PolylinePoints`]/[`PolylineDrawProgress`].
///
/// Colour is deliberately *not* in here. A polyline does nothing with it but forward it to
/// the [`Line`]s it builds, so it lives in the ordinary [`Color`] component like every other
/// primitive's does -- which is what makes [`Grows::color`](crate::Grows::color) and
/// `Motion::Color` reach a polyline at all. Held here it was spawn-only, and animating it
/// silently did nothing.
#[derive(Component, Copy, Clone)]
pub struct PolylineStyle {
    pub weight: i32,
    pub dash: Option<DashPattern>,
}
impl Default for PolylineStyle {
    fn default() -> Self {
        Self {
            weight: crate::MIN_LINE_WEIGHT,
            dash: None,
        }
    }
}

/// A repeating on/off run length along the path, in px, continuous across vertices (the
/// pattern doesn't reset at a bend) -- see `dashed_segments` (engine-internal).
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
    color: Color,
    draw_progress: PolylineDrawProgress,
    dropped_points: PolylineDroppedPoints,
}
impl Default for PolylineSprout {
    fn default() -> Self {
        Self {
            leaf: LeafSprout::default(),
            points: None,
            style: PolylineStyle::default(),
            color: Color::default(),
            draw_progress: PolylineDrawProgress::default(),
            dropped_points: PolylineDroppedPoints::default(),
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
    /// Clamped to [`MIN_LINE_WEIGHT`](crate::MIN_LINE_WEIGHT), the same floor the segments
    /// themselves take, which is why this shares the constant rather than carrying its own.
    pub fn weight(mut self, w: i32) -> Self {
        self.style.weight = w.max(crate::MIN_LINE_WEIGHT);
        self
    }
    /// Forwarded to every [`Line`] the polyline builds. Rewritable after spawn through
    /// [`Grows::color`](crate::Grows::color), and animatable as `Motion::Color`.
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }
    pub fn dash(mut self, d: DashPattern) -> Self {
        self.style.dash = Some(d);
        self
    }
    /// Starting reveal fraction -- see [`PolylineDrawProgress`]. Almost always left at the
    /// default (fully drawn); set this when a polyline should *start* mid-draw-in rather
    /// than snap to it after spawning.
    pub fn draw_progress(mut self, t: f32) -> Self {
        self.draw_progress = PolylineDrawProgress(t.clamp(0.0, 1.0));
        self
    }
}
impl Author for PolylineSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Polyline {},
            self.points.expect("Polyline::points(..) is required"),
            self.style,
            self.color,
            self.draw_progress,
            self.dropped_points,
            Grid::default(),
        )
    }
    fn build(this: Entity, tree: &mut crate::Tree) {
        // Persistent pools, one entity per segment/joint -- reconciled in place on every
        // write rather than torn down and rebuilt. Pool size is driven by the *full* point
        // list only, never by
        // `PolylineDrawProgress` -- see below -- so an existing polyline's points moving,
        // or its draw-in progress advancing, is nothing more than new `Location`/`Color`/
        // `Opacity` values on entities that already exist. `Location` already implements
        // `Animate` (the same machinery driving every other animated position in this
        // crate), so a caller can `tree.animate(polyline, ..)` a `PolylinePoints` change
        // and get real interpolation, not a series of respawns. Only an actual topology
        // change (a point added/removed, or dash toggling) touches `reconcile`'s
        // spawn/despawn path at all, and only for the entities the count delta requires.
        //
        // Both `reconcile` and the value-diff caches below key everything by raw index,
        // and `reconcile` only grows/shrinks the pool from the tail. That's exactly right
        // for append-only growth (a stock ticker pushing new points on the end: every
        // earlier index's data is untouched, only the new trailing index appears) but
        // wrong for a moving/sliding window that evicts from the *front* (dropping the
        // oldest point): `segment_data.len()` shrinks by one, so `reconcile` would pop the
        // *tail* entity (the newest segment, which should survive) instead of the one that
        // actually disappeared, and every surviving index's data would shift down by one,
        // making the value-diff cache see everything as "changed." `PolylineDroppedPoints`
        // exists to correct exactly that: trimming that many entries from the *front* of
        // the pools/caches below, before `reconcile` runs, so surviving entities keep their
        // identity across an eviction instead of just their raw index. Getting this wrong
        // (a stale, too-large, or simply never-set count) only ever costs a perf
        // regression, never correctness or a panic -- `segment_data`/`joint_data` are
        // always recomputed fresh and in full from the *actual current* `PolylinePoints`
        // below, completely independent of what this says; the drop count only chooses
        // which already-existing entities get reused instead of respawned.
        // An animated `Color` is mutated in place by the animation runner rather than
        // inserted, so the `Insert` reaction below never sees it. Every other primitive is
        // unaffected: they draw from the component themselves and change detection carries
        // it to the renderer. A polyline is the one that has to *forward* its colour to the
        // children it builds, so it has to hear about the write -- which is what the
        // runner's `Resolve<Animation<Color>>` is for. Re-inserting the value it just wrote
        // turns the mutation back into a real write, exactly as `Node` does for an animated
        // `Opacity`.
        tree.subscribe(
            this,
            |trigger: Trigger<crate::Resolve<crate::Animation<Color>>>,
             colors: Query<&Color>,
             mut tree: Tree| {
                if let Ok(c) = colors.get(trigger.event_target()) {
                    tree.write_to(trigger.event_target(), *c);
                }
            },
        );
        // TODO this just gets moved into the closure? how can we access again from outside?
        let mut segments: Vec<Entity> = Vec::new();
        let mut joints: Vec<Entity> = Vec::new();
        // What each pool entity was last actually written with -- `None` means "currently
        // hidden" (mirrors the `Opacity(0.0)` branch below). A long, mostly-settled
        // polyline (e.g. a growing history of points where only the newest segment is
        // still animating in) would otherwise re-`write_to` every visible entity on every
        // single reactive fire, even the ones whose data didn't change at all this frame.
        //
        // Colour is part of what is cached, not just geometry. It is forwarded to the
        // children rather than drawn here, so a recolour with every point unmoved is a real
        // change to what they hold -- keyed on position alone the diff below would find the
        // entry unchanged and skip the write, and the polyline would keep the colour it was
        // spawned with however many times it was set.
        let mut segment_cache: Vec<Option<(Position<Logical>, Position<Logical>, Color)>> =
            Vec::new();
        let mut joint_cache: Vec<Option<(Position<Logical>, Color)>> = Vec::new();
        // Cumulative `PolylineDroppedPoints` value as of the last fire -- diffed against
        // the current one to get this tick's actual front-eviction count, so re-sending
        // the same total (or never touching the component at all) is always a no-op.
        let mut last_dropped: usize = 0;
        tree.react_any::<(
            PolylinePoints,
            PolylineStyle,
            Color,
            PolylineDrawProgress,
            PolylineDroppedPoints,
        ), _>(
            this,
            move |trigger: Trigger<
                Insert,
                (
                    PolylinePoints,
                    PolylineStyle,
                    Color,
                    PolylineDrawProgress,
                    PolylineDroppedPoints,
                ),
            >,
                  points_q: Query<&PolylinePoints>,
                  styles: Query<&PolylineStyle>,
                  colors: Query<&Color>,
                  progresses: Query<&PolylineDrawProgress>,
                  drops: Query<&PolylineDroppedPoints>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let points = points_q.get(e).unwrap().0.clone();
                let style = *styles.get(e).unwrap();
                // Defaulted rather than unwrapped: `Color` is an ordinary component now, so
                // nothing stops it being stripped, and a polyline with no colour should
                // draw in the default one rather than panic.
                let color = colors.get(e).copied().unwrap_or_default();
                let progress = progresses
                    .get(e)
                    .copied()
                    .unwrap_or_default()
                    .0
                    .clamp(0.0, 1.0);
                let dropped_total = drops.get(e).copied().unwrap_or_default().0;
                let newly_dropped = dropped_total.saturating_sub(last_dropped);
                last_dropped = dropped_total;
                if points.len() < 2 {
                    for child in segments.drain(..) {
                        tree.remove(child);
                    }
                    for child in joints.drain(..) {
                        tree.remove(child);
                    }
                    segment_cache.clear();
                    joint_cache.clear();
                    last_dropped = 0;
                    return;
                }
                // Only meaningful without a dash pattern -- a dash's segment boundaries
                // don't correspond 1:1 with point count (see `PolylineDroppedPoints`'s own
                // docs), so there's no safe way to know how many *segments* a given number
                // of dropped *points* removed from the front. `reconcile`'s ordinary
                // tail-based logic still produces a fully correct (just unoptimized)
                // result for that combination.
                if newly_dropped > 0 && style.dash.is_none() {
                    let n = newly_dropped.min(segments.len());
                    for child in segments.drain(..n) {
                        tree.remove(child);
                    }
                    segment_cache.drain(..n.min(segment_cache.len()));
                    let n = newly_dropped.min(joints.len());
                    for child in joints.drain(..n) {
                        tree.remove(child);
                    }
                    joint_cache.drain(..n.min(joint_cache.len()));
                }
                // Pool size always comes from the *full* path -- drawing in from 0 to 1
                // never spawns or despawns a single entity after this first reconcile.
                let (segment_data, joint_data) = match style.dash {
                    None => straight_segments(&points),
                    Some(dash) => dashed_segments(&points, dash),
                };
                // What's actually *visible* right now: the same segments/joints, but
                // computed from a path truncated to `progress` of the total arc length.
                // Since both segment functions are deterministic prefix walks over the
                // points they're given, `visible_segment_data`/`visible_joint_data` are
                // always exact prefixes of `segment_data`/`joint_data` -- so index `i`
                // corresponds to the same entity in both, and "beyond the visible prefix"
                // just means "not drawn yet."
                let truncated = truncate_path(&points, progress);
                let (visible_segment_data, visible_joint_data) = if truncated.len() < 2 {
                    (Vec::new(), Vec::new())
                } else {
                    match style.dash {
                        None => straight_segments(&truncated),
                        Some(dash) => dashed_segments(&truncated, dash),
                    }
                };

                reconcile(
                    &mut tree,
                    e,
                    &mut segments,
                    segment_data.len(),
                    |tree, parent| tree.branch(parent, Line::new(1).elevate(Elevation::up(1))),
                );
                segment_cache.resize(segments.len(), None);
                for (i, child) in segments.iter().enumerate() {
                    let value = visible_segment_data
                        .get(i)
                        .copied()
                        .map(|(a, b)| (a, b, color));
                    if segment_cache[i] == value {
                        continue;
                    }
                    match value {
                        Some((a, b, color)) => {
                            tree.write_to(
                                *child,
                                (
                                    Line::new_marker(style.weight),
                                    color,
                                    Opacity::new(1.0),
                                    Location::new().xs(
                                        a.left().px().as_x().with(a.top().px().as_y()),
                                        b.left().px().as_x().with(b.top().px().as_y()),
                                    ),
                                ),
                            );
                        }
                        None => tree.write_to(*child, Opacity::new(0.0)),
                    }
                    segment_cache[i] = value;
                }

                // The joint is exactly the segment's width now, with nothing subtracted. It
                // used to be shrunk, because `Line`'s AA faded inward only -- 0% coverage at
                // its true edge ramping to 100% a full px inside it -- which put a segment's
                // *perceived* edge about half a px inside its true one while `Polygon`'s sat
                // right on it, so a joint drawn at the stated weight bulged past the segments
                // it joined. `line.wgsl`'s feather is centered on the true edge now, same as
                // `Polygon`'s, so both perceived edges land in the same place and the two
                // agree by construction. A whole number either way, which is what keeps the
                // joint centered: `polygon/pipeline.rs` snaps position and area to whole
                // physical px *independently*, and a `left` derived from a fractional
                // center/width pair can round to a different sub-pixel offset than its width
                // did, visibly shifting the shape off its own center.
                let diameter = style.weight as f32;

                reconcile(
                    &mut tree,
                    e,
                    &mut joints,
                    joint_data.len(),
                    |tree, parent| {
                        tree.branch(
                            parent,
                            Polygon::new().rounding(1.0).elevate(Elevation::up(2)),
                        )
                    },
                );
                joint_cache.resize(joints.len(), None);
                for (i, child) in joints.iter().enumerate() {
                    let value = visible_joint_data.get(i).copied().map(|j| (j, color));
                    if joint_cache[i] == value {
                        continue;
                    }
                    match value {
                        Some((j, color)) => {
                            tree.write_to(
                                *child,
                                (
                                    color,
                                    Opacity::new(1.0),
                                    Location::new().xs(
                                        j.left().px().as_center_x().with(diameter.px().as_width()),
                                        j.top().px().as_center_y().with(diameter.px().as_height()),
                                    ),
                                ),
                            );
                        }
                        None => tree.write_to(*child, Opacity::new(0.0)),
                    }
                    joint_cache[i] = value;
                }
            },
        );
    }
}

/// The sub-path of `points` covered by the first `t` (0.0..=1.0) of its total arc length --
/// full vertices up to wherever `t` lands, then one lerped point ending the visible prefix
/// exactly on the path (not just at the nearest vertex), so [`PolylineDrawProgress`]'s
/// draw-in looks continuous rather than jumping vertex to vertex.
fn truncate_path(points: &[Position<Logical>], t: f32) -> Vec<Position<Logical>> {
    if points.len() < 2 || t >= 1.0 {
        return points.to_vec();
    }
    let total: f32 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
    let target = total * t.clamp(0.0, 1.0);
    let mut traveled = 0.0;
    let mut out = vec![points[0]];
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = a.distance(b);
        if traveled + len >= target {
            let frac = if len > 0.0 {
                (target - traveled) / len
            } else {
                0.0
            };
            out.push(lerp(a, b, frac));
            return out;
        }
        traveled += len;
        out.push(b);
    }
    out
}

/// Sprout or shrinks `pool` to exactly `target_len` entities -- spawning via `spawn` (given
/// only minimal/placeholder data; the caller overwrites every pool member, new or
/// surviving, with real data in the pass right after calling this) or despawning from the
/// tail. Entities that already existed at the right index are left completely untouched
/// here, which is the whole point: a `PolylinePoints` write that doesn't change the
/// segment/joint count never spawns or despawns anything, only rewrites existing entities'
/// `Location`/`Color`.
fn reconcile(
    tree: &mut Tree,
    parent: Entity,
    pool: &mut Vec<Entity>,
    target_len: usize,
    mut spawn: impl FnMut(&mut Tree, Entity) -> Entity,
) {
    while pool.len() < target_len {
        pool.push(spawn(tree, parent));
    }
    while pool.len() > target_len {
        tree.remove(pool.pop().unwrap());
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
) -> (
    Vec<(Position<Logical>, Position<Logical>)>,
    Vec<Position<Logical>>,
) {
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
) -> (
    Vec<(Position<Logical>, Position<Logical>)>,
    Vec<Position<Logical>>,
) {
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
