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
    ) -> Option<(Section<LogicalContext>, (bool, usize, GridAspect), (bool, usize, GridAspect))> {
        let mut resolution = Section::<LogicalContext>::default();
        let mut aw = (false, 0, GridAspect::default());
        let mut ah = (false, 0, GridAspect::default());
        for (aspect_config, aspect_value) in self.configurations.iter() {
            let pair_config = (
                aspect_value.aspects[0].aspect,
                aspect_value.aspects[1].aspect,
            );
            let (a, auto_a_found) = aspect_value.aspects[0].value.resolve(stem, screen);
            let (b, auto_b_found) = aspect_value.aspects[1].value.resolve(stem, screen);
            let data = (a, b);
            match aspect_config {
                Configuration::Horizontal => {
                    if pair_config == (GridAspect::Left, GridAspect::Right) {
                        resolution.position.set_x(data.0);
                        resolution.area.set_width(data.1 - data.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::CenterX) {
                        resolution.position.set_x(data.0);
                        resolution.area.set_width((data.1 - data.0) * 2.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::Width) {
                        if auto_b_found {
                            aw.0 = true;
                            aw.1 = 1;
                            aw.2 = GridAspect::Left;
                        }
                        resolution.position.set_x(data.0);
                        resolution.area.set_width(data.1);
                    } else if pair_config == (GridAspect::Width, GridAspect::CenterX) {
                        if auto_a_found {
                            aw.0 = true;
                            aw.2 = GridAspect::CenterX;
                        }
                        resolution.position.set_x(data.1 - data.0 / 2.0);
                        resolution.area.set_width(data.0);
                    } else if pair_config == (GridAspect::Width, GridAspect::Right) {
                        if auto_a_found {
                            aw.0 = true;
                            aw.2 = GridAspect::Right;
                        }
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
                        if auto_b_found {
                            ah.0 = true;
                            ah.1 = 1;
                            ah.2 = GridAspect::Top;
                        }
                        resolution.position.set_y(data.0);
                        resolution.area.set_height(data.1);
                    } else if pair_config == (GridAspect::Height, GridAspect::CenterY) {
                        if auto_a_found {
                            ah.0 = true;
                            ah.2 = GridAspect::CenterY;
                        }
                        resolution.position.set_y(data.1 - data.0 / 2.0);
                        resolution.area.set_height(data.0);
                    } else if pair_config == (GridAspect::Height, GridAspect::Bottom) {
                        if auto_a_found {
                            ah.0 = true;
                            ah.2 = GridAspect::Bottom;
                        }
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
