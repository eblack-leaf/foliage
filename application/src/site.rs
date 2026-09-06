//! The page: one leaf, and the three places it goes.
//!
//! The leaf is a mosaic of loose polygons inside a traced outline. It arrives in three passes --
//! the outline as dashes, then the tiles sweeping along the ramp they are coloured from, then what
//! is cut out of them -- and each is one [`animate`](Grow::animate) per element with a delay taken
//! from where that element sits.

use foliage::{
    Boxed, Cap, Color, Ease, Elevation, FontSize, Grove, Grow, HAIRLINE, Key, Leaf, Line, Location,
    Motion, Palette, Panel, Place, Point, Pollen, Polygon, Root, Shape, Source, Stem, Text, Timing,
    center_x, center_y, content, left, top,
};
use tracing::{debug, info};

use crate::mosaic::{Scatter, Silhouette};

/// Where the page goes, as the three tiles that go there.
struct Destination {
    /// What is cut out of the tile.
    label: &'static str,
    /// Where a press on it goes.
    url: &'static str,
    /// Where the tile sits, in unit space.
    at: (f32, f32),
    /// How far it is turned, in radians.
    turn: f32,
}

/// The three, one to a prong of the leaf.
const DESTINATIONS: [Destination; 3] = [
    Destination {
        label: "book",
        url: "https://eblack-leaf.github.io/foliage/book/",
        at: (0.200, 0.270),
        turn: 0.18,
    },
    Destination {
        label: "docs",
        url: "https://eblack-leaf.github.io/foliage/api/foliage/",
        at: (0.471, 0.153),
        turn: -0.24,
    },
    Destination {
        label: "github",
        url: "https://github.com/eblack-leaf/foliage",
        at: (0.630, 0.470),
        turn: 0.34,
    },
];

/// How wide a destination's tile is, in unit space, and how far around it the mosaic keeps clear.
const PRONG: f32 = 0.150;
const CLEAR: f32 = 0.088;

/// Where the wordmark is cut out of the leaf, in unit space.
const CUTOUT: (f32, f32) = (0.380, 0.420);

/// The ramp the leaf is coloured from: gold at the tips, deep red toward the stem.
const RAMP: [(f32, f32, f32); 4] = [
    (0.97, 0.76, 0.30),
    (0.93, 0.52, 0.17),
    (0.82, 0.28, 0.13),
    (0.58, 0.12, 0.11),
];

/// How far one tile's colour may sit off the ramp, so that no two neighbours read as one shape.
const DRIFT: f32 = 0.035;

/// How much nearer white a destination's tile is drawn while it is held.
const LIFT: f32 = 0.16;

/// What the outline is dashed in.
const EDGE: (f32, f32, f32) = (0.80, 0.40, 0.19);

/// How much of its final size a tile starts at, before it grows into the shape.
const SEED: f32 = 0.22;

/// How long one dash takes to appear, and the step from one to the next.
const EDGE_MS: u64 = 240;
const EDGE_STEP: u64 = 6;

/// When the mosaic begins, and how long its sweep along the ramp takes.
const MOSAIC_AT: u64 = 620;
const MOSAIC_SPREAD: f32 = 1150.0;

/// How long one tile takes to fade in, and to grow into its shape.
const TILE_MS: u64 = 300;
const GROW_MS: u64 = 540;

/// When what is cut out of the leaf appears, and how long it takes.
const CUTOUT_AT: u64 = 2200;
const CUTOUT_MS: u64 = 420;

/// The app.
///
/// Whatever it keeps is its own. Nothing here is handed to the engine, and the engine has no way to
/// reach it: the value stays on this side and touches the tree only through the [`Grove`] it is
/// lent for the length of a frame.
pub(crate) struct Site {
    /// The three tiles that go somewhere, in the order [`DESTINATIONS`] gives them.
    prongs: [Prong; DESTINATIONS.len()],
}

/// One destination, as the page holds it once it is grown.
struct Prong {
    /// The tile itself, which is the whole of what receives a gesture.
    tile: Leaf,
    /// Where a press on it goes.
    url: &'static str,
    /// What it is drawn in at rest, and while it is held.
    rest: Color,
    lit: Color,
}

