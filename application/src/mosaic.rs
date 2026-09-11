//! The leaf's geometry: the outline it is cut from, and the tiles that fill it.
//!
//! Everything here is plain arithmetic in **unit space** -- `x` over `0.0..1.0`, `y` over
//! `0.0..`[`Silhouette::height`] -- which is isotropic, so a square is a square whatever box the
//! leaf is finally drawn into. Nothing in this module reaches the engine: what comes out is
//! positions and sizes, and the page states them as placements.

use core::f32::consts::TAU;

/// The page the outline was traced on.
///
/// Kept because the two axes are only comparable through it: the outline is stated as fractions of
/// a page that is wider than it is tall, and a distance is not a distance until both axes are in
/// the same unit.
const SKETCH: (f32, f32) = (2000.0, 1500.0);

/// The outline, clockwise from the tip, as fractions of the page it was traced on.
///
/// Closed implicitly: the last vertex joins the first. The stem is part of it rather than a second
/// shape, so it is dashed, filled and measured with everything else.
///
/// Stated exactly as it was drawn -- lying on its side -- and stood up by [`upright`], so what was
/// traced and which way it faces stay separable.
const OUTLINE: [(f32, f32); 75] = [
    (0.480, 0.073),
    (0.520, 0.115),
    (0.560, 0.175),
    (0.600, 0.245),
    (0.630, 0.290),
    (0.645, 0.310),
    (0.652, 0.345),
    (0.663, 0.315),
    (0.684, 0.312),
    (0.700, 0.360),
    (0.712, 0.405),
    (0.722, 0.455),
    (0.730, 0.500),
    (0.735, 0.535),
    (0.800, 0.552),
    (0.870, 0.578),
    (0.925, 0.615),
    (0.958, 0.660),
    (0.966, 0.690),
    (0.950, 0.700),
    (0.930, 0.665),
    (0.890, 0.625),
    (0.820, 0.595),
    (0.750, 0.575),
    (0.737, 0.572),
    (0.728, 0.620),
    (0.716, 0.670),
    (0.700, 0.718),
    (0.686, 0.752),
    (0.660, 0.762),
    (0.640, 0.745),
    (0.618, 0.782),
    (0.590, 0.800),
    (0.552, 0.833),
    (0.512, 0.862),
    (0.480, 0.882),
    (0.448, 0.887),
    (0.410, 0.878),
    (0.372, 0.872),
    (0.352, 0.862),
    (0.358, 0.815),
    (0.362, 0.772),
    (0.356, 0.735),
    (0.366, 0.700),
    (0.340, 0.676),
    (0.300, 0.665),
    (0.262, 0.650),
    (0.222, 0.632),
    (0.180, 0.612),
    (0.140, 0.592),
    (0.100, 0.570),
    (0.072, 0.553),
    (0.052, 0.532),
    (0.048, 0.492),
    (0.056, 0.455),
    (0.072, 0.432),
    (0.092, 0.412),
    (0.100, 0.385),
    (0.118, 0.352),
    (0.142, 0.330),
    (0.178, 0.310),
    (0.215, 0.283),
    (0.246, 0.256),
    (0.262, 0.238),
    (0.300, 0.248),
    (0.345, 0.262),
    (0.388, 0.283),
    (0.380, 0.230),
    (0.383, 0.185),
    (0.396, 0.140),
    (0.412, 0.152),
    (0.424, 0.120),
    (0.440, 0.135),
    (0.452, 0.105),
    (0.466, 0.090),
];

/// How far apart tile centres sit.
const PITCH: f32 = 0.068;

/// How much of a pitch a tile spans, before the outline has its say.
///
/// Over one, and it has to be: a tile is a polygon inscribed in this box and covers about two
/// thirds of it, so tiles the size of their own cell leave the gaps between them showing. They
/// overlap instead, which is what a mosaic is.
const SPAN: (f32, f32) = (1.45, 1.85);

/// How far a tile centre wanders off its cell, as a fraction of the pitch.
///
/// What makes the fill a mosaic rather than a grid. Large enough that no two rows line up, small
/// enough that the cells still tile the shape.
const WANDER: f32 = 0.26;

/// How much of the room the outline leaves a tile may take.
///
/// Above one, because a tile is a polygon inscribed in the box being sized here: its corners sit
/// inside the box, so a box that crosses the outline by a little draws a shape that does not.
const ROOM: f32 = 3.4;

/// A tile the outline leaves less room than this is not drawn at all.
const LEAST: f32 = 0.015;

/// One dash, and the gap after it.
const DASH: f32 = 0.026;
const GAP: f32 = 0.020;

