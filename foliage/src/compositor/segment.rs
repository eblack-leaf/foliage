use crate::compositor::layout::Layout;
use crate::compositor::{ViewHandle, ViewHandleOffset};
use crate::coordinate::layer::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::component::Component;
use std::collections::{HashMap, HashSet};

#[derive(Component, Clone)]
pub struct ResponsiveSegment {
    pub view_handle: ViewHandle,
    pub segment: Segment,
    pub negations: HashSet<Layout>,
    pub layer: Layer,
}
impl ResponsiveSegment {
    pub fn new<S: Into<Segment>>(s: S) -> Self {
        Self {
            view_handle: ViewHandle::default(),
            segment: s.into(),
            negations: HashSet::new(),
            layer: Layer::default(),
        }
    }
    pub fn viewed_at<VH: Into<ViewHandle>>(mut self, vh: VH) -> Self {
        self.view_handle = vh.into();
        self
    }
    pub fn at_layer<L: Into<Layer>>(mut self, l: L) -> Self {
        self.layer = l.into();
        self
    }
    pub fn coordinate(
        &self,
        layout: Layout,
        section: Section<InterfaceContext>,
    ) -> Option<Coordinate<InterfaceContext>> {
        if self.negations.contains(&layout) {
            return None;
        }
        Some(
            Coordinate::default()
                .with_position((
                    self.segment
                        .x
                        .calc_horizontal(self.view_handle, layout, section),
                    self.segment
                        .y
                        .calc_vertical(self.view_handle, layout, section),
                ))
                .with_area((
                    self.segment
                        .w
                        .calc_horizontal(self.view_handle, layout, section),
                    self.segment
                        .h
                        .calc_vertical(self.view_handle, layout, section),
                ))
                .with_layer(self.layer),
        )
    }
    pub fn x_exception<SUD: Into<SegmentUnitDescriptor>>(mut self, l: Layout, sud: SUD) -> Self {
        self.segment.x.exceptions.insert(l, sud.into());
        self
    }
    pub fn y_exception<SUD: Into<SegmentUnitDescriptor>>(mut self, l: Layout, sud: SUD) -> Self {
        self.segment.y.exceptions.insert(l, sud.into());
        self
    }
    pub fn w_exception<SUD: Into<SegmentUnitDescriptor>>(mut self, l: Layout, sud: SUD) -> Self {
        self.segment.w.exceptions.insert(l, sud.into());
        self
    }
    pub fn h_exception<SUD: Into<SegmentUnitDescriptor>>(mut self, l: Layout, sud: SUD) -> Self {
        self.segment.h.exceptions.insert(l, sud.into());
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
#[derive(Clone, Default)]
pub struct Segment {
    x: SegmentUnit,
    y: SegmentUnit,
    w: SegmentUnit,
    h: SegmentUnit,
}
impl Segment {
    pub fn new<SU: Into<SegmentUnit>>(x: SU, y: SU, w: SU, h: SU) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            w: w.into(),
            h: h.into(),
        }
    }
}
#[derive(Clone, Default)]
pub struct SegmentUnit {
    base: SegmentUnitDescriptor,
    exceptions: HashMap<Layout, SegmentUnitDescriptor>,
}
impl<SUD: Into<SegmentUnitDescriptor>> From<SUD> for SegmentUnit {
    fn from(value: SUD) -> Self {
        Self::new(value)
    }
}
impl SegmentUnit {
    pub fn new<SUD: Into<SegmentUnitDescriptor>>(sud: SUD) -> Self {
        Self {
            base: sud.into(),
            exceptions: HashMap::new(),
        }
    }
    pub fn calc_horizontal(
        &self,
        vh: ViewHandle,
        l: Layout,
        vs: Section<InterfaceContext>,
    ) -> CoordinateUnit {
        self.exceptions
            .get(&l)
            .cloned()
            .unwrap_or(self.base)
            .calc(vh.0, vs.width())
    }
    pub fn calc_vertical(
        &self,
        vh: ViewHandle,
        l: Layout,
        vs: Section<InterfaceContext>,
    ) -> CoordinateUnit {
        self.exceptions
            .get(&l)
            .cloned()
            .unwrap_or(self.base)
            .calc(vh.1, vs.height())
    }
}
#[derive(Copy, Clone, Default)]
pub struct SegmentUnitDescriptor {
    base: CoordinateUnit,
    fixed: bool,
    min: Option<CoordinateUnit>,
    max: Option<CoordinateUnit>,
    offset: CoordinateUnit,
}
impl SegmentUnitDescriptor {
    pub fn new(base: CoordinateUnit) -> Self {
        Self {
            base,
            fixed: false,
            min: None,
            max: None,
            offset: 0.0,
        }
    }
    pub fn relative(mut self) -> Self {
        self.fixed = false;
        self
    }
    pub fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }
    pub fn max(mut self, m: CoordinateUnit) -> Self {
        self.max.replace(m);
        self
    }
    pub fn min(mut self, m: CoordinateUnit) -> Self {
        self.min.replace(m);
        self
    }
    pub fn offset(mut self, o: CoordinateUnit) -> Self {
        self.offset = o;
        self
    }
    pub fn calc(&self, handle_offset: ViewHandleOffset, dim: CoordinateUnit) -> CoordinateUnit {
        let factor = if self.fixed { 1.0 } else { dim };
        let num = handle_offset as CoordinateUnit * dim + self.base * factor + self.offset;
        num.min(self.max.unwrap_or(CoordinateUnit::MAX))
            .max(self.min.unwrap_or(CoordinateUnit::MIN))
    }
}
pub trait SegmentUnitNumber {
    fn relative(self) -> SegmentUnitDescriptor;
    fn fixed(self) -> SegmentUnitDescriptor;
}
impl SegmentUnitNumber for CoordinateUnit {
    fn relative(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self).relative()
    }

    fn fixed(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self).fixed()
    }
}
impl SegmentUnitNumber for i32 {
    fn relative(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).relative()
    }

    fn fixed(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).fixed()
    }
}
impl SegmentUnitNumber for u32 {
    fn relative(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).relative()
    }

    fn fixed(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).fixed()
    }
}
impl SegmentUnitNumber for u64 {
    fn relative(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).relative()
    }

    fn fixed(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).fixed()
    }
}
impl SegmentUnitNumber for i64 {
    fn relative(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).relative()
    }

    fn fixed(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).fixed()
    }
}
impl SegmentUnitNumber for f64 {
    fn relative(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).relative()
    }

    fn fixed(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).fixed()
    }
}
impl SegmentUnitNumber for usize {
    fn relative(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).relative()
    }

    fn fixed(self) -> SegmentUnitDescriptor {
        SegmentUnitDescriptor::new(self as CoordinateUnit).fixed()
    }
}
