//! Logical pixels, with the origin at the top-left of the surface and `y` increasing downward.
//!
//! One unit throughout. Device pixels exist only inside the render backend, where the scale factor
//! is applied; nothing an app writes or reads is in them.

/// A point on the surface, in logical pixels.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A width and a height, in logical pixels.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
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
#[derive(Copy, Clone, Debug, Default, PartialEq)]
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
