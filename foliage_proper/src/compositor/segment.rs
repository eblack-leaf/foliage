use crate::compositor::layout::Layout;
use crate::compositor::ViewHandle;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::Div;

#[derive(Resource, Default)]
pub struct ResponsiveGrid {
    view_configs: HashMap<ViewHandle, Grid>,
}
impl ResponsiveGrid {
    pub fn current(&self, view_handle: ViewHandle) -> &Grid {
        self.view_configs
            .get(&view_handle)
            .expect("view-not-configured-with-grid")
    }
    pub fn configure_view(&mut self, view_handle: ViewHandle, grid: Grid) {
        self.view_configs.insert(view_handle, grid);
    }
}
#[derive(Default, Copy, Clone)]
pub struct GridTemplate {
    columns: SegmentValue,
    rows: SegmentValue,
}

impl GridTemplate {
    fn new(columns: SegmentValue, rows: SegmentValue) -> GridTemplate {
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
#[derive(Default, Copy, Clone)]
pub struct Grid {
    gap_x: CoordinateUnit,
    gap_y: CoordinateUnit,
    template: GridTemplate,
}
impl Grid {
    pub const DEFAULT_GAP: CoordinateUnit = 8.0;
    pub fn new(columns: SegmentValue, rows: SegmentValue) -> Self {
        Self {
            gap_x: Self::DEFAULT_GAP,
            gap_y: Self::DEFAULT_GAP,
            template: GridTemplate::new(columns, rows),
        }
    }
    pub fn gap_x(&self, dim: CoordinateUnit) -> CoordinateUnit {
        self.gap_x.min(self.column_element_width(dim) * 0.25)
    }
    pub fn gap_y(&self, dim: CoordinateUnit) -> CoordinateUnit {
        self.gap_x.min(self.row_element_height(dim) * 0.25)
    }
    pub fn horizontal(&self, area: Area<InterfaceContext>, unit: SegmentUnit) -> GridRelativeValue {
        if let Some(f) = unit.fixed {
            return GridRelativeValue::Fixed(f);
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
        if let Some(f) = unit.fixed {
            return GridRelativeValue::Fixed(f);
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
        self.column_width(dim) - self.gap_x * 2f32
    }
    pub fn row_element_height(&self, dim: CoordinateUnit) -> CoordinateUnit {
        self.row_height(dim) - self.gap_y * 2f32
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
    view_handle: ViewHandle,
    base: Segment,
    horizontal_exceptions: HashMap<Layout, SegmentUnitDescriptor>,
    vertical_exceptions: HashMap<Layout, SegmentUnitDescriptor>,
    negations: HashSet<Layout>,
    layer: Layer,
    justification: Option<Justify>,
}
impl ResponsiveSegment {
    pub fn coordinate(
        &self,
        layout: Layout,
        section: Section<InterfaceContext>,
        grid: &ResponsiveGrid,
    ) -> Option<Coordinate<InterfaceContext>> {
        if self.negations.contains(&layout) {
            return None;
        }
        let current = grid.current(self.view_handle);
        let left = current
            .horizontal(section.area, self.horizontal_value(&layout).begin)
            .value();
        let top = current
            .vertical(section.area, self.vertical_value(&layout).begin)
            .value();
        let width_or_right = current.horizontal(section.area, self.horizontal_value(&layout).end);
        let height_or_bottom = current.vertical(section.area, self.vertical_value(&layout).end);
        let width = match width_or_right {
            GridRelativeValue::Anchored(value) => value - left,
            GridRelativeValue::Fixed(value) => value,
        };
        let height = match height_or_bottom {
            GridRelativeValue::Anchored(value) => value - top,
            GridRelativeValue::Fixed(value) => value,
        };
        let width = if let Some(w) = self.horizontal_value(&layout).min {
            let bounded = width.max(w);
            bounded
        } else {
            width
        };
        let (left, width) = if let Some(w) = self.horizontal_value(&layout).max {
            let bounded = width.min(w);
            let adjusted_left = if bounded < width {
                let diff = width - bounded;
                let justification = self.justification.unwrap_or(Justify::Center);
                match justification {
                    Justify::Center | Justify::Top | Justify::Bottom => left + diff.div(2.0),
                    Justify::Right | Justify::RightTop | Justify::RightBottom => left + diff,
                    _ => left,
                }
            } else {
                left
            };
            (adjusted_left, bounded)
        } else {
            (left, width)
        };
        let height = if let Some(h) = self.vertical_value(&layout).min {
            let bounded = height.max(h);
            bounded
        } else {
            height
        };
        let (top, height) = if let Some(h) = self.vertical_value(&layout).max {
            let bounded = height.min(h);
            let adjusted_top = if bounded < height {
                let diff = height - bounded;
                let justification = self.justification.unwrap_or(Justify::Center);
                match justification {
                    Justify::Center | Justify::Left | Justify::Right => top + diff.div(2.0),
                    Justify::Bottom | Justify::RightBottom | Justify::LeftBottom => top + diff,
                    _ => top,
                }
            } else {
                top
            };
            (adjusted_top, bounded)
        } else {
            (top, height)
        };
        Some(Coordinate::new(
            Section::new(
                (left + section.left(), top - section.top()),
                (width, height),
            ),
            self.layer,
        ))
    }
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justification.replace(justify);
        self
    }
    fn vertical_value(&self, layout: &Layout) -> SegmentUnitDescriptor {
        self.vertical_exceptions
            .get(layout)
            .cloned()
            .unwrap_or(self.base.vertical)
    }

    fn horizontal_value(&self, layout: &Layout) -> SegmentUnitDescriptor {
        self.horizontal_exceptions
            .get(layout)
            .cloned()
            .unwrap_or(self.base.horizontal)
    }
    pub fn base(
        horizontal: WellFormedSegmentUnitDescriptor,
        vertical: WellFormedSegmentUnitDescriptor,
    ) -> Self {
        Self {
            view_handle: ViewHandle::default(),
            base: Segment::new(horizontal.normal(), vertical.normal()),
            horizontal_exceptions: Default::default(),
            vertical_exceptions: Default::default(),
            negations: HashSet::new(),
            layer: Layer::default(),
            justification: None,
        }
    }
    pub fn at_layer<L: Into<Layer>>(mut self, l: L) -> Self {
        self.layer = l.into();
        self
    }
    pub fn horizontal_exception<L: AsRef<[Layout]>>(
        mut self,
        layouts: L,
        exc: WellFormedSegmentUnitDescriptor,
    ) -> Self {
        let layouts = layouts.as_ref();
        for l in layouts.iter() {
            self.horizontal_exceptions.insert(*l, exc.normal());
        }
        self
    }
    pub fn vertical_exception<L: AsRef<[Layout]>>(
        mut self,
        layouts: L,
        exc: WellFormedSegmentUnitDescriptor,
    ) -> Self {
        for l in layouts.as_ref().iter() {
            self.vertical_exceptions.insert(*l, exc.normal());
        }
        self
    }
    pub fn viewed_at(mut self, view_handle: ViewHandle) -> Self {
        self.view_handle = view_handle;
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
    horizontal: SegmentUnitDescriptor,
    vertical: SegmentUnitDescriptor,
}

impl Segment {
    pub fn new(horizontal: SegmentUnitDescriptor, vertical: SegmentUnitDescriptor) -> Segment {
        Self {
            horizontal,
            vertical,
        }
    }
}
pub trait SegmentUnitDesc {
    fn near(self) -> SegmentUnit;
    fn far(self) -> SegmentUnit;
    fn fixed(self) -> SegmentUnit;
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
            fn fixed(self) -> SegmentUnit {
                SegmentUnit::fixed(self as CoordinateUnit)
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
    fixed: Option<CoordinateUnit>,
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
            fixed: None,
            offset: None,
        }
    }
    pub fn fixed(value: CoordinateUnit) -> Self {
        Self {
            value: SegmentValue::default(),
            bias: SegmentBias::Near,
            fixed: Some(value),
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
    fn normal(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::with_bounds(self.begin, self.end, self.min, self.max)
    }
}
