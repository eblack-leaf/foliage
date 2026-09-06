//! The page: one leaf, and the three places it goes.
//!
//! The leaf is a mosaic of loose polygons inside a traced outline. It arrives in three passes --
//! the outline as dashes, then the tiles sweeping along the ramp they are coloured from, then what
//! is cut out of them -- and each is one [`animate`](Grow::animate) per element with a delay taken
//! from where that element sits.
//!
//! Once the whole of that is over, a gust crosses the leaf and it settles: the polygons are carried
//! and turned, let go past where they rest, and brought back.

use foliage::{
    Boxed, Cap, Color, Ease, Elevation, FontSize, Grove, Grow, HAIRLINE, Key, Leaf, Line, Location,
    Motion, Palette, Panel, Place, Point, Pollen, Polygon, Root, Sequence, Shape, Source, Stem,
    Text, Timing, Tween, anchor, center_x, center_y, content, left, right, top,
};
use tracing::{debug, info};

use crate::mosaic::{Scatter, Silhouette};

/// Where the page goes, as the three tiles that go there.
struct Destination {
    /// What is cut out of the tile.
    label: &'static str,
    /// Where a press on it goes.
    url: &'static str,
    /// Where the tile sits, in the sketch's own terms, which the leaf converts.
    at: (f32, f32),
    /// How far it is turned, in radians.
    turn: f32,
}

/// The three, one to a prong of the leaf.
const DESTINATIONS: [Destination; 3] = [
    Destination {
        label: "book",
        url: "https://eblack-leaf.github.io/foliage/book/",
        at: (0.232, 0.404),
        turn: 0.18,
    },
    Destination {
        label: "docs",
        url: "https://eblack-leaf.github.io/foliage/api/foliage/",
        at: (0.480, 0.260),
        turn: -0.24,
    },
    Destination {
        label: "github",
        url: "https://github.com/eblack-leaf/foliage",
        at: (0.610, 0.630),
        turn: 0.34,
    },
];

/// How wide a destination's tile is, in unit space, and how far around it the mosaic keeps clear.
///
/// A quarter of the leaf's width. Nothing is drawn over the middle of the leaf any more, so a
/// destination is as large as the shape will hold rather than as large as what shared it allowed.
const PRONG: f32 = 0.250;
const CLEAR: f32 = 0.135;

/// Where the wordmark sits against the leaf, as fractions of the leaf's own box: how far in from
/// its left edge the wordmark ends, and how far down the box it is centred.
///
/// Inside the box rather than beside it. A leaf is not a rectangle, and the corner above its widest
/// point is empty at every size it is drawn -- so the wordmark tucks into the notch beside the tip,
/// where it costs the shape nothing and gives the leaf back the room it used to be set aside.
const NOTCH: (f32, f32) = (0.28, 0.14);

/// The band of the leaf that carries anything, as fractions of its height: from the top of the
/// wordmark down to the bottom of the lowest destination.
///
/// Above it is the tip and below it is the stem, and there is nothing in either. So this is what is
/// centred in the viewport rather than the box -- a window too short for the whole leaf then runs
/// out of leaf at the two ends that can afford it, instead of cutting the wordmark off the top.
const BAND: (f32, f32) = (0.100, 0.695);

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

/// How long one dash takes to appear, and how long the trace takes to go all the way round.
///
/// The sweep is stated rather than a step from one dash to the next, so how long the outline takes
/// is what it says here whatever the perimeter divides into: the shape reads as being drawn at one
/// speed, and the pause before the fill stays the pause it was written as.
const EDGE_MS: u64 = 300;
const EDGE_SWEEP: u64 = 1240;

/// When the mosaic begins, and how long its sweep along the ramp takes.
///
/// After the outline is closed, with a beat to spare: the shape is what the page is, and the fill
/// arriving into a leaf already drawn reads as filling one rather than as forming one.
const MOSAIC_AT: u64 = 1800;
const MOSAIC_SPREAD: f32 = 1150.0;

/// How long one tile takes to fade in, and to grow into its shape.
const TILE_MS: u64 = 300;
const GROW_MS: u64 = 540;

/// When what is cut out of the leaf appears, and how long it takes.
const CUTOUT_AT: u64 = 3400;
const CUTOUT_MS: u64 = 420;

