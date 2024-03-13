use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::layout::{AspectRatio, Layout};
use bevy_ecs::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::Div;

#[derive(Default, Copy, Clone)]
pub struct GridTemplate {
    columns: SegmentValue,
    rows: SegmentValue,
}

impl GridTemplate {
    const fn new(columns: SegmentValue, rows: SegmentValue) -> GridTemplate {
        Self { columns, rows }
    }
}
pub enum GridRelativeValue {
    Anchored(CoordinateUnit),
    Fixed(CoordinateUnit),
}
impl GridRelativeValue {
    fn value(self) -> CoordinateUnit {
        match self {
            GridRelativeValue::Anchored(value) => value,
            GridRelativeValue::Fixed(value) => value,
        }
    }
}
#[derive(Resource, Default, Copy, Clone)]
pub struct MacroGrid {
    gap_x: CoordinateUnit,
    gap_y: CoordinateUnit,
    template: GridTemplate,
}
impl MacroGrid {
    pub const GAP_RATIO: CoordinateUnit = 0.15;
    pub const DEFAULT_GAP: CoordinateUnit = 8.0;
    pub const fn new(columns: SegmentValue, rows: SegmentValue) -> Self {
        Self {
            gap_x: Self::DEFAULT_GAP,
            gap_y: Self::DEFAULT_GAP,
            template: GridTemplate::new(columns, rows),
        }
    }
    pub const fn explicit(
        columns: SegmentValue,
        rows: SegmentValue,
        gx: CoordinateUnit,
        gy: CoordinateUnit,
    ) -> Self {
        Self {
            gap_x: gx,
            gap_y: gy,
            template: GridTemplate::new(columns, rows),
        }
    }
    pub fn gap_x(&self, dim: CoordinateUnit) -> CoordinateUnit {
        self.gap_x.min(self.column_width(dim) * Self::GAP_RATIO)
    }
    pub fn gap_y(&self, dim: CoordinateUnit) -> CoordinateUnit {
        self.gap_x.min(self.row_height(dim) * Self::GAP_RATIO)
    }
    pub fn horizontal(&self, area: Area<InterfaceContext>, unit: SegmentUnit) -> GridRelativeValue {
        if let Some(a) = unit.absolute {
            return GridRelativeValue::Fixed(a + unit.offset.unwrap_or_default());
        }
        if let Some(r) = unit.relative {
            return GridRelativeValue::Fixed(r * area.width + unit.offset.unwrap_or_default());
        }
        GridRelativeValue::Anchored(
            unit.value() * self.column_element_width(area.width)
                + unit.value() * self.gap_x(area.width)
                + (unit.value() - 1f32).max(0.0) * self.gap_x(area.width)
                - self.column_element_width(area.width) * unit.bias.factor()
                + unit.offset.unwrap_or_default(),
        )
    }
    pub fn vertical(&self, area: Area<InterfaceContext>, unit: SegmentUnit) -> GridRelativeValue {
        if let Some(a) = unit.absolute {
            return GridRelativeValue::Fixed(a + unit.offset.unwrap_or_default());
        }
        if let Some(r) = unit.relative {
            return GridRelativeValue::Fixed(r * area.height + unit.offset.unwrap_or_default());
        }
        GridRelativeValue::Anchored(
            unit.value() * self.row_element_height(area.height)
                + unit.value() * self.gap_y(area.height)
                + (unit.value() - 1f32).max(0.0) * self.gap_y(area.height)
                - self.row_element_height(area.height) * unit.bias.factor()
                + unit.offset.unwrap_or_default(),
        )
    }
    pub fn column_width(&self, dim: CoordinateUnit) -> CoordinateUnit {
        dim / self.template.columns as CoordinateUnit
    }
    pub fn row_height(&self, dim: CoordinateUnit) -> CoordinateUnit {
        dim / self.template.rows as CoordinateUnit
    }
    pub fn column_element_width(&self, dim: CoordinateUnit) -> CoordinateUnit {
        self.column_width(dim) - self.gap_x(dim) * 2f32
    }
    pub fn row_element_height(&self, dim: CoordinateUnit) -> CoordinateUnit {
        self.row_height(dim) - self.gap_y(dim) * 2f32
    }
    pub fn assign_gap(mut self, descriptor: GapDescriptor, value: CoordinateUnit) -> Self {
        match descriptor {
            GapDescriptor::Horizontal => {
                self.gap_x = value;
            }
            GapDescriptor::Vertical => {
                self.gap_y = value;
            }
            GapDescriptor::Both => {
                self.gap_x = value;
                self.gap_y = value;
            }
        }
        self
    }
    pub fn calculate_coordinate(
        &self,
        section: Section<InterfaceContext>,
        horizontal: SegmentUnitDescriptor,
        vertical: SegmentUnitDescriptor,
        layer: Layer,
        justification: Option<Justify>,
        aspect_ratio: Option<AspectRatio>,
    ) -> Coordinate<InterfaceContext> {
        let left = self.horizontal(section.area, horizontal.begin).value();
        let top = self.vertical(section.area, vertical.begin).value();
        let width_or_right = self.horizontal(section.area, horizontal.end);
        let height_or_bottom = self.vertical(section.area, vertical.end);
        let width = match width_or_right {
            GridRelativeValue::Anchored(value) => value - left,
            GridRelativeValue::Fixed(value) => value,
        };
        let height = match height_or_bottom {
            GridRelativeValue::Anchored(value) => value - top,
            GridRelativeValue::Fixed(value) => value,
        };
        let initial_w = width;
        let initial_h = height;
        let width = if let Some(w) = horizontal.min {
            width.max(w)
        } else {
            width
        };
        let height = if let Some(h) = vertical.min {
            height.max(h)
        } else {
            height
        };
        let (width, height) = if let Some(ar) = aspect_ratio {
            let d = ar.determine((width, height));
            (d.width, d.height)
        } else {
            (width, height)
        };
        let width = if let Some(w) = horizontal.max {
            width.min(w)
        } else {
            width
        };
        let height = if let Some(h) = vertical.max {
            height.min(h)
        } else {
            height
        };
        let (left, width) = if width < initial_w {
            let diff = initial_w - width;
            let justification = justification.unwrap_or(Justify::Center);
            let adjusted_left = match justification {
                Justify::Center | Justify::Top | Justify::Bottom => left + diff.div(2.0),
                Justify::Right | Justify::RightTop | Justify::RightBottom => left + diff,
                _ => left,
            };
            (adjusted_left, width)
        } else {
            (left, width)
        };
        let (top, height) = if height < initial_h {
            let diff = initial_h - height;
            let justification = justification.unwrap_or(Justify::Center);
            let adjusted_top = match justification {
                Justify::Center | Justify::Left | Justify::Right => top + diff.div(2.0),
                Justify::Bottom | Justify::RightBottom | Justify::LeftBottom => top + diff,
                _ => top,
            };
            (adjusted_top, height)
        } else {
            (top, height)
        };
        Coordinate::new(
            Section::new(
                (left + section.left(), top - section.top()),
                (width, height),
            ),
            layer,
        )
    }
}
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq)]
pub struct Gap {
    value: SegmentValue,
}
#[derive(Copy, Clone, Serialize, Deserialize, Hash, PartialEq)]
pub enum GapDescriptor {
    Horizontal,
    Vertical,
    Both,
}
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Default)]
pub enum Justify {
    #[default]
    Center,
    Left,
    Right,
    Bottom,
    Top,
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
}
#[derive(Clone, Component)]
pub struct ResponsiveSegment {
    base: Segment,
    justification: Option<Justify>,
    layer: Layer,
    exceptions: HashMap<Layout, Segment>,
    negations: HashSet<Layout>,
}
impl ResponsiveSegment {
    pub fn coordinate(
        &self,
        layout: Layout,
        section: Section<InterfaceContext>,
        current: &MacroGrid,
    ) -> Option<Coordinate<InterfaceContext>> {
        if self.negations.contains(&layout) {
            return None;
        }
        Some(current.calculate_coordinate(
            section,
            self.horizontal_value(&layout),
            self.vertical_value(&layout),
            self.layer,
            self.justification,
            self.aspect_ratio(&layout),
        ))
    }
    pub fn aspect_ratio(&self, layout: &Layout) -> Option<AspectRatio> {
        self.exceptions
            .get(layout)
            .unwrap_or(&self.base)
            .aspect_ratio
    }
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justification.replace(justify);
        self
    }
    fn vertical_value(&self, layout: &Layout) -> SegmentUnitDescriptor {
        if let Some(exc) = self.exceptions.get(layout).cloned() {
            exc.vertical
        } else {
            self.base.vertical
        }
    }

    fn horizontal_value(&self, layout: &Layout) -> SegmentUnitDescriptor {
        if let Some(exc) = self.exceptions.get(layout).cloned() {
            exc.horizontal
        } else {
            self.base.horizontal
        }
    }
    pub fn base(segment: Segment) -> Self {
        Self {
            base: segment,
            layer: Default::default(),
            justification: None,
            exceptions: Default::default(),
            negations: HashSet::new(),
        }
    }
    pub fn at_layer<L: Into<Layer>>(mut self, l: L) -> Self {
        self.layer = l.into();
        self
    }
    pub fn exception<L: AsRef<[Layout]>>(mut self, layouts: L, segment: Segment) -> Self {
        let layouts = layouts.as_ref();
        for l in layouts.iter() {
            self.exceptions.insert(*l, segment);
        }
        self
    }
    pub fn without_portrait_mobile(mut self) -> Self {
        self.negations.insert(Layout::PORTRAIT_MOBILE);
        self
    }
    pub fn without_portrait_tablet(mut self) -> Self {
        self.negations.insert(Layout::PORTRAIT_TABLET);
        self
    }
    pub fn without_portrait_desktop(mut self) -> Self {
        self.negations.insert(Layout::PORTRAIT_DESKTOP);
        self
    }
    pub fn without_portrait_workstation(mut self) -> Self {
        self.negations.insert(Layout::PORTRAIT_WORKSTATION);
        self
    }
    pub fn without_landscape_mobile(mut self) -> Self {
        self.negations.insert(Layout::LANDSCAPE_MOBILE);
        self
    }
    pub fn without_landscape_tablet(mut self) -> Self {
        self.negations.insert(Layout::LANDSCAPE_TABLET);
        self
    }
    pub fn without_landscape_desktop(mut self) -> Self {
        self.negations.insert(Layout::LANDSCAPE_DESKTOP);
        self
    }
    pub fn without_landscape_workstation(mut self) -> Self {
        self.negations.insert(Layout::LANDSCAPE_WORKSTATION);
        self
    }
}
#[derive(Copy, Clone)]
pub struct SegmentUnitDescriptor {
    begin: SegmentUnit,
    end: SegmentUnit,
    min: Option<CoordinateUnit>,
    max: Option<CoordinateUnit>,
}
impl SegmentUnitDescriptor {
    pub fn new(begin: SegmentUnit, end: SegmentUnit) -> SegmentUnitDescriptor {
        Self {
            begin,
            end,
            min: None,
            max: None,
        }
    }
    pub fn with_bounds(
        begin: SegmentUnit,
        end: SegmentUnit,
        min: Option<CoordinateUnit>,
        max: Option<CoordinateUnit>,
    ) -> Self {
        Self {
            begin,
            end,
            min,
            max,
        }
    }
}
#[derive(Copy, Clone)]
pub struct Segment {
    pub horizontal: SegmentUnitDescriptor,
    pub vertical: SegmentUnitDescriptor,
    pub aspect_ratio: Option<AspectRatio>,
}
impl<AR: Into<AspectRatio>>
    From<(
        WellFormedSegmentUnitDescriptor,
        WellFormedSegmentUnitDescriptor,
        AR,
    )> for Segment
{
    fn from(
        value: (
            WellFormedSegmentUnitDescriptor,
            WellFormedSegmentUnitDescriptor,
            AR,
        ),
    ) -> Self {
        Self::new(value.0, value.1).with_aspect(value.2.into())
    }
}
impl
    From<(
        WellFormedSegmentUnitDescriptor,
        WellFormedSegmentUnitDescriptor,
    )> for Segment
{
    fn from(
        value: (
            WellFormedSegmentUnitDescriptor,
            WellFormedSegmentUnitDescriptor,
        ),
    ) -> Self {
        Self::new(value.0, value.1)
    }
}
impl Segment {
    pub fn new(
        horizontal: WellFormedSegmentUnitDescriptor,
        vertical: WellFormedSegmentUnitDescriptor,
    ) -> Segment {
        Self {
            horizontal: horizontal.normal(),
            vertical: vertical.normal(),
            aspect_ratio: None,
        }
    }
    pub fn with_aspect<AR: Into<AspectRatio>>(mut self, aspect_ratio: AR) -> Self {
        self.aspect_ratio.replace(aspect_ratio.into());
        self
    }
}
pub trait SegmentUnitDesc {
    fn near(self) -> SegmentUnit;
    fn far(self) -> SegmentUnit;
    fn absolute(self) -> SegmentUnit;
    fn relative(self) -> SegmentUnit;
}
macro_rules! impl_segment_unit_desc {
    ($($elem:ty),*) => {
        $(impl SegmentUnitDesc for $elem {
            fn near(self) -> SegmentUnit {
                SegmentUnit::new(self as SegmentValue, SegmentBias::Near)
            }
            fn far(self) -> SegmentUnit {
                SegmentUnit::new(self as SegmentValue, SegmentBias::Far)
            }
            fn absolute(self) -> SegmentUnit {
                SegmentUnit::absolute(self as CoordinateUnit)
            }
            fn relative(self) -> SegmentUnit {
                SegmentUnit::relative(self as CoordinateUnit)
            }
        })*
    };
}
impl_segment_unit_desc!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);
#[derive(Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Default)]
pub enum SegmentBias {
    #[default]
    Near,
    Far,
}
impl SegmentBias {
    pub(crate) fn factor(self) -> CoordinateUnit {
        match self {
            SegmentBias::Near => 1.0,
            SegmentBias::Far => 0.0,
        }
    }
}
pub type SegmentValue = u8;
#[derive(Copy, Clone, Default)]
pub struct SegmentUnit {
    value: SegmentValue,
    bias: SegmentBias,
    absolute: Option<CoordinateUnit>,
    relative: Option<CoordinateUnit>,
    offset: Option<CoordinateUnit>,
}
impl SegmentUnit {
    fn value(&self) -> CoordinateUnit {
        self.value as CoordinateUnit
    }
    pub fn new(value: SegmentValue, bias: SegmentBias) -> Self {
        Self {
            value,
            bias,
            absolute: None,
            relative: None,
            offset: None,
        }
    }
    pub fn absolute(value: CoordinateUnit) -> Self {
        Self {
            value: SegmentValue::default(),
            bias: SegmentBias::Near,
            absolute: Some(value),
            relative: None,
            offset: None,
        }
    }
    pub fn relative(value: CoordinateUnit) -> Self {
        Self {
            value: 0,
            bias: SegmentBias::Near,
            absolute: None,
            relative: Some(value),
            offset: None,
        }
    }
    pub fn to(self, su: SegmentUnit) -> WellFormedSegmentUnitDescriptor {
        WellFormedSegmentUnitDescriptor::new(self, su)
    }
    pub fn offset(mut self, o: CoordinateUnit) -> Self {
        self.offset.replace(o);
        self
    }
}
#[derive(Copy, Clone)]
pub struct WellFormedSegmentUnitDescriptor {
    begin: SegmentUnit,
    end: SegmentUnit,
    min: Option<CoordinateUnit>,
    max: Option<CoordinateUnit>,
}
impl WellFormedSegmentUnitDescriptor {
    pub fn new(begin: SegmentUnit, end: SegmentUnit) -> Self {
        Self {
            begin,
            end,
            min: None,
            max: None,
        }
    }
    pub fn minimum(mut self, m: CoordinateUnit) -> Self {
        self.min.replace(m);
        self
    }
    pub fn maximum(mut self, m: CoordinateUnit) -> Self {
        self.max.replace(m);
        self
    }
    pub fn fixed(self, f: CoordinateUnit) -> Self {
        self.minimum(f).maximum(f)
    }
    pub(crate) fn normal(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::with_bounds(self.begin, self.end, self.min, self.max)
    }
}
