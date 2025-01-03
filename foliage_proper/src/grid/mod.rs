mod aspect_ratio;
mod layout;
pub(crate) mod location;
mod view;

use crate::foliage::{DiffMarkers, Foliage, MainMarkers};
pub(crate) use crate::grid::layout::viewport_changed;
pub use crate::grid::location::{
    auto, stack, Justify, LocationAxisDescriptor, LocationAxisType, Padding,
};
use crate::grid::view::{extent_check, prepare_extent, ExtentCheckIds};
use crate::{Attachment, Component, CoordinateUnit, Logical, Section};
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
        foliage.world.insert_resource(ExtentCheckIds::default());
        foliage
            .main
            .add_systems(viewport_changed.in_set(MainMarkers::External));
        foliage.diff.add_systems(
            (prepare_extent, extent_check)
                .chain()
                .in_set(DiffMarkers::Prepare),
        );
        foliage.define(Location::update_from_visibility);
        foliage.define(Location::update_location);
    }
}
pub trait GridExt {
    fn px(self) -> GridUnit;
    fn pct(self) -> GridUnit;
    fn col(self) -> GridUnit;
    fn row(self) -> GridUnit;
}
impl GridExt for i32 {
    fn px(self) -> GridUnit {
        GridUnit::Scalar(ScalarUnit::Px(self as CoordinateUnit))
    }

    fn pct(self) -> GridUnit {
        GridUnit::Scalar(ScalarUnit::Pct(self as CoordinateUnit / 100.0))
    }

    fn col(self) -> GridUnit {
        GridUnit::Aligned(AlignedUnit::Columns(self))
    }

