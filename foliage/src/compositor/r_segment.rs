use crate::compositor::layout::Layout;
use crate::compositor::ViewHandle;
use crate::coordinate::area::Area;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ResponsiveGrid {
    base: Grid,
    exceptions: HashMap<Layout, Grid>,
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
    gap: HashMap<GapCategory, Gap>,
    padding: HashMap<PaddingCategory, Padding>,
    template: GridTemplate,
}
impl Grid {
    pub fn new(columns: SegmentValue, rows: SegmentValue) -> Self {
        Self {
            gap: Default::default(),
            padding: Default::default(),
            template: GridTemplate::new(columns, rows),
        }
    }
    pub fn horizontal(area: Area<InterfaceContext>, unit: SegmentUnit) -> CoordinateUnit {
        todo!()
    }
    pub fn vertical(area: Area<InterfaceContext>, unit: SegmentUnit) -> CoordinateUnit {
        todo!()
    }
    pub fn column_height(&self, area: Area<InterfaceContext>) -> CoordinateUnit {
        todo!()
    }
    pub fn row_height(&self, area: Area<InterfaceContext>) -> CoordinateUnit {
        todo!()
    }
    pub fn column_element_height(&self, area: Area<InterfaceContext>) -> CoordinateUnit {
        todo!()
    }
    pub fn row_element_height(&self, area: Area<InterfaceContext>) -> CoordinateUnit {
        todo!()
    }
    pub fn assign_gap(&mut self, descriptor: GapDescriptor, value: SegmentValue) {
        todo!()
    }
    pub fn assign_padding(&mut self, descriptor: PaddingDescriptor, value: SegmentValue) {
        todo!()
    }
}
#[derive(Copy, Clone, Default, Serialize, Deserialize, Hash, PartialEq)]
pub struct Padding {
    value: SegmentValue,
}
#[derive(Copy, Clone, Serialize, Deserialize, Hash, PartialEq)]
enum PaddingCategory {
    Left,
    Top,
    Right,
    Bottom,
    Horizontal,
    Vertical,
}
#[derive(Copy, Clone, Serialize, Deserialize, Hash, PartialEq)]
pub enum PaddingDescriptor {
    Left,
    Top,
    Right,
    Bottom,
    Horizontal,
    Vertical,
    All,
}
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq)]
pub struct Gap {
    value: SegmentValue,
}
#[derive(Copy, Clone, Serialize, Deserialize, Hash, PartialEq)]
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
}
impl ResponsiveSegment {
    pub fn coordinate(
        &self,
        layout: Layout,
        area: Area<InterfaceContext>,
        grid: &ResponsiveGrid,
    ) -> Option<Coordinate<InterfaceContext>> {
        todo!()
    }
    pub fn mobile(horizontal: SegmentUnitDescriptor, vertical: SegmentUnitDescriptor) -> Self {
        Self {
            view_handle: ViewHandle::default(),
            base: Segment::new(horizontal, vertical),
            horizontal_exceptions: Default::default(),
            vertical_exceptions: Default::default(),
        }
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
    pub fn factor(self) -> () {}
}
pub type SegmentValue = u8;
#[derive(Copy, Clone)]
pub struct SegmentUnit {
    value: SegmentValue,
    bias: SegmentBias,
}
impl SegmentUnit {
    pub fn new(value: SegmentValue, bias: SegmentBias) -> Self {
        Self { value, bias }
    }
    pub fn to_end(self, su: SegmentUnit) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self, su)
    }
}
