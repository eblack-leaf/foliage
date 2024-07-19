use crate::coordinate::layer::Layer;
use crate::coordinate::placement::Placement;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, Coordinates, LogicalContext};
use crate::ginkgo::viewport::ViewportHandle;
use bevy_ecs::prelude::{Component, ResMut, Resource};
use bitflags::bitflags;
use std::collections::HashMap;

#[derive(Copy, Clone, Component)]
pub struct Grid {
    cols: i32,
    rows: i32,
    column_size: CoordinateUnit,
    row_size: CoordinateUnit,
    placement: Placement<LogicalContext>,
    gap: Coordinates,
}

impl Grid {
    pub fn new(cols: i32, rows: i32) -> Self {
        Self {
            cols,
            rows,
            column_size: 0.0,
            row_size: 0.0,
            placement: Default::default(),
            gap: (8, 8).into(),
        }
    }
    pub fn config(&mut self, c: i32, r: i32, size: Option<Placement<LogicalContext>>) {
        self.cols = c;
        self.rows = r;
        let placement = if let Some(s) = size {
            s
        } else {
            self.placement
        };
        self.size_to(placement);
    }
    pub fn with_gap<C: Into<Coordinates>>(mut self, c: C) -> Self {
        self.gap = c.into();
        self
    }
    pub fn size_to(&mut self, placement: Placement<LogicalContext>) {
        self.placement = placement;
        self.column_size = placement.section.width() / self.cols as CoordinateUnit;
        self.row_size = placement.section.height() / self.rows as CoordinateUnit;
    }
    pub fn sized(mut self, placement: Placement<LogicalContext>) -> Self {
        self.size_to(placement);
        self
    }
    pub fn place(
        &self,
        grid_placement: &GridPlacement,
        layout: Layout,
    ) -> Placement<LogicalContext> {
        let horizontal = grid_placement.horizontal(layout);
        let vertical = grid_placement.vertical(layout);
        let x = if let Some(px) = horizontal.start.px {
            self.placement.section.x() + px
        } else {
            horizontal.start.col.unwrap() as CoordinateUnit * self.column_size - self.column_size
                + self.gap.horizontal()
                + grid_placement.padding.horizontal()
        };
        let y = if let Some(px) = vertical.start.px {
            self.placement.section.y() + px
        } else {
            vertical.start.row.unwrap() as CoordinateUnit * self.row_size - self.row_size
                + self.gap.vertical()
                + grid_placement.padding.vertical()
        };
        let w = if let Some(px) = horizontal.end.px {
            px
        } else {
            horizontal.end.col.unwrap() as CoordinateUnit * self.column_size
                - self.gap.horizontal()
                - grid_placement.padding.horizontal()
                - x
        };
        let h = if let Some(px) = vertical.end.px {
            px
        } else {
            vertical.end.row.unwrap() as CoordinateUnit * self.row_size
                - self.gap.vertical()
                - grid_placement.padding.vertical()
                - y
        };
        let placed = Placement::new(
            (
                (
                    self.placement.section.x() + x,
                    self.placement.section.y() + y,
                ),
                (w, h),
            ),
            self.placement.layer + grid_placement.layer_offset,
        );
        println!("placing {:?}", placed);
        placed
    }
}
#[derive(Clone, Component)]
pub struct GridPlacement {
    horizontal: GridRange,
    horizontal_exceptions: HashMap<Layout, GridRange>,
    vertical: GridRange,
    vertical_exceptions: HashMap<Layout, GridRange>,
    layer_offset: Layer,
    padding: Coordinates,
}
impl GridPlacement {
    pub fn new(horizontal: GridRange, vertical: GridRange) -> Self {
        Self {
            horizontal,
            horizontal_exceptions: Default::default(),
            vertical,
            vertical_exceptions: Default::default(),
            layer_offset: Default::default(),
            padding: Default::default(),
        }
    }
    pub fn padded<C: Into<Coordinates>>(mut self, c: C) -> Self {
        self.padding = c.into();
        self
    }
    pub fn offset_layer<L: Into<Layer>>(mut self, l: L) -> Self {
        self.layer_offset = l.into();
        self
    }
    pub fn horizontal(&self, layout: Layout) -> GridRange {
        let mut accepted = self.horizontal;
        for (l, except) in self.horizontal_exceptions.iter() {
            if LayoutFilter::from(*l).accepts(layout) {
                accepted = *except;
            }
        }
        accepted
    }
    pub fn vertical(&self, layout: Layout) -> GridRange {
        let mut accepted = self.vertical;
        for (l, except) in self.vertical_exceptions.iter() {
            if LayoutFilter::from(*l).accepts(layout) {
                accepted = *except;
            }
        }
        accepted
    }
    pub fn except(mut self, layout: Layout, horizontal: GridRange, vertical: GridRange) -> Self {
        self.horizontal_exceptions.insert(layout, horizontal);
        self.vertical_exceptions.insert(layout, vertical);
        self
    }
}
#[derive(Resource, Copy, Clone, Eq, Hash, PartialEq, Ord, PartialOrd, Debug)]
pub struct Layout(u16);
// set of layouts this will signal at
#[derive(Component, Copy, Clone)]
pub struct LayoutFilter {
    pub(crate) config: Layout,
}
impl From<Layout> for LayoutFilter {
    fn from(value: Layout) -> Self {
        Self::new(value)
    }
}
impl LayoutFilter {
    pub fn new(config: Layout) -> Self {
        Self { config }
    }
    pub fn accepts(&self, current: Layout) -> bool {
        self.config.contains(current)
    }
}
bitflags! {
    impl Layout: u16 {
        const SQUARE = 1;
        const SQUARE_EXT = 1 << 1;
        const SQUARE_MAX = 1 << 2;
        const PORTRAIT_MOBILE = 1 << 3;
        const PORTRAIT_EXT = 1 << 4;
        const LANDSCAPE_MOBILE = 1 << 5;
        const LANDSCAPE_EXT = 1 << 6;
        const TALL_DESKTOP = 1 << 7;
        const WIDE_DESKTOP = 1 << 8;
    }
}
#[cfg(test)]
#[test]
fn bitflags_test() {
    let config = Layout::LANDSCAPE_MOBILE | Layout::LANDSCAPE_EXT;
    let filter = LayoutFilter::from(config);
    let accept = filter.accepts(Layout::LANDSCAPE_MOBILE);
    assert_eq!(accept, true);
}
#[derive(Resource)]
pub struct LayoutGrid {
    pub(crate) grid: Grid,
}
impl LayoutGrid {
    pub(crate) fn new(grid: Grid) -> Self {
        Self { grid }
    }
    pub(crate) const SMALL_HORIZONTAL_THRESHOLD: f32 = 440.0;
    pub(crate) const LARGE_HORIZONTAL_THRESHOLD: f32 = 900.0;
    pub(crate) const SMALL_VERTICAL_THRESHOLD: f32 = 440.0;
    pub(crate) const LARGE_VERTICAL_THRESHOLD: f32 = 800.0;
    pub(crate) fn configuration(coordinates: Coordinates) -> (Layout, (i32, i32)) {
        let mut columns = 4;
        if coordinates.horizontal() > Self::SMALL_HORIZONTAL_THRESHOLD {
            columns = 8
        }
        if coordinates.horizontal() > Self::LARGE_HORIZONTAL_THRESHOLD {
            columns = 12;
        }
        let mut rows = 4;
        if coordinates.vertical() > Self::SMALL_VERTICAL_THRESHOLD {
            rows = 8;
        }
        if coordinates.vertical() > Self::LARGE_VERTICAL_THRESHOLD {
            rows = 12;
        }
        let orientation = if columns == 4 && rows == 8 {
            Layout::PORTRAIT_MOBILE
        } else if columns == 4 && rows == 12 {
            Layout::PORTRAIT_EXT
        } else if columns == 8 && rows == 4 {
            Layout::LANDSCAPE_MOBILE
        } else if columns == 8 && rows == 8 {
            Layout::SQUARE_EXT
        } else if columns == 8 && rows == 12 {
            Layout::TALL_DESKTOP
        } else if columns == 12 && rows == 4 {
            Layout::LANDSCAPE_EXT
        } else if columns == 12 && rows == 8 {
            Layout::WIDE_DESKTOP
        } else if columns == 12 && rows == 12 {
            Layout::SQUARE_MAX
        } else {
            Layout::SQUARE
        };
        (orientation, (columns, rows))
    }
}
pub trait GridCoordinate {
    fn px(self) -> GridIndex;
    fn col(self) -> GridIndex;
    fn row(self) -> GridIndex;
}
impl GridCoordinate for i32 {
    fn px(self) -> GridIndex {
        GridIndex::px(self as CoordinateUnit)
    }