    fn row(self) -> GridUnit {
        GridUnit::Aligned(AlignedUnit::Rows(self))
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
    pub fn column_metrics(
        &self,
        layout: Layout,
        stem: Section<Logical>,
    ) -> (CoordinateUnit, CoordinateUnit) {
        let columns = self.config(layout).columns;
        let c = match columns.unit {
            GridAxisUnit::Infinite(inf) => match inf {
                ScalarUnit::Px(px) => px,
                ScalarUnit::Pct(pct) => stem.width() * pct,
            },
            GridAxisUnit::Explicit(exp) => {
                let without_gap = stem.width() - columns.gap.amount * (exp.value() + 1.0);

                without_gap / exp.value()
            }
        };
        (c, columns.gap.amount)
    }
    pub fn column(
        &self,
        layout: Layout,
        stem: Section<Logical>,
        aligned_unit: AlignedUnit,
        inclusive: bool,
    ) -> CoordinateUnit {
        let num = aligned_unit.value();
        let (column_size, gap) = self.column_metrics(layout, stem);
        match aligned_unit {
            AlignedUnit::Columns(_) => {
                (num - 1.0 * CoordinateUnit::from(!inclusive)) * column_size + num * gap
            }
            AlignedUnit::Rows(_) => {
                panic!("Rows are not supported in horizontal.");
            }
        }
    }
    pub fn row_metrics(
        &self,
        layout: Layout,
        stem: Section<Logical>,
    ) -> (CoordinateUnit, CoordinateUnit) {
        let rows = self.config(layout).rows;
        let c = match rows.unit {
            GridAxisUnit::Infinite(inf) => match inf {
                ScalarUnit::Px(px) => px,
                ScalarUnit::Pct(pct) => stem.height() * pct,
            },
            GridAxisUnit::Explicit(exp) => {
                let without_gap = stem.height() - rows.gap.amount * (exp.value() + 1.0);

                without_gap / exp.value()
            }
        };
        (c, rows.gap.amount)
    }
    pub fn row(
        &self,
        layout: Layout,
        stem: Section<Logical>,
        aligned_unit: AlignedUnit,
        inclusive: bool,
    ) -> CoordinateUnit {
        let num = aligned_unit.value();
        let (row_size, gap) = self.row_metrics(layout, stem);
        match aligned_unit {
            AlignedUnit::Columns(_) => {
                panic!("Columns are not supported in vertical.");
            }
            AlignedUnit::Rows(_) => {
                (num - 1.0 * CoordinateUnit::from(!inclusive)) * row_size + num * gap
            }
        }
    }
}
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
#[derive(Copy, Clone)]
pub struct GridAxisDescriptor {
    pub unit: GridAxisUnit,
    pub gap: Gap,
}
impl From<GridUnit> for GridAxisDescriptor {
    fn from(unit: GridUnit) -> Self {
        match unit {
            GridUnit::Aligned(a) => GridAxisDescriptor {
                unit: GridAxisUnit::Explicit(a),
                gap: Gap::default(),
            },
            GridUnit::Scalar(s) => GridAxisDescriptor {
                unit: GridAxisUnit::Infinite(s),
                gap: Gap::default(),
            },
            _ => panic!("only aligned and scalar allowed"),
        }
    }
}
#[derive(Copy, Clone)]
pub enum GridAxisUnit {
    Infinite(ScalarUnit),
    Explicit(AlignedUnit),
}
#[derive(Copy, Clone, PartialEq, Debug, PartialOrd)]
pub enum ScalarUnit {
    Px(CoordinateUnit),
    Pct(f32),
}
impl From<GridUnit> for ScalarUnit {
    fn from(grid: GridUnit) -> Self {
        match grid {
            GridUnit::Scalar(s) => s,
            _ => panic!("not scalar"),
        }
    }
}
impl ScalarUnit {
    pub(crate) fn vertical(&self, stem: Section<Logical>) -> CoordinateUnit {
        match self {
            ScalarUnit::Px(px) => stem.top() + px,
            ScalarUnit::Pct(pct) => stem.height() * pct,
        }
    }
    pub fn horizontal(self, stem: Section<Logical>) -> CoordinateUnit {
        match self {
            ScalarUnit::Px(px) => stem.left() + px,
            ScalarUnit::Pct(pct) => stem.width() * pct,
        }
    }
}
#[derive(Copy, Clone)]
pub struct Gap {
    pub amount: CoordinateUnit,
}
impl Default for Gap {
    fn default() -> Self {
        Self { amount: 8.0 }
    }
}
impl From<i32> for Gap {
    fn from(x: i32) -> Self {
        Gap {
            amount: x as CoordinateUnit,
        }
    }
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GridUnit {
    Aligned(AlignedUnit),
    Scalar(ScalarUnit),
    Stack,
    Auto,
}
impl GridUnit {
    pub fn gap<G: Into<Gap>>(self, g: G) -> GridAxisDescriptor {
        GridAxisDescriptor {
            unit: match self {
                GridUnit::Aligned(a) => GridAxisUnit::Explicit(a),
                GridUnit::Scalar(s) => GridAxisUnit::Infinite(s),
                _ => panic!("not grid axis unit"),
            },
            gap: g.into(),
        }
    }
    pub fn y<Y: Into<GridUnit>>(self, y: Y) -> LocationAxisDescriptor {
        let y = y.into();
        debug_assert_ne!(self, GridUnit::Auto);
        debug_assert_ne!(y, GridUnit::Stack);
        debug_assert_ne!(y, GridUnit::Auto);
        LocationAxisDescriptor {
            a: self,
            b: y,
            ty: LocationAxisType::Point,
            padding: Padding::default(),
            justify: Justify::default(),
            max: None,
            min: None,
        }
    }
    pub fn to<T: Into<GridUnit>>(self, t: T) -> LocationAxisDescriptor {
        let t = t.into();
        debug_assert_ne!(self, GridUnit::Auto);
        debug_assert_ne!(t, GridUnit::Stack);
        LocationAxisDescriptor {
            a: self,
            b: t,
            ty: LocationAxisType::To,
            padding: Default::default(),
            justify: Default::default(),
            max: None,
            min: None,
        }
    }
    pub fn span<S: Into<GridUnit>>(self, s: S) -> LocationAxisDescriptor {
        let s = s.into();
        debug_assert_ne!(self, GridUnit::Auto);
        debug_assert_ne!(s, GridUnit::Stack);
        LocationAxisDescriptor {
            a: self,
            b: s,
            ty: LocationAxisType::Span,
            padding: Default::default(),
            justify: Default::default(),
            max: None,
            min: None,
        }
    }
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AlignedUnit {
    Columns(i32),
    Rows(i32),
}
impl AlignedUnit {
    pub fn value(self) -> CoordinateUnit {
        match self {
            AlignedUnit::Columns(c) => c as CoordinateUnit,
            AlignedUnit::Rows(r) => r as CoordinateUnit,
        }
    }
}
