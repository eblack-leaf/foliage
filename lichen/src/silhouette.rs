//! The shape a mosaic is cut from.
//!
//! Plain arithmetic in unit space -- `x` over `0.0..1.0`, `y` over `0.0..height` -- which is
//! isotropic, so a square is a square whatever box the shape is finally drawn into. Nothing in here
//! reaches the engine: what comes out is positions and sizes, and only [`Mosaic`](crate::Mosaic)
//! reads them.

use core::f32::consts::TAU;

use crate::Scatter;

/// How much of a pitch a tile spans, before the outline has its say. Over one, and it has to be: a
/// tile is a polygon inscribed in this box and covers about two thirds of it, so tiles the size of
/// their own cell leave the gaps between them showing. They overlap instead, which is what a mosaic
/// is.
const SPAN: (f32, f32) = (1.45, 1.85);

/// How far a tile centre wanders off its cell, as a fraction of the pitch.
const WANDER: f32 = 0.26;

/// How much of the room the outline leaves a tile may take. Above one, because a tile is a polygon
/// inscribed in the box being sized here: its corners sit inside the box.
const ROOM: f32 = 3.4;

/// A tile the outline leaves less room than this is not drawn at all.
const LEAST: f32 = 0.015;

/// One dash, and the gap after it.
const DASH: f32 = 0.026;
const GAP: f32 = 0.020;

/// How far a region is pulled off the vertex it is named for, toward the middle of the shape. A
/// region centred on the vertex itself would hang half of itself outside the outline.
const INSET: f32 = 0.28;

/// How much of the room between the two closest regions is left showing between them, so that
/// regions sized by that pair are neighbours rather than neighbours that touch.
const APART: f32 = 0.08;

/// Clockwise quarter turns of the sketch, applied before it is scaled to unit width.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Turn {
    None,
    Quarter,
    Half,
    ThreeQuarter,
}

/// A closed outline in unit space.
#[derive(Clone, Debug)]
pub struct Silhouette {
    /// The outline, in unit space, in the order it was traced.
    pub(crate) outline: Vec<(f32, f32)>,
    /// How tall it is per unit of width.
    pub(crate) height: f32,
}

/// One cell of the grid a mosaic is laid on: where its tile sits, how large it is, and what shape
/// it settles into.
pub(crate) struct Cell {
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

impl Silhouette {
    /// `outline` as fractions of `page`, turned, then scaled so its width is 1.
    ///
    /// The page is kept only long enough to make the two axes comparable: an outline traced on a
    /// 2000×1500 page has its x in `0..2000` and its y in `0..1500`, and only by multiplying each
    /// by its own side are they the same kind of number. The height falls out of that rather than
    /// being stated, which is what keeps the shape the proportions it was drawn in.
    pub fn traced(outline: &[(f32, f32)], page: (f32, f32), turn: Turn) -> Self {
        assert!(outline.len() >= 3, "an outline is three or more points");
        let turned: Vec<(f32, f32)> = outline
            .iter()
            .map(|&(x, y)| {
                let (x, y) = (x * page.0, y * page.1);
                match turn {
                    Turn::None => (x, y),
                    Turn::Quarter => (-y, x),
                    Turn::Half => (-x, -y),
                    Turn::ThreeQuarter => (y, -x),
                }
            })
            .collect();
        let reach = |axis: fn(&(f32, f32)) -> f32| {
            let low = turned.iter().map(axis).fold(f32::MAX, f32::min);
            let high = turned.iter().map(axis).fold(f32::MIN, f32::max);
            (low, high - low)
        };
        let ((left, width), (top, tall)) = (reach(|point| point.0), reach(|point| point.1));
        Self {
            outline: turned
                .iter()
                .map(|&(x, y)| ((x - left) / width, (y - top) / width))
                .collect(),
            height: tall / width,
        }
    }

