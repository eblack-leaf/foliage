mod aspect_ratio;
mod layout;
pub(crate) mod location;
pub(crate) mod view;

use crate::foliage::{DiffMarkers, Foliage, MainMarkers};
pub(crate) use crate::grid::layout::viewport_changed;
pub use crate::grid::location::{auto, stack, Adjust, Justify, StackDescriptor};
pub use crate::grid::location::{GridExt, LocationValue};
use crate::grid::view::extent_check_v2;
use crate::{Attachment, Component, CoordinateUnit};
pub use aspect_ratio::AspectRatio;
use bevy_ecs::prelude::IntoSystemConfigs;
pub use layout::Layout;
pub use location::Location;
pub use location::Stack;
pub use location::StackDeps;
pub use view::View;

impl Attachment for Grid {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(Layout::Xs);
        foliage
            .main
            .add_systems(viewport_changed.in_set(MainMarkers::External));
        foliage
            .diff
            .add_systems(extent_check_v2.in_set(DiffMarkers::Prepare));
    }
}
#[derive(Component, Copy, Clone)]
#[require(View)]
pub struct Grid {
    pub xs: GridConfiguration,
    pub sm: Option<GridConfiguration>,
    pub md: Option<GridConfiguration>,
    pub lg: Option<GridConfiguration>,
    pub xl: Option<GridConfiguration>,
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(1.col(), 1.row())
    }
}
impl Grid {
    pub fn new<HA: Into<GridAxisDescriptor>, VA: Into<GridAxisDescriptor>>(ha: HA, va: VA) -> Self {
        Self {
            xs: (ha.into(), va.into()).into(),
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
    pub fn sm<HA: Into<GridAxisDescriptor>, VA: Into<GridAxisDescriptor>>(
        mut self,
        ha: HA,
        va: VA,
    ) -> Self {
        self.sm.replace((ha.into(), va.into()).into());
        self
    }
    pub fn md<HA: Into<GridAxisDescriptor>, VA: Into<GridAxisDescriptor>>(
        mut self,
        ha: HA,
        va: VA,
    ) -> Self {
        self.md.replace((ha.into(), va.into()).into());
        self
    }
    pub fn lg<HA: Into<GridAxisDescriptor>, VA: Into<GridAxisDescriptor>>(
        mut self,
        ha: HA,
        va: VA,
    ) -> Self {
        self.lg.replace((ha.into(), va.into()).into());
        self
    }
    pub fn xl<HA: Into<GridAxisDescriptor>, VA: Into<GridAxisDescriptor>>(
        mut self,
        ha: HA,
        va: VA,
    ) -> Self {
        self.xl.replace((ha.into(), va.into()).into());
        self
    }
    fn at_least_sm(&self) -> GridConfiguration {
        if let Some(sm) = &self.sm {
            *sm
        } else {
            self.xs
        }
    }
    fn at_least_md(&self) -> GridConfiguration {
        if let Some(md) = &self.md {
            *md
        } else {
            self.at_least_sm()
        }
    }
    fn at_least_lg(&self) -> GridConfiguration {
        if let Some(lg) = &self.lg {
            *lg
        } else {
            self.at_least_md()
        }
    }
    pub fn config(&self, layout: Layout) -> GridConfiguration {
        match layout {
            Layout::Xs => self.xs,
            Layout::Sm => self.at_least_sm(),
            Layout::Md => self.at_least_md(),
            Layout::Lg => self.at_least_lg(),
            Layout::Xl => {
                if let Some(xl) = &self.xl {
                    *xl
                } else {
                    self.at_least_lg()
                }
            }
        }
    }
}
#[derive(Copy, Clone)]
pub struct GridConfiguration {
    pub columns: GridAxisDescriptor,
    pub rows: GridAxisDescriptor,
}
#[derive(Copy, Clone)]
pub struct GridAxisDescriptor {
    pub value: LocationValue,
    pub gap: Gap,
}
impl GridAxisDescriptor {
    pub fn gap(mut self, g: CoordinateUnit) -> Self {
        self.gap = Gap { amount: g };
        self
    }
}
impl From<LocationValue> for GridAxisDescriptor {
    fn from(value: LocationValue) -> Self {
        GridAxisDescriptor {
            value,
            gap: Gap::default(),
        }
    }
}
#[derive(Copy, Clone, Debug)]
pub struct Gap {
    pub amount: CoordinateUnit,
}
impl Default for Gap {
    fn default() -> Self {
        Self { amount: 0.0 }
    }
}
impl From<i32> for Gap {
    fn from(x: i32) -> Self {
        Gap {
            amount: x as CoordinateUnit,
        }
    }
}
