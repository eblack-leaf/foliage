//! A silhouette filled with tiles.

use foliage::{
    Boxed, Cap, Ease, Elevation, Grove, Grow, HAIRLINE, Leaf, Line, Location, Motion, Place, Point,
    Pollen, Polygon, Sequence, Shape, Source, Stem, Timing, left, top,
};

use crate::{Ramp, Rgb, Scatter, Silhouette, fill, shifted};

/// How far one tile's colour may sit off the ramp, so that no two neighbours read as one shape.
const DRIFT: f32 = 0.035;

/// How much of its final size a tile starts at, before it grows into the shape.
const SEED: f32 = 0.22;

/// How long one dash takes to appear, and how long the trace takes to go all the way round.
const EDGE_MS: u64 = 240;
const EDGE_SWEEP: u64 = 620;

/// When the tiles begin, and how long their sweep along the ramp takes.
const MOSAIC_AT: u64 = 860;
const MOSAIC_SPREAD: f32 = 600.0;

/// How long one tile takes to fade in, and to grow into its shape.
const TILE_MS: u64 = 240;
const GROW_MS: u64 = 400;

/// How long one tile takes to turn, and how long the turn takes to cross a region.
const EMPHASIS_MS: u64 = 260;
const EMPHASIS_SPREAD: f32 = 220.0;

/// A [`Silhouette`] outlined in dashes and filled with polygons coloured along a [`Ramp`].
///
/// Built ungrown, which costs nothing, and grown once into a box. [`form`](Self::form) brings it
/// in -- the outline arrives as dashes, then the tiles sweep in along the ramp -- and
/// [`frame`](Self::frame) carries that until [`formed`](Self::formed) says it is over.
///
/// [`regions`](Self::regions) stands a square on each vertex the caller names, and hands back a
/// stem over each to hang something off. [`emphasize`](Self::emphasize) recolours the tiles under
/// one of those onto another ramp until [`unemphasize`](Self::unemphasize) puts them back.
pub struct Mosaic {
    silhouette: Silhouette,
    ramp: Ramp,
    /// Which way the ramp runs across the shape, as a direction across its box.
    along: (f32, f32),
    /// What the outline is dashed in.
    edge: Rgb,
    /// Tile spacing, as a fraction of the shape's width.
    pitch: f32,
    /// What it is all branched from. `None` until it is grown.
    root: Option<Leaf>,
    /// The outline, and when each dash of it appears. Empty until it is grown.
    dashes: Vec<(Leaf, u64)>,
    /// Every tile, in the order they were grown. Empty until it is grown.
    tiles: Vec<Tile>,
    /// The whole arrival, as one group. `None` until it is formed.
    forming: Option<Sequence>,
    /// Whether that group has landed, latched, so that asking after the frame it lands still answers.
    arrived: bool,
}

/// One polygon of the mosaic, as grown.
struct Tile {
    /// The polygon itself.
    leaf: Leaf,
    /// Where it sits, in unit space, and how large it is.
    center: (f32, f32),
    size: f32,
    /// The shape it settles into.
    shape: Shape,
    /// When it arrives, as a delay into the form.
    after: u64,
    /// What it rests at, kept because an emphasis has to have somewhere to put it back to.
    hue: Rgb,
    /// How far off the ramp it was put, kept so that it is put the same distance off whichever ramp
    /// it is currently coloured from -- the scatter is what keeps neighbours apart, and a region
    /// that lost it would read as one shape.
    drift: f32,
    /// Which region it sits under, where it sits under one at all.
    mark: Option<Mark>,
}

/// Where one tile sits within a region.
struct Mark {
    /// Which region holds it.
    region: usize,
    /// How far out of that region's middle it sits, `0.0` at the middle to `1.0` at the rim.
    depth: f32,
}

impl Mosaic {
    /// An ungrown mosaic, to be placed by [`grow`](Self::grow).
    pub fn new(silhouette: Silhouette, ramp: Ramp) -> Self {
        let edge = ramp.at(0.5);
        Self {
            silhouette,
            ramp,
            along: (0.0, 1.0),
            edge,
            pitch: 0.068,
            root: None,
            dashes: Vec::new(),
            tiles: Vec::new(),
            forming: None,
            arrived: false,
        }
    }

    /// Which way the ramp runs across the shape, as a direction across its box with both axes read
    /// `0.0..1.0`. Top to bottom unless stated. Only the direction matters: the ramp is spread over
    /// the range the tiles actually cover along it, so both ends are reached.
    pub fn along(mut self, direction: (f32, f32)) -> Self {
        self.along = direction;
        self
    }

    /// What the outline is dashed in. The ramp's middle unless stated.
    pub fn edge(mut self, rgb: Rgb) -> Self {
        self.edge = rgb;
        self
    }

