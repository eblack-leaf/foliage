use crate::coordinate::area::Area;
use crate::coordinate::points::Points;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::grid::animation::{GridLocationAnimationHook, PointDrivenAnimationHook};
use crate::grid::aspect::{AspectConfiguration, GridAspect, LocationAspect};
use crate::grid::resolve::{ReferentialData, ResolvedLocation};
use crate::grid::token::{LocationAspectDescriptorValue, SpecifiedDescriptorValue};
use crate::layout::Layout;
use bevy_ecs::component::Component;
use smallvec::SmallVec;

#[derive(Clone, Component, Default)]
pub struct GridLocation {
    configurations: [(AspectConfiguration, LocationAspect); 4],
    exceptions: SmallVec<[(GridLocationException, LocationAspect); 1]>,
    pub(crate) animation_hook: GridLocationAnimationHook,
}

impl GridLocation {
    pub(crate) fn resolve(
        &self,
        stem: ReferentialData,
        screen: ReferentialData,
        layout: Layout,
    ) -> Option<ResolvedLocation> {
        let mut resolution = ResolvedLocation::new();
        for (aspect_config, location_aspect) in self.configurations.iter() {
            let mut to_use = None;
            let base = location_aspect;
            for except in self.exceptions.iter() {
                if except.0.layout.contains(layout) && aspect_config == &except.0.config {
                    to_use = Some(*except.1);
                }
            }
            let to_use = to_use.unwrap_or(base);
            let a = match &to_use.aspects[0].value {
                LocationAspectDescriptorValue::Existing => {
                    base.resolve_grid_aspect(stem, screen, to_use.aspects[0].aspect)
                }
                LocationAspectDescriptorValue::Specified(spec) => spec.resolve(stem, screen),
            };
            let b = match &to_use.aspects[1].value {
                LocationAspectDescriptorValue::Existing => {
                    base.resolve_grid_aspect(stem, screen, to_use.aspects[1].aspect)
                }
                LocationAspectDescriptorValue::Specified(spec) => spec.resolve(stem, screen),
            };
            let (pair_config, data) = if to_use.aspects[0].aspect < to_use.aspects[1].aspect {
                ((to_use.aspects[0].aspect, to_use.aspects[1].aspect), (a, b))
            } else {
                ((to_use.aspects[1].aspect, to_use.aspects[0].aspect), (b, a))
            };
            match aspect_config {
                AspectConfiguration::Horizontal => {
                    if pair_config == (GridAspect::Left, GridAspect::Right) {
                        resolution.section.position.set_x(data.0);
                        resolution.section.area.set_width(data.1 - data.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::CenterX) {
                        resolution.section.position.set_x(data.0);
                        resolution.section.area.set_width((data.1 - data.0) * 2.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::Width) {
                        resolution.section.position.set_x(data.0);
                        resolution.section.area.set_width(data.1);
                    } else if pair_config == (GridAspect::Width, GridAspect::CenterX) {
                        resolution.section.position.set_x(data.1 - data.0 / 2.0);
                        resolution.section.area.set_width(data.0);
                    } else if pair_config == (GridAspect::Width, GridAspect::Right) {
                        resolution.section.position.set_x(data.1 - data.0);
                        resolution.section.area.set_width(data.0);
                    } else if pair_config == (GridAspect::CenterX, GridAspect::Right) {
                        let diff = data.1 - data.0;
                        resolution.section.position.set_x(data.0 - diff);
                        resolution.section.area.set_width(diff * 2.0);
                    }
                }
                AspectConfiguration::Vertical => {
                    if pair_config == (GridAspect::Top, GridAspect::Bottom) {
                        resolution.section.position.set_y(data.0);
                        resolution.section.area.set_height(data.1 - data.0);
                    } else if pair_config == (GridAspect::Top, GridAspect::CenterY) {
                        resolution.section.position.set_y(data.0);
                        resolution.section.area.set_height((data.1 - data.0) * 2.0);
                    } else if pair_config == (GridAspect::Top, GridAspect::Height) {
                        resolution.section.position.set_y(data.0);
                        resolution.section.area.set_height(data.1);
                    } else if pair_config == (GridAspect::Height, GridAspect::CenterY) {
                        resolution.section.position.set_y(data.1 - data.0 / 2.0);
                        resolution.section.area.set_height(data.0);
                    } else if pair_config == (GridAspect::Height, GridAspect::Bottom) {
                        resolution.section.position.set_y(data.1 - data.0);
                        resolution.section.area.set_height(data.0);
                    } else if pair_config == (GridAspect::CenterY, GridAspect::Bottom) {
                        let diff = data.1 - data.0;
                        resolution.section.position.set_y(data.0 - diff);
                        resolution.section.area.set_height(diff * 2.0);
                    }
                }
                AspectConfiguration::PointA
                | AspectConfiguration::PointB
                | AspectConfiguration::PointC
                | AspectConfiguration::PointD => {
                    if resolution.points.is_none() {
                        resolution.points.replace(Points::default());
                    }
                    match aspect_config {
                        AspectConfiguration::PointA => {
                            if pair_config == (GridAspect::PointAX, GridAspect::PointAY) {
                                resolution.points.as_mut()?.data[0] = data.into();
                            } else {
                                panic!("invalid-configuration aspect")
                            }
                        }
                        AspectConfiguration::PointB => {
                            if pair_config == (GridAspect::PointBX, GridAspect::PointBY) {
                                resolution.points.as_mut()?.data[1] = data.into();
                            } else {
                                panic!("invalid-configuration aspect")
                            }
                        }
                        AspectConfiguration::PointC => {
                            if pair_config == (GridAspect::PointCX, GridAspect::PointCY) {
                                resolution.points.as_mut()?.data[2] = data.into();
                            } else {
                                panic!("invalid-configuration aspect")
                            }
                        }
                        AspectConfiguration::PointD => {
                            if pair_config == (GridAspect::PointDX, GridAspect::PointDY) {
                                resolution.points.as_mut()?.data[3] = data.into();
                            } else {
                                panic!("invalid-configuration aspect")
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        match &self.animation_hook {
            GridLocationAnimationHook::SectionDriven(s) => {
                if s.hook_changed {
                    if s.create_diff {
                        let diff = s.last - resolution.section;
                        let offset = diff * s.hook_percent;
                        resolution.section += offset;
                        resolution
                            .hook_update
                            .replace([Some(diff), None, None, None]);
                    } else {
                        resolution.section += s.diff * s.hook_percent;
                    }
                }
            }
            GridLocationAnimationHook::PointDriven(p) => {
                if p.point_a.create_diff
                    || p.point_b.create_diff
                    || p.point_c.create_diff
                    || p.point_d.create_diff
                {
                    resolution.hook_update.replace([None, None, None, None]);
                }
                if p.point_a.hook_changed {
                    if p.point_a.create_diff {
                        let diff = p.point_a.last
                            - Section::new(resolution.points.as_ref()?.data[0], Area::default());
                        let offset = diff * p.point_a.hook_percent;
                        resolution.points.as_mut()?.data[0] +=
                            Position::new(offset.position.coordinates);
                        resolution.hook_update?.get_mut(0)?.replace(offset);
                    } else {
                        let offset = p.point_a.diff * p.point_a.hook_percent;
                        resolution.points.as_mut()?.data[0] +=
                            Position::new(offset.position.coordinates);
                    }
                }
                if p.point_b.hook_changed {
                    if p.point_b.create_diff {
                        let diff = p.point_b.last
                            - Section::new(resolution.points.as_ref()?.data[1], Area::default());
                        let offset = diff * p.point_b.hook_percent;
                        resolution.points.as_mut()?.data[1] +=
                            Position::new(offset.position.coordinates);
                        resolution.hook_update?.get_mut(1)?.replace(offset);
                    } else {
                        let offset = p.point_b.diff * p.point_b.hook_percent;
                        resolution.points.as_mut()?.data[1] +=
                            Position::new(offset.position.coordinates);
                    }
                }
                if p.point_c.hook_changed {
                    if p.point_c.create_diff {
                        let diff = p.point_c.last
                            - Section::new(resolution.points.as_ref()?.data[2], Area::default());
                        let offset = diff * p.point_c.hook_percent;
                        resolution.points.as_mut()?.data[2] +=
                            Position::new(offset.position.coordinates);
                        resolution.hook_update?.get_mut(2)?.replace(offset);
                    } else {
                        let offset = p.point_c.diff * p.point_c.hook_percent;
                        resolution.points.as_mut()?.data[2] +=
                            Position::new(offset.position.coordinates);
                    }
                }
                if p.point_d.hook_changed {
                    if p.point_d.create_diff {
                        let diff = p.point_d.last
                            - Section::new(resolution.points.as_ref()?.data[3], Area::default());
                        let offset = diff * p.point_d.hook_percent;
                        resolution.points.as_mut()?.data[3] +=
                            Position::new(offset.position.coordinates);
                        resolution.hook_update?.get_mut(3)?.replace(offset);
                    } else {
                        let offset = p.point_d.diff * p.point_d.hook_percent;
                        resolution.points.as_mut()?.data[3] +=
                            Position::new(offset.position.coordinates);
                    }
                }
            }
        }
        if let Some(pts) = resolution.points.as_mut() {
            resolution.section = pts.bbox();
        }
        if self.configurations.is_empty() {
            return None;
        }
        Some(resolution)
    }
    pub fn new() -> Self {
        Self {
            configurations: Default::default(),
            exceptions: Default::default(),
            animation_hook: Default::default(),
        }
    }
    pub fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Top,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Bottom,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn height<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Height,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn center_y<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::CenterY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn left<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Left,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn right<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Right,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn width<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Width,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn center_x<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::CenterX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    fn point_driven_check(&mut self) {
        if let GridLocationAnimationHook::SectionDriven(_) = self.animation_hook {
            self.animation_hook =
                GridLocationAnimationHook::PointDriven(PointDrivenAnimationHook::default());
        }
    }
    pub fn point_ax<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        self.point_driven_check();
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointA.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointAX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn point_ay<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        self.point_driven_check();
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointA.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointAY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn point_bx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        self.point_driven_check();
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointB.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointBX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn point_by<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        self.point_driven_check();
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointB.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointBY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn point_cx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        self.point_driven_check();
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointC.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointCX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn point_cy<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        self.point_driven_check();
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointC.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointCY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn point_dx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointD.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointDX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self.point_driven_check();
        self
    }
    pub fn point_dy<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointD.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointDY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self.point_driven_check();
        self
    }
    pub fn except_at<LA: Into<LocationConfiguration>>(mut self, layout: Layout, la: LA) -> Self {
        let config = la.into();
        for c in config.configurations {
            self.exceptions
                .push((GridLocationException::new(layout, c.0), c.1));
        }
        self
    }
}

pub struct LocationConfiguration {
    configurations: [(AspectConfiguration, LocationAspect); 4],
}

impl LocationConfiguration {
    pub fn new() -> Self {
        Self {
            configurations: Default::default(),
        }
    }
    pub fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Top,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_top(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            aspect
                .1
                .set(GridAspect::Top, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Bottom,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_bottom(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            aspect
                .1
                .set(GridAspect::Bottom, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn height<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Height,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_height(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            aspect
                .1
                .set(GridAspect::Height, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn center_y<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::CenterY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_center_y(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Vertical.value())
        {
            aspect
                .1
                .set(GridAspect::CenterY, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn left<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Left,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_left(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            aspect
                .1
                .set(GridAspect::Left, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn right<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Right,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_right(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            aspect
                .1
                .set(GridAspect::Right, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn width<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Width,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_width(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            aspect
                .1
                .set(GridAspect::Width, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn center_x<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::CenterX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_center_x(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::Horizontal.value())
        {
            aspect
                .1
                .set(GridAspect::CenterX, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn point_ax<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointA.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointAX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_point_ax(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointA.value())
        {
            aspect
                .1
                .set(GridAspect::PointAX, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn point_ay<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointA.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointAY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_point_ay(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointA.value())
        {
            aspect
                .1
                .set(GridAspect::PointAY, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn point_bx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointB.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointBX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_point_bx(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointB.value())
        {
            aspect
                .1
                .set(GridAspect::PointBX, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn point_by<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointB.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointBY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_point_by(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointB.value())
        {
            aspect
                .1
                .set(GridAspect::PointBY, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn point_cx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointC.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointCX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_point_cx(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointC.value())
        {
            aspect
                .1
                .set(GridAspect::PointCX, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn point_cy<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointC.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointCY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_point_cy(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointC.value())
        {
            aspect
                .1
                .set(GridAspect::PointCY, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn point_dx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointD.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointDX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_point_dx(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointD.value())
        {
            aspect
                .1
                .set(GridAspect::PointDX, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn point_dy<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointD.value())
        {
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::PointDY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_point_dy(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(AspectConfiguration::PointD.value())
        {
            aspect
                .1
                .set(GridAspect::PointDY, LocationAspectDescriptorValue::Existing);
        }
        self
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub(crate) struct GridLocationException {
    layout: Layout,
    config: AspectConfiguration,
}

impl GridLocationException {
    fn new(layout: Layout, config: AspectConfiguration) -> GridLocationException {
        Self { layout, config }
    }
}
