use crate::compositor::layout::AspectRatio;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::prelude::Component;
#[derive(Component, Copy, Clone)]
pub struct MicroGrid {
    pub aspect: Option<AspectRatio>,
    pub min_width: Option<CoordinateUnit>,
    pub min_height: Option<CoordinateUnit>,
    pub max_width: Option<CoordinateUnit>,
    pub max_height: Option<CoordinateUnit>,
}
impl MicroGrid {
    pub fn determine(
        &self,
        coordinate: Coordinate<InterfaceContext>,
        alignment: &Alignment,
    ) -> Coordinate<InterfaceContext> {
        let anchor = self.adjusted(coordinate);
        let w = self.calc_w(anchor, alignment.w);
        let h = self.calc_h(anchor, alignment.h);
        let x = self.calc_x(anchor, alignment.x, w);
        let y = self.calc_y(anchor, alignment.y, h);
        Coordinate::default()
            .with_position((x, y))
            .with_area((w, h))
            .with_layer(anchor.layer + alignment.layer)
    }
    pub fn adjusted(
        &self,
        coordinate: Coordinate<InterfaceContext>,
    ) -> Coordinate<InterfaceContext> {
        let area = coordinate.section.area;
        let area = if let Some(w) = self.min_width {
            (area.width.max(w), area.height).into()
        } else {
            area
        };
        let area = if let Some(w) = self.max_width {
            (area.width.min(w), area.height).into()
        } else {
            area
        };
        let area = if let Some(h) = self.min_height {
            (area.width, area.height.max(h)).into()
        } else {
            area
        };
        let area = if let Some(h) = self.max_height {
            (area.width, area.height.min(h)).into()
        } else {
            area
        };
        let area = if let Some(ar) = self.aspect {
            ar.determine(area)
        } else {
            area
        };
        let pos = coordinate.section.position;
        let pos = if area.height < coordinate.section.height() {
            (
                pos.x,
                pos.y + (coordinate.section.height() - area.height) / 2f32,
            )
                .into()
        } else {
            pos
        };
        let pos = if area.width < coordinate.section.width() {
            (
                pos.x,
                pos.y + (coordinate.section.width() - area.width) / 2f32,
            )
                .into()
        } else {
            pos
        };
        Coordinate::default()
            .with_position(pos)
            .with_area(area)
            .with_layer(coordinate.layer)
    }
    pub fn calc_x(
        &self,
        anchor: Coordinate<InterfaceContext>,
        relative_alignment: RelativeAlignment,
        w: CoordinateUnit,
    ) -> CoordinateUnit {
        let location = relative_alignment.marker.location(anchor);
        let unit = relative_alignment.unit.value
            * match relative_alignment.unit.op {
                AlignmentOp::Fixed => 1.0,
                AlignmentOp::Percent => anchor.section.width(),
            };
        location.x + unit - w / 2f32
    }
    pub fn calc_y(
        &self,
        anchor: Coordinate<InterfaceContext>,
        relative_alignment: RelativeAlignment,
        h: CoordinateUnit,
    ) -> CoordinateUnit {
        let location = relative_alignment.marker.location(anchor);
        let unit = relative_alignment.unit.value
            * match relative_alignment.unit.op {
                AlignmentOp::Fixed => 1.0,
                AlignmentOp::Percent => anchor.section.width(),
            };
        location.x + unit - h / 2f32
    }
    pub fn calc_w(
        &self,
        anchor: Coordinate<InterfaceContext>,
        alignment_unit: AlignmentUnit,
    ) -> CoordinateUnit {
        alignment_unit.value
            * match alignment_unit.op {
                AlignmentOp::Fixed => 1.0,
                AlignmentOp::Percent => anchor.section.width(),
            }
    }
    pub fn calc_h(
        &self,
        anchor: Coordinate<InterfaceContext>,
        alignment_unit: AlignmentUnit,
    ) -> CoordinateUnit {
        alignment_unit.value
            * match alignment_unit.op {
                AlignmentOp::Fixed => 1.0,
                AlignmentOp::Percent => anchor.section.width(),
            }
    }
}
#[derive(Component, Copy, Clone)]
pub struct Alignment {
    pub x: RelativeAlignment,
    pub y: RelativeAlignment,
    pub w: AlignmentUnit,
    pub h: AlignmentUnit,
    pub layer: Layer,
}
impl Alignment {
    pub fn new<L: Into<Layer>>(
        x: RelativeAlignment,
        y: RelativeAlignment,
        w: AlignmentUnit,
        h: AlignmentUnit,
        l: L,
    ) -> Self {
        Self {
            x,
            y,
            w,
            h,
            layer: l.into(),
        }
    }
}
#[derive(Copy, Clone)]
pub enum RelativeMarker {
    Center,
    Left,
    Right,
    Top,
    Bottom,
    MidLeft,
    MidRight,
    MidTop,
    MidBottom,
}
impl RelativeMarker {
    pub fn location(self, anchor: Coordinate<InterfaceContext>) -> Position<InterfaceContext> {
        let center = anchor.section.center();
        let width = anchor.section.width();
        let height = anchor.section.height();
        match self {
            RelativeMarker::Center => center,
            RelativeMarker::Left => (0, 0).into(),
            RelativeMarker::Right => (anchor.section.right(), center.y).into(),
            RelativeMarker::Top => (center.x, 0.0).into(),
            RelativeMarker::Bottom => (center.x, anchor.section.bottom()).into(),
            RelativeMarker::MidLeft => center - (width / 4f32, 0.0).into(),
            RelativeMarker::MidRight => center + (width / 4f32, 0.0).into(),
            RelativeMarker::MidTop => center - (0.0, height / 4f32).into(),
            RelativeMarker::MidBottom => center + (0.0, height / 4f32).into(),
        }
    }
}
#[derive(Copy, Clone)]
pub struct RelativeAlignment {
    pub marker: RelativeMarker,
    pub unit: AlignmentUnit,
}
impl RelativeAlignment {
    pub fn new(marker: RelativeMarker, unit: AlignmentUnit) -> Self {
        Self { marker, unit }
    }
}
#[derive(Copy, Clone)]
pub struct AlignmentUnit {
    pub value: CoordinateUnit,
    pub op: AlignmentOp,
}
impl AlignmentUnit {
    pub fn new(value: CoordinateUnit, op: AlignmentOp) -> Self {
        Self { value, op }
    }
}
#[derive(Copy, Clone)]
pub enum AlignmentOp {
    Fixed,
    Percent,
}
pub trait AlignmentDesc {
    fn fixed_from(self, relative_marker: RelativeMarker) -> RelativeAlignment;
    fn percent_from(self, relative_marker: RelativeMarker) -> RelativeAlignment;
    fn fixed(self) -> AlignmentUnit;
    fn percent(self) -> AlignmentUnit;
}
macro_rules! impl_alignment_desc {
    ($($t:ty),*) => {
        $(
        impl AlignmentDesc for $t {
            fn fixed_from(self, relative_marker: RelativeMarker) -> RelativeAlignment {
                RelativeAlignment::new(relative_marker, AlignmentUnit::new(self as CoordinateUnit, AlignmentOp::Fixed))
            }
            fn percent_from(self, relative_marker: RelativeMarker) -> RelativeAlignment {
                RelativeAlignment::new(relative_marker, AlignmentUnit::new(self as CoordinateUnit, AlignmentOp::Percent))
            }
            fn fixed(self) -> AlignmentUnit {
                AlignmentUnit::new(self as CoordinateUnit, AlignmentOp::Fixed)
            }
            fn percent(self) -> AlignmentUnit {
                AlignmentUnit::new(self as CoordinateUnit, AlignmentOp::Percent)
            }
        }
        )*
    };
}
impl_alignment_desc!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);