mod aspect_ratio;
mod layout;
pub(crate) mod location;
pub(crate) mod view;

use crate::foliage::{DiffMarkers, Foliage, MainMarkers};
pub(crate) use crate::grid::layout::viewport_changed;
pub use crate::grid::location::{
    Adjust, AnchorDescriptor, ConfigurationDescriptor, Justify, ValueDescriptor, anchor,
    text_content,
};
pub use crate::grid::location::{GridExt, LocationValue};
use crate::grid::view::{ScrolledViews, coast, extent_check, propagate_offsets};
use crate::{Attachment, Component, CoordinateUnit};
pub use aspect_ratio::AspectRatio;
use bevy_ecs::prelude::IntoScheduleConfigs;
pub use layout::{Layout, Short};
pub use location::Anchor;
pub use location::AnchorDeps;
pub use location::Location;
pub use view::{ScrollMomentum, ScrollProgress, ScrollTo, View};

impl Attachment for Grid {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(Layout::Xs);
        foliage.world.insert_resource(Short::No);
        foliage.world.insert_resource(ScrollMomentum::default());
        foliage.world.insert_resource(ScrolledViews::default());
        foliage
            .main
            .add_systems(viewport_changed.in_set(MainMarkers::External));
        foliage.main.add_systems(coast.in_set(MainMarkers::Process));
        foliage.diff.add_systems(
            (extent_check, propagate_offsets)
                .chain()
                .in_set(DiffMarkers::Prepare),
        );
    }
}
/// The coordinate system a parent offers its children, per breakpoint.
///
/// Any entity whose children position themselves with `.col()`/`.row()` needs one -- and
/// in fact **every** parent of a positioned child needs a `Grid`, even a single-cell
/// `Grid::new(1.col().gap(0), 1.row().gap(0))`, since that is what a child's `Location`
/// resolves against.
///
/// Columns and rows can be sized in fractions of the parent (`3.col()`), fixed pixels, or
/// characters (`1.letters()`, taking its pitch from the parent's own
/// [`FontSize`](crate::FontSize)). `Gap` (engine-internal) is the space *between* tracks,
/// never outside them.
///
/// Requires [`View`], which is not optional plumbing: a parent's `View` carries the
/// accumulated scroll offset every child subtracts to land on screen -- that is how
/// children move when a parent scrolls -- as well as the extent the overscroll chain
/// walks. Both are total, which is why every positioned child's parent needs a `Grid`: it
/// is what guarantees the `View` exists. Most carry an offset of zero forever, and that is
/// fine.
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
    /// A grid used at every breakpoint: `Grid::new(3.col().gap(8), 2.row())`. Add
    /// exceptions with [`sm`](Self::sm)/[`md`](Self::md)/[`lg`](Self::lg)/[`xl`](Self::xl).
    pub fn new<HA: Into<GridAxisDescriptor>, VA: Into<GridAxisDescriptor>>(ha: HA, va: VA) -> Self {
        Self {
            xs: (ha.into(), va.into()).into(),
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
    /// Overrides the configuration from the `sm` breakpoint up.
    pub fn sm<HA: Into<GridAxisDescriptor>, VA: Into<GridAxisDescriptor>>(
        mut self,
        ha: HA,
        va: VA,
    ) -> Self {
        self.sm.replace((ha.into(), va.into()).into());
        self
    }
    /// Overrides the configuration from the `md` breakpoint up.
    pub fn md<HA: Into<GridAxisDescriptor>, VA: Into<GridAxisDescriptor>>(
        mut self,
        ha: HA,
        va: VA,
    ) -> Self {
        self.md.replace((ha.into(), va.into()).into());
        self
    }
    /// Overrides the configuration from the `lg` breakpoint up.
    pub fn lg<HA: Into<GridAxisDescriptor>, VA: Into<GridAxisDescriptor>>(
        mut self,
        ha: HA,
        va: VA,
    ) -> Self {
        self.lg.replace((ha.into(), va.into()).into());
        self
    }
    /// Overrides the configuration from the `xl` breakpoint up.
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
    /// The configuration in force at `layout`, falling back down the breakpoints to `xs`.
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
/// One breakpoint's worth of grid: how the horizontal and vertical axes are divided.
#[derive(Copy, Clone)]
pub struct GridConfiguration {
    pub columns: GridAxisDescriptor,
    pub rows: GridAxisDescriptor,
}
impl From<(GridAxisDescriptor, GridAxisDescriptor)> for GridConfiguration {
    fn from((columns, rows): (GridAxisDescriptor, GridAxisDescriptor)) -> Self {
        Self { columns, rows }
    }
}
/// One axis of a [`Grid`]: how it is divided, and the space between the resulting tracks.
#[derive(Copy, Clone)]
pub struct GridAxisDescriptor {
    pub value: LocationValue,
    pub gap: Gap,
}
impl GridAxisDescriptor {
    /// Sets the gap between tracks on this axis, in logical pixels.
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
/// Space between adjacent tracks, in logical pixels. Applies only between tracks, so an
/// n-track axis has n-1 gaps and no outer margin.
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