    /// Tile spacing as a fraction of the shape's width.
    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    /// Height per unit width, for sizing the box it is grown into.
    pub fn aspect(&self) -> f32 {
        self.silhouette.height
    }

    /// Grows it into `location` off `at`, and hands back what everything is branched from -- to
    /// move it, to hang something off it, or to take it away.
    ///
    /// Grown unseen: the outline is drawn but not inked and every tile is a seed of itself, so
    /// nothing shows until [`form`](Self::form) brings it in. Growing one that is already grown
    /// does nothing and hands back what it grew.
    pub fn grow(
        &mut self,
        grove: &mut Grove,
        at: Leaf,
        location: Location,
        scatter: &mut Scatter,
    ) -> Leaf {
        if let Some(root) = self.root {
            return root;
        }
        let height = self.silhouette.height;
        let root = grove.branch(at, Stem::new().at(location));

        // The outline, drawn first and left in front of the fill, so the shape is legible before
        // there is anything inside it and stays legible after.
        let dashes = self.silhouette.dashes();
        let trace = EDGE_SWEEP as f32 / dashes.len() as f32;
        self.dashes = dashes
            .iter()
            .enumerate()
            .map(|(n, (from, to))| {
                let dash = grove.branch(
                    root,
                    Line::new()
                        .between(spot(height, *from), spot(height, *to))
                        .weight(HAIRLINE * 1.5)
                        .color(fill(self.edge))
                        .cap(Cap::Round)
                        .opacity(0.0)
                        .elevate(Elevation::up(2)),
                );
                (dash, (n as f32 * trace) as u64)
            })
            .collect();

        // The fill. Each tile is one colour and carries no gradient of its own; what builds one is
        // where each sits on the ramp, and the delay taken from the same number sweeps the shape
        // in the order the ramp runs. The ramp is spread over the range the tiles actually cover
        // rather than the box, so a ramp read against corners the shape does not have is not spent
        // on them.
        let cells = self.silhouette.cells(self.pitch, scatter);
        let reading = |(x, y): (f32, f32)| self.along.0 * x + self.along.1 * (y / height);
        let low = cells
            .iter()
            .map(|cell| reading(cell.center))
            .fold(f32::MAX, f32::min);
        let high = cells
            .iter()
            .map(|cell| reading(cell.center))
            .fold(f32::MIN, f32::max);
        let span = (high - low).max(f32::EPSILON);
        self.tiles = cells
            .iter()
            .map(|cell| {
                let warmth = ((reading(cell.center) - low) / span).clamp(0.0, 1.0);
                let drift = scatter.between(-DRIFT, DRIFT);
                let hue = shifted(self.ramp.at(warmth), drift);
                let after =
                    MOSAIC_AT + (warmth * MOSAIC_SPREAD) as u64 + scatter.between(0.0, 90.0) as u64;
                let shape = Shape {
                    sides: cell.sides,
                    rounding: cell.rounding,
                    rotation: cell.rotation,
                };
                let leaf = grove.branch(
                    root,
                    Polygon::new()
                        .sides(3.0)
                        .rounding(0.85)
                        .rotation(cell.rotation - 0.7)
                        .color(fill(hue))
                        .intangible()
                        .opacity(0.0)
                        .at(placed(height, cell.center, cell.size * SEED)),
                );
                Tile {
                    leaf,
                    center: cell.center,
                    size: cell.size,
                    shape,
                    after,
                    hue,
                    drift,
                    mark: None,
                }
            })
            .collect();

        self.root = Some(root);
        root
    }

    /// A square on each named vertex -- `(vertex index, drop)` -- sized so no two meet, and a stem
    /// over each to hang something on. The tiles under a square are what `emphasize` recolours.
    ///
    /// Once, after `grow`; nothing on one that was never grown. Branched after every tile, so what
    /// is hung off a region is branched after every part of the mosaic rather than into the middle
    /// of it. Hands back the stems to keep, in the order they were named.
    pub fn regions(&mut self, grove: &mut Grove, at: &[(usize, f32)]) -> Vec<Leaf> {
        let Some(root) = self.root else {
            return Vec::new();
        };
        let (regions, size) = self.silhouette.regions(at);
        let half = size / 2.0;
        for tile in &mut self.tiles {
            // How far out of a region's middle the tile sits, along whichever axis it sits furthest
            // out on, which is `1.0` exactly at the rim. No two regions meet, so at most one
            // answers and the first that does is it.
            tile.mark = regions.iter().enumerate().find_map(|(region, center)| {
                let out = (tile.center.0 - center.0)
                    .abs()
                    .max((tile.center.1 - center.1).abs());
                (out <= half).then_some(Mark {
                    region,
                    depth: out / half,
                })
            });
        }
        let height = self.silhouette.height;
        regions
            .iter()
            .map(|&center| grove.branch(root, Stem::new().at(placed(height, center, size))))
            .collect()
    }

