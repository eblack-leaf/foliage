//! Blueprint plates: the site's analytical layer.
//!
//! A plate is a technical drawing, not a chart. A ruled scale down the left edge, dashed rows
//! running out from its ticks, a dashed path traversing a set of shapes, and annotations
//! centred on a chosen few of them. It is the counterweight to the poly shapes: the shapes are
//! fluid and expressive, and the measure around them is exact, so each makes the other read
//! stronger than it does alone.
//!
//! Deliberately not a line plot. A plot has to be *about* its data or it reads as a chart
//! someone made up, and it spends the whole field on a trend line -- where a drawing can put
//! its shapes where they compose.
//!
//! Everything here is one of three weights, and which family a mark belongs to has to be
//! legible at a glance: ticks are solid and full strength, rows are dashed and muted, the
//! traverse is dashed and coloured. When the rows were solid they argued with the ticks at any
//! opacity that made them visible; broken line is what separates them. Leader lines out to the
//! annotations were a fourth family and there was no room for it, so proximity does that job
//! instead.
//!
//! The annotations are pseudo-metrics on purpose. They are not instrumentation -- they are
//! the *look* of instrumentation, which is what makes a still figure feel mid-calculation.
//!
//! Built to be reused: the sections' embedded demos are meant to be a live widget in the
//! field with labels naming what it is doing, which is this module with the nodes swapped for
//! content.

use foliage::{
    Animation, Color, DashPattern, Ease, Elevation, Entity, Foliage, FontSize, Grid,
    GridExt, HorizontalAlignment, Stem, Line, Location, Logical, Opacity, Polygon, Polyline,
    PolylineDrawProgress, PolylinePoints, Position, Query, Res, Section, Sprout, Text, Time, Tree,
    VerticalAlignment, component,
};

use crate::site::{motion, role, space, type_scale};

/// A shape in the field, positioned in plate-normalized coordinates: `(0.0, 0.0)` is the
/// field's top-left, `(1.0, 1.0)` its bottom-right, y measured downward like every other
/// coordinate in this framework.
pub(crate) struct Node {
    pub(crate) at: (f32, f32),
    pub(crate) sides: f32,
    pub(crate) size: i32,
    /// Text beside this node. `None` leaves it muted -- which is the whole reason the
    /// annotated ones read as the subject rather than as three of six equals.
    pub(crate) label: Option<Label>,
}

/// Text centred on a node and clear of it, close enough that the pairing is obvious without a
/// line drawn between them.
///
/// Always centred, only ever above or under -- no per-label placement. Free placement made
/// every label a separate decision to get right and a separate thing to re-check at each
/// breakpoint; this makes the *node* the thing to place well, which is one job instead of two.
/// Put a labelled node at a turning point and the open side of the turn is clear by
/// construction.
pub(crate) struct Label {
    pub(crate) text: &'static str,
    /// Which way it sits off the node. Centred either way -- only the direction changes, so
    /// this is the one call a labelled node still makes, and it is decided by where the node
    /// is in the field rather than by nudging px.
    pub(crate) side: Side,
}

/// Above or under, for a node near the top or the bottom of its path.
pub(crate) enum Side {
    Above,
    Under,
}

/// Everything a plate draws. Static so a page declares its figures as consts and the builder
/// stays a pure function of them.
pub(crate) struct PlateSpec {
    /// Drawn in order; the traverse path follows the same order.
    ///
    /// Keep the leftmost node clear of `TICK_MAJOR` plus its own half-width: the ticks reach
    /// into the field from x = 0, and the narrowest breakpoint is where a node first touches
    /// one.
    pub(crate) nodes: &'static [Node],
    /// The ruled scale down the left edge, top to bottom: y fraction, and the number beside
    /// it. An empty string is a minor tick -- shorter, and no number.
    pub(crate) scale: &'static [(f32, &'static str)],
    /// Bottom-left, in the manner of a plate in a manual. Keep it short: it is one line, and
    /// the narrowest column is not wide.
    pub(crate) caption: &'static str,
}

