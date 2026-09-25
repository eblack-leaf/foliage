//! The shape a mosaic is cut from.
//!
//! Plain arithmetic in unit space -- `x` over `0.0..1.0`, `y` over `0.0..height` -- which is
//! isotropic, so a square is a square whatever box the shape is finally drawn into. Nothing in here
//! reaches the engine: what comes out is positions and sizes, and only [`Mosaic`](crate::Mosaic)
//! reads them.

use core::f32::consts::TAU;

use foliage::{Location, Source, left, top};

use crate::Scatter;

/// How much of a pitch a tile spans, before the outline has its say. Well over one, and it has to
/// be: a tile is a polygon inscribed in this box and covers about two thirds of it, so tiles the
/// size of their own cell leave the gaps between them showing. They overlap instead, which is what
/// a mosaic is, and at a fine pitch they overlap by more than they would at a coarse one because
/// a gap between fine tiles reads as a hole rather than as grout. The floor is what keeps the
/// holes out: a pentagon at the small end, wandered off its cell and turned edge-on, still has to
/// meet its neighbours over the gap between them. As low as that allows, and no lower, so tiles
/// read as tiles rather than as the fragments of them that heavier overlap leaves showing.
const SPAN: (f32, f32) = (1.7, 2.0);

/// How far a tile centre wanders off its cell, as a fraction of the pitch. Enough that the grid
/// is not read; not so much that two neighbours can both leave the same gap.
const WANDER: f32 = 0.18;

/// How much of the room the outline leaves a tile may take. Above one, because a tile is a polygon
/// inscribed in the box being sized here: its corners sit inside the box.
const ROOM: f32 = 3.4;

/// A tile the outline leaves less room than this is not drawn at all.
const LEAST: f32 = 0.015;

/// One dash, and the gap after it. A short gap: the round caps eat half a weight of it at each
/// end, and what is left reads as a line made of pieces rather than a dashed one. `0.020` is a
/// dashed line; `0.0` is a solid one, the pieces joined by their caps.
const DASH: f32 = 0.026;
const GAP: f32 = 0.010;

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

/// A place on a shape, in its unit space: `x` over `0.0..1.0`, `y` over `0.0..` its
/// [`aspect`](Silhouette::aspect).
///
/// What a [`Mosaic`](crate::Mosaic) stands something on, and recolours when it is emphasized.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Region {
    /// A square standing on vertex `n` of the outline, pulled in toward the middle of the shape
    /// until it sits over the vertex, and nudged by as much again on each axis -- for a vertex that
    /// is the corner of its feature rather than the middle of it. The shape's points, its tips and
    /// lobes, are these.
    Vertex(usize, (f32, f32)),
    /// A square centred at a point. Sized with the vertex squares, all alike.
    Point((f32, f32)),
    /// A box: its top-left corner, and its width and height. Sized as stated.
    Box((f32, f32), (f32, f32)),
}

/// Where a region sits, as it was resolved: its middle, and how far it reaches either way.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Area {
    pub(crate) center: (f32, f32),
    pub(crate) half: (f32, f32),
}

/// A closed outline in unit space.
#[derive(Clone, Debug)]
pub struct Silhouette {
    /// The outline, in unit space, in the order it was traced.
    pub(crate) outline: Vec<(f32, f32)>,
    /// How tall it is per unit of width.
    pub(crate) height: f32,
    /// How a point of the sketch is brought into unit space: the page it was traced on, how it
    /// was turned, where the turned sketch's near corner landed, and what its width was divided
    /// by. Kept so that another outline traced on the same page can be brought into the same
    /// space after the fact, which is what a [`part`](Self::part) is.
    page: (f32, f32),
    turn: Turn,
    origin: (f32, f32),
    scale: f32,
}

