use crate::compositor::layout::{Layout, Orientation, Threshold};
use crate::compositor::ViewHandle;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::component::Component;
use std::collections::HashMap;

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
            let x = self.handle.0 as CoordinateUnit * section.area.width
                + seg.x.base * if seg.x.fixed { 1.0 } else { section.area.width }
                + seg.x.offset;
            let x = match seg.x.min {
                None => x,
                Some(min) => x.max(min),
            };
            let x = match seg.x.max {
                None => x,
                Some(max) => x.min(max),
            };
            let y = self.handle.1 as CoordinateUnit * section.area.height
                + seg.y.base
                    * if seg.y.fixed {
                        1.0
                    } else {
                        section.area.height
                    }
                + seg.y.offset;
            let y = match seg.y.min {
                None => y,
                Some(min) => y.max(min),
            };
            let y = match seg.y.max {
                None => y,
                Some(max) => y.min(max),
            };
            let w = seg.w.base * if seg.w.fixed { 1.0 } else { section.area.width } + seg.w.offset;
            let w = match seg.w.min {
                None => w,
                Some(min) => w.max(min),
            };
            let w = match seg.w.max {
                None => w,
                Some(max) => w.min(max),
            };
            let h = seg.h.base
                * if seg.h.fixed {
                    1.0
                } else {
                    section.area.height
                }
                + seg.h.offset;
            let h = match seg.h.min {
                None => h,
                Some(min) => h.max(min),
            };
            let h = match seg.h.max {
                None => h,
                Some(max) => h.min(max),
            };
            let coordinate = Coordinate::default()
                .with_position((x, y))
                .with_area((w, h))
                .with_layer(self.layer);
            return Some(coordinate);
        }
        None
    }
    pub fn all(self, segment: Segment) -> Self {
        self.with_portrait(segment).with_landscape(segment)
    }
    pub fn with_landscape(self, segment: Segment) -> Self {
        self.with_landscape_mobile(segment)
            .with_landscape_tablet(segment)
            .with_landscape_desktop(segment)
            .with_landscape_workstation(segment)
    }
    pub fn with_portrait(self, segment: Segment) -> Self {
        self.with_portrait_mobile(segment)
            .with_portrait_tablet(segment)
            .with_portrait_desktop(segment)
            .with_portrait_workstation(segment)
    }
    pub fn without_portrait_mobile(mut self) -> Self {
        self.segments
            .remove(&Layout::new(Orientation::Portrait, Threshold::Mobile));
        self
    }
    pub fn without_landscape_mobile(mut self) -> Self {
        self.segments
            .remove(&Layout::new(Orientation::Landscape, Threshold::Mobile));
        self
    }
    pub fn without_portrait_tablet(mut self) -> Self {
        self.segments
            .remove(&Layout::new(Orientation::Portrait, Threshold::Tablet));
        self
    }
    pub fn without_landscape_tablet(mut self) -> Self {
        self.segments
            .remove(&Layout::new(Orientation::Landscape, Threshold::Tablet));
        self
    }
    pub fn without_portrait_desktop(mut self) -> Self {
        self.segments
            .remove(&Layout::new(Orientation::Portrait, Threshold::Desktop));
        self
    }
    pub fn without_landscape_desktop(mut self) -> Self {
        self.segments
            .remove(&Layout::new(Orientation::Landscape, Threshold::Desktop));
        self
    }
    pub fn without_portrait_workstation(mut self) -> Self {
        self.segments
            .remove(&Layout::new(Orientation::Portrait, Threshold::Workstation));
        self
    }
    pub fn without_landscape_workstation(mut self) -> Self {
        self.segments
            .remove(&Layout::new(Orientation::Landscape, Threshold::Workstation));
        self
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
