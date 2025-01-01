mod aspect_ratio;
mod layout;
mod location;
mod view;

use crate::grid::layout::viewport_changed;
use crate::grid::location::Justify::{Center, Left};
pub use crate::grid::location::{
    auto, stack, Justify, LocationAxisDescriptor, LocationAxisType, Padding,
};
use crate::grid::view::{extent_check, prepare_extent};
use crate::{Attachment, Component, CoordinateUnit, Coordinates, DiffMarkers, Foliage};
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
        foliage.main.add_systems(viewport_changed);
        foliage.diff.add_systems(
            (prepare_extent, extent_check)
                .chain()
                .in_set(DiffMarkers::Prepare),
        );
        foliage.define(Location::update_from_visibility);
        foliage.define(Location::update_location);
    }
}
#[test]
fn behavior() {
    use crate::FontSize;
    let grid = Grid::new(12.col().gap(4), 8.px().gap(4))
        .md(12.col().gap(4), 8.px().gap(4))
        .lg(12.col().gap(8), 16.px().gap(8))
        .xl(12.col().gap(12), 24.px().gap(12)) // canon
        .max(12.col().gap(12), 24.px().gap(12)); // canon
    let root = Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct()));
    // let view = View::context(root); // scrolling
    let location = Location::new().xs(50.px().y(100.px()), 50.px().y(150.px())); // points
    let location = Location::new().xs(1.col().to(12.col()), 1.row().to(19.row()));
    let location = Location::new()
        .xs(
            2.col().to(11.col()).max(400.px()).justify(Center).pad(4),
            4.row().to(10.row()).pad((4, 8)), // debug-assert max only on width
        )
        .sm(
            3.col().to(10.col()).max(500.px()).justify(Center),
            4.row().to(10.row()),
        )
        .xl(
            3.col().to(10.col()).max(500.px()).justify(Center),
            4.row().to(10.row()),
        );
    let aspect_ratio = AspectRatio::new().sm(16.0 / 9.0);
    let font_size = FontSize::default().sm(20).md(24).lg(32);
    let location = Location::new()
        .xs(
            3.col().to(10.col()).max(300.px()).justify(Left),
            6.row().to(9.row()),
        )
        .sm(
            4.col().to(9.col()).max(400.px()).justify(Left),
            6.row().to(9.row()),
        );
    let location = Location::new().xs(1.col().to(1.col()), 2.row().to(auto()));
    let location = Location::new().xs(1.col().to(1.col()), stack().to(auto())); // stack uses stem().bottom() as this.top()
    let location = Location::new().xs(1.col().to(1.col()), stack().to(25.row())); // explicit back to grid (acceptable content-range) or keep stacking
    let location = Location::new().xs(1.col().to(1.col()), stack().span(5.row()));
    // span cause unknown end (to)
    // span(5.row()) => 5 row lengths from current px() in stack (not necessarily aligned)
    // but spacing relative is guaranteed 5 rows for content
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
    pub fn config(&self, layout: Layout) -> GridConfiguration {
        match layout {
            Layout::Xs => self.xs,
            Layout::Sm => {
                todo!()
            }
            Layout::Md => {
                todo!()
            }
            Layout::Lg => {
                todo!()
            }
            Layout::Xl => {
                todo!()
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
#[derive(Copy, Clone)]
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
            amount: (x, x).into(),
        }
    }
}
#[derive(Copy, Clone)]
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
        LocationAxisDescriptor {
            a: self,
            b: y.into(),
            ty: LocationAxisType::Point,
            padding: Padding::default(),
            justify: Justify::default(),
            max: None,
        }
    }
    pub fn to<T: Into<GridUnit>>(self, t: T) -> LocationAxisDescriptor {
        LocationAxisDescriptor {
            a: self,
            b: t.into(),
            ty: LocationAxisType::To,
            padding: Default::default(),
            justify: Default::default(),
            max: None,
        }
    }
    pub fn span<S: Into<GridUnit>>(self, s: S) -> LocationAxisDescriptor {
        LocationAxisDescriptor {
            a: self,
            b: s.into(),
            ty: LocationAxisType::Span,
            padding: Default::default(),
            justify: Default::default(),
            max: None,
        }
    }
}
#[derive(Copy, Clone)]
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
