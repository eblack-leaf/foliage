//! A line of beads between two points.

use core::f32::consts::TAU;

use foliage::{
    Boxed, Ease, Elevation, Grove, Grow, HorizontalCoordinate, Leaf, Location, Motion, Place,
    Pollen, Polygon, Sequence, Shape, Source, Stem, Timing, VerticalCoordinate, center_x, center_y,
    left, top,
};

use crate::{Ramp, Scatter, fill, shifted};

/// How far a bead wanders off the line, in pixels, on each axis.
///
/// Both axes rather than across the line, because which way the line runs is not known here -- it is
/// whatever the two ends resolve to this frame. A scatter that is the same either way needs no
/// direction to be right.
const WANDER: f32 = 6.0;

/// How large a bead is, in pixels.
const BEAD: (f32, f32) = (7.0, 13.0);

/// How much of its final size a bead is retracted to. Not zero: what leaves has to be seen leaving.
const SEED: f32 = 0.2;

/// How much a bead's colour may sit off the ramp, so that no two neighbours read as one shape.
const DRIFT: f32 = 0.04;

/// How long one bead takes to fade, and to grow into or out of its shape.
const BEAD_MS: u64 = 240;
const GROW_MS: u64 = 380;

/// How long the front takes to run the whole line, either way.
const SPREAD: f32 = 460.0;

/// How large the box at the trail's far end is, in pixels.
///
/// Nothing is drawn in it: it is somewhere to anchor to. A box rather than a point because an anchor
/// is read by its edges and its extents, and something square has a middle that is the same middle
/// whichever of those is asked for.
const TAIL: f32 = 28.0;

/// A run of small polygons from one point to another, coloured start to end along a [`Ramp`].
///
/// Scattered around the line between the two points rather than laid on it, the way a mosaic is
/// scattered inside its outline: what reads as a line is the run of them, no one of them.
///
/// Both ends are stated as placements rather than as numbers, and every bead is stated as one too:
/// a bead sits `from + (to - from) * t`, which is a placement the engine resolves every frame like
/// any other. So an end may be another element -- `anchor().center_x()` on a region is that
/// region's centre, wherever the layout has since put it -- or a point in the caller's own space,
/// or either with pixels added to it. Nothing is measured here and nothing has to be measured per
/// frame: what the trail is given is what the engine is given.
///
/// Grown retracted -- nothing showing. [`expand`](Self::expand) runs it out to the far end and
/// [`retract`](Self::retract) draws it back in, each in the order the line runs, so the far end is
/// the last to arrive and the first to leave. Either may be asked for while the other is still
/// crossing: the trail turns around from wherever it has got to rather than from where it was
/// going. [`frame`](Self::frame) carries whichever is in flight, and [`expanded`](Self::expanded)
/// and [`retracted`](Self::retracted) say when it is all the way out or all the way back.
/// [`end`](Self::end) is a box on the far end, to hang the next thing off.
pub struct Trail {
    ramp: Ramp,
    /// What the whole of it is branched from. `None` until it is grown.
    root: Option<Leaf>,
    /// A box sitting on the far end, to hang something off. `None` until it is grown.
    tail: Option<Leaf>,
    /// Every bead, in the order the line runs. Empty until it is grown.
    beads: Vec<Bead>,
    /// Where the trail is between its two ends.
    along: Along,
}

/// One polygon of the trail.
struct Bead {
    /// The polygon itself.
    leaf: Leaf,
    /// Where it sits, as the placement it was stated at. Kept whole rather than as a number: it is
    /// restated every time the bead is grown or seeded, and it is the engine that reads it.
    at: (HorizontalCoordinate, VerticalCoordinate),
    /// How large it is expanded, in pixels.
    size: f32,
    /// The shape it settles into once it is out.
    shape: Shape,
    /// Where it stands along the line, `0.0` at the near end and `1.0` at the far one. What colours
    /// it, and what delays it.
    front: f32,
}