/// Numbers live left of the axis, ticks live right of it. Nothing overlaps because they are
/// on opposite sides of the line, rather than both starting from it.
const GUTTER: i32 = 32;
/// Ticks read as machined marks rather than as more hairlines, so they are the heaviest thing
/// in the rule.
const TICK_WEIGHT: i32 = 3;
const TICK_MAJOR: i32 = 17;
const TICK_MINOR: i32 = 10;
/// Reserved at the bottom for the caption, outside the field, so a long one can never be
/// squeezed against the last rule of the drawing.
const CAPTION_H: i32 = 22;

/// Wide enough for the longest annotation. Only the box -- the text is centred inside it, so
/// an over-wide box costs nothing visually.
const LABEL_W: i32 = 128;
const LABEL_H: i32 = 16;
/// Between a marker's edge and its label.
const LABEL_GAP: i32 = space::SM;

/// How much bigger a node gets once the plate has `lg`'s room. Matched to the plate's own
/// height step (248 -> 340), so the shapes keep their share of the field rather than the field
/// growing away from them.
fn scaled(size: i32) -> i32 {
    size * 7 / 5
}

const PATH_WEIGHT: i32 = 3;
const DASH_ON: f32 = 11.0;
const DASH_OFF: f32 = 7.0;

/// The ruled rows. `MIN_LINE_WEIGHT` is the floor for anything a polyline draws, so tone and
/// dash length are the only ways down from the traverse -- a shorter dash with more gap than
/// [`DASH_ON`]/[`DASH_OFF`] reads as finer even at the same weight.
const ROW_WEIGHT: i32 = foliage::MIN_LINE_WEIGHT;
const ROW_DASH_ON: f32 = 4.0;
const ROW_DASH_OFF: f32 = 8.0;
/// Well under the ticks, which sit at full opacity. Major rows carry a number in the gutter,
/// so they earn a little more presence than the halves between them.
const ROW_MAJOR: f32 = 0.5;
const ROW_MINOR: f32 = 0.3;

/// Annotated nodes take the warm tones; everything else takes this. With no leader lines,
/// the muting is the *only* thing pairing a label with its shape, so the gap between the two
/// tones has to be wide.
fn muted() -> Color {
    Color::stone(500)
}
/// The accent plus the two tones the destination buttons already use. No new hues.
const TONES: [fn() -> Color; 3] = [role::accent, || Color::amber(400), || Color::rose(400)];

/// How long the path takes to draw itself in, matched to the shapes' own morph so the drawing
/// and the shapes in it resolve as one gesture.
const DRAW_SECS: f32 = motion::ENTRANCE as f32 / 1000.0;

/// Drives a plate's traverse path: keeps its vertices resolved against the current field
/// size, and runs the one-shot draw-in.
///
/// Normalized rather than authored in px because `PolylinePoints` is local px space, which
/// does not stretch with a percentage-width box -- so the field's own size has to be read back
/// and the points rewritten. Rewriting them is the supported path (`PolylinePoints` is
/// documented as a whole-unit write) and costs nothing after the first frame: point *count*
/// never changes, so the segment pool is reused rather than respawned.
#[component]
pub(crate) struct Traverse {
    vertices: Vec<(f32, f32)>,
    elapsed: f32,
    /// Last field size the points were written against, so a still figure at a steady size
    /// does no work.
    resolved: Option<(f32, f32)>,
}

fn drive_traverses(
    time: Res<Time>,
    sections: Query<&Section<Logical>>,
    mut paths: Query<(Entity, &mut Traverse)>,
    mut tree: Tree,
) {
    for (entity, mut path) in paths.iter_mut() {
        let Ok(section) = sections.get(entity) else {
            continue;
        };
        let size = (section.width(), section.height());
        if size.0 <= 0.0 || size.1 <= 0.0 {
            continue;
        }
        if path.resolved != Some(size) {
            path.resolved = Some(size);
            let points = path
                .vertices
                .iter()
                .map(|&(x, y)| Position::<Logical>::new((x * size.0, y * size.1)))
                .collect::<Vec<_>>();
            tree.write_to(entity, PolylinePoints(points));
        }
        if path.elapsed < DRAW_SECS {
            path.elapsed += time.frame_diff().as_secs_f32();
            let t = (path.elapsed / DRAW_SECS).clamp(0.0, 1.0);
            tree.write_to(entity, PolylineDrawProgress(t));
        }
    }
}

pub(crate) fn attach(foliage: &mut Foliage) {
    foliage.user.add_systems(drive_traverses);
}

