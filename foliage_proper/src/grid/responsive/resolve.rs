use crate::coordinate::points::Points;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::grid::aspect::{
    Configuration, ConfigurationDescriptor, GridAspect, PointAspectConfiguration,
};
use crate::grid::responsive::evaluate::ReferentialData;
use bevy_ecs::component::Component;

#[derive(Component, Clone, Default)]
pub(crate) struct ResolvedConfiguration {
    pub(crate) configurations: [(Configuration, ConfigurationDescriptor); 2],
}

impl ResolvedConfiguration {
    pub(crate) fn evaluate(
        &self,
        stem: ReferentialData,
        screen: ReferentialData,
    ) -> Option<(Section<LogicalContext>, bool, bool)> {
        let mut resolution = Section::default();
        let mut aw = false;
        let mut ah = false;
        for (aspect_config, aspect_value) in self.configurations.iter() {
            let pair_config = (
                aspect_value.aspects[0].aspect,
                aspect_value.aspects[1].aspect,
            );
            let (x, auto_w_found) = aspect_value.aspects[0].value.resolve(stem, screen);
            let (y, auto_h_found) = aspect_value.aspects[1].value.resolve(stem, screen);
            if auto_w_found {
                aw = true;
            }
            if auto_h_found {
                ah = true;
            }
            let data = (x, y);
            match aspect_config {
                Configuration::Horizontal => {
                    if pair_config == (GridAspect::Left, GridAspect::Right) {
                        resolution.position.set_x(data.0);
                        resolution.area.set_width(data.1 - data.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::CenterX) {
                        resolution.position.set_x(data.0);
                        resolution.area.set_width((data.1 - data.0) * 2.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::Width) {
                        resolution.position.set_x(data.0);
                        resolution.area.set_width(data.1);
                    } else if pair_config == (GridAspect::Width, GridAspect::CenterX) {
                        resolution.position.set_x(data.1 - data.0 / 2.0);
                        resolution.area.set_width(data.0);
                    } else if pair_config == (GridAspect::Width, GridAspect::Right) {
                        resolution.position.set_x(data.1 - data.0);
                        resolution.area.set_width(data.0);
                    } else if pair_config == (GridAspect::CenterX, GridAspect::Right) {
                        let diff = data.1 - data.0;
                        resolution.position.set_x(data.0 - diff);
                        resolution.area.set_width(diff * 2.0);
                    }
                }
                Configuration::Vertical => {
                    if pair_config == (GridAspect::Top, GridAspect::Bottom) {
                        resolution.position.set_y(data.0);
                        resolution.area.set_height(data.1 - data.0);
                    } else if pair_config == (GridAspect::Top, GridAspect::CenterY) {
                        resolution.position.set_y(data.0);
                        resolution.area.set_height((data.1 - data.0) * 2.0);
                    } else if pair_config == (GridAspect::Top, GridAspect::Height) {
                        resolution.position.set_y(data.0);
                        resolution.area.set_height(data.1);
                    } else if pair_config == (GridAspect::Height, GridAspect::CenterY) {
                        resolution.position.set_y(data.1 - data.0 / 2.0);
                        resolution.area.set_height(data.0);
                    } else if pair_config == (GridAspect::Height, GridAspect::Bottom) {
                        resolution.position.set_y(data.1 - data.0);
                        resolution.area.set_height(data.0);
                    } else if pair_config == (GridAspect::CenterY, GridAspect::Bottom) {
                        let diff = data.1 - data.0;
                        resolution.position.set_y(data.0 - diff);
                        resolution.area.set_height(diff * 2.0);
                    }
                }
            }
        }
        resolution.position += stem.view.position;
        Some((resolution, aw, ah))
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
                b.aspects[0].value.resolve(stem, screen).0,
                b.aspects[1].value.resolve(stem, screen).0,
            );
            match a {
                PointAspectConfiguration::PointA => {
                    if pair_config == (GridAspect::PointAX, GridAspect::PointAY) {
                        resolution.data[0] =
                            Position::<LogicalContext>::from(data) + stem.view.position;
                    } else {
                        panic!("invalid-configuration aspect")
                    }
                }
                PointAspectConfiguration::PointB => {
                    if pair_config == (GridAspect::PointBX, GridAspect::PointBY) {
                        resolution.data[1] =
                            Position::<LogicalContext>::from(data) + stem.view.position;
                    } else {
                        panic!("invalid-configuration aspect")
                    }
                }
                PointAspectConfiguration::PointC => {
                    if pair_config == (GridAspect::PointCX, GridAspect::PointCY) {
                        resolution.data[2] =
                            Position::<LogicalContext>::from(data) + stem.view.position;
                    } else {
                        panic!("invalid-configuration aspect")
                    }
                }
                PointAspectConfiguration::PointD => {
                    if pair_config == (GridAspect::PointDX, GridAspect::PointDY) {
                        resolution.data[3] =
                            Position::<LogicalContext>::from(data) + stem.view.position;
                    } else {
                        panic!("invalid-configuration aspect")
                    }
                }
            }
        }
        Some(resolution)
    }
}