/// Where the trail is between its two ends.
#[derive(Copy, Clone)]
enum Along {
    /// All the way in, nothing showing.
    Retracted,
    /// Running out, and the group carrying it.
    Expanding(Sequence),
    /// All the way out.
    Expanded,
    /// Coming back in, and the group carrying it.
    Retracting(Sequence),
}

impl Trail {
    /// An ungrown trail, coloured from `ramp`: its start at `0.0`, its end at `1.0`.
    pub fn new(ramp: Ramp) -> Self {
        Self {
            ramp,
            root: None,
            tail: None,
            beads: Vec::new(),
            along: Along::Retracted,
        }
    }

    /// Grows `beads` of it off `at`, from one placement to another, and hands back the leaf the
    /// whole of it is branched from -- to move it, or to take it away.
    ///
    /// `reads` is what `anchor()` reads in either end, so a trail off a region is
    /// `(anchor().center_x(), anchor().center_y())` handed that region's stem, with pixels added to
    /// stand it clear, and a far end of `(50.pct(), 50.pct())` is the middle of `at`. Percentages
    /// in an end are read against `at`, whose box the trail covers whole.
    ///
    /// How many beads is asked for rather than worked out: the line's length is not known here and
    /// is not fixed -- it is whatever the two ends resolve to this frame -- so a count is what keeps
    /// the trail the same trail as the layout moves under it.
    ///
    /// Grown retracted: every bead is a seed of itself with no ink in it, so nothing shows until
    /// [`expand`](Self::expand). Growing one that is already grown does nothing and hands back what
    /// it grew.
    pub fn grow(
        &mut self,
        grove: &mut Grove,
        at: Leaf,
        reads: Leaf,
        from: (HorizontalCoordinate, VerticalCoordinate),
        to: (HorizontalCoordinate, VerticalCoordinate),
        beads: usize,
        scatter: &mut Scatter,
    ) -> Leaf {
        if let Some(root) = self.root {
            return root;
        }
        let root = grove.branch(
            at,
            Stem::new().at(Location::new().xs(
                left(0.0.pct()).width(100.0.pct()),
                top(0.0.pct()).height(100.0.pct()),
            )),
        );
        let beads = beads.max(2);
        self.beads = (0..beads)
            .map(|bead| {
                let front = bead as f32 / (beads - 1) as f32;
                // Where it sits: the near end, plus the whole of the way to the far end taken
                // `front` of the way along, plus its own wander. One expression per axis, resolved
                // by the engine -- so an end that is another element goes on being that element.
                let at = (
                    from.0.clone()
                        + ((to.0.clone() - from.0.clone()) * front
                            + scatter.between(-WANDER, WANDER).px()),
                    from.1.clone()
                        + ((to.1.clone() - from.1.clone()) * front
                            + scatter.between(-WANDER, WANDER).px()),
                );
                let size = scatter.between(BEAD.0, BEAD.1);
                let shape = Shape {
                    sides: scatter.between(5.0, 7.0).round(),
                    rounding: scatter.between(0.03, 0.18),
                    rotation: scatter.next() * TAU,
                };
                let hue = shifted(self.ramp.at(front), scatter.between(-DRIFT, DRIFT));
                let leaf = grove.branch(
                    root,
                    Polygon::new()
                        .sides(3.0)
                        .rounding(0.85)
                        .rotation(shape.rotation - 0.7)
                        .color(fill(hue))
                        .intangible()
                        .opacity(0.0)
                        .anchored(reads)
                        .at(placed(&at, size * SEED))
                        .elevate(Elevation::up(1)),
                );
                Bead {
                    leaf,
                    at,
                    size,
                    shape,
                    front,
                }
            })
            .collect();
        // The far end, as something to hang the next thing off. Anchored the way a bead is and
        // placed at the same end a bead is placed against, so it is the trail's end however the
        // layout moves -- and grown whether or not anything is ever hung on it, which costs one
        // stem and saves the caller having to state that end a second time.
        self.tail = Some(grove.branch(root, Stem::new().anchored(reads).at(placed(&to, TAIL))));
        self.root = Some(root);
        root
    }