impl Root for Site {
    fn take_root(grove: &mut Grove) -> Self {
        let silhouette = Silhouette::traced();
        let height = silhouette.height();
        let page = grove.plant(Panel::new().color(Palette::Surface));
        let leaf = grove.branch(page, Stem::new().at(frame(height)));

        // The outline, drawn first and left in front of the fill, so the shape is legible before
        // there is anything inside it and stays legible after.
        let dashes = silhouette.dashes();
        for (n, (from, to)) in dashes.iter().enumerate() {
            let dash = grove.branch(
                leaf,
                Line::new()
                    .between(spot(height, *from), spot(height, *to))
                    .weight(HAIRLINE * 1.5)
                    .color(fill(EDGE))
                    .cap(Cap::Round)
                    .opacity(0.0)
                    .elevate(Elevation::up(2)),
            );
            grove.animate(
                dash,
                Motion::Opacity(1.0),
                Timing::ms(EDGE_MS)
                    .after(n as u64 * EDGE_STEP)
                    .ease(Ease::Decelerate),
            );
        }

        // The fill. Each tile is one colour and carries no gradient of its own; what builds one is
        // where each sits on the ramp, and the delay taken from the same number sweeps the leaf in
        // the order the ramp runs.
        let taken: Vec<((f32, f32), f32)> = DESTINATIONS
            .iter()
            .map(|destination| (destination.at, CLEAR))
            .collect();
        let tiles = silhouette.tiles(&taken);
        let ramp = Ramp::over(
            height,
            tiles
                .iter()
                .map(|tile| tile.center)
                .chain(DESTINATIONS.iter().map(|destination| destination.at))
                .collect::<Vec<_>>()
                .into_iter(),
        );
        let mut scatter = Scatter::new();
        for tile in &tiles {
            let warmth = ramp.warmth(height, tile.center);
            let hue = shifted(ember(warmth), scatter.between(-DRIFT, DRIFT));
            let after = MOSAIC_AT + (warmth * MOSAIC_SPREAD) as u64 + scatter.between(0.0, 90.0) as u64;
            let polygon = grove.branch(
                leaf,
                Polygon::new()
                    .sides(3.0)
                    .rounding(0.85)
                    .rotation(tile.rotation - 0.7)
                    .color(fill(hue))
                    .intangible()
                    .opacity(0.0)
                    .at(placed(height, tile.center, tile.size * SEED)),
            );
            morph(
                grove,
                polygon,
                placed(height, tile.center, tile.size),
                Shape {
                    sides: tile.sides,
                    rounding: tile.rounding,
                    rotation: tile.rotation,
                },
                after,
            );
        }

        // The three that go somewhere: the same tiles, larger, in the hit test, and each carrying
        // its own cutout.
        let prongs = std::array::from_fn(|n| {
            let destination = &DESTINATIONS[n];
            let warmth = ramp.warmth(height, destination.at);
            let hue = ember(warmth);
            let (rest, lit) = (fill(hue), fill(shifted(hue, LIFT)));
            let after = MOSAIC_AT + (warmth * MOSAIC_SPREAD) as u64;
            let tile = grove.branch(
                leaf,
                Polygon::new()
                    .sides(3.0)
                    .rounding(0.85)
                    .rotation(destination.turn - 0.7)
                    .color(rest)
                    // What puts it in the hit test at all, against the ellipse its own shape sits
                    // in rather than the box around it.
                    .interactive()
                    .round_hit_area()
                    .opacity(0.0)
                    .elevate(Elevation::up(1))
                    .at(placed(height, destination.at, PRONG * SEED)),
            );
            morph(
                grove,
                tile,
                placed(height, destination.at, PRONG),
                Shape {
                    sides: 6.0,
                    rounding: 0.16,
                    rotation: destination.turn,
                },
                after,
            );
            // Grown under the tile, so it travels with it while it is still arriving, and filled
            // with the page's own tone, so it reads as a hole rather than as a label.
            let label = grove.branch(
                tile,
                Text::new(destination.label)
                    .color(Palette::Surface)
                    .font_size(FontSize::new().xs(10).sm(12).md(16).lg(21).xl(25).short(11))
                    .intangible()
                    .opacity(0.0)
                    .at(Location::new().xs(
                        center_x(50.pct()).width(content()),
                        center_y(50.pct()).height(content()),
                    )),
            );
            reveal(grove, label);
            Prong {
                tile,
                url: destination.url,
                rest,
                lit,
            }
        });

        let cutout = grove.branch(
            leaf,
            Text::new("FOLIAGE")
                .color(Palette::Surface)
                .font_size(FontSize::new().xs(20).sm(24).md(34).lg(44).xl(52).short(22))
                .intangible()
                .opacity(0.0)
                .elevate(Elevation::up(3))
                .at(Location::new().xs(
                    center_x((CUTOUT.0 * 100.0).pct()).width(content()),
                    center_y((CUTOUT.1 / height * 100.0).pct()).height(content()),
                )),
        );
        reveal(grove, cutout);

        info!(
            dashes = dashes.len(),
            tiles = tiles.len(),
            "leaf grown"
        );
        Site { prongs }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        for prong in &self.prongs {
            if pollen.engaged(prong.tile) {
                grove.animate(prong.tile, Motion::Color(prong.lit), Timing::ms(120));
            }
            if pollen.disengaged(prong.tile) {
                grove.animate(prong.tile, Motion::Color(prong.rest), Timing::ms(200));
            }
            // A tile is a link, so the keyboard reaches it the way the pointer does. Focus rests
            // only on what asked to receive input, which is the tile and nothing over it.
            let entered = pollen
                .keys(prong.tile)
                .iter()
                .any(|stroke| stroke.key == Key::Enter);
            if pollen.clicked(prong.tile) || entered {
                debug!(url = prong.url, "leaving");
                grove.navigate(prong.url);
            }
        }
    }
}

