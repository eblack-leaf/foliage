use std::collections::HashMap;

use bevy_ecs::prelude::{Component, ResMut, Resource};
use bitflags::bitflags;

use crate::anim::{Animate, Interpolations};
use crate::coordinate::elevation::Elevation;
use crate::coordinate::placement::Placement;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, Coordinates, LogicalContext};
use crate::ginkgo::viewport::ViewportHandle;

#[derive(Copy, Clone, Component)]
pub struct Grid {
    cols: i32,
    rows: i32,
    column_size: CoordinateUnit,
    row_size: CoordinateUnit,
    placement: Placement<LogicalContext>,
    gap: Coordinates,
}
impl Default for Grid {
    fn default() -> Self {
        Grid::new(1, 1)
    }
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
    pub fn unaligned_point(&self, grid_point: GridLocation) -> Option<Position<LogicalContext>> {
        if grid_point.point.is_some() {
            let mut p = Position::default();
            p.set_x(
                grid_point
                    .start
                    .px
                    .unwrap_or(grid_point.start.percent? * self.placement.section.width())
                    + grid_point
                        .start
                        .adjust
                        .unwrap_or_default()
                        .resolve(self.placement.section.width()),
            );
            p.set_y(
                grid_point
                    .end
                    .px
                    .unwrap_or(grid_point.end.percent? * self.placement.section.height())
                    + grid_point
                        .end
                        .adjust
                        .unwrap_or_default()
                        .resolve(self.placement.section.height()),
            );
            return Some(p);
        }
        None
    }
    pub fn place(
        &self,
        grid_placement: &GridPlacement,
        elevation: Elevation,
        layout: Layout,
    ) -> (Placement<LogicalContext>, Option<Section<LogicalContext>>) {
        let horizontal = grid_placement.horizontal(layout);
        let vertical = grid_placement.vertical(layout);
        let (x, y, w, h) =
            if horizontal.point.is_some() && vertical.is_area {
                let w =
                    vertical.start.px.unwrap_or(
                        vertical.start.percent.unwrap() * self.placement.section.width(),
                    ) + vertical
                        .start
                        .adjust
                        .unwrap_or_default()
                        .resolve(self.placement.section.width());
                let h = vertical
                    .end
                    .px
                    .unwrap_or(vertical.end.percent.unwrap() * self.placement.section.height())
                    + vertical
                        .end
                        .adjust
                        .unwrap_or_default()
                        .resolve(self.placement.section.height());
                let (w_diff, h_diff) = match horizontal.point.unwrap() {
                    PointAlignment::Center => (-w / 2f32, -h / 2f32),
                    PointAlignment::TopLeft => (0.0, 0.0),
                    PointAlignment::TopRight => (-w, 0.0),
                    PointAlignment::BotLeft => (0.0, -h),
                    PointAlignment::BotRight => (-w, -h),
                    PointAlignment::CenterRight => (-w, -h / 2f32),
                    PointAlignment::CenterLeft => (0.0, -h / 2f32),
                };
                let point = (
                    horizontal.start.px.unwrap_or(
                        horizontal.start.percent.unwrap() * self.placement.section.width(),
                    ) + horizontal
                        .start
                        .adjust
                        .unwrap_or_default()
                        .resolve(self.placement.section.width())
                        + w_diff,
                    horizontal.end.px.unwrap_or(
                        horizontal.end.percent.unwrap() * self.placement.section.height(),
                    ) + horizontal
                        .end
                        .adjust
                        .unwrap_or_default()
                        .resolve(self.placement.section.height())
                        + h_diff,
                );
                (point.0, point.1, w, h)
            } else {
                let mut x = if let Some(px) = horizontal.start.px {
                    px
                } else if let Some(p) = horizontal.start.percent {
                    self.placement.section.width() * p / 100f32
                } else {
                    horizontal.start.col.unwrap() as CoordinateUnit * self.column_size
                        - self.column_size
                        + self.gap.horizontal()
                        + grid_placement.padding.horizontal()
                } + horizontal
                    .start
                    .adjust
                    .unwrap_or_default()
                    .resolve(self.placement.section.width());
                let mut y = if let Some(px) = vertical.start.px {
                    px
                } else if let Some(p) = vertical.start.percent {
                    self.placement.section.height() * p / 100f32
                } else {
                    vertical.start.row.unwrap() as CoordinateUnit * self.row_size - self.row_size
                        + self.gap.vertical()
                        + grid_placement.padding.vertical()
                } + vertical
                    .start
                    .adjust
                    .unwrap_or_default()
                    .resolve(self.placement.section.height());
                let mut w = if let Some(px) = horizontal.end.px {
                    px - x
                } else if let Some(fs) = horizontal.end.fixed {
                    if let Some(p) = fs.percent {
                        p * self.placement.section.width()
                    } else {
                        fs.px.expect("fixed can only be percent/px based")
                    }
                } else if let Some(p) = horizontal.end.percent {
                    let percent = self.placement.section.width() * p / 100f32;
                    percent - x
                } else {
                    horizontal.end.col.unwrap() as CoordinateUnit * self.column_size
                        - self.gap.horizontal()
                        - grid_placement.padding.horizontal()
                        - x
                } + horizontal
                    .end
                    .adjust
                    .unwrap_or_default()
                    .resolve(self.placement.section.width());
                let mut h = if let Some(px) = vertical.end.px {
                    px - y
                } else if let Some(fs) = vertical.end.fixed {
                    if let Some(p) = fs.percent {
                        p * self.placement.section.height()
                    } else {
                        fs.px.expect("fixed can only be percent/px based")
                    }
                } else if let Some(p) = vertical.end.percent {
                    let percent = self.placement.section.height() * p / 100f32;
                    percent - y
                } else {
                    vertical.end.row.unwrap() as CoordinateUnit * self.row_size
                        - self.gap.vertical()
                        - grid_placement.padding.vertical()
                        - y
                } + vertical
                    .end
                    .adjust
                    .unwrap_or_default()
                    .resolve(self.placement.section.height());
                if let Some(f) = horizontal.fixed {
                    let f = if let Some(p) = f.percent {
                        p * self.placement.section.width()
                    } else {
                        f.px.expect("fixed can only be percent/px based")
                    };
                    let diff = (w - f) / 2f32;
                    x += diff;
                    w = f;
                }
                if let Some(f) = vertical.fixed {
                    let f = if let Some(p) = f.percent {
                        p * self.placement.section.height()
                    } else {
                        f.px.expect("fixed can only be percent/px based")
                    };
                    let diff = (h - f) / 2f32;
                    y += diff;
                    h = f;
                }
                (x, y, w, h)
            };
        let mut placed = Placement::new(
            (
                (
                    self.placement.section.x() + x,
                    self.placement.section.y() + y,
                ),
                (w, h),
            ),
            self.placement.render_layer.0 + elevation.0,
        );
        let offset = if let Some(queued) = grid_placement.queued_offset {
            grid_placement.offset + queued - placed.section
        } else {
            grid_placement.offset
        };
        placed.section += offset;
        (
            placed,
            if offset == Section::default() {
                None
            } else {
                Some(offset)
            },
        )
    }
}
#[derive(Clone, Component)]
pub struct GridPlacement {
    horizontal: GridLocation,
    horizontal_exceptions: HashMap<Layout, GridLocation>,
    vertical: GridLocation,
    vertical_exceptions: HashMap<Layout, GridLocation>,
    padding: Coordinates,
    pub(crate) queued_offset: Option<Section<LogicalContext>>,
    offset: Section<LogicalContext>,
}
impl GridPlacement {
    pub fn new<GL: Into<GridLocation>, GLI: Into<GridLocation>>(
        horizontal: GL,
        vertical: GLI,
    ) -> Self {
        Self {
            horizontal: horizontal.into(),
            horizontal_exceptions: Default::default(),
            vertical: vertical.into(),
            vertical_exceptions: Default::default(),
            padding: Default::default(),
            queued_offset: None,
            offset: Default::default(),
        }
    }
    pub fn padded<C: Into<Coordinates>>(mut self, c: C) -> Self {
        self.padding = c.into();
        self
    }
    pub fn horizontal(&self, layout: Layout) -> GridLocation {
        let mut accepted = self.horizontal;
        for (l, except) in self.horizontal_exceptions.iter() {
            if LayoutFilter::from(*l).accepts(layout) {
                accepted = *except;
            }
        }
        accepted
    }
    pub fn vertical(&self, layout: Layout) -> GridLocation {
        let mut accepted = self.vertical;
        for (l, except) in self.vertical_exceptions.iter() {
            if LayoutFilter::from(*l).accepts(layout) {
                accepted = *except;
            }
        }
        accepted
    }
    pub fn except<GL: Into<GridLocation>, GLI: Into<GridLocation>>(
        mut self,
        layout: Layout,
        horizontal: GL,
        vertical: GLI,
    ) -> Self {
        self.horizontal_exceptions.insert(layout, horizontal.into());
        self.vertical_exceptions.insert(layout, vertical.into());
        self
    }
    pub(crate) fn update_queued_offset(&mut self, o: Option<Section<LogicalContext>>) {
        self.queued_offset.take();
        if let Some(o) = o {
            self.offset = o;
        }
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
    assert!(accept);
}
#[derive(Resource)]
pub struct LayoutGrid {
    pub(crate) grid: Grid,
}
impl LayoutGrid {
    pub(crate) fn new(grid: Grid) -> Self {
        Self { grid }
    }
    pub(crate) const SMALL_HORIZONTAL_THRESHOLD: f32 = 640.0;
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
    fn percent(self) -> GridIndex;
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

