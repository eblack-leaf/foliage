//! Logical pixels, with the origin at the top-left of the surface and `y` increasing downward.
//!
//! One unit throughout. Device pixels exist only inside the render backend, where the scale factor
//! is applied; nothing an app writes or reads is in them.
//!
//! Every type here is `#[repr(C)]` over `f32` and nothing else, which is the layout a shader reads
//! a `vec2` and a `vec4` in. That is a commitment rather than an accident: it is what lets a
//! renderer's instance be built from these directly instead of from a mirror of them that has to
//! be kept in step.

use bytemuck::{Pod, Zeroable};

/// A point on the surface, in logical pixels.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// How far `self` is from `other`, as a pair of signed distances.
    pub fn to(self, other: Self) -> Self {
        Self::new(other.x - self.x, other.y - self.y)
    }

    /// This point moved by `offset`.
    pub fn moved(self, offset: Self) -> Self {
        Self::new(self.x + offset.x, self.y + offset.y)
    }

    /// The distance along one axis.
    pub(crate) fn along(self, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => self.x,
            Axis::Vertical => self.y,
        }
    }
}

/// A width and a height, in logical pixels.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Area {
    pub width: f32,
    pub height: f32,
}

impl Area {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Where an element is and how large it is.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Section {
    pub position: Position,
    pub area: Area,
}

impl Section {
    pub fn new(position: Position, area: Area) -> Self {
        Self { position, area }
    }

    /// A section from its four edges. `right` and `bottom` below their counterparts give a zero
    /// extent rather than a negative one.
    pub fn from_edges(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            position: Position::new(left, top),
            area: Area::new((right - left).max(0.0), (bottom - top).max(0.0)),
        }
    }

    pub fn left(&self) -> f32 {
        self.position.x
    }

    pub fn top(&self) -> f32 {
        self.position.y
    }

    pub fn right(&self) -> f32 {
        self.position.x + self.area.width
    }

    pub fn bottom(&self) -> f32 {
        self.position.y + self.area.height
    }

    pub fn width(&self) -> f32 {
        self.area.width
    }

    pub fn height(&self) -> f32 {
        self.area.height
    }

    /// The midpoint of the box.
    pub fn center(&self) -> Position {
        Position::new(
            self.position.x + self.area.width / 2.0,
            self.position.y + self.area.height / 2.0,
        )
    }

    /// The box both of these cover, which is empty where they do not overlap.
    pub fn intersect(&self, other: Section) -> Section {
        Section::from_edges(
            self.left().max(other.left()),
            self.top().max(other.top()),
            self.right().min(other.right()),
            self.bottom().min(other.bottom()),
        )
    }

    /// Whether the box covers anything at all.
    pub fn is_empty(&self) -> bool {
        self.area.width <= 0.0 || self.area.height <= 0.0
    }

    /// Whether `point` falls inside the box, near edges inclusive and far edges exclusive.
    pub fn contains(&self, point: Position) -> bool {
        point.x >= self.left()
            && point.x < self.right()
            && point.y >= self.top()
            && point.y < self.bottom()
    }
}

/// Which of the two axes a value belongs to.
///
/// The two are not interchangeable: the horizontal axis resolves first, so a horizontal length is
/// available to a vertical role but never the reverse. See [`placement`](crate::placement).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Axis {
    Horizontal,
    Vertical,
}

/// Which axes a declaration covers.
///
/// The two axes are stated together because the answer is almost never the same for both: a column
/// scrolls down and not across, a carousel across and not down. One value naming which is meant is
/// what keeps the pair from being two independent flags that can disagree.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Axes {
    Horizontal,
    Vertical,
    Both,
}

impl Axes {
    pub(crate) fn covers(self, axis: Axis) -> bool {
        match self {
            Axes::Horizontal => axis == Axis::Horizontal,
            Axes::Vertical => axis == Axis::Vertical,
            Axes::Both => true,
        }
    }
}