    fn col(self) -> GridIndex {
        GridIndex::col(self)
    }

    fn row(self) -> GridIndex {
        GridIndex::row(self)
    }
}
#[derive(Copy, Clone)]
pub struct GridIndex {
    px: Option<CoordinateUnit>,
    col: Option<i32>,
    row: Option<i32>,
}
impl GridIndex {
    pub fn px(px: CoordinateUnit) -> Self {
        Self {
            px: Some(px),
            col: None,
            row: None,
        }
    }
    pub fn col(c: i32) -> Self {
        Self {
            px: None,
            col: Some(c),
            row: None,
        }
    }
    pub fn row(r: i32) -> Self {
        Self {
            px: None,
            col: None,
            row: Some(r),
        }
    }
    pub fn to<GI: Into<GridIndex>>(self, gi: GI) -> GridRange {
        // TODO sanitize row/col differences
        GridRange::new(self, gi.into())
    }
}
#[derive(Copy, Clone)]
pub struct GridRange {
    start: GridIndex,
    end: GridIndex,
}
impl GridRange {
    pub fn new(start: GridIndex, end: GridIndex) -> Self {
        Self { start, end }
    }
}
#[cfg(test)]
#[test]
fn api_test() {
    let mut grid = Grid::new(3, 4);
    grid.size_to(Placement::default());
    let grid_placement = GridPlacement::new(20.px().to(3.col()), 20.px().to(3.row()));
    let layout = Layout::LANDSCAPE_MOBILE;
    let placement = grid.place(&grid_placement, layout);
}
pub(crate) fn viewport_changes_layout(
    mut viewport_handle: ResMut<ViewportHandle>,
    mut layout_grid: ResMut<LayoutGrid>,
    mut layout: ResMut<Layout>,
) {
    if viewport_handle.updated() {
        let (l, (c, r)) = LayoutGrid::configuration(viewport_handle.section().area.coordinates);
        if &l != layout.as_ref() {
            tracing::trace!("grid-layout:{:?}", l);
            *layout = l;
        }
        let placement = Placement::new(
            (
                viewport_handle.section().position.coordinates,
                viewport_handle.section().area.coordinates,
            ),
            0,
        );
        layout_grid.grid.config(c, r, Some(placement));
    }
}