/// One tile of the mosaic: where it sits, how large it is, and what shape it settles into.
pub(crate) struct Tile {
    /// Its centre, in unit space.
    pub(crate) center: (f32, f32),
    /// How wide and tall it is. Square, because unit space is isotropic.
    pub(crate) size: f32,
    /// How many sides it has when it has finished arriving.
    pub(crate) sides: f32,
    /// How round its corners are, `0.0` sharp to `1.0` a circle.
    pub(crate) rounding: f32,
    /// How far it is turned, in radians.
    pub(crate) rotation: f32,
}

/// The leaf's outline, and what can be asked of it.
pub(crate) struct Silhouette {
    /// The outline in unit space, in order, closed implicitly.
    outline: Vec<(f32, f32)>,
    /// How tall the shape is for every unit of its width.
    height: f32,
    /// Where the turned sketch's near corner landed, and what its width was divided by. Kept so
    /// that a point can be brought into unit space after the fact, which is what [`at`] is.
    ///
    /// [`at`]: Silhouette::at
    origin: (f32, f32),
    scale: f32,
}

impl Silhouette {
    /// The outline, brought into unit space.
    ///
    /// The traced fractions are scaled by the page they were traced on, then moved and divided so
    /// that the shape spans exactly `0.0..1.0` across. Its height falls out of that rather than
    /// being stated, which is what keeps the leaf the proportions it was drawn in.
    pub(crate) fn traced() -> Self {
        let turned: Vec<(f32, f32)> = OUTLINE.iter().copied().map(upright).collect();
        let reach = |axis: fn(&(f32, f32)) -> f32| {
            let low = turned.iter().map(axis).fold(f32::MAX, f32::min);
            let high = turned.iter().map(axis).fold(f32::MIN, f32::max);
            (low, high - low)
        };
        let ((left, width), (top, tall)) = (reach(|point| point.0), reach(|point| point.1));
        let mut silhouette = Self {
            outline: Vec::new(),
            height: tall / width,
            origin: (left, top),
            scale: width,
        };
        silhouette.outline = OUTLINE
            .iter()
            .copied()
            .map(|point| silhouette.at(point))
            .collect();
        silhouette
    }

    /// A point of the sketch, in unit space.
    ///
    /// What lets the page say where a destination sits in the terms the leaf was drawn in. Read off
    /// the drawing once, they then survive the shape being turned, rescaled or retraced, where
    /// coordinates taken from the finished unit space would have to be derived again each time.
    pub(crate) fn at(&self, point: (f32, f32)) -> (f32, f32) {
        let (x, y) = upright(point);
        (
            (x - self.origin.0) / self.scale,
            (y - self.origin.1) / self.scale,
        )
    }

    /// How tall the shape is for every unit of its width.
    pub(crate) fn height(&self) -> f32 {
        self.height
    }

    /// The outline as dashes, in order around it.
    ///
    /// Walked by arc length rather than by vertex, so a dash is the same length on a long edge and
    /// on a short one and the rhythm survives the jagged parts of the outline.
    pub(crate) fn dashes(&self) -> Vec<((f32, f32), (f32, f32))> {
        let perimeter = self.perimeter();
        let stride = DASH + GAP;
        let count = (perimeter / stride).floor().max(1.0) as usize;
        // The stride is spread over what is actually there, so the last dash meets the first rather
        // than leaving a gap of whatever the division happened to leave over.
        let stride = perimeter / count as f32;
        (0..count)
            .map(|n| {
                let from = n as f32 * stride;
                (self.along(from), self.along(from + stride - GAP))
            })
            .collect()
    }

    /// The mosaic: one tile per cell of a jittered grid that the outline holds.
    ///
    /// A tile never crosses the outline. What the edge leaves is what the tile gets, so the fill
    /// thins toward the boundary and stops rather than being cut off, and the shape is legible from
    /// the tiles alone.
    ///
    /// `taken` are circles already spoken for -- the tiles the page places itself -- given as a
    /// centre and a radius.
    pub(crate) fn tiles(&self, taken: &[((f32, f32), f32)]) -> Vec<Tile> {
        let mut scatter = Scatter::new();
        let mut tiles = Vec::new();
        let columns = (1.0 / PITCH).ceil() as i32;
        let rows = (self.height / PITCH).ceil() as i32;
        for row in 0..rows {
            for column in 0..columns {
                let center = (
                    (column as f32 + 0.5) * PITCH + scatter.between(-WANDER, WANDER) * PITCH,
                    (row as f32 + 0.5) * PITCH + scatter.between(-WANDER, WANDER) * PITCH,
                );
                let size = PITCH * scatter.between(SPAN.0, SPAN.1);
                let sides = scatter.between(5.0, 7.0).round();
                let rounding = scatter.between(0.03, 0.18);
                let rotation = scatter.next() * TAU;
                if !self.contains(center) {
                    continue;
                }
                if taken
                    .iter()
                    .any(|(at, radius)| distance(center, *at) < *radius)
                {
                    continue;
                }
                let size = size.min(self.clearance(center) * ROOM);
                if size < LEAST {
                    continue;
                }
                tiles.push(Tile {
                    center,
                    size,
                    sides,
                    rounding,
                    rotation,
                });
            }
        }
        tiles
    }