/// One cell of the grid a mosaic is laid on: where its tile sits, how large it is, what shape it
/// settles into, and the rest of what the scatter decided for it.
///
/// Everything the scatter decides is decided here, cell by cell in grid order and before the
/// outline has its say, so that what a tile is does not depend on which tiles the outline let
/// through before it.
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
    /// How far off a ramp its colour sits, `-1.0..1.0`.
    pub(crate) drift: f32,
    /// How far off its true distance from anything it is read as sitting, `-1.0..1.0`.
    pub(crate) jitter: f32,
    /// How much later than its place in a sweep it moves, `0.0..1.0`.
    pub(crate) lag: f32,
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
            .map(|&(x, y)| turned((x * page.0, y * page.1), turn))
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
            page,
            turn,
            origin: (left, top),
            scale: width,
        }
    }

    /// Another outline traced on the same page, in this shape's own unit space: the same box,
    /// the same height, so it can be laid over a mosaic of this. For a piece of the shape, drawn
    /// over it.
    pub fn part(&self, outline: &[(f32, f32)]) -> Self {
        assert!(outline.len() >= 3, "an outline is three or more points");
        Self {
            outline: outline.iter().map(|&point| self.at(point)).collect(),
            ..self.clone()
        }
    }

    /// The same shape's space, holding `outline` -- already in unit space -- instead.
    pub(crate) fn outlined(&self, outline: Vec<(f32, f32)>) -> Self {
        Self {
            outline,
            ..self.clone()
        }
    }

    /// A box in unit space -- its top-left corner, and its width and height -- as the placement the
    /// box the shape is grown into resolves it against. For standing something on the shape that
    /// the mosaic need not know about.
    pub fn boxed(&self, (x, y): (f32, f32), (width, height): (f32, f32)) -> Location {
        Location::new().xs(
            left((x * 100.0).pct()).width((width * 100.0).pct()),
            top((y / self.height * 100.0).pct()).height((height / self.height * 100.0).pct()),
        )
    }

    /// How tall it is per unit of width.
    pub fn aspect(&self) -> f32 {
        self.height
    }

    /// A point of the sketch this was traced from, in unit space.
    pub fn at(&self, point: (f32, f32)) -> (f32, f32) {
        let (x, y) = turned((point.0 * self.page.0, point.1 * self.page.1), self.turn);
        (
            (x - self.origin.0) / self.scale,
            (y - self.origin.1) / self.scale,
        )
    }

    /// How large a tile centred at `center` may be for the outline to hold it whole, or `None`
    /// for a centre the outline does not hold, or holds too near its edge for any tile at all.
    /// What [`cells`](Self::cells) sizes a tile by, asked again of another outline.
    pub(crate) fn room(&self, center: (f32, f32)) -> Option<f32> {
        if !self.contains(center) {
            return None;
        }
        let room = self.clearance(center) * ROOM;
        (room >= LEAST).then_some(room)
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

    /// The outline as dashes, in order around it: each the points it runs through, its two ends
    /// and every vertex of the outline between them.
    ///
    /// Walked by arc length rather than by vertex, so a dash is the same length on a long edge and
    /// on a short one and the rhythm survives the jagged parts of the outline. Drawn through the
    /// vertices it passes rather than straight between its ends, so it follows the outline over
    /// those parts rather than cutting the corners off them.
    pub(crate) fn dashes(&self) -> Vec<Vec<(f32, f32)>> {
        let perimeter = self.perimeter();
        let stride = DASH + GAP;
        let count = (perimeter / stride).floor().max(1.0) as usize;
        // The stride is spread over what is actually there, so the last dash meets the first rather
        // than leaving a gap of whatever the division happened to leave over.
        let stride = perimeter / count as f32;
        (0..count)
            .map(|n| {
                let from = n as f32 * stride;
                self.stretch(from, from + stride - GAP)
            })
            .collect()
    }

    /// The mosaic: one tile per cell of a jittered grid that the outline holds, `pitch` apart.
    ///
    /// Every other row is set half a pitch over, so a row's tiles sit over the gaps in the row
    /// before rather than in line with its tiles: the packing that covers most for the count, and
    /// what lets the tiles be as small as they are without a gap opening where four corners meet.
    /// Every row runs a cell past the box, so the rows set over still reach its far edge.
    ///
    /// A tile never crosses the outline. What the edge leaves is what the tile gets, so the fill
    /// thins toward the boundary and stops rather than being cut off, and the shape is legible from
    /// the tiles alone.
    pub(crate) fn cells(&self, pitch: f32, scatter: &mut Scatter) -> Vec<Cell> {
        let mut cells = Vec::new();
        let columns = (1.0 / pitch).ceil() as i32;
        let rows = (self.height / pitch).ceil() as i32;
        for row in 0..rows {
            let over = match row % 2 {
                0 => 0.5,
                _ => 0.0,
            };
            for column in 0..=columns {
                let center = (
                    (column as f32 + over) * pitch + scatter.between(-WANDER, WANDER) * pitch,
                    (row as f32 + 0.5) * pitch + scatter.between(-WANDER, WANDER) * pitch,
                );
                let size = pitch * scatter.between(SPAN.0, SPAN.1);
                let sides = scatter.between(5.0, 7.0).round();
                let rounding = scatter.between(0.03, 0.18);
                let rotation = scatter.next() * TAU;
                let drift = scatter.between(-1.0, 1.0);
                let jitter = scatter.between(-1.0, 1.0);
                let lag = scatter.next();
                let Some(room) = self.room(center) else {
                    continue;
                };
                let size = size.min(room);
                cells.push(Cell {
                    center,
                    size,
                    sides,
                    rounding,
                    rotation,
                    drift,
                    jitter,
                    lag,
                });
            }
        }
        cells
    }

    /// Where each of `at` sits, and how far it reaches either way from its middle.
    ///
    /// A square -- on a vertex, or at a point -- is pulled off a vertex toward the middle of the
    /// shape so that it sits over the vertex rather than half outside it. Every square is the same
    /// size: `size` where one is stated, else the most the closest pair of them leaves -- two equal
    /// squares centred at `a` and `b` clear each other exactly when their side is no longer than
    /// the greater of `|dx|` and `|dy|`, so the least of those over every pair is what every square
    /// gets, and one square alone gets the largest the shape's box holds.
    ///
    /// Held inside the shape's own box, because a vertex on its edge would have a square hanging
    /// over the side. Sized again afterwards, since holding a square in moves it toward the
    /// others, and then nudged by whatever it asks for, in unit space on each axis. A box is where
    /// it was stated and nothing else.
    pub(crate) fn regions(&self, at: &[Region], size: Option<f32>) -> Vec<Area> {
        let middle = self.middle();
        let squares: Vec<(f32, f32)> = at
            .iter()
            .filter_map(|region| match *region {
                Region::Vertex(vertex, _) => {
                    let (x, y) = self.outline[vertex];
                    Some((x + (middle.0 - x) * INSET, y + (middle.1 - y) * INSET))
                }
                Region::Point(center) => Some(center),
                Region::Box(..) => None,
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
        let half = size.unwrap_or_else(|| room(&squares)) / 2.0;
        let held: Vec<(f32, f32)> = squares
            .iter()
            .map(|&(x, y)| (x.clamp(half, 1.0 - half), y.clamp(half, self.height - half)))
            .collect();
        let half = size.unwrap_or_else(|| room(&held) * (1.0 - APART)) / 2.0;
        // Nudged last, so that a square asking to sit elsewhere moves only itself.
        let mut held = held.into_iter();
        at.iter()
            .map(|region| match *region {
                Region::Vertex(_, (dx, dy)) => {
                    let (x, y) = held.next().expect("a square for every square");
                    Area {
                        center: (x + dx, y + dy),
                        half: (half, half),
                    }
                }
                Region::Point(_) => Area {
                    center: held.next().expect("a square for every square"),
                    half: (half, half),
                },
                Region::Box((x, y), (width, height)) => Area {
                    center: (x + width / 2.0, y + height / 2.0),
                    half: (width / 2.0, height / 2.0),
                },
            })
            .collect()
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

    /// The outline from `from` to `to`, both distances along it from its first vertex: the point
    /// at each end, and every vertex strictly between them in order. `to` past `from`, and by
    /// less than the whole way round; past the end of the outline is round to its start.
    fn stretch(&self, from: f32, to: f32) -> Vec<(f32, f32)> {
        let perimeter = self.perimeter();
        let from = from.rem_euclid(perimeter);
        let to = from + (to - from).min(perimeter);
        let mut points = vec![self.along(from)];
        // Twice round, since a stretch may run past the end of the outline and pick up vertices
        // from its start. Each edge's end vertex sits at `reached` along the outline.
        let mut reached = 0.0;
        for (a, b) in self.edges().chain(self.edges()) {
            reached += self::distance(a, b);
            if reached >= to {
                break;
            }
            if reached > from {
                points.push(b);
            }
        }
        points.push(self.along(to));
        points
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

/// A point of the sketch, turned.
fn turned((x, y): (f32, f32), turn: Turn) -> (f32, f32) {
    match turn {
        Turn::None => (x, y),
        Turn::Quarter => (-y, x),
        Turn::Half => (-x, -y),
        Turn::ThreeQuarter => (y, -x),
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
