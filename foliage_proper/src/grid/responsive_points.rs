use crate::anim::{Animate, Interpolations};
use crate::coordinate::points::Points;
use crate::coordinate::LogicalContext;
use crate::grid::aspect::{ConfigurationDescriptor, GridAspect, PointAspectConfiguration};
use crate::grid::responsive_section::{ConfigureFromLayout, ReferentialData};
use crate::grid::token::{AspectValue, AspectValueWrapper};
use crate::layout::Layout;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use smallvec::SmallVec;

impl ResponsivePoints {
    pub fn point_ax<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointA.value())
        {
            aspect.0 = PointAspectConfiguration::PointA;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::PointAX, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_point_ax(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointA.value())
        {
            aspect.0 = PointAspectConfiguration::PointA;
            aspect
                .1
                .set(GridAspect::PointAX, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn point_ay<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointA.value())
        {
            // sanitize that other is compatible
            aspect.0 = PointAspectConfiguration::PointA;
            aspect
                .1
                .set(GridAspect::PointAY, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_point_ay(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointA.value())
        {
            aspect.0 = PointAspectConfiguration::PointA;
            aspect
                .1
                .set(GridAspect::PointAY, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn point_bx<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointB.value())
        {
            aspect.0 = PointAspectConfiguration::PointB;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::PointBX, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_point_bx(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointB.value())
        {
            aspect.0 = PointAspectConfiguration::PointB;
            aspect
                .1
                .set(GridAspect::PointBX, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn point_by<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointB.value())
        {
            aspect.0 = PointAspectConfiguration::PointB;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::PointBY, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_point_by(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointB.value())
        {
            aspect.0 = PointAspectConfiguration::PointB;
            aspect
                .1
                .set(GridAspect::PointBY, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn point_cx<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        // TODO < 3 check + reserve?
        if let Some(mut aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointC.value())
        {
            aspect.0 = PointAspectConfiguration::PointC;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::PointCX, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_point_cx(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointC.value())
        {
            aspect.0 = PointAspectConfiguration::PointC;
            aspect
                .1
                .set(GridAspect::PointCX, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn point_cy<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointC.value())
        {
            aspect.0 = PointAspectConfiguration::PointC;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::PointCY, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_point_cy(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointC.value())
        {
            aspect.0 = PointAspectConfiguration::PointC;
            aspect
                .1
                .set(GridAspect::PointCY, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn point_dx<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointD.value())
        {
            aspect.0 = PointAspectConfiguration::PointD;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::PointDX, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_point_dx(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointD.value())
        {
            aspect.0 = PointAspectConfiguration::PointD;
            aspect
                .1
                .set(GridAspect::PointDX, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn point_dy<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointD.value())
        {
            aspect.0 = PointAspectConfiguration::PointD;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::PointDY, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_point_dy(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(PointAspectConfiguration::PointD.value())
        {
            aspect.0 = PointAspectConfiguration::PointD;
            aspect
                .1
                .set(GridAspect::PointDY, AspectValueWrapper::Existing);
        }
        self
    }
}
#[derive(Component, Clone, Default)]
pub struct ResponsivePoints {
    pub(crate) configurations: [(PointAspectConfiguration, ConfigurationDescriptor); 4],
}
#[derive(Default, Component, Clone)]
pub(crate) struct PointExceptions {
    pub(crate) exceptions: SmallVec<[(PointException, ConfigurationDescriptor); 2]>,
}
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct PointException {
    pub(crate) layout: Layout,
    pub(crate) pac: PointAspectConfiguration,
}
impl PointException {
    pub(crate) fn new(
        layout: Layout,
        point_aspect_configuration: PointAspectConfiguration,
    ) -> Self {
        Self {
            layout,
            pac: point_aspect_configuration,
        }
    }
}
#[derive(Component)]
pub struct ResponsivePointsAnimPackage {
    pub(crate) base_points: ResponsivePoints,
    pub(crate) exceptions: PointExceptions,
}
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct ResponsivePointsAnimationHook {
    pub(crate) points: Points<LogicalContext>,
    pub(crate) percent: f32,
}
impl ResponsivePointsAnimationHook {
    pub(crate) fn value(&self) -> Points<LogicalContext> {
        self.points * self.percent
    }
}
impl Animate for ResponsivePointsAnimationHook {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        todo!()
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(s) = interpolations.read(0) {
            self.percent = s;
        }
    }
}
#[derive(Component, Clone, Default)]
pub(crate) struct ResolvedPoints {
    pub(crate) configurations: [(PointAspectConfiguration, ConfigurationDescriptor); 4],
}
impl ResolvedPoints {
    pub(crate) fn evaluate(
        &self,
        stem: ReferentialData,
        screen: ReferentialData,
    ) -> Option<Points<LogicalContext>> {
        let mut resolution = Points::default();
        for (a, b) in self.configurations.iter() {
            if b.count == 0 {
                continue;
            }
            let pair_config = (b.aspects[0].aspect, b.aspects[1].aspect);
            let data = (
                b.aspects[0].value.resolve(stem, screen),
                b.aspects[1].value.resolve(stem, screen),
            );
            match a {
                PointAspectConfiguration::PointA => {
                    if pair_config == (GridAspect::PointAX, GridAspect::PointAY) {
                        resolution.data[0] = data.into();
                    } else {
                        panic!("invalid-configuration aspect")
                    }
                }
                PointAspectConfiguration::PointB => {
                    if pair_config == (GridAspect::PointBX, GridAspect::PointBY) {
                        resolution.data[1] = data.into();
                    } else {
                        panic!("invalid-configuration aspect")
                    }
                }
                PointAspectConfiguration::PointC => {
                    if pair_config == (GridAspect::PointCX, GridAspect::PointCY) {
                        resolution.data[2] = data.into();
                    } else {
                        panic!("invalid-configuration aspect")
                    }
                }
                PointAspectConfiguration::PointD => {
                    if pair_config == (GridAspect::PointDX, GridAspect::PointDY) {
                        resolution.data[3] = data.into();
                    } else {
                        panic!("invalid-configuration aspect")
                    }
                }
            }
        }
        Some(resolution)
    }
}
#[derive(Bundle, Default)]
pub struct ResponsivePointBundle {
    pub(crate) points: ResolvedPoints,
    pub(crate) exceptions: PointExceptions,
    pub(crate) base_points: ResponsivePoints,
    layout_check: ConfigureFromLayout,
    diff: ResponsivePointsAnimationHook,
}
impl ResponsivePointBundle {
    pub fn point_ax<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base_points
            .configurations
            .get_mut(PointAspectConfiguration::PointA.value())
        {
            // sanitize that other is compatible
            aspect.0 = PointAspectConfiguration::PointA;
            aspect
                .1
                .set(GridAspect::PointAX, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn point_ay<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base_points
            .configurations
            .get_mut(PointAspectConfiguration::PointA.value())
        {
            // sanitize that other is compatible
            aspect.0 = PointAspectConfiguration::PointA;
            aspect
                .1
                .set(GridAspect::PointAY, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn point_bx<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base_points
            .configurations
            .get_mut(PointAspectConfiguration::PointB.value())
        {
            // sanitize that other is compatible
            aspect.0 = PointAspectConfiguration::PointB;
            aspect
                .1
                .set(GridAspect::PointBX, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn point_by<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base_points
            .configurations
            .get_mut(PointAspectConfiguration::PointB.value())
        {
            // sanitize that other is compatible
            aspect.0 = PointAspectConfiguration::PointB;
            aspect
                .1
                .set(GridAspect::PointBY, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn point_cx<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base_points
            .configurations
            .get_mut(PointAspectConfiguration::PointC.value())
        {
            // sanitize that other is compatible
            aspect.0 = PointAspectConfiguration::PointC;
            aspect
                .1
                .set(GridAspect::PointCX, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn point_cy<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base_points
            .configurations
            .get_mut(PointAspectConfiguration::PointC.value())
        {
            // sanitize that other is compatible
            aspect.0 = PointAspectConfiguration::PointC;
            aspect
                .1
                .set(GridAspect::PointCY, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn point_dx<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base_points
            .configurations
            .get_mut(PointAspectConfiguration::PointD.value())
        {
            // sanitize that other is compatible
            aspect.0 = PointAspectConfiguration::PointD;
            aspect
                .1
                .set(GridAspect::PointDX, AspectValueWrapper::Specified(d.into()));
        }

        self
    }
    pub fn point_dy<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base_points
            .configurations
            .get_mut(PointAspectConfiguration::PointD.value())
        {
            // sanitize that other is compatible
            aspect.0 = PointAspectConfiguration::PointD;
            aspect
                .1
                .set(GridAspect::PointDY, AspectValueWrapper::Specified(d.into()));
        }

        self
    }
    pub fn except_at<RP: Into<ResponsivePoints>>(mut self, layout: Layout, rp: RP) -> Self {
        let config = rp.into();
        for (c, l) in config.configurations {
            self.exceptions
                .exceptions
                .push((PointException::new(layout, c), l));
        }
        self
    }
}