    /// Whether the outline holds `point`, by counting the crossings of a ray cast from it.
    fn contains(&self, (x, y): (f32, f32)) -> bool {
        let mut inside = false;
        let mut previous = *self.outline.last().expect("the outline has vertices");
        for &vertex in &self.outline {
            let ((x0, y0), (x1, y1)) = (previous, vertex);
            if (y0 > y) != (y1 > y) && x < (x1 - x0) * (y - y0) / (y1 - y0) + x0 {
                inside = !inside;
            }
            previous = vertex;
        }
        inside
    }

    /// How far `point` is from the nearest edge of the outline.
    ///
    /// What sizes a tile, and what limits how far one may be moved afterwards: the outline is drawn
    /// once and does not move, so nothing inside it may be carried out of it.
    pub(crate) fn clearance(&self, point: (f32, f32)) -> f32 {
        self.edges()
            .map(|(from, to)| span(point, from, to))
            .fold(f32::MAX, f32::min)
    }

    /// The point `distance` along the outline from its first vertex.
    fn along(&self, distance: f32) -> (f32, f32) {
        let mut remaining = distance.rem_euclid(self.perimeter());
        for (from, to) in self.edges() {
            let length = self::distance(from, to);
            if remaining <= length {
                let at = match length > 0.0 {
                    true => remaining / length,
                    false => 0.0,
                };
                return (from.0 + (to.0 - from.0) * at, from.1 + (to.1 - from.1) * at);
            }
            remaining -= length;
        }
        *self.outline.last().expect("the outline has vertices")
    }

    /// How far it is all the way round.
    fn perimeter(&self) -> f32 {
        self.edges().map(|(from, to)| distance(from, to)).sum()
    }

    /// Every edge of the outline, the closing one included.
    fn edges(&self) -> impl Iterator<Item = ((f32, f32), (f32, f32))> + '_ {
        self.outline
            .iter()
            .zip(self.outline.iter().cycle().skip(1))
            .take(self.outline.len())
            .map(|(from, to)| (*from, *to))
    }
}

/// A point of the sketch, stood upright.
///
/// The leaf was drawn lying on its side with its stem out to the right, and a leaf hangs from its
/// stem -- so a quarter turn clockwise puts the stem at the bottom and the blade above it. The two
/// axes are scaled by the page first, because a turn is only a turn once they are comparable.
///
/// Applied in one place, so everything stated in the sketch's own terms turns together.
fn upright((x, y): (f32, f32)) -> (f32, f32) {
    let (x, y) = (x * SKETCH.0, y * SKETCH.1);
    (-y, x)
}

/// How far apart two points are.
fn distance(from: (f32, f32), to: (f32, f32)) -> f32 {
    ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt()
}

/// How far `point` is from the segment between `from` and `to`.
fn span(point: (f32, f32), from: (f32, f32), to: (f32, f32)) -> f32 {
    let run = (to.0 - from.0, to.1 - from.1);
    let length = run.0 * run.0 + run.1 * run.1;
    if length == 0.0 {
        return distance(point, from);
    }
    let at = (((point.0 - from.0) * run.0 + (point.1 - from.1) * run.1) / length).clamp(0.0, 1.0);
    distance(point, (from.0 + run.0 * at, from.1 + run.1 * at))
}

/// The jitter the mosaic is built from.
///
/// Its own generator rather than the platform's, so the leaf is the same leaf on every run and on
/// every target: the mosaic is a drawing, and a drawing that came out differently each time would
/// be a different drawing.
pub(crate) struct Scatter(u64);

impl Scatter {
    /// The one sequence, from a fixed seed.
    pub(crate) fn new() -> Self {
        Self(0x9E37_79B9_7F4A_7C15)
    }

    /// The next number in `0.0..1.0`.
    pub(crate) fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1 << 24) as f32
    }

    /// The next number between two bounds.
    pub(crate) fn between(&mut self, low: f32, high: f32) -> f32 {
        low + self.next() * (high - low)
    }
}