/// Brings one tile in: it fades, grows into its box, and settles into its shape together.
///
/// Three motions rather than one because they are three properties, and one delay because they are
/// one arrival. Each is grown at what it is leaving, so the animation carries the whole of the
/// difference and nothing has to be written back when it lands.
fn morph(grove: &mut Grove, tile: Leaf, into: Location, shape: Shape, after: u64) {
    grove.animate(
        tile,
        Motion::Opacity(1.0),
        Timing::ms(TILE_MS).after(after).ease(Ease::Decelerate),
    );
    grove.animate(
        tile,
        Motion::Location(into),
        Timing::ms(GROW_MS).after(after).ease(Ease::Decelerate),
    );
    grove.animate(
        tile,
        Motion::Polygon(shape),
        Timing::ms(GROW_MS).after(after).ease(Ease::Decelerate),
    );
}

/// Fades in what is cut out of the leaf, once the leaf is there to cut it out of.
fn reveal(grove: &mut Grove, leaf: Leaf) {
    grove.animate(
        leaf,
        Motion::Opacity(1.0),
        Timing::ms(CUTOUT_MS).after(CUTOUT_AT).ease(Ease::Decelerate),
    );
}

/// How large the leaf is drawn, per breakpoint.
///
/// Only the width is stated. The height follows from the proportions the outline was traced in, so
/// the mosaic inside it -- every part of which is a fraction of this box -- is never distorted.
fn frame(height: f32) -> Location {
    let across = |width: f32| center_x(50.pct()).width(width.px());
    let down = |width: f32| center_y(50.pct()).height((width * height).px());
    Location::new()
        .xs(across(312.0), down(312.0))
        .sm(across(380.0), down(380.0))
        .md(across(540.0), down(540.0))
        .lg(across(720.0), down(720.0))
        .xl(across(840.0), down(840.0))
        .short(across(340.0), down(340.0))
}

/// A square in unit space, as the placement the leaf's own box resolves it against.
fn placed(height: f32, (x, y): (f32, f32), size: f32) -> Location {
    Location::new().xs(
        left(((x - size / 2.0) * 100.0).pct()).width((size * 100.0).pct()),
        top(((y - size / 2.0) / height * 100.0).pct()).height((size / height * 100.0).pct()),
    )
}

/// A point in unit space, as the coordinate a stroke's end is stated in.
fn spot(height: f32, (x, y): (f32, f32)) -> Point {
    Point::new((x * 100.0).pct(), (y / height * 100.0).pct())
}

/// Where a point sits across the leaf: `0.0` at the top and the tips, `1.0` toward the stem.
///
/// Unnormalised. A leaf is not a rectangle, so the corners of the box this is stated over are not
/// in the shape and the raw reading never reaches either end -- which is what [`Ramp`] is for.
fn along(height: f32, (x, y): (f32, f32)) -> f32 {
    0.55 * x + 0.45 * (y / height)
}

/// The range the leaf actually occupies across the ramp.
///
/// Measured from the tiles rather than assumed, so the gold end and the deep red end are both
/// reached: a ramp read against the box would spend its ends on corners the shape does not have.
struct Ramp {
    low: f32,
    span: f32,
}

impl Ramp {
    /// The range `centers` cover.
    fn over(height: f32, centers: impl Iterator<Item = (f32, f32)> + Clone) -> Self {
        let reading = |center| along(height, center);
        let low = centers.clone().map(reading).fold(f32::MAX, f32::min);
        let high = centers.map(reading).fold(f32::MIN, f32::max);
        Self {
            low,
            span: (high - low).max(f32::EPSILON),
        }
    }

    /// Where a point sits on the ramp, in `0.0..=1.0`.
    fn warmth(&self, height: f32, center: (f32, f32)) -> f32 {
        ((along(height, center) - self.low) / self.span).clamp(0.0, 1.0)
    }
}

/// The colour at `warmth` along the ramp.
fn ember(warmth: f32) -> (f32, f32, f32) {
    let steps = (RAMP.len() - 1) as f32;
    let at = (warmth * steps).floor().min(steps - 1.0);
    let into = warmth * steps - at;
    let (from, to) = (RAMP[at as usize], RAMP[at as usize + 1]);
    (
        from.0 + (to.0 - from.0) * into,
        from.1 + (to.1 - from.1) * into,
        from.2 + (to.2 - from.2) * into,
    )
}

/// The same colour nearer white, or nearer black where `by` is negative.
fn shifted((red, green, blue): (f32, f32, f32), by: f32) -> (f32, f32, f32) {
    (
        (red + by).clamp(0.0, 1.0),
        (green + by).clamp(0.0, 1.0),
        (blue + by).clamp(0.0, 1.0),
    )
}

/// A colour off the ramp, as what an element is filled with.
///
/// Stated outright rather than as a [`Palette`] role: a ramp of this many steps is not a scheme,
/// and a tile that followed one would lose the gradient the mosaic is built to carry.
fn fill((red, green, blue): (f32, f32, f32)) -> Color {
    Color::rgb(red, green, blue)
}
