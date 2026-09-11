//! A puff of motes in a box.

use core::f32::consts::TAU;

use foliage::{
    Boxed, Ease, Elevation, Grove, Grow, Leaf, Location, Motion, Place, Polygon, Shape, Source,
    Timing, center_x, center_y,
};

use crate::{Ramp, Rgb, Scatter, fill, shifted};

/// How many polygons one cluster is.
const MOTES: usize = 13;

/// How large each mote is, as a fraction of the box.
const MOTE: (f32, f32) = (0.44, 0.66);

/// How much of the box the motes keep to, so that what is clicked is a little larger than what is
/// drawn and neighbouring clusters do not read as a row of touching boxes.
const HELD: f32 = 0.75;

/// How much a mote's colour may sit off the ramp, so that no two neighbours read as one shape.
const DRIFT: f32 = 0.05;

/// How long one mote takes to turn, and how long a turn takes to cross the cluster.
const TURN_MS: u64 = 240;
const TURN_SPREAD: f32 = 130.0;

/// Small polygons scattered through a box the caller grew, coloured from a [`Ramp`] by how deep
/// in the box each sits: the ramp's start at the middle, its end at the rim.
///
/// The box is the caller's: where it is anchored, how it is elevated, whether it is interactive,
/// what it fades to, and what is hung on top of it. This only puts polygons in it and recolours
/// them: [`tint`](Self::tint) sweeps them onto another ramp from the middle out, and
/// [`restore`](Self::restore) puts them back.
pub struct Cluster {
    motes: Vec<Mote>,
}

/// One polygon of the cluster.
struct Mote {
    /// The polygon itself.
    leaf: Leaf,
    /// What it rests at, kept because a tint has to have somewhere to put it back to.
    hue: Rgb,
    /// How far off the ramp it was put, kept so that it is put the same distance off either ramp.
    drift: f32,
    /// How far out of the box's middle it sits, `0.0` at the middle to `1.0` at the rim. What
    /// colours it on either ramp, and what delays its turn.
    depth: f32,
}

impl Cluster {
    /// Fills `pad` -- `size` px square -- with motes coloured from `ramp`.
    ///
    /// Placed in the pad's own box in pixels: the box is a fixed size, so nothing in a cluster is a
    /// reading of anything outside it.
    pub fn grow(
        grove: &mut Grove,
        pad: Leaf,
        size: f32,
        ramp: &Ramp,
        scatter: &mut Scatter,
    ) -> Self {
        let held = size * HELD / 2.0;
        let motes = (0..MOTES)
            .map(|_| {
                let (x, y) = (scatter.between(-held, held), scatter.between(-held, held));
                let depth = (x.abs().max(y.abs()) / held).min(1.0);
                let across = size * scatter.between(MOTE.0, MOTE.1);
                let shape = Shape {
                    sides: scatter.between(5.0, 7.0).round(),
                    rounding: scatter.between(0.03, 0.18),
                    rotation: scatter.next() * TAU,
                };
                let drift = scatter.between(-DRIFT, DRIFT);
                let hue = shifted(ramp.at(depth), drift);
                let leaf = grove.branch(
                    pad,
                    Polygon::new()
                        .sides(shape.sides)
                        .rounding(shape.rounding)
                        .rotation(shape.rotation)
                        .color(fill(hue))
                        .intangible()
                        .at(placed((x, y), across))
                        .elevate(Elevation::up(1)),
                );
                Mote {
                    leaf,
                    hue,
                    drift,
                    depth,
                }
            })
            .collect();
        Self { motes }
    }

    /// Sweeps the motes onto `ramp`, from the middle out.
    ///
    /// Each is put the same distance off the new ramp as it was off the one it was grown from, so
    /// the scatter that keeps neighbours apart survives the change of colour.
    pub fn tint(&self, grove: &mut Grove, ramp: &Ramp) {
        for mote in &self.motes {
            turn(grove, mote, shifted(ramp.at(mote.depth), mote.drift));
        }
    }

    /// Puts them back to what they were grown at.
    pub fn restore(&self, grove: &mut Grove) {
        for mote in &self.motes {
            turn(grove, mote, mote.hue);
        }
    }
}

/// Moves one mote onto a colour, delayed by how deep in the box it sits.
///
/// Stated as where the colour is going rather than as a step from where it is, so the same call
/// tints a cluster and puts it back, and a turn interrupted mid-way still lands on what it was told.
fn turn(grove: &mut Grove, mote: &Mote, hue: Rgb) {
    grove.animate(
        mote.leaf,
        Motion::Color(fill(hue)),
        Timing::ms(TURN_MS)
            .after((mote.depth * TURN_SPREAD) as u64)
            .ease(Ease::Decelerate),
    );
}

/// A polygon `size` pixels across, sitting `at` pixels off the middle of the box.
fn placed((x, y): (f32, f32), size: f32) -> Location {
    Location::new().xs(
        center_x(50.0.pct() + x.px()).width(size.px()),
        center_y(50.0.pct() + y.px()).height(size.px()),
    )
}
