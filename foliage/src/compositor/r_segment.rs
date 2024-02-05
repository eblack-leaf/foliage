use crate::compositor::layout::Layout;
use crate::compositor::ViewHandle;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct ResponsiveGrid {
    base: Grid,
    exceptions: HashMap<Layout, Grid>,
    view_exceptions: HashMap<ViewHandle, Grid>,
}
impl ResponsiveGrid {
    pub fn current(&self, layout: Layout, view_handle: ViewHandle) -> &Grid {
        self.view_exceptions
            .get(&view_handle)
            .unwrap_or(self.exceptions.get(&layout).unwrap_or(&self.base))
    }
    // TODO cfg stuff here
}
#[derive(Default)]
pub struct GridTemplate {
    columns: SegmentValue,
    rows: SegmentValue,
}

impl GridTemplate {
    fn new(columns: SegmentValue, rows: SegmentValue) -> GridTemplate {
        Self { columns, rows }
    }
}
#[derive(Default)]
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
    pub fn horizontal(&self, area: Area<InterfaceContext>, unit: SegmentUnit) -> CoordinateUnit {
        unit.value() * self.column_element_width(area.width) + unit.value() * self.gap_x
            - self.column_element_width(area.width) * unit.bias.factor()
    }
    pub fn vertical(&self, area: Area<InterfaceContext>, unit: SegmentUnit) -> CoordinateUnit {
        unit.value() * self.row_element_height(area.height) + unit.value() * self.gap_y
            - self.row_element_height(area.height) * unit.bias.factor()
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
    pub fn assign_gap(&mut self, descriptor: GapDescriptor, value: SegmentValue) {
        todo!()
    }
}
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq)]
pub struct Gap {
    value: SegmentValue,
}
#[derive(Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
enum GapCategory {
    Horizontal,
    Vertical,
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
        area: Area<InterfaceContext>,
        grid: &ResponsiveGrid,
    ) -> Option<Coordinate<InterfaceContext>> {
        if self.negations.contains(&layout) {
            return None;
        }
        let current = grid.current(layout, self.view_handle);
        let left = current.horizontal(area, self.horizontal_value(&layout).begin);
        let top = current.vertical(area, self.vertical_value(&layout).begin);
        let right = current.horizontal(area, self.horizontal_value(&layout).end);
        let bottom = current.vertical(area, self.vertical_value(&layout).end);
        Some(Coordinate::new(
            Section::from_left_top_right_bottom(left, top, right, bottom),
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
    pub fn mobile(horizontal: SegmentUnitDescriptor, vertical: SegmentUnitDescriptor) -> Self {
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
    fn new(begin: SegmentUnit, end: SegmentUnit) -> SegmentUnitDescriptor {
        Self { begin, end }
    }
}
#[derive(Copy, Clone)]
pub struct Segment {
    horizontal: SegmentUnitDescriptor,
    vertical: SegmentUnitDescriptor,
}

impl Segment {
    fn new(horizontal: SegmentUnitDescriptor, vertical: SegmentUnitDescriptor) -> Segment {
        Self {
            horizontal,
            vertical,
        }
    }
}

pub trait SegmentUnitDesc {
    fn near(self) -> SegmentUnit;
    fn far(self) -> SegmentUnit;
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
        })*
    };
}
impl_segment_unit_desc!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);
#[derive(Copy, Clone, Serialize, Deserialize, Hash, PartialEq)]
pub enum SegmentBias {
    Near,
    Far,
}
impl SegmentBias {
    pub fn factor(self) -> CoordinateUnit {
        match self {
            SegmentBias::Near => 0.0,
            SegmentBias::Far => 1.0,
        }
    }
}
pub type SegmentValue = u8;
#[derive(Copy, Clone)]
pub struct SegmentUnit {
    value: SegmentValue,
    bias: SegmentBias,
}
impl SegmentUnit {
    fn value(&self) -> CoordinateUnit {
        self.value as CoordinateUnit
    }
    pub fn new(value: SegmentValue, bias: SegmentBias) -> Self {
        Self { value, bias }
    }
    pub fn to_end(self, su: SegmentUnit) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self, su)
    }
}
