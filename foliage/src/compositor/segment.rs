use crate::compositor::layout::Layout;
use crate::compositor::ViewHandle;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct GridException(Layout, Option<ViewHandle>);
impl GridException {
    pub fn global(layout: Layout) -> Self {
        Self(layout, None)
    }
    pub fn view_specific(layout: Layout, view_handle: ViewHandle) -> Self {
        Self(layout, Some(view_handle))
    }
}
#[derive(Resource, Default)]
pub struct ResponsiveGrid {
    base: Grid,
    exceptions: HashMap<GridException, Grid>,
}
impl ResponsiveGrid {
    pub fn current(&self, layout: Layout, view_handle: ViewHandle) -> &Grid {
        self.exceptions
            .get(&GridException::view_specific(layout, view_handle))
            .unwrap_or(&self.base)
    }
    pub fn add_base(&mut self, base: Grid) {
        self.base = base;
    }
    pub fn add_global_exception(&mut self, layouts: &[Layout], exc: Grid) {
        for l in layouts.iter() {
            self.exceptions.insert(GridException::global(*l), exc);
        }
    }
    pub fn add_view_specific_exceptions(
        &mut self,
        layouts: &[Layout],
        view_handle: ViewHandle,
        exc: Grid,
    ) {
        for l in layouts {
            self.exceptions
                .insert(GridException::view_specific(*l, view_handle), exc);
        }
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
    pub fn horizontal(&self, area: Area<InterfaceContext>, unit: SegmentUnit) -> GridRelativeValue {
        if let Some(f) = unit.fixed {
            return GridRelativeValue::Fixed(f);
        }
        GridRelativeValue::Anchored(
            unit.value() * self.column_element_width(area.width)
                + unit.value() * self.gap_x
                + (unit.value() - 1f32).max(0.0) * self.gap_x
                - self.column_element_width(area.width) * unit.bias.factor(),
        )
    }
    pub fn vertical(&self, area: Area<InterfaceContext>, unit: SegmentUnit) -> GridRelativeValue {
        if let Some(f) = unit.fixed {
            return GridRelativeValue::Fixed(f);
        }
        GridRelativeValue::Anchored(
            unit.value() * self.row_element_height(area.height)
                + unit.value() * self.gap_y
                + (unit.value() - 1f32).max(0.0) * self.gap_y
                - self.row_element_height(area.height) * unit.bias.factor(),
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
#[derive(Clone, Component)]
pub struct ResponsiveSegment {
    view_handle: ViewHandle,
    base: Segment,
    horizontal_exceptions: HashMap<Layout, SegmentUnitDescriptor>,
    vertical_exceptions: HashMap<Layout, SegmentUnitDescriptor>,
    negations: HashSet<Layout>,
    layer: Layer,
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
        let current = grid.current(layout, self.view_handle);
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
        Some(Coordinate::new(
            Section::new(
                (left + section.left(), top + section.top()),
                (width, height),
            ),
            self.layer,
        ))
    }

    fn vertical_value(&self, layout: &Layout) -> SegmentUnitDescriptor {
        self.vertical_exceptions
            .get(&layout)
            .cloned()
            .unwrap_or(self.base.vertical)
    }

    fn horizontal_value(&self, layout: &Layout) -> SegmentUnitDescriptor {
        self.horizontal_exceptions
            .get(&layout)
            .cloned()
            .unwrap_or(self.base.horizontal)
    }
    pub fn base(horizontal: SegmentUnitDescriptor, vertical: SegmentUnitDescriptor) -> Self {
        Self {
            view_handle: ViewHandle::default(),
            base: Segment::new(horizontal, vertical),
            horizontal_exceptions: Default::default(),
            vertical_exceptions: Default::default(),
            negations: HashSet::new(),
            layer: Layer::default(),
        }
    }
    pub fn at_layer<L: Into<Layer>>(mut self, l: L) -> Self {
        self.layer = l.into();
        self
    }
    pub fn horizontal_exception<L: AsRef<[Layout]>>(
        mut self,
        layouts: L,
        exc: SegmentUnitDescriptor,
    ) -> Self {
        let layouts = layouts.as_ref();
        for l in layouts.iter() {
            self.horizontal_exceptions.insert(*l, exc);
        }
        self
    }
    pub fn vertical_exception(mut self, layouts: &[Layout], exc: SegmentUnitDescriptor) -> Self {
        for l in layouts.iter() {
            self.vertical_exceptions.insert(*l, exc);
        }
        self
    }
    pub fn viewed_at(mut self, view_handle: ViewHandle) -> Self {
        self.view_handle = view_handle;
        self
    }
}
#[derive(Copy, Clone)]
pub struct SegmentUnitDescriptor {
    begin: SegmentUnit,
    end: SegmentUnit,
}
impl SegmentUnitDescriptor {
    pub fn new(begin: SegmentUnit, end: SegmentUnit) -> SegmentUnitDescriptor {
        Self { begin, end }
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
    pub fn factor(self) -> CoordinateUnit {
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
        }
    }
    pub fn fixed(value: CoordinateUnit) -> Self {
        Self {
            value: SegmentValue::default(),
            bias: SegmentBias::Near,
            fixed: Some(value),
        }
    }
    pub fn to_end(self, su: SegmentUnit) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self, su)
    }
}