/// How far the wind carries a polygon at the height of the gust, in unit space, and how far it
/// turns one, in radians.
///
/// Stated for the tip, which is the free end. What sits nearer the stem gets a fraction of it.
const SWAY: f32 = 0.013;
const SPIN: f32 = 0.22;

/// How much of the sway the stem end keeps.
///
/// The blade is held at the bottom, so the tip travels and the stem barely does. Not zero: a mosaic
/// with a still half in it reads as half a gust.
const HELD: f32 = 0.30;

/// Which way the gust runs, as a direction in unit space: across the leaf, and pressing down.
const WIND: (f32, f32) = (0.94, 0.34);

/// How much of the room the outline leaves a polygon it may be carried into.
///
/// The outline is drawn once and does not move, so nothing inside it may be carried out of it: what
/// sits close to an edge is carried little, however far the wind would otherwise take it.
const KEEP: f32 = 0.30;

/// One pass of the gust.
struct Pass {
    /// How far it carries a polygon, against that polygon's own sway. Negative is back the way the
    /// wind came, which is what something let go of does.
    carry: f32,
    /// How long one polygon takes to answer it.
    ms: u64,
    /// How long the front takes to cross the leaf.
    sweep: u64,
    /// The shape a polygon moves in.
    ease: Ease,
}

/// The gust, in order: it arrives, it lets go, and the leaf settles.
///
/// Three rather than one, because a leaf struck by wind does not return by the way it went: it is
/// carried, released past where it rests, and comes back the rest of the way with nothing behind
/// it. Each pass is weaker and quicker to cross than the one before.
const GUST: [Pass; 3] = [
    Pass {
        carry: 1.0,
        ms: 340,
        sweep: 520,
        ease: Ease::Emphasis,
    },
    Pass {
        carry: -0.42,
        ms: 430,
        sweep: 360,
        ease: Ease::Emphasis,
    },
    Pass {
        carry: 0.0,
        ms: 640,
        sweep: 240,
        ease: Ease::Decelerate,
    },
];

/// The app.
///
/// Whatever it keeps is its own. Nothing here is handed to the engine, and the engine has no way to
/// reach it: the value stays on this side and touches the tree only through the [`Grove`] it is
/// lent for the length of a frame.
pub(crate) struct Site {
    /// The three tiles that go somewhere, in the order [`DESTINATIONS`] gives them.
    prongs: [Prong; DESTINATIONS.len()],
    /// Every polygon the wind reaches -- the mosaic and the three alike -- in the order they were
    /// grown.
    blades: Vec<Blade>,
    /// How tall the leaf is for every unit of its width, which is what a placement in unit space is
    /// stated against. Kept because every pass of the gust states a new one.
    height: f32,
    /// The whole arrival, as one group. The gust is let go the frame it is over.
    grown: Sequence,
    /// How many polygons the gust is still carrying, so that the leaf settling is one report rather
    /// than one per polygon and a settled leaf costs nothing per frame.
    rustling: usize,
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

/// One polygon of the leaf, as the wind holds it.
struct Blade {
    /// The polygon itself.
    leaf: Leaf,
    /// Where it sits with nothing blowing, and how large it is. Both, because a placement is one
    /// value: a pass states a whole one rather than an offset from the last.
    rest: (f32, f32),
    size: f32,
    /// The shape it settled into, which is what a pass turns it from.
    shape: Shape,
    /// What the gust does to it.
    wind: Wind,
    /// Where it is in the gust.
    rustle: Rustle,
}

/// What the gust does to one polygon, which is a question of where that polygon sits.
struct Wind {
    /// How far it is carried at the height of the gust, in unit space.
    carried: (f32, f32),
    /// How far it is turned with that, in radians.
    spin: f32,
    /// Where it stands in the front, `0.0` where the gust arrives and `1.0` where it leaves.
    front: f32,
}

/// Where one polygon is in the gust.
#[derive(Copy, Clone)]
enum Rustle {
    /// Nothing to advance: it is still arriving, or it has settled.
    Still,
    /// Answering a pass of [`GUST`], and the motion whose end carries it into the next.
    Carried { pass: usize, moving: Tween },
}

impl Root for Site {
    fn take_root(grove: &mut Grove) -> Self {
        let silhouette = Silhouette::traced();
        let height = silhouette.height();
        let page = grove.plant(Panel::new().color(Palette::Surface));
        let leaf = grove.branch(page, Stem::new().at(frame(height)));
        // Everything the leaf arrives by is counted into one group, so what follows it waits for
        // the whole arrival rather than for whichever part of it was written last.
        let grown = grove.sequence();

        // The outline, drawn first and left in front of the fill, so the shape is legible before
        // there is anything inside it and stays legible after.
        let dashes = silhouette.dashes();
        let trace = EDGE_SWEEP as f32 / dashes.len() as f32;
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
                    .after((n as f32 * trace) as u64)
                    .ease(Ease::Decelerate)
                    .within(grown),
            );
        }

