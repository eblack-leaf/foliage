use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::{CoordinateContext, CoordinateUnit, DeviceContext, InterfaceContext};
use bevy_ecs::bundle::Bundle;
use serde::{Deserialize, Serialize};

#[derive(Bundle, Copy, Clone, PartialOrd, PartialEq, Default, Serialize, Deserialize)]
pub struct Section<Context: CoordinateContext> {
    pub position: Position<Context>,
    pub area: Area<Context>,
}

impl<Context: CoordinateContext> Section<Context> {
    pub fn new<P: Into<Position<Context>>, A: Into<Area<Context>>>(position: P, area: A) -> Self {
        Self {
            position: position.into(),
            area: area.into(),
        }
    }
    #[allow(unused)]
    pub(crate) fn center(&self) -> Position<Context> {
        let x = self.position.x + self.width() / 2f32;
        let y = self.position.y + self.height() / 2f32;
        Position::new(x, y)
    }
    /// Can be instantiated with specific points
    pub fn from_left_top_right_bottom(
        left: CoordinateUnit,
        top: CoordinateUnit,
        right: CoordinateUnit,
        bottom: CoordinateUnit,
    ) -> Self {
        Self {
            position: (left, top).into(),
            area: (right - left, bottom - top).into(),
        }
    }
    pub fn width(&self) -> CoordinateUnit {
        self.area.width
    }
    pub fn height(&self) -> CoordinateUnit {
        self.area.height
    }
    pub fn left(&self) -> CoordinateUnit {
        self.position.x
    }
    pub fn right(&self) -> CoordinateUnit {
        self.position.x + self.area.width
    }
    pub fn top(&self) -> CoordinateUnit {
        self.position.y
    }
    pub fn bottom(&self) -> CoordinateUnit {
        self.position.y + self.area.height
    }
    /// returns if any port of this section is touching the other
    pub fn is_touching(&self, other: Self) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }
    /// returns true if section overlaps the other
    pub fn is_overlapping(&self, other: Self) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }
    /// returns true if the position resides in the section
    pub fn contains(&self, position: Position<Context>) -> bool {
        if position.x >= self.left()
            && position.x <= self.right()
            && position.y >= self.top()
            && position.y <= self.bottom()
        {
            return true;
        }
        false
    }
    /// returns an Option of the overlap between the sections
    pub fn intersection(&self, other: Self) -> Option<Self> {
        if !self.is_overlapping(other) {
            return None;
        }
        let top = self.top().max(other.top());
        let bottom = self.bottom().min(other.bottom());
        let left = self.left().max(other.left());
        let right = self.right().min(other.right());
        Option::from(Self::from_left_top_right_bottom(left, top, right, bottom))
    }
    pub fn with_position(mut self, position: Position<Context>) -> Self {
        self.position = position;
        self
    }
    pub fn with_area(mut self, area: Area<Context>) -> Self {
        self.area = area;
        self
    }
}

impl Section<InterfaceContext> {
    #[allow(unused)]
    pub(crate) fn to_device(self, scale_factor: CoordinateUnit) -> Section<DeviceContext> {
        Section::<DeviceContext>::new(
            self.position.to_device(scale_factor),
            self.area.to_device(scale_factor),
        )
    }
}

impl Section<DeviceContext> {
    #[allow(unused)]
    pub(crate) fn to_interface(self, scale_factor: CoordinateUnit) -> Section<InterfaceContext> {
        Section::<InterfaceContext>::new(
            self.position.to_interface(scale_factor),
            self.area.to_interface(scale_factor),
        )
    }
}

impl<Context: CoordinateContext, P: Into<Position<Context>>, A: Into<Area<Context>>> From<(P, A)>
    for Section<Context>
{
    fn from(value: (P, A)) -> Self {
        Self::new(value.0.into(), value.1.into())
    }
}