    /// The box on the far end, to anchor a sub-arrangement to. `None` until it is grown.
    ///
    /// It does not move with the expanding: the trail runs out to where this already is, so what is
    /// hung here is placed against where the trail ends rather than against how far along it has got.
    pub fn end(&self) -> Option<Leaf> {
        self.tail
    }

    /// Runs it out to the far end.
    ///
    /// Each bead fades in and grows into its shape, delayed by how far along the line it sits, so
    /// what arrives is a front travelling rather than a line switched on. Expanding one that is
    /// already out does nothing; expanding one that is still coming back in turns it around from
    /// wherever each bead has got to. Nothing happens on one that was never grown.
    pub fn expand(&mut self, grove: &mut Grove) {
        if matches!(self.along, Along::Expanded | Along::Expanding(_)) {
            return;
        }
        self.along = Along::Expanding(self.run(grove, true));
    }

    /// Draws it back in, from the far end.
    ///
    /// The order the line runs, as [`expand`](Self::expand) has it, so the bead that arrived last
    /// is the one that leaves first. Retracting one that is already in does nothing, and retracting
    /// one still running out turns it around.
    pub fn retract(&mut self, grove: &mut Grove) {
        if matches!(self.along, Along::Retracted | Along::Retracting(_)) {
            return;
        }
        self.along = Along::Retracting(self.run(grove, false));
    }

    /// Carries whatever is in flight. Call once a frame while it is on screen.
    pub fn frame(&mut self, pollen: &Pollen) {
        let (Along::Expanding(group) | Along::Retracting(group)) = self.along else {
            return;
        };
        if !pollen.sequence_finished(group) {
            return;
        }
        self.along = match self.along {
            Along::Expanding(_) => Along::Expanded,
            _ => Along::Retracted,
        };
    }

    /// Whether it is all the way out. True from the frame the last bead lands, not only on it.
    pub fn expanded(&self) -> bool {
        matches!(self.along, Along::Expanded)
    }

    /// Whether it is all the way in, which it is before it has ever been expanded.
    pub fn retracted(&self) -> bool {
        matches!(self.along, Along::Retracted)
    }

    /// Sends the front along the line, either way, and hands back the group carrying it.
    ///
    /// Three motions per bead because they are three properties, and one delay because a bead
    /// arriving or leaving is one thing. Each states where the bead is going rather than how far it
    /// is moved, so a trail turned around mid-run lands on exactly what it was grown at.
    fn run(&mut self, grove: &mut Grove, out: bool) -> Sequence {
        let group = grove.sequence();
        for bead in &self.beads {
            // Out, the near end goes first; back, the far end does -- one line running one way.
            let after = ((if out { bead.front } else { 1.0 - bead.front }) * SPREAD) as u64;
            let timing = |ms: u64| {
                Timing::ms(ms)
                    .after(after)
                    .ease(Ease::Decelerate)
                    .within(group)
            };
            let (opacity, size, shape) = match out {
                true => (1.0, bead.size, bead.shape),
                false => (
                    0.0,
                    bead.size * SEED,
                    Shape {
                        sides: 3.0,
                        rounding: 0.85,
                        rotation: bead.shape.rotation - 0.7,
                    },
                ),
            };
            grove.animate(bead.leaf, Motion::Opacity(opacity), timing(BEAD_MS));
            grove.animate(
                bead.leaf,
                Motion::Location(placed(&bead.at, size)),
                timing(GROW_MS),
            );
            grove.animate(bead.leaf, Motion::Polygon(shape), timing(GROW_MS));
        }
        group
    }
}

/// A bead `size` pixels across, centred on the placement it was stated at.
///
/// Centred rather than cornered, so that the placement is the bead's own middle whatever size it is
/// currently at: a bead growing out of its seed grows where it stands rather than walking as it
/// grows. Only the size is in pixels; where it sits is whatever the two ends were stated in.
fn placed((x, y): &(HorizontalCoordinate, VerticalCoordinate), size: f32) -> Location {
    Location::new().xs(
        center_x(x.clone()).width(size.px()),
        center_y(y.clone()).height(size.px()),
    )
}