    fn percent(self) -> GridIndex {
        GridIndex::percent(self as f32)
    }
}
pub trait GridPoint {
    fn center(self) -> GridLocation;
    fn area(self) -> GridLocation;
}
impl GridPoint for (GridIndex, GridIndex) {
    fn center(self) -> GridLocation {
        GridLocation::point(self.0, self.1, PointAlignment::Center)
    }
    // ...
    fn area(self) -> GridLocation {
        GridLocation::area(self.0, self.1)
    }
}
#[derive(Copy, Clone)]
pub struct GridIndex {
    px: Option<CoordinateUnit>,
    col: Option<i32>,
    row: Option<i32>,
    percent: Option<f32>,
    fixed: Option<FixedIndex>,
    adjust: Option<FixedIndex>,
}
impl GridIndex {
    pub fn px(px: CoordinateUnit) -> Self {
        Self {
            px: Some(px),
            col: None,
            row: None,
            percent: None,
            fixed: None,
            adjust: None,
        }
    }
    pub fn col(c: i32) -> Self {
        Self {
            px: None,
            col: Some(c),
            row: None,
            percent: None,
            fixed: None,
            adjust: None,
        }
    }
    pub fn row(r: i32) -> Self {
        Self {
            px: None,
            col: None,
            row: Some(r),
            percent: None,
            fixed: None,
            adjust: None,
        }
    }
    pub fn percent(p: f32) -> Self {
        Self {
            px: None,
            col: None,
            row: None,
            percent: Some(p.clamp(0.0, 100.0)),
            fixed: None,
            adjust: None,
        }
    }
    pub(crate) fn fixed(f: FixedIndex) -> Self {
        Self {
            px: None,
            col: None,
            row: None,
            percent: None,
            fixed: Some(f),
            adjust: None,
        }
    }
    pub fn adjust<FI: Into<FixedIndex>>(mut self, amt: FI) -> Self {
        self.adjust.replace(amt.into());
        self
    }
    pub fn span<FI: Into<FixedIndex>>(self, f: FI) -> GridLocation {
        GridLocation::new(self, GridIndex::fixed(f.into()))
    }
    pub fn to<GI: Into<GridIndex>>(self, gi: GI) -> GridLocation {
        // TODO sanitize row/col differences
        GridLocation::new(self, gi.into())
    }
}
#[derive(Copy, Clone)]
pub struct GridLocation {
    start: GridIndex,
    end: GridIndex,
    fixed: Option<FixedIndex>,
    point: Option<PointAlignment>,
    is_area: bool,
}
#[derive(Copy, Clone, Default)]
pub struct FixedIndex {
    px: Option<f32>,
    percent: Option<f32>,
}
impl FixedIndex {
    pub fn resolve(&self, dim: CoordinateUnit) -> CoordinateUnit {
        self.px.unwrap_or(self.percent.unwrap_or_default() * dim)
    }
    pub fn new(px: Option<f32>, percent: Option<f32>) -> Self {
        Self { px, percent }
    }
}
impl From<GridIndex> for FixedIndex {
    fn from(value: GridIndex) -> Self {
        let px = value.px;
        let percent = value.percent;
        Self::new(px, percent)
    }
}
impl From<FixedIndex> for GridIndex {
    fn from(value: FixedIndex) -> Self {
        if let Some(p) = value.px {
            Self::px(p)
        } else {
            Self::percent(value.percent.unwrap())
        }
    }
}
#[derive(Copy, Clone)]
pub enum PointAlignment {
    Center,
    TopLeft,
    TopRight,
    BotLeft,
    BotRight,
    CenterRight,
    CenterLeft,
}
impl GridLocation {
    pub fn point<FI: Into<GridIndex>, FII: Into<GridIndex>>(
        a: FI,
        b: FII,
        point_alignment: PointAlignment,
    ) -> Self {
        Self {
            start: a.into(),
            end: b.into(),
            fixed: None,
            point: Some(point_alignment),
            is_area: false,
        }
    }
    pub fn area<FI: Into<GridIndex>, FII: Into<GridIndex>>(w: FI, h: FII) -> Self {
        Self {
            start: w.into(),
            end: h.into(),
            fixed: None,
            point: None,
            is_area: true,
        }
    }
    pub fn fixed<FI: Into<FixedIndex>>(mut self, f: FI) -> Self {
        self.fixed.replace(f.into());
        self
    }
    pub fn new(start: GridIndex, end: GridIndex) -> Self {
        Self {
            start,
            end,
            fixed: None,
            point: None,
            is_area: false,
        }
    }
}
pub(crate) fn viewport_changes_layout(
    mut viewport_handle: ResMut<ViewportHandle>,
    mut layout_grid: ResMut<LayoutGrid>,
    mut layout: ResMut<Layout>,
) {
    if viewport_handle.updated() {
        let (l, (c, r)) = LayoutGrid::configuration(viewport_handle.section().area.coordinates);
        if &l != layout.as_ref() {
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
impl Animate for GridPlacement {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new()
            .with(start.offset.x(), end.offset.x())
            .with(start.offset.y(), end.offset.y())
            .with(start.offset.width(), end.offset.width())
            .with(start.offset.height(), end.offset.height())
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(x) = interpolations.read(0) {
            self.offset.set_x(x);
        }
        if let Some(y) = interpolations.read(1) {
            self.offset.set_y(y);
        }
        if let Some(w) = interpolations.read(2) {
            self.offset.set_width(w);
        }
        if let Some(h) = interpolations.read(3) {
            self.offset.set_height(h);
        }
    }
}
