use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::layout::AspectRatio;
use bevy_ecs::prelude::Component;
#[derive(Component, Copy, Clone)]
pub struct MicroGrid {
    aspect: Option<AspectRatio>,
    min_width: Option<CoordinateUnit>,
    min_height: Option<CoordinateUnit>,
    max_width: Option<CoordinateUnit>,
    max_height: Option<CoordinateUnit>,
}
impl Default for MicroGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl MicroGrid {
    pub fn new() -> Self {
        Self {
            aspect: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }
    pub fn aspect_ratio<AR: Into<AspectRatio>>(mut self, ar: AR) -> Self {
        self.aspect.replace(ar.into());
        self
    }
    pub fn min_width(mut self, mw: CoordinateUnit) -> Self {
        self.min_width.replace(mw);
        self
    }
    pub fn max_width(mut self, mw: CoordinateUnit) -> Self {
        self.max_width.replace(mw);
        self
    }
    pub fn min_height(mut self, mw: CoordinateUnit) -> Self {
        self.min_height.replace(mw);
        self
    }
    pub fn max_height(mut self, mw: CoordinateUnit) -> Self {
        self.max_height.replace(mw);
        self
    }
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
        let area = if let Some(ar) = self.aspect {
            ar.determine(area)
        } else {
            area
        };
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
                pos.x + (coordinate.section.width() - area.width) / 2f32,
                pos.y,
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
        let x_offset = if let Some(a) = relative_alignment.unit.align {
            match a {
                Align::Center => w / 2f32,
                Align::Left => 0.0,
                Align::Right => w,
                _ => 0.0,
            }
        } else {
            match relative_alignment.marker {
                RelativeMarker::Left => 0.0,
                RelativeMarker::Right => w,
                _ => w / 2f32,
            }
        };
        location.x + unit - x_offset
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
                AlignmentOp::Percent => anchor.section.height(),
            };
        let y_offset = if let Some(a) = relative_alignment.unit.align {
            match a {
                Align::Center => h / 2f32,
                Align::Top => 0.0,
                Align::Bottom => h,
                _ => 0.0,
            }
        } else {
            match relative_alignment.marker {
                RelativeMarker::Top => 0.0,
                RelativeMarker::Bottom => h,
                _ => h / 2f32,
            }
        };
        location.y + unit - y_offset
    }
    pub fn calc_w(
        &self,
        anchor: Coordinate<InterfaceContext>,
        alignment_unit: AlignmentUnit,
    ) -> CoordinateUnit {
        alignment_unit.value
            * match alignment_unit.op {
                AlignmentOp::Fixed => 1.0,
                AlignmentOp::Percent => {
                    if let Some(AnchorDim::Height) = alignment_unit.dim {
                        anchor.section.height()
                    } else {
                        anchor.section.width()
                    }
                }
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
                AlignmentOp::Percent => {
                    if let Some(AnchorDim::Width) = alignment_unit.dim {
                        anchor.section.width()
                    } else {
                        anchor.section.height()
                    }
                }
            }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct Alignment {
    pub x: RelativeAlignment,
    pub y: RelativeAlignment,
    pub w: AlignmentUnit,
    pub h: AlignmentUnit,
    pub layer: Layer,
}
impl Alignment {
    pub fn new(
        x: RelativeAlignment,
        y: RelativeAlignment,
        w: AlignmentUnit,
        h: AlignmentUnit,
    ) -> Self {
        Self {
            x,
            y,
            w,
            h,
            layer: Layer::default(),
        }
    }
    pub fn with_layer<L: Into<Layer>>(mut self, l: L) -> Self {
        self.layer = l.into();
        self
    }
}
#[derive(Copy, Clone, Default)]
pub enum RelativeMarker {
    #[default]
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
            RelativeMarker::Left => (anchor.section.left(), center.y).into(),
            RelativeMarker::Right => (anchor.section.right(), center.y).into(),
            RelativeMarker::Top => (center.x, anchor.section.top()).into(),
            RelativeMarker::Bottom => (center.x, anchor.section.bottom()).into(),
            RelativeMarker::MidLeft => center - (width / 4f32, 0.0).into(),
            RelativeMarker::MidRight => center + (width / 4f32, 0.0).into(),
            RelativeMarker::MidTop => center - (0.0, height / 4f32).into(),
            RelativeMarker::MidBottom => center + (0.0, height / 4f32).into(),
        }
    }
}
#[derive(Copy, Clone, Default)]
pub struct RelativeAlignment {
    pub marker: RelativeMarker,
    pub unit: AlignmentUnit,
}
impl RelativeAlignment {
    pub fn new(marker: RelativeMarker, unit: AlignmentUnit) -> Self {
        Self { marker, unit }
    }
    pub fn align(mut self, align: Align) -> Self {
        self.unit = self.unit.align(align);
        self
    }
}
#[derive(Copy, Clone, Default)]
pub struct AlignmentUnit {
    pub value: CoordinateUnit,
    pub op: AlignmentOp,
    pub dim: Option<AnchorDim>,
    pub align: Option<Align>,
}
#[derive(Copy, Clone, Default)]
pub enum Align {
    #[default]
    Center,
    Left,
    Right,
    Top,
    Bottom,
}
impl AlignmentUnit {
    pub fn new(
        value: CoordinateUnit,
        op: AlignmentOp,
        dim: Option<AnchorDim>,
        align: Option<Align>,
    ) -> Self {
        Self {
            value,
            op,
            dim,
            align,
        }
    }
    pub fn align(mut self, align: Align) -> Self {
        self.align.replace(align);
        self
    }
}
#[derive(Copy, Clone, Default)]
pub enum AlignmentOp {
    Fixed,
    #[default]
    Percent,
}
#[derive(Copy, Clone, Default)]
pub enum AnchorDim {
    #[default]
    Width,
    Height,
}
pub trait AlignmentDesc {
    fn fixed_from(self, relative_marker: RelativeMarker) -> RelativeAlignment;
    fn percent_from(self, relative_marker: RelativeMarker) -> RelativeAlignment;
    fn fixed(self) -> AlignmentUnit;
    fn percent_of(self, dim: AnchorDim) -> AlignmentUnit;
}
macro_rules! impl_alignment_desc {
    ($($t:ty),*) => {
        $(
        impl AlignmentDesc for $t {
            fn fixed_from(self, relative_marker: RelativeMarker) -> RelativeAlignment {
                RelativeAlignment::new(
                    relative_marker,
                    AlignmentUnit::new(
                        self as CoordinateUnit,
                        AlignmentOp::Fixed,
                        None,
                        None
                    )
                )
            }
            fn percent_from(self, relative_marker: RelativeMarker) -> RelativeAlignment {
                RelativeAlignment::new(
                    relative_marker,
                    AlignmentUnit::new(
                        self as CoordinateUnit,
                        AlignmentOp::Percent,
                        None,
                        None
                    )
                )
            }
            fn fixed(self) -> AlignmentUnit {
                AlignmentUnit::new(
                    self as CoordinateUnit,
                    AlignmentOp::Fixed,
                    None,
                    None
                )
            }
            fn percent_of(self, dim: AnchorDim) -> AlignmentUnit {
                AlignmentUnit::new(
                    self as CoordinateUnit,
                    AlignmentOp::Percent,
                    Some(dim),
                    None
                )
            }
        }
        )*
    };
}
impl_alignment_desc!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);
