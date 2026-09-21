//! A silhouette filled with tiles.

use core::f32::consts::TAU;

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

/// How long a change of the whole mosaic -- appearing, vanishing, cropping -- takes to cross it,
/// and how much later than its place in that a tile may move.
const CHANGE_SPREAD: f32 = 420.0;
const CHANGE_LAG: f32 = 90.0;

/// How far a tile may be read as sitting off its true distance from a region, as a fraction of
/// the region's size either way. What makes a rim ragged. About a tile's worth at a fine pitch:
/// more, and the rim is not ragged but a band of tiles that could be either side of it.
const RAGGED: f32 = 0.10;

/// How far a tinge's rim wanders in and out as it goes round, on three lobes, five and seven, as
/// fractions of its radius; and how far one tile may be read off its distance from the middle,
/// as a fraction of the tinge's width. The wander is what makes the rim a grown shape, and it is
/// smooth -- a tile and its neighbour read nearly the same of it -- so the tinge stays solid to
/// its rim. The jitter is kept small for the same reason: it is what puts a tile of the other
/// colour inside the rim, and a little is a ragged edge where a lot is a striped one.
const TINGE_WOBBLE: (f32, f32, f32) = (0.13, 0.09, 0.05);
const TINGE_RAGGED: f32 = 0.03;

/// How far the distance from a region swells and shrinks as it goes round, as fractions of
/// itself, on three lobes and on five. What makes a rim a shape rather than a rule.
const LOBES: (f32, f32) = (0.16, 0.10);

/// A [`Silhouette`] outlined in dashes and filled with polygons coloured along a [`Ramp`].
///
/// Built ungrown, which costs nothing, and grown once into a box. [`form`](Self::form) brings it
/// in -- the outline arrives as dashes, then the tiles sweep in along the ramp -- and
/// [`frame`](Self::frame) carries that until [`formed`](Self::formed) says it is over.
/// [`vanish`](Self::vanish) takes it away toward a point and [`appear`](Self::appear) brings it
/// back out from one, the arrival run backwards and forwards.
///
/// [`regions`](Self::regions) stands a square on each vertex the caller names, and hands back a
/// stem over each to hang something off. [`emphasize`](Self::emphasize) recolours the tiles under
/// one of those onto another ramp until [`unemphasize`](Self::unemphasize) puts them back.
///
/// [`crop`](Self::crop) cuts the mosaic down to a part of its shape, in place -- what is outside
/// goes, what crosses the edge is cut to it, what is inside is recoloured over the part -- until
/// [`uncrop`](Self::uncrop) puts the whole back. The outline stays through both.
pub struct Mosaic {
    silhouette: Silhouette,
    ramp: Ramp,
    /// Which way the ramp runs across the shape, as a direction across its box.
    along: (f32, f32),
    /// What the outline is dashed in.
    edge: Rgb,
    /// Tile spacing, as a fraction of the shape's width.
    pitch: f32,
    /// How large a region is, where that is stated rather than left to the regions.
    square: Option<f32>,
    /// What it is all branched from. `None` until it is grown.
    root: Option<Leaf>,
    /// The outline, dash by dash. Empty until it is grown.
    dashes: Vec<Dash>,
    /// Every tile, in the order they were grown. Empty until it is grown.
    tiles: Vec<Tile>,
    /// Where each region's middle is, in unit space, and how large every one of them is. Empty
    /// until regions are stood.
    regions: Vec<(f32, f32)>,
    region: f32,
    /// The whole arrival, as one group. `None` until it is formed.
    forming: Option<Sequence>,
    /// Whether that group has landed, latched, so that asking after the frame it lands still answers.
    arrived: bool,
}

/// One dash of the outline, as grown.
struct Dash {
    leaf: Leaf,
    /// Its middle, in unit space, which is where it is measured from.
    at: (f32, f32),
    /// When it appears, as a delay into the form.
    after: u64,
}

/// One polygon of the mosaic, as grown.
struct Tile {
    /// The polygon itself.
    leaf: Leaf,
    /// Where it sits, in unit space, and how large it is.
    center: (f32, f32),
    size: f32,
    /// The shape it settles into, and the seed it starts from.
    shape: Shape,
    seed: Shape,
    /// When it arrives, as a delay into the form.
    after: u64,
    /// What it rests at, kept because an emphasis has to have somewhere to put it back to.
    hue: Rgb,
    /// How far off the ramp it was put, kept so that it is put the same distance off whichever ramp
    /// it is currently coloured from -- the scatter is what keeps neighbours apart, and a region
    /// that lost it would read as one shape.
    drift: f32,
    /// How far off its true distance from a region it is read as sitting, `-1.0..1.0`, so that
    /// the rim of a region is ragged rather than drawn with a rule.
    jitter: f32,
    /// How much later than its place in a sweep across the whole mosaic it moves, `0.0..1.0`.
    lag: f32,
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
            square: None,
            root: None,
            dashes: Vec::new(),
            tiles: Vec::new(),
            regions: Vec::new(),
            region: 0.0,
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

    /// How large a region is, as a fraction of the shape's width. As large as the closest pair of
    /// regions leaves unless stated, which is right for regions that are the shape's features and
    /// wrong for regions that are places on it something else is sized to.
    pub fn square(mut self, size: f32) -> Self {
        self.square = Some(size);
        self
    }

    /// Height per unit width, for sizing the box it is grown into.
    pub fn aspect(&self) -> f32 {
        self.silhouette.height
    }

    /// Where the middle of a region is, in unit space. `None` for one that was never stood.
    pub fn middle_of(&self, region: usize) -> Option<(f32, f32)> {
        self.regions.get(region).copied()
    }