    /// The same outline flipped left-to-right.
    ///
    /// The vertices keep their order, so an index into the outline names the same vertex either
    /// way round; only the winding changes, and nothing here reads it.
    pub fn mirrored(mut self) -> Self {
        for point in &mut self.outline {
            point.0 = 1.0 - point.0;
        }
        self
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

    /// The mosaic: one tile per cell of a jittered grid that the outline holds, `pitch` apart.
    ///
    /// A tile never crosses the outline. What the edge leaves is what the tile gets, so the fill
    /// thins toward the boundary and stops rather than being cut off, and the shape is legible from
    /// the tiles alone.
    pub(crate) fn cells(&self, pitch: f32, scatter: &mut Scatter) -> Vec<Cell> {
        let mut cells = Vec::new();
        let columns = (1.0 / pitch).ceil() as i32;
        let rows = (self.height / pitch).ceil() as i32;
        for row in 0..rows {
            for column in 0..columns {
                let center = (
                    (column as f32 + 0.5) * pitch + scatter.between(-WANDER, WANDER) * pitch,
                    (row as f32 + 0.5) * pitch + scatter.between(-WANDER, WANDER) * pitch,
                );
                let size = pitch * scatter.between(SPAN.0, SPAN.1);
                let sides = scatter.between(5.0, 7.0).round();
                let rounding = scatter.between(0.03, 0.18);
                let rotation = scatter.next() * TAU;
                if !self.contains(center) {
                    continue;
                }
                let size = size.min(self.clearance(center) * ROOM);
                if size < LEAST {
                    continue;
                }
                cells.push(Cell {
                    center,
                    size,
                    sides,
                    rounding,
                    rotation,
                });
            }
        }
        cells
    }

    /// Where a region sits on each named vertex -- `(vertex index, nudge)` -- and how large every
    /// one of them is.
    ///
    /// Each is pulled off its vertex toward the middle of the shape so that a square centred there
    /// sits over the vertex rather than half outside it. All of them are the same size, and that
    /// size is the most the closest pair of them leaves: two equal squares centred at `a` and `b`
    /// clear each other exactly when their side is no longer than the greater of `|dx|` and `|dy|`,
    /// so the least of those over every pair is what every region gets. One region alone gets the
    /// largest square the shape's box holds.
    ///
    /// Held inside the shape's own box, because a vertex on its edge would have a region hanging
    /// over the side. Sized again afterwards, since holding a region in moves it toward the others,
    /// and then nudged by whatever its vertex asks for, in unit space on each axis.
    pub(crate) fn regions(&self, at: &[(usize, (f32, f32))]) -> (Vec<(f32, f32)>, f32) {
        let middle = self.middle();
        let centers: Vec<(f32, f32)> = at
            .iter()
            .map(|&(vertex, _)| {
                let (x, y) = self.outline[vertex];
                (x + (middle.0 - x) * INSET, y + (middle.1 - y) * INSET)
            })
            .collect();
        let apart = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs().max((a.1 - b.1).abs());
        let room = |centers: &[(f32, f32)]| {
            let mut room = self.height.min(1.0);
            for (n, &a) in centers.iter().enumerate() {
                for &b in &centers[n + 1..] {
                    room = room.min(apart(a, b));
                }
            }
            room
        };
        let half = room(&centers) / 2.0;
        let held: Vec<(f32, f32)> = centers
            .iter()
            .map(|&(x, y)| (x.clamp(half, 1.0 - half), y.clamp(half, self.height - half)))
            .collect();
        let size = room(&held) * (1.0 - APART);
        // Nudged last, so that a region asking to sit elsewhere moves only itself.
        let regions = held
            .iter()
            .zip(at)
            .map(|(&(x, y), &(_, (dx, dy)))| (x + dx, y + dy))
            .collect();
        (regions, size)
    }

    /// The middle of the shape, as the average of the outline's vertices. What a region is pulled
    /// toward, which asks only for somewhere inside the shape and away from every vertex.
    fn middle(&self) -> (f32, f32) {
        let count = self.outline.len() as f32;
        self.outline.iter().fold((0.0, 0.0), |(x, y), vertex| {
            (x + vertex.0 / count, y + vertex.1 / count)
        })
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

    /// How far `point` is from the nearest edge of the outline. What sizes a tile.
    fn clearance(&self, point: (f32, f32)) -> f32 {
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
