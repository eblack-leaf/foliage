use crate::anim::Animate;
use crate::grid::aspect::{
    Configuration, ConfigurationDescriptor, GridAspect, PointAspectConfiguration,
};
use crate::grid::token::{AspectValue, AspectValueWrapper};
use crate::grid::unit::TokenUnit;
use crate::layout::Layout;
use anim::{ResponsiveAnimationHook, ResponsivePointsAnimationHook};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::event::Event;
use configure::ConfigureFromLayout;
use resolve::{ResolvedConfiguration, ResolvedPoints};
use smallvec::SmallVec;

pub(crate) mod anim;
pub mod configure;
pub mod evaluate;
pub(crate) mod resolve;

#[derive(Bundle, Clone)]
pub struct ResponsiveLocation {
    pub(crate) resolved_configuration: ResolvedConfiguration,
    pub(crate) base: ResponsiveSection,
    pub(crate) exceptions: ResponsiveConfigurationException,
    pub(crate) layout_check: ConfigureFromLayout,
    pub(crate) diff: ResponsiveAnimationHook,
}
impl ResponsiveLocation {
    pub fn new() -> Self {
        ResponsiveLocation {
            resolved_configuration: ResolvedConfiguration::default(),
            base: Default::default(),
            exceptions: Default::default(),
            layout_check: Default::default(),
            diff: Default::default(),
        }
    }
    pub fn points() -> ResponsivePointBundle {
        ResponsivePointBundle::default()
    }
}
impl ResponsiveLocation {
    pub fn top<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Vertical.value())
        {
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::Top, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn bottom<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::Bottom, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn auto_height(mut self) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Vertical.value())
        {
            aspect.0 = Configuration::Vertical;
            aspect.1.set(GridAspect::Height, AspectValueWrapper::Auto);
        }
        self
    }
    pub fn height<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::Height, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn center_y<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::CenterY, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn left<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = Configuration::Horizontal;
            aspect
                .1
                .set(GridAspect::Left, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn right<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = Configuration::Horizontal;
            aspect
                .1
                .set(GridAspect::Right, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn auto_width(mut self) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            aspect.0 = Configuration::Horizontal;
            aspect.1.set(GridAspect::Width, AspectValueWrapper::Auto);
        }
        self
    }
    pub fn width<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = Configuration::Horizontal;
            aspect
                .1
                .set(GridAspect::Width, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn center_x<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .base
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = Configuration::Horizontal;
            aspect
                .1
                .set(GridAspect::CenterX, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn except_at<LA: Into<ResponsiveSection>>(mut self, layout: Layout, la: LA) -> Self {
        let config = la.into();
        for c in config.configurations {
            self.exceptions
                .exceptions
                .push((SectionException::new(layout, c.0), c.1));
        }
        self
    }
}
#[derive(Component, Default, Clone)]
pub struct ResponsiveConfigurationException {
    pub exceptions: SmallVec<[(SectionException, ConfigurationDescriptor); 2]>,
}
#[derive(Clone, Component, Default)]
pub struct ResponsiveSection {
    configurations: [(Configuration, ConfigurationDescriptor); 2],
}
impl ResponsiveSection {
    pub fn new() -> Self {
        Self {
            configurations: Default::default(),
        }
    }
    pub fn top<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(Configuration::Vertical.value()) {
            // sanitize that other is compatible
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::Top, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_top(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(Configuration::Vertical.value()) {
            aspect.0 = Configuration::Vertical;
            aspect.1.set(GridAspect::Top, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn bottom<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(Configuration::Vertical.value()) {
            // sanitize that other is compatible
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::Bottom, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_bottom(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(Configuration::Vertical.value()) {
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::Bottom, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn height<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(Configuration::Vertical.value()) {
            // sanitize that other is compatible
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::Height, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn auto_height(mut self) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(Configuration::Vertical.value()) {
            aspect.0 = Configuration::Vertical;
            aspect.1.set(GridAspect::Height, AspectValueWrapper::Auto);
        }
        self
    }
    pub fn existing_height(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(Configuration::Vertical.value()) {
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::Height, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn center_y<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(Configuration::Vertical.value()) {
            // sanitize that other is compatible
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::CenterY, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_center_y(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(Configuration::Vertical.value()) {
            aspect.0 = Configuration::Vertical;
            aspect
                .1
                .set(GridAspect::CenterY, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn left<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = Configuration::Horizontal;
            aspect
                .1
                .set(GridAspect::Left, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_left(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            aspect.0 = Configuration::Horizontal;
            aspect.1.set(GridAspect::Left, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn right<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            aspect.0 = Configuration::Horizontal;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::Right, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_right(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            aspect.0 = Configuration::Horizontal;
            aspect
                .1
                .set(GridAspect::Right, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn width<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            aspect.0 = Configuration::Horizontal;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::Width, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn auto_width(mut self) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            aspect.0 = Configuration::Horizontal;
            aspect.1.set(GridAspect::Width, AspectValueWrapper::Auto);
        }
        self
    }
    pub fn existing_width(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            aspect.0 = Configuration::Horizontal;
            aspect
                .1
                .set(GridAspect::Width, AspectValueWrapper::Existing);
        }
        self
    }
    pub fn center_x<LAD: Into<AspectValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            aspect.0 = Configuration::Horizontal;
            // sanitize that other is compatible
            aspect
                .1
                .set(GridAspect::CenterX, AspectValueWrapper::Specified(d.into()));
        }
        self
    }
    pub fn existing_center_x(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(Configuration::Horizontal.value())
        {
            aspect.0 = Configuration::Horizontal;
            aspect
                .1
                .set(GridAspect::CenterX, AspectValueWrapper::Existing);
        }
        self
    }
}
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct SectionException {
    layout: Layout,
    config: Configuration,
}
impl SectionException {
    fn new(layout: Layout, config: Configuration) -> SectionException {
        Self { layout, config }
    }
}

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

#[derive(Bundle, Default, Clone)]
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