    /// Brings it in: the outline dashes round, then the tiles sweep in along the ramp.
    ///
    /// Counted into one group, so what waits on the arrival waits on the whole of it rather than on
    /// whichever part of it was written last. Forming one that has already been formed does nothing,
    /// and forming one that has not been grown does nothing.
    pub fn form(&mut self, grove: &mut Grove) {
        if self.root.is_none() || self.forming.is_some() {
            return;
        }
        let forming = grove.sequence();
        for (dash, after) in &self.dashes {
            grove.animate(
                *dash,
                Motion::Opacity(1.0),
                Timing::ms(EDGE_MS)
                    .after(*after)
                    .ease(Ease::Decelerate)
                    .within(forming),
            );
        }
        let height = self.silhouette.height;
        for tile in &self.tiles {
            // Three motions rather than one because they are three properties, and one delay
            // because they are one arrival. Each was grown at what it is leaving, so the animation
            // carries the whole of the difference and nothing has to be written back when it lands.
            let timing = |ms: u64| {
                Timing::ms(ms)
                    .after(tile.after)
                    .ease(Ease::Decelerate)
                    .within(forming)
            };
            grove.animate(tile.leaf, Motion::Opacity(1.0), timing(TILE_MS));
            grove.animate(
                tile.leaf,
                Motion::Location(placed(height, tile.center, tile.size)),
                timing(GROW_MS),
            );
            grove.animate(tile.leaf, Motion::Polygon(tile.shape), timing(GROW_MS));
        }
        self.forming = Some(forming);
    }

    /// Whether the arrival is over -- the outline closed, the mosaic filled.
    ///
    /// True from that frame on, not only on it, so it can be asked whenever whatever waits on the
    /// mosaic is ready rather than only on the one frame the arrival lands. False until it is formed.
    pub fn formed(&self, pollen: &Pollen) -> bool {
        self.arrived
            || self
                .forming
                .is_some_and(|forming| pollen.sequence_finished(forming))
    }

    /// Carries the arrival. Call once a frame while it is on screen.
    pub fn frame(&mut self, pollen: &Pollen) {
        if !self.arrived && self.formed(pollen) {
            self.arrived = true;
        }
    }

    /// Recolours the tiles under one region onto `ramp`, palest at its middle and deepest at the
    /// rim, swept out from the middle.
    ///
    /// Read outward from the region rather than along the mosaic, so the colour reads as something
    /// sitting on that region rather than as a stretch of the mosaic's own ramp recoloured. Only the
    /// colour moves. Emphasizing a region that is already emphasized runs it again from wherever it
    /// currently is; emphasizing one on a mosaic with no regions does nothing.
    pub fn emphasize(&self, grove: &mut Grove, region: usize, ramp: &Ramp) {
        for tile in &self.tiles {
            let Some(mark) = tile.mark.as_ref().filter(|mark| mark.region == region) else {
                continue;
            };
            tint(
                grove,
                tile.leaf,
                shifted(ramp.at(mark.depth), tile.drift),
                mark.depth,
            );
        }
    }

    /// Puts one region back on the ramp it was grown from, drawn in from the rim.
    ///
    /// Every tile is returned to its own colour rather than to the ramp read afresh, so however
    /// often a region is emphasized it comes back to exactly the mosaic that was grown.
    /// Unemphasizing a region that is not emphasized states what it is already at, which is nothing
    /// to look at.
    pub fn unemphasize(&self, grove: &mut Grove, region: usize) {
        for tile in &self.tiles {
            let Some(mark) = tile.mark.as_ref().filter(|mark| mark.region == region) else {
                continue;
            };
            tint(grove, tile.leaf, tile.hue, 1.0 - mark.depth);
        }
    }
}

/// Moves one tile onto a colour, `front` of the way into the sweep that carries it there.
///
/// Stated as where the colour is going rather than as a step from where it is, so the same call
/// emphasizes a region and puts it back, and a turn interrupted mid-way still lands on what it was
/// told.
fn tint(grove: &mut Grove, tile: Leaf, hue: Rgb, front: f32) {
    grove.animate(
        tile,
        Motion::Color(fill(hue)),
        Timing::ms(EMPHASIS_MS)
            .after((front * EMPHASIS_SPREAD) as u64)
            .ease(Ease::Decelerate),
    );
}

/// A square in unit space, as the placement the mosaic's own box resolves it against.
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
