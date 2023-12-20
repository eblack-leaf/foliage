use crate::compositor::layout::{Layout, Orientation, Threshold};
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use std::collections::HashMap;
#[derive(Copy, Clone)]
pub enum SegmentUnit {
    Fixed(CoordinateUnit),
    Relative(f32),
}
pub trait SegmentDesc {
    fn fixed(&self) -> SegmentUnit;
    fn relative(&self) -> SegmentUnit;
}
impl SegmentDesc for i32 {
    fn fixed(&self) -> SegmentUnit {
        SegmentUnit::Fixed(*self as f32)
    }

    fn relative(&self) -> SegmentUnit {
        SegmentUnit::Relative(*self as f32)
    }
}
impl SegmentDesc for u32 {
    fn fixed(&self) -> SegmentUnit {
        SegmentUnit::Fixed(*self as f32)
    }

    fn relative(&self) -> SegmentUnit {
        SegmentUnit::Relative(*self as f32)
    }
}
impl SegmentDesc for f32 {
    fn fixed(&self) -> SegmentUnit {
        SegmentUnit::Fixed(*self)
    }

    fn relative(&self) -> SegmentUnit {
        SegmentUnit::Relative(*self)
    }
}
#[derive(Copy, Clone)]
pub struct SegmentPosition {
    pub x: SegmentUnit,
    pub y: SegmentUnit,
}
impl SegmentPosition {
    pub fn new(x: SegmentUnit, y: SegmentUnit) -> Self {
        Self { x, y }
    }
}
impl From<(SegmentUnit, SegmentUnit)> for SegmentPosition {
    fn from(value: (SegmentUnit, SegmentUnit)) -> Self {
        SegmentPosition::new(value.0, value.1)
    }
}
impl SegmentPosition {
    pub(crate) fn calc(
        &self,
        viewport_section: Section<InterfaceContext>,
    ) -> Position<InterfaceContext> {
        let x = match self.x {
            SegmentUnit::Fixed(fix) => fix,
            SegmentUnit::Relative(rel) => viewport_section.width() * rel,
        };
        let y = match self.y {
            SegmentUnit::Fixed(fix) => fix,
            SegmentUnit::Relative(rel) => viewport_section.height() * rel,
        };
        (x, y).into()
    }
}
impl From<(SegmentUnit, SegmentUnit)> for SegmentArea {
    fn from(value: (SegmentUnit, SegmentUnit)) -> Self {
        SegmentArea::new(value.0, value.1)
    }
}
#[derive(Copy, Clone)]
pub struct SegmentArea {
    pub width: SegmentUnit,
    pub height: SegmentUnit,
}
impl SegmentArea {
    pub fn new(width: SegmentUnit, height: SegmentUnit) -> Self {
        Self { width, height }
    }
}

impl SegmentArea {
    pub(crate) fn calc(
        &self,
        viewport_section: Section<InterfaceContext>,
    ) -> Area<InterfaceContext> {
        let w = match self.width {
            SegmentUnit::Fixed(fix) => fix,
            SegmentUnit::Relative(rel) => viewport_section.width() * rel,
        };
        let h = match self.height {
            SegmentUnit::Fixed(fix) => fix,
            SegmentUnit::Relative(rel) => viewport_section.height() * rel,
        };
        (w, h).into()
    }
}
#[derive(Copy, Clone)]
pub struct Segment {
    pub pos: SegmentPosition,
    pub area: SegmentArea,
    pub layer: Layer,
}
impl Segment {
    pub fn new<SP: Into<SegmentPosition>, SA: Into<SegmentArea>, L: Into<Layer>>(
        pos: SP,
        area: SA,
        layer: L,
    ) -> Self {
        Self {
            pos: pos.into(),
            area: area.into(),
            layer: layer.into(),
        }
    }
    pub fn with_area<SA: Into<SegmentArea>>(mut self, sa: SA) -> Self {
        self.area = sa.into();
        self
    }
}
pub struct ResponsiveSegment(pub HashMap<Layout, Segment>);

impl ResponsiveSegment {
    pub(crate) fn coordinate(
        &self,
        layout: &Layout,
        viewport_section: Section<InterfaceContext>,
    ) -> Option<Coordinate<InterfaceContext>> {
        if let Some(segment) = self.0.get(layout) {
            let mut coordinate = Coordinate::default();
            coordinate.section.position = segment.pos.calc(viewport_section);
            coordinate.section.area = segment.area.calc(viewport_section);
            coordinate.layer = segment.layer;
            Some(coordinate)
        } else {
            None
        }
    }
}

