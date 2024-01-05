use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::r_compositor::layout::{Layout, Orientation, Threshold};
use crate::r_compositor::ViewHandle;
use bevy_ecs::component::Component;
use std::collections::HashMap;
use crate::coordinate::layer::Layer;

#[derive(Component)]
pub struct ResponsiveSegment {
    pub handle: ViewHandle,
    pub segments: HashMap<Layout, Segment>,
    pub layer: Layer,
}
impl ResponsiveSegment {
    pub fn new<VH: Into<ViewHandle>, L: Into<Layer>>(vh: VH, l: L) -> Self {
        Self {
            handle: vh.into(),
            segments: HashMap::new(),
            layer: l.into(),
        }
    }
    pub fn coordinate(
        &self,
        layout: Layout,
        section: Section<InterfaceContext>,
    ) -> Option<Coordinate<InterfaceContext>> {
        if let Some(seg) = self.segments.get(&layout) {
            let x_offset = seg.x.base * if seg.x.fixed { 1.0 } else { section.area.width };
            let x_start = self.handle.0 as CoordinateUnit * section.area.width;
            return Some(coord);
        }
        None
    }
    pub fn all(mut self, segment: Segment) -> Self {
        self.with_portrait(segment).with_landscape(segment)
    }
    pub fn with_landscape(mut self, segment: Segment) -> Self {
        self.with_landscape_mobile(segment)
            .with_landscape_tablet(segment)
            .with_landscape_desktop(segment)
            .with_landscape_workstation(segment)
    }
    pub fn with_portrait(mut self, segment: Segment) -> Self {
        self.with_portrait_mobile(segment)
            .with_portrait_tablet(segment)
            .with_portrait_desktop(segment)
            .with_portrait_workstation(segment)
    }
    pub fn with_portrait_mobile(mut self, segment: Segment) -> Self {
        self.segments.insert(
            Layout::new(Orientation::Portrait, Threshold::Mobile),
            segment,
        );
        self
    }
    pub fn with_landscape_mobile(mut self, segment: Segment) -> Self {
        self.segments.insert(
            Layout::new(Orientation::Landscape, Threshold::Mobile),
            segment,
        );
        self
    }
    pub fn with_portrait_tablet(mut self, segment: Segment) -> Self {
        self.segments.insert(
            Layout::new(Orientation::Portrait, Threshold::Tablet),
            segment,
        );
        self
    }
    pub fn with_landscape_tablet(mut self, segment: Segment) -> Self {
        self.segments.insert(
            Layout::new(Orientation::Landscape, Threshold::Tablet),
            segment,
        );
        self
    }
    pub fn with_portrait_desktop(mut self, segment: Segment) -> Self {
        self.segments.insert(
            Layout::new(Orientation::Portrait, Threshold::Desktop),
            segment,
        );
        self
    }
    pub fn with_landscape_desktop(mut self, segment: Segment) -> Self {
        self.segments.insert(
            Layout::new(Orientation::Landscape, Threshold::Desktop),
            segment,
        );
        self
    }
    pub fn with_portrait_workstation(mut self, segment: Segment) -> Self {
        self.segments.insert(
            Layout::new(Orientation::Portrait, Threshold::Workstation),
            segment,
        );
        self
    }
    pub fn with_landscape_workstation(mut self, segment: Segment) -> Self {
        self.segments.insert(
            Layout::new(Orientation::Landscape, Threshold::Workstation),
            segment,
        );
        self
    }
}
#[derive(Clone, Copy)]
pub struct Segment {
    pub x: SegmentUnit,
    pub y: SegmentUnit,
    pub w: SegmentUnit,
    pub h: SegmentUnit,
}
impl Segment {
    pub fn new() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            w: Default::default(),
            h: Default::default(),
        }
    }
    pub fn with_x<SU: Into<SegmentUnit>>(mut self, unit: SU) -> Self {
        self.x = unit.into();
        self
    }
    pub fn with_y<SU: Into<SegmentUnit>>(mut self, unit: SU) -> Self {
        self.y = unit.into();
        self
    }
    pub fn with_w<SU: Into<SegmentUnit>>(mut self, unit: SU) -> Self {
        self.w = unit.into();
        self
    }
    pub fn with_h<SU: Into<SegmentUnit>>(mut self, unit: SU) -> Self {
        self.h = unit.into();
        self
    }
}
#[derive(Copy, Clone, Default)]
pub struct SegmentUnit {
    base: CoordinateUnit,
    fixed: bool,
    min: Option<CoordinateUnit>,
    max: Option<CoordinateUnit>,
    offset: CoordinateUnit,
}
impl SegmentUnit {
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
}