    /// Grows it into `location` off `at`, and hands back what everything is branched from -- to
    /// move it, to hang something off it, or to take it away.
    ///
    /// Grown unseen: the outline is drawn but not inked and every tile is a seed of itself, so
    /// nothing shows until [`form`](Self::form) or [`appear`](Self::appear) brings it in. Growing
    /// one that is already grown does nothing and hands back what it grew.
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
                let leaf = grove.branch(
                    root,
                    Line::new()
                        .between(spot(height, *from), spot(height, *to))
                        .weight(HAIRLINE * 1.5)
                        .color(fill(self.edge))
                        .cap(Cap::Round)
                        .opacity(0.0)
                        .elevate(Elevation::up(2)),
                );
                Dash {
                    leaf,
                    at: ((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0),
                    after: (n as f32 * trace) as u64,
                }
            })
            .collect();

        // The fill. Each tile is one colour and carries no gradient of its own; what builds one is
        // where each sits on the ramp, and the delay taken from the same number sweeps the shape
        // in the order the ramp runs. The ramp is spread over the range the tiles actually cover
        // rather than the box, so a ramp read against corners the shape does not have is not spent
        // on them.
        let cells = self.silhouette.cells(self.pitch, scatter);
        let range = self.range(cells.iter().map(|cell| cell.center));
        self.tiles = cells
            .iter()
            .map(|cell| {
                let warmth = self.warmth(cell.center, range);
                let drift = cell.drift * DRIFT;
                let hue = shifted(self.ramp.at(warmth), drift);
                let after = MOSAIC_AT + (warmth * MOSAIC_SPREAD + cell.lag * CHANGE_LAG) as u64;
                let shape = Shape {
                    sides: cell.sides,
                    rounding: cell.rounding,
                    rotation: cell.rotation,
                };
                let seed = Shape {
                    sides: 3.0,
                    rounding: 0.85,
                    rotation: cell.rotation - 0.7,
                };
                let leaf = grove.branch(
                    root,
                    Polygon::new()
                        .sides(seed.sides)
                        .rounding(seed.rounding)
                        .rotation(seed.rotation)
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
                    seed,
                    after,
                    hue,
                    drift,
                    jitter: cell.jitter,
                    lag: cell.lag,
                    mark: None,
                }
            })
            .collect();

        self.root = Some(root);
        root
    }

    /// A square on each named vertex -- `(vertex index, nudge)` -- sized so no two meet unless
    /// [`square`](Self::square) said how large, and a stem over each to hang something on. The
    /// tiles under a square are what `emphasize` recolours. The nudge is how far the square is
    /// moved off where it would otherwise sit, in unit space on each axis, for a vertex that is a
    /// corner of its feature rather than the middle of it.
    ///
    /// Once, after `grow`; nothing on one that was never grown. Branched after every tile, so what
    /// is hung off a region is branched after every part of the mosaic rather than into the middle
    /// of it. Hands back the stems to keep, in the order they were named.
    pub fn regions(&mut self, grove: &mut Grove, at: &[(usize, (f32, f32))]) -> Vec<Leaf> {
        let Some(root) = self.root else {
            return Vec::new();
        };
        let (regions, size) = self.silhouette.regions(at, self.square);
        let half = size / 2.0;
        for tile in &mut self.tiles {
            // How far out of a region's middle the tile sits, which is `1.0` at the rim. No two
            // regions meet, so at most one answers and the first that does is it.
            tile.mark = regions.iter().enumerate().find_map(|(region, &center)| {
                let out = reach(tile.center, center, region) + tile.jitter * RAGGED * size;
                (out <= half).then_some(Mark {
                    region,
                    depth: (out / half).clamp(0.0, 1.0),
                })
            });
        }
        let height = self.silhouette.height;
        let stems = regions
            .iter()
            .map(|&center| grove.branch(root, Stem::new().at(placed(height, center, size))))
            .collect();
        self.regions = regions;
        self.region = size;
        stems
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
        for dash in &self.dashes {
            grove.animate(
                dash.leaf,
                Motion::Opacity(1.0),
                Timing::ms(EDGE_MS)
                    .after(dash.after)
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

    /// Takes it away, swept in toward `toward` -- a point in unit space -- from the far side,
    /// `after` a delay: every tile shrinks back to its seed and goes to `ground` -- the page's
    /// own colour -- as it goes, and every dash goes out, the arrival run backwards. What has
    /// vanished is still grown, and [`appear`](Self::appear) brings it back.
    ///
    /// Gone to the ground's colour rather than faded, and that is deliberate: tiles overlap, and
    /// one part way to no opacity shows what is under it through its own edges, where one the
    /// colour of the ground, shrunk to a seed, is a speck of ground on the ground.
    ///
    /// Nothing on one that was never grown.
    pub fn vanish(&self, grove: &mut Grove, toward: (f32, f32), after: u64, ground: Rgb) {
        let far = self.far(toward);
        for tile in &self.tiles {
            let front = 1.0 - distance(tile.center, toward) / far;
            self.go(grove, tile, after + (front * CHANGE_SPREAD) as u64, ground);
        }
        for dash in &self.dashes {
            let front = 1.0 - distance(dash.at, toward) / far;
            grove.animate(
                dash.leaf,
                Motion::Opacity(0.0),
                Timing::ms(EDGE_MS)
                    .after(after + (front * CHANGE_SPREAD) as u64)
                    .ease(Ease::Accelerate),
            );
        }
    }

    /// Brings it back, swept out from `from` -- a point in unit space -- to the far side, `after`
    /// a delay: every tile grows out of its seed into its shape, and every dash comes in. The
    /// arrival again, from wherever it is asked from rather than along the ramp.
    ///
    /// Nothing on one that was never grown. On one that was grown and never formed, this is its
    /// arrival.
    pub fn appear(&self, grove: &mut Grove, from: (f32, f32), after: u64) {
        let far = self.far(from);
        for tile in &self.tiles {
            let front = distance(tile.center, from) / far;
            self.come(grove, tile, after + (front * CHANGE_SPREAD) as u64);
        }
        for dash in &self.dashes {
            let front = distance(dash.at, from) / far;
            grove.animate(
                dash.leaf,
                Motion::Opacity(1.0),
                Timing::ms(EDGE_MS)
                    .after(after + (front * CHANGE_SPREAD) as u64)
                    .ease(Ease::Decelerate),
            );
        }
    }

    /// Cuts the mosaic down to `within` -- an outline in the same unit space, a
    /// [`part`](Silhouette::part) of its own shape -- in place, swept in toward `toward` from the
    /// far side. Every tile the part does not hold goes to `ground`, as [`vanish`](Self::vanish)
    /// takes it;
    /// every tile it holds but not whole is cut down to what it holds; and every tile it holds is
    /// recoloured from the ramp spread over the part alone, so the part carries the whole ramp
    /// rather than the stretch of it it happened to sit on. `tinge` lays another ramp over the
    /// tiles within a rough circle -- where, how wide, and what -- palest at its middle and
    /// deepest at its rim, where it meets the section's own colour outright. A circle rather than
    /// a region's lobed square, because what is wanted here is a spot of colour on the section
    /// and not a shape of its own; rough, because a compass circle is not a spot either; and
    /// rough off `scatter`, so that two crops of the same place are the same spot drawn twice
    /// rather than the same drawing.
    ///
    /// The outline is not touched. A section of a shape is still the shape, and the outline is
    /// what says so; what is missing reads as missing because the edge of it is still there.
    ///
    /// Cropping again crops from wherever the mosaic is, so a crop over a crop is the second
    /// crop; [`uncrop`](Self::uncrop) is what puts the whole back. Nothing on one that was never
    /// grown.
    pub fn crop(
        &self,
        grove: &mut Grove,
        within: &Silhouette,
        toward: (f32, f32),
        ground: Rgb,
        tinge: Option<((f32, f32), f32, &Ramp)>,
        scatter: &mut Scatter,
    ) {
        let far = self.far(toward);
        let height = self.silhouette.height;
        let phases = (
            scatter.between(0.0, TAU),
            scatter.between(0.0, TAU),
            scatter.between(0.0, TAU),
        );
        let rooms: Vec<Option<f32>> = self
            .tiles
            .iter()
            .map(|tile| within.room(tile.center))
            .collect();
        let range = self.range(
            self.tiles
                .iter()
                .zip(&rooms)
                .filter(|(_, room)| room.is_some())
                .map(|(tile, _)| tile.center),
        );
        for (tile, room) in self.tiles.iter().zip(&rooms) {
            let front = 1.0 - distance(tile.center, toward) / far;
            let after = (front * CHANGE_SPREAD) as u64;
            let Some(room) = *room else {
                self.go(grove, tile, after, ground);
                continue;
            };
            let timing = |ms: u64| {
                Timing::ms(ms)
                    .after(after + (tile.lag * CHANGE_LAG) as u64)
                    .ease(Ease::Decelerate)
            };
            let mut hue = shifted(self.ramp.at(self.warmth(tile.center, range)), tile.drift);
            if let Some((center, size, ramp)) = tinge {
                let angle = (tile.center.1 - center.1).atan2(tile.center.0 - center.0);
                let wander = 1.0
                    + TINGE_WOBBLE.0 * (3.0 * angle + phases.0).sin()
                    + TINGE_WOBBLE.1 * (5.0 * angle + phases.1).sin()
                    + TINGE_WOBBLE.2 * (7.0 * angle + phases.2).sin();
                let out = distance(tile.center, center) / wander
                    + scatter.between(-1.0, 1.0) * TINGE_RAGGED * size;
                let depth = out / (size / 2.0);
                if depth <= 1.0 {
                    hue = shifted(ramp.at(depth.max(0.0)), tile.drift);
                }
            }
            grove.animate(tile.leaf, Motion::Color(fill(hue)), timing(TILE_MS));
            // Cut down to what the part holds, in place. Only stated for what has to give, so a
            // tile the part holds whole is not told to be what it is.
            if room < tile.size {
                grove.animate(
                    tile.leaf,
                    Motion::Location(placed(height, tile.center, room)),
                    timing(GROW_MS),
                );
            }
        }
    }

    /// Puts the whole back as it was grown, swept out from `from` to the far side: what went
    /// comes as [`appear`](Self::appear) brings it, what was cut down grows back, and what was
    /// recoloured is returned to its own colour.
    ///
    /// Told to every tile, a tile already at what it is told being nothing to look at, so however
    /// often the mosaic is cropped it comes back to exactly what was grown. Nothing on one that
    /// was never grown.
    pub fn uncrop(&self, grove: &mut Grove, from: (f32, f32)) {
        let far = self.far(from);
        for tile in &self.tiles {
            let front = distance(tile.center, from) / far;
            self.come(grove, tile, (front * CHANGE_SPREAD) as u64);
        }
    }

    /// Recolours one region onto `ramp` -- the tiles under it, and the dashes of the outline that
    /// run through it -- palest at its middle and deepest at the rim, swept out from the middle.
    ///
    /// Read outward from the region rather than along the mosaic, so the colour reads as something
    /// sitting on that region rather than as a stretch of the mosaic's own ramp recoloured. Only the
    /// colour moves. Emphasizing a region that is already emphasized runs it again from wherever it
    /// currently is; emphasizing one on a mosaic with no regions does nothing.
    pub fn emphasize(&self, grove: &mut Grove, region: usize, ramp: &Ramp) {
        let Some(&center) = self.regions.get(region) else {
            return;
        };
        for tile in &self.tiles {
            let Some(mark) = tile.mark.as_ref().filter(|mark| mark.region == region) else {
                continue;
            };
            let hue = shifted(ramp.at(mark.depth), tile.drift);
            tint(grove, tile.leaf, hue, mark.depth);
        }
        for dash in &self.dashes {
            let depth = reach(dash.at, center, region) / (self.region / 2.0);
            if depth <= 1.0 {
                tint(grove, dash.leaf, ramp.at(depth), depth);
            }
        }
    }

    /// Puts one region back on the ramp it was grown from, drawn in from the rim.
    ///
    /// Every tile and every dash is returned to its own colour rather than to the ramp read
    /// afresh, so however often a region is emphasized it comes back to exactly the mosaic that
    /// was grown. Unemphasizing a region that is not emphasized states what it is already at,
    /// which is nothing to look at.
    pub fn unemphasize(&self, grove: &mut Grove, region: usize) {
        let Some(&center) = self.regions.get(region) else {
            return;
        };
        for tile in &self.tiles {
            let Some(mark) = tile.mark.as_ref().filter(|mark| mark.region == region) else {
                continue;
            };
            tint(grove, tile.leaf, tile.hue, 1.0 - mark.depth);
        }
        for dash in &self.dashes {
            let depth = reach(dash.at, center, region) / (self.region / 2.0);
            if depth <= 1.0 {
                tint(grove, dash.leaf, self.edge, 1.0 - depth);
            }
        }
    }

    /// Takes one tile away, `after` a delay: back to its seed, and to the ground's colour. The
    /// arrival run backwards, but for the colour.
    fn go(&self, grove: &mut Grove, tile: &Tile, after: u64, ground: Rgb) {
        let height = self.silhouette.height;
        let timing = |ms: u64| {
            Timing::ms(ms)
                .after(after + (tile.lag * CHANGE_LAG) as u64)
                .ease(Ease::Accelerate)
        };
        grove.animate(tile.leaf, Motion::Color(fill(ground)), timing(TILE_MS));
        grove.animate(
            tile.leaf,
            Motion::Location(placed(height, tile.center, tile.size * SEED)),
            timing(GROW_MS),
        );
        grove.animate(tile.leaf, Motion::Polygon(tile.seed), timing(GROW_MS));
    }

    /// Brings one tile in, `after` a delay: out of its seed, into its shape, and in its own
    /// colour. The arrival, and what puts back whatever a crop or a vanish did to it.
    fn come(&self, grove: &mut Grove, tile: &Tile, after: u64) {
        let height = self.silhouette.height;
        let timing = |ms: u64| {
            Timing::ms(ms)
                .after(after + (tile.lag * CHANGE_LAG) as u64)
                .ease(Ease::Decelerate)
        };
        grove.animate(tile.leaf, Motion::Opacity(1.0), timing(TILE_MS));
        grove.animate(tile.leaf, Motion::Color(fill(tile.hue)), timing(TILE_MS));
        grove.animate(
            tile.leaf,
            Motion::Location(placed(height, tile.center, tile.size)),
            timing(GROW_MS),
        );
        grove.animate(tile.leaf, Motion::Polygon(tile.shape), timing(GROW_MS));
    }

    /// What a point reads along the ramp's direction, before it is spread.
    fn reading(&self, (x, y): (f32, f32)) -> f32 {
        self.along.0 * x + self.along.1 * (y / self.silhouette.height)
    }

    /// The range `centers` cover along the ramp: the lowest reading among them, and how far it
    /// is from there to the highest. What the ramp is spread over, so both ends of it are reached
    /// over whatever the centers are.
    fn range(&self, centers: impl Iterator<Item = (f32, f32)>) -> (f32, f32) {
        let (low, high) = centers
            .map(|center| self.reading(center))
            .fold((f32::MAX, f32::MIN), |(low, high), reading| {
                (low.min(reading), high.max(reading))
            });
        (low, (high - low).max(f32::EPSILON))
    }

    /// Where on the ramp a point sits, with the ramp spread over `range`: `0.0` at its low end
    /// and `1.0` at its high.
    fn warmth(&self, center: (f32, f32), (low, span): (f32, f32)) -> f32 {
        ((self.reading(center) - low) / span).clamp(0.0, 1.0)
    }

    /// How far the furthest of the mosaic is from `point`: what a sweep between it and the far
    /// side is spread over, so the far end moves first or last whatever the shape.
    fn far(&self, point: (f32, f32)) -> f32 {
        self.tiles
            .iter()
            .map(|tile| distance(tile.center, point))
            .chain(self.dashes.iter().map(|dash| distance(dash.at, point)))
            .fold(0.0, f32::max)
            .max(f32::EPSILON)
    }
}

/// How far apart two points are.
fn distance(from: (f32, f32), to: (f32, f32)) -> f32 {
    ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt()
}

/// How far `at` is out of `region`'s middle at `center`.
///
/// Neither the box's own reading, which puts a straight side on everything measured from it, nor
/// a circle's, which puts nothing of the box in it: the fourth-power mean of the two distances,
/// which is `1.0` on a rounded square. Then lobed -- the distance swells and shrinks as it goes
/// round, on two counts that do not divide each other, phased by the region so that no two
/// regions are lobed alike -- because a rounded square is still a shape drawn with a rule, and
/// what is wanted is one that could have grown there.
fn reach((x, y): (f32, f32), (cx, cy): (f32, f32), region: usize) -> f32 {
    let (dx, dy) = (x - cx, y - cy);
    let plain = (dx.powi(4) + dy.powi(4)).sqrt().sqrt();
    let angle = dy.atan2(dx);
    let phase = region as f32 * 1.7;
    let lobed =
        1.0 + LOBES.0 * (3.0 * angle + phase).sin() + LOBES.1 * (5.0 * angle - 2.0 * phase).sin();
    plain / lobed
}

/// Moves one element onto a colour, `front` of the way into the sweep across a region that
/// carries it there.
///
/// Stated as where the colour is going rather than as a step from where it is, so the same call
/// emphasizes a region and puts it back, and a turn interrupted mid-way still lands on what it was
/// told.
fn tint(grove: &mut Grove, leaf: Leaf, hue: Rgb, front: f32) {
    grove.animate(
        leaf,
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