impl ResponsiveSegment {
    pub fn all(segment: Segment) -> Self {
        Self::portrait_mobile(segment)
            .with_portrait_tablet(segment)
            .with_portrait_desktop(segment)
            .with_portrait_workstation(segment)
            .with_landscape_mobile(segment)
            .with_landscape_tablet(segment)
            .with_landscape_desktop(segment)
            .with_landscape_workstation(segment)
    }
    pub fn portrait_mobile(segment: Segment) -> Self {
        Self {
            0: {
                let mut map = HashMap::new();
                map.insert(
                    Layout::new(Orientation::Portrait, Threshold::Mobile),
                    segment,
                );
                map
            },
        }
    }
    pub fn landscape_mobile(segment: Segment) -> Self {
        Self {
            0: {
                let mut map = HashMap::new();
                map.insert(
                    Layout::new(Orientation::Landscape, Threshold::Mobile),
                    segment,
                );
                map
            },
        }
    }
    pub fn portrait_tablet(segment: Segment) -> Self {
        Self {
            0: {
                let mut map = HashMap::new();
                map.insert(
                    Layout::new(Orientation::Portrait, Threshold::Tablet),
                    segment,
                );
                map
            },
        }
    }
    pub fn landscape_tablet(segment: Segment) -> Self {
        Self {
            0: {
                let mut map = HashMap::new();
                map.insert(
                    Layout::new(Orientation::Landscape, Threshold::Tablet),
                    segment,
                );
                map
            },
        }
    }
    pub fn portrait_desktop(segment: Segment) -> Self {
        Self {
            0: {
                let mut map = HashMap::new();
                map.insert(
                    Layout::new(Orientation::Portrait, Threshold::Desktop),
                    segment,
                );
                map
            },
        }
    }
    pub fn landscape_desktop(segment: Segment) -> Self {
        Self {
            0: {
                let mut map = HashMap::new();
                map.insert(
                    Layout::new(Orientation::Landscape, Threshold::Desktop),
                    segment,
                );
                map
            },
        }
    }
    pub fn portrait_workstation(segment: Segment) -> Self {
        Self {
            0: {
                let mut map = HashMap::new();
                map.insert(
                    Layout::new(Orientation::Portrait, Threshold::Workstation),
                    segment,
                );
                map
            },
        }
    }
    pub fn landscape_workstation(segment: Segment) -> Self {
        Self {
            0: {
                let mut map = HashMap::new();
                map.insert(
                    Layout::new(Orientation::Landscape, Threshold::Workstation),
                    segment,
                );
                map
            },
        }
    }
    pub fn with_landscape_mobile(mut self, segment: Segment) -> Self {
        self.0.insert(
            Layout::new(Orientation::Landscape, Threshold::Mobile),
            segment,
        );
        self
    }
    pub fn with_portrait_tablet(mut self, segment: Segment) -> Self {
        self.0.insert(
            Layout::new(Orientation::Portrait, Threshold::Tablet),
            segment,
        );
        self
    }
    pub fn with_landscape_tablet(mut self, segment: Segment) -> Self {
        self.0.insert(
            Layout::new(Orientation::Landscape, Threshold::Tablet),
            segment,
        );
        self
    }
    pub fn with_portrait_desktop(mut self, segment: Segment) -> Self {
        self.0.insert(
            Layout::new(Orientation::Portrait, Threshold::Desktop),
            segment,
        );
        self
    }
    pub fn with_landscape_desktop(mut self, segment: Segment) -> Self {
        self.0.insert(
            Layout::new(Orientation::Landscape, Threshold::Desktop),
            segment,
        );
        self
    }
    pub fn with_portrait_workstation(mut self, segment: Segment) -> Self {
        self.0.insert(
            Layout::new(Orientation::Portrait, Threshold::Workstation),
            segment,
        );
        self
    }
    pub fn with_landscape_workstation(mut self, segment: Segment) -> Self {
        self.0.insert(
            Layout::new(Orientation::Landscape, Threshold::Workstation),
            segment,
        );
        self
    }
}