/// Draws `spec` into `plate` -- a region the caller owns, typically from `Column::region`.
///
/// `seq` and `start` place the plate's entrance in the page's own staggered arrival.
pub(crate) fn plate(tree: &mut Tree, plate: Entity, spec: &PlateSpec, seq: Entity, start: u64) {
    // Two boxes sharing one vertical extent: the frame carries the rule's numbers, the field
    // carries everything drawn. Splitting them is what lets a y fraction mean the same thing
    // to a number in the gutter and a tick inside the drawing.
    let frame = tree.branch(
        plate,
        Stem::new()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct()
                    .as_top()
                    .with(100.pct().as_bottom().adjust(-CAPTION_H)),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
    let field = tree.branch(
        frame,
        Stem::new()
            .at(Location::new().xs(
                GUTTER.px().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );

    // fades rather than draws: the traverse is the thing that draws itself in, and a drawing
    // whose every line animated would be a race rather than a gesture
    let fade_to = |tree: &mut Tree, entity: Entity, alpha: f32, delay: u64| {
        tree.animate(
            Animation::new(Opacity::new(alpha))
                .targeting(entity)
                .during(seq)
                .start(start + delay)
                .finish(start + delay + motion::FADE)
                .eased(Ease::Linear),
        );
    };
    let rule = |tree: &mut Tree,
                parent: Entity,
                from: (i32, f32),
                to: (i32, f32),
                weight: i32,
                delay: u64| {
        let entity = tree.branch(
            parent,
            Line::new(weight)
                .color(role::outline())
                .at(Location::new().xs(
                    from.0.px().as_x().with((from.1 * 100.0).pct().as_y()),
                    to.0.px().as_x().with((to.1 * 100.0).pct().as_y()),
                ))
                .elevate(Elevation::up(1))
                .with(Opacity::new(0.0)),
        );
        fade_to(tree, entity, 1.0, delay);
    };

    // A tick per row, a number per major row, and a dashed run continuing from each tick
    // across the field.
    //
    // The runs were solid hairlines once, and at any opacity that made them visible they read
    // as a second family of horizontals arguing with the ticks. Dashed is what fixes that:
    // broken line is legibly *not* a rule you are meant to read along, so it can sit well under
    // the ticks in tone and still be seen. It is the paper the drawing sits on.
    //
    // The axis they used to hang off is gone for its own reason: a column of ticks at a shared
    // left edge already reads as a scale, and the line joining them was the one vertical in a
    // drawing that has no other verticals.
    for (i, &(y, number)) in spec.scale.iter().enumerate() {
        let major = !number.is_empty();
        let delay = i as u64 * 30;
        let row = tree.branch(
            field,
            Polyline::new()
                .points(vec![(0, 0); 2])
                .weight(ROW_WEIGHT)
                .color(role::outline())
                .dash(DashPattern::new(ROW_DASH_ON, ROW_DASH_OFF))
                // starts past the tick, so the two read as one mark continuing rather than as
                // a rule with a heavier stub sitting on its left end
                .at(Location::new().xs(
                    (TICK_MAJOR + space::SM)
                        .px()
                        .as_left()
                        .with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .with((
                    Traverse {
                        vertices: vec![(0.0, y), (1.0, y)],
                        // already drawn -- `elapsed` past the duration skips the draw-in and
                        // leaves only the point resolve. Seven rows racing the traverse would
                        // bury the one line that is meant to be watched.
                        elapsed: DRAW_SECS,
                        resolved: None,
                    },
                    Opacity::new(0.0),
                )),
        );
        fade_to(tree, row, if major { ROW_MAJOR } else { ROW_MINOR }, delay);
        rule(
            tree,
            field,
            (0, y),
            (if major { TICK_MAJOR } else { TICK_MINOR }, y),
            TICK_WEIGHT,
            delay,
        );
        if !major {
            continue;
        }
        let label = tree.branch(
            frame,
            Text::new(number)
                .size(FontSize::new(type_scale::LABEL))
                .color(role::on_surface_variant())
                .at(Location::new().xs(
                    0.px().as_left().with((GUTTER - space::SM).px().as_width()),
                    (y * 100.0).pct().as_center_y().with(16.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .with((
                    HorizontalAlignment::Right,
                    VerticalAlignment::Middle,
                    Opacity::new(0.0),
                )),
        );
        crate::site::fade_in(tree, label, seq, start + delay);
    }

    tree.branch(
        field,
        Polyline::new()
            // seeded degenerate and immediately rewritten by `drive_traverses` -- `points` is
            // required, and the real ones need a resolved size that does not exist at spawn
            .points(spec.nodes.iter().map(|_| (0, 0)).collect::<Vec<_>>())
            .weight(PATH_WEIGHT)
            .color(role::accent())
            .dash(DashPattern::new(DASH_ON, DASH_OFF))
            .draw_progress(0.0)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(2))
            .with(Traverse {
                vertices: spec.nodes.iter().map(|n| n.at).collect(),
                elapsed: 0.0,
                resolved: None,
            }),
    );

    let mut annotated = 0usize;
    for (i, node) in spec.nodes.iter().enumerate() {
        let (x, y) = node.at;
        let tone = match node.label {
            Some(_) => {
                let tone = TONES[annotated % TONES.len()]();
                annotated += 1;
                tone
            }
            None => muted(),
        };
        // Positions are fractions and so scale with the field on their own; sizes are px and
        // do not, which left the shapes as specks once a wide window gave the plate a third
        // more height. The subject of a drawing has to grow with the drawing.
        let sized = |s: i32| {
            Location::new()
                .xs(
                    (x * 100.0).pct().as_center_x().with(s.px().as_width()),
                    (y * 100.0).pct().as_center_y().with(s.px().as_height()),
                )
                .lg(
                    (x * 100.0)
                        .pct()
                        .as_center_x()
                        .with(scaled(s).px().as_width()),
                    (y * 100.0)
                        .pct()
                        .as_center_y()
                        .with(scaled(s).px().as_height()),
                )
        };
        let marker = tree.branch(
            field,
            Polygon::new()
                .sides(3.0)
                .rounding(0.3)
                .rotation(0.22)
                .color(tone)
                .at(sized(node.size))
                .elevate(Elevation::up(3))
                .with(Opacity::new(0.0)),
        );
        crate::site::morph_in(
            tree,
            marker,
            seq,
            node.sides,
            0.3,
            start + i as u64 * motion::STAGGER,
        );

        let Some(label) = &node.label else {
            continue;
        };
        // No leader. Proximity is enough to say which shape a label belongs to, and a leader
        // put a third family of lines into a drawing that already has an axis, seven ticks and
        // a traverse -- at which point nothing reads as the subject.
        //
        // Clear of the marker by its own half-width, which scales with it at `lg` -- a fixed
        // offset would have the label creeping under a grown shape on a wide window.
        let off = |s: i32| match label.side {
            Side::Above => -(s / 2 + LABEL_GAP + LABEL_H),
            Side::Under => s / 2 + LABEL_GAP,
        };
        let centred = |s: i32| {
            (
                (x * 100.0)
                    .pct()
                    .as_center_x()
                    .with(LABEL_W.px().as_width()),
                (y * 100.0)
                    .pct()
                    .as_top()
                    .adjust(off(s))
                    .with(LABEL_H.px().as_height()),
            )
        };
        let (h_xs, v_xs) = centred(node.size);
        let (h_lg, v_lg) = centred(scaled(node.size));
        let caption = tree.branch(
            field,
            Text::new(label.text)
                .size(FontSize::new(type_scale::LABEL))
                // the shapes are the subject of a plate; an annotation is a note about one,
                // and reading brighter than what it annotates inverted that
                .color(role::on_surface_variant())
                .at(Location::new().xs(h_xs, v_xs).lg(h_lg, v_lg))
                .elevate(Elevation::up(4))
                .with((
                    HorizontalAlignment::Center,
                    VerticalAlignment::Middle,
                    Opacity::new(0.0),
                )),
        );
        crate::site::fade_in(tree, caption, seq, start + motion::STAGGER * (i as u64 + 2));
    }

    // in its own strip below the field, so it can never be crowded by the drawing
    let caption = tree.branch(
        plate,
        Text::new(spec.caption)
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_surface_variant())
            .at(Location::new().xs(
                GUTTER.px().as_left().with(100.pct().as_right()),
                100.pct().as_bottom().with(CAPTION_H.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
            )),
    );
    crate::site::fade_in(tree, caption, seq, start + motion::STAGGER * 4);
}