        // The fill. Each tile is one colour and carries no gradient of its own; what builds one is
        // where each sits on the ramp, and the delay taken from the same number sweeps the leaf in
        // the order the ramp runs.
        let taken: Vec<((f32, f32), f32)> = DESTINATIONS
            .iter()
            .map(|destination| (silhouette.at(destination.at), CLEAR))
            .collect();
        let tiles = silhouette.tiles(&taken);
        let ramp = Ramp::over(
            height,
            tiles
                .iter()
                .map(|tile| tile.center)
                .chain(
                    DESTINATIONS
                        .iter()
                        .map(|destination| silhouette.at(destination.at)),
                )
                .collect::<Vec<_>>()
                .into_iter(),
        );
        let mut scatter = Scatter::new();
        let mut blades = Vec::with_capacity(tiles.len() + DESTINATIONS.len());
        for tile in &tiles {
            let warmth = ramp.warmth(height, tile.center);
            let hue = shifted(ember(warmth), scatter.between(-DRIFT, DRIFT));
            let after = MOSAIC_AT + (warmth * MOSAIC_SPREAD) as u64 + scatter.between(0.0, 90.0) as u64;
            let shape = Shape {
                sides: tile.sides,
                rounding: tile.rounding,
                rotation: tile.rotation,
            };
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
                shape,
                after,
                grown,
            );
            blades.push(Blade {
                leaf: polygon,
                rest: tile.center,
                size: tile.size,
                shape,
                wind: Wind::on(&silhouette, height, tile.center, &mut scatter),
                rustle: Rustle::Still,
            });
        }

        // The three that go somewhere: the same tiles, larger, in the hit test, and each carrying
        // its own cutout.
        let prongs = std::array::from_fn(|n| {
            let destination = &DESTINATIONS[n];
            let at = silhouette.at(destination.at);
            let warmth = ramp.warmth(height, at);
            let hue = ember(warmth);
            let (rest, lit) = (fill(hue), fill(shifted(hue, LIFT)));
            let after = MOSAIC_AT + (warmth * MOSAIC_SPREAD) as u64;
            let shape = Shape {
                sides: 6.0,
                rounding: 0.16,
                rotation: destination.turn,
            };
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
                    .at(placed(height, at, PRONG * SEED)),
            );
            morph(grove, tile, placed(height, at, PRONG), shape, after, grown);
            blades.push(Blade {
                leaf: tile,
                rest: at,
                size: PRONG,
                shape,
                wind: Wind::on(&silhouette, height, at, &mut scatter).unturned(),
                rustle: Rustle::Still,
            });
            // Grown under the tile, so it travels with it while it is still arriving, and filled
            // with the page's own tone, so it reads as a hole rather than as a label.
            let label = grove.branch(
                tile,
                Text::new(destination.label)
                    .color(Palette::Surface)
                    .font_size(FontSize::new().xs(11).sm(13).md(18).lg(21).xl(24).short(11))
                    .intangible()
                    .opacity(0.0)
                    .at(Location::new().xs(
                        center_x(50.pct()).width(content()),
                        center_y(50.pct()).height(content()),
                    )),
            );
            reveal(grove, label, grown);
            Prong {
                tile,
                url: destination.url,
                rest,
                lit,
            }
        });

        // Over the leaf's own box rather than beside it, and its own element rather than a hole in
        // it: a cutout reads as one only where there is fill behind every glyph, which the tips and
        // the notches of a leaf do not offer. Stated in fractions of the box it is set against, so
        // the two read as one composition at every size, and set in the corner the blade leaves
        // empty, so what the wordmark takes is room the leaf was never using.
        let wordmark = grove.branch(
            page,
            Text::new("foliage")
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(22).sm(28).md(36).lg(46).xl(56).short(28))
                .intangible()
                .opacity(0.0)
                .anchored(leaf)
                .at(Location::new().xs(
                    right(anchor().left() + anchor().width() * NOTCH.0).width(content()),
                    center_y(anchor().top() + anchor().height() * NOTCH.1).height(content()),
                )),
        );
        reveal(grove, wordmark, grown);

        info!(
            dashes = dashes.len(),
            tiles = tiles.len(),
            "leaf grown"
        );
        Site {
            prongs,
            blades,
            height,
            grown,
            rustling: 0,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        self.rustle(grove, &pollen);
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

impl Site {
    /// Runs the gust: it is let go the frame the leaf has finished arriving, and each polygon takes
    /// the next pass the frame the one it is answering lands.
    ///
    /// Chained per polygon rather than per pass, which is what keeps the front travelling: the tip
    /// is already coming back while the stem is still being pushed. A pass that waited on the whole
    /// of the last would hold the leaf still between them, and a gust does not stop.
    fn rustle(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if pollen.sequence_finished(self.grown) {
            debug!(polygons = self.blades.len(), "gust");
            self.rustling = self.blades.len();
            for blade in &mut self.blades {
                blade.answer(grove, self.height, 0);
            }
            return;
        }
        if self.rustling == 0 {
            return;
        }
        for blade in &mut self.blades {
            let Rustle::Carried { pass, moving } = blade.rustle else {
                continue;
            };
            if !pollen.finished(moving) {
                continue;
            }
            match pass + 1 < GUST.len() {
                true => blade.answer(grove, self.height, pass + 1),
                false => {
                    blade.rustle = Rustle::Still;
                    self.rustling -= 1;
                }
            }
        }
        if self.rustling == 0 {
            info!("leaf settled");
        }
    }
}

impl Blade {
    /// Answers one pass of the gust: carried to where that pass leaves it, turned with it, and left
    /// waiting on the motion that gets it there.
    ///
    /// Both motions state where the polygon is going rather than how far it has been moved, and the
    /// last pass carries it by nothing -- so the leaf settles on exactly what it was grown at, and
    /// a gust cannot walk it anywhere over however many passes it is given.
    fn answer(&mut self, grove: &mut Grove, height: f32, pass: usize) {
        let gust = &GUST[pass];
        let timing = Timing::ms(gust.ms)
            .after((self.wind.front * gust.sweep as f32) as u64)
            .ease(gust.ease);
        let carried = (
            self.rest.0 + self.wind.carried.0 * gust.carry,
            self.rest.1 + self.wind.carried.1 * gust.carry,
        );
        let moving = grove.animate(
            self.leaf,
            Motion::Location(placed(height, carried, self.size)),
            timing,
        );
        // Nothing is queued: a second motion on a property replaces the first, so the turn is only
        // stated for what turns at all, and the chain hangs off the one motion every polygon has.
        if self.wind.spin != 0.0 {
            grove.animate(
                self.leaf,
                Motion::Polygon(Shape {
                    rotation: self.shape.rotation + self.wind.spin * gust.carry,
                    ..self.shape
                }),
                timing,
            );
        }
        self.rustle = Rustle::Carried { pass, moving };
    }
}

impl Wind {
    /// What the gust does to something sitting at `center`.
    ///
    /// Mostly a question of how far up the leaf it is: the blade is held at the stem, so the tip
    /// travels and the foot of it barely does. The rest is the room the outline leaves it, which is
    /// what keeps the fill inside an edge that does not move, and a jitter, which is what keeps a
    /// gust from reading as one rigid sheet.
    fn on(silhouette: &Silhouette, height: f32, center: (f32, f32), scatter: &mut Scatter) -> Self {
        let free = 1.0 - center.1 / height;
        let reach = (SWAY * (HELD + (1.0 - HELD) * free) * scatter.between(0.75, 1.25))
            .min(silhouette.clearance(center) * KEEP);
        Self {
            carried: (WIND.0 * reach, WIND.1 * reach),
            // Against the sway it was allowed rather than the sway it asked for, so what the
            // outline holds back is held back whole rather than pinned in place and left spinning.
            spin: SPIN * (reach / SWAY) * scatter.between(-1.0, 1.0),
            // Across the leaf, and a little up it: the gust reaches the tip before the stem.
            front: 0.78 * center.0 + 0.22 * free,
        }
    }

    /// The same, for a tile that carries a word: turned by nothing.
    ///
    /// A cutout is set square in its tile and does not turn with it, so a tile that turned under one
    /// would read as a crooked hole rather than as a tile in the wind.
    fn unturned(mut self) -> Self {
        self.spin = 0.0;
        self
    }
}

/// Brings one tile in: it fades, grows into its box, and settles into its shape together.
///
/// Three motions rather than one because they are three properties, and one delay because they are
/// one arrival. Each is grown at what it is leaving, so the animation carries the whole of the
/// difference and nothing has to be written back when it lands.
fn morph(
    grove: &mut Grove,
    tile: Leaf,
    into: Location,
    shape: Shape,
    after: u64,
    grown: Sequence,
) {
    grove.animate(
        tile,
        Motion::Opacity(1.0),
        Timing::ms(TILE_MS)
            .after(after)
            .ease(Ease::Decelerate)
            .within(grown),
    );
    grove.animate(
        tile,
        Motion::Location(into),
        Timing::ms(GROW_MS)
            .after(after)
            .ease(Ease::Decelerate)
            .within(grown),
    );
    grove.animate(
        tile,
        Motion::Polygon(shape),
        Timing::ms(GROW_MS)
            .after(after)
            .ease(Ease::Decelerate)
            .within(grown),
    );
}

/// Fades in what is cut out of the leaf, once the leaf is there to cut it out of.
fn reveal(grove: &mut Grove, leaf: Leaf, grown: Sequence) {
    grove.animate(
        leaf,
        Motion::Opacity(1.0),
        Timing::ms(CUTOUT_MS)
            .after(CUTOUT_AT)
            .ease(Ease::Decelerate)
            .within(grown),
    );
}

/// How large the leaf is drawn, per breakpoint, and where its box sits.
///
/// Only the width is stated. The height follows from the proportions the outline was traced in, so
/// the mosaic inside it -- every part of which is a fraction of this box -- is never distorted. Each
/// width is one whose [`BAND`] fits the shortest viewport its breakpoint can see, which is the
/// 400 logical pixels below which the engine reads the viewport as cramped and `short` takes over.
fn frame(height: f32) -> Location {
    // The wordmark ends inside the leaf's box, so the only part of it outside the composition is
    // what reaches past the leaf's left edge, and the leaf moves right by half of that: the pair is
    // centred rather than the leaf alone. The second number is the wordmark's width -- seven
    // characters of the size it is set at for that breakpoint -- so the two are stated together and
    // cannot drift apart.
    let across = |width: f32, wordmark: f32| {
        let over = (wordmark - NOTCH.0 * width).max(0.0);
        center_x(50.pct() + (over / 2.0).px()).width(width.px())
    };
    // The box is centred on the band rather than on itself, so it moves down by however far its own
    // middle sits above the band's. What that costs is leaf off the top and the bottom, which is
    // what the two ends of the shape are there to give.
    let down = |width: f32| {
        let carried = (0.5 - (BAND.0 + BAND.1) / 2.0) * width * height;
        center_y(50.pct() + carried.px()).height((width * height).px())
    };
    Location::new()
        .xs(across(240.0, 98.0), down(240.0))
        .sm(across(230.0, 119.0), down(230.0))
        .md(across(330.0, 154.0), down(330.0))
        .lg(across(400.0, 196.0), down(400.0))
        .xl(across(430.0, 238.0), down(430.0))
        .short(across(215.0, 119.0), down(215.0))
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

/// Where a point sits along the leaf: `0.0` at the tip, `1.0` toward the stem.
///
/// Unnormalised. A leaf is not a rectangle, so the corners of the box this is stated over are not
/// in the shape and the raw reading never reaches either end -- which is what [`Ramp`] is for.
fn along(height: f32, (x, y): (f32, f32)) -> f32 {
    // Mostly down the leaf, now that the leaf stands up: gold at the tip, deepening toward the
    // stem. The across term is what keeps the bands from reading as stripes.
    0.25 * x + 0.75 * (y / height)
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
