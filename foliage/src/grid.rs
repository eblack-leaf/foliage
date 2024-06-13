use bevy_ecs::component::Component;
use bevy_ecs::prelude::DetectChanges;
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res, ResMut, Resource};
use bitflags::bitflags;

use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::placement::Placement;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, Coordinates, LogicalContext};
use crate::ginkgo::viewport::ViewportHandle;

#[derive(Clone)]
pub struct Grid {
    pub(crate) gap: Gap,
    placement: Placement<LogicalContext>,
    column_size: CoordinateUnit,
    row_size: CoordinateUnit,
    pub(crate) grid_template: GridTemplate,
}

impl Grid {
    pub fn new(grid_template: GridTemplate) -> Self {
        Self {
            gap: Gap { x: 8.0, y: 8.0 },
            placement: Placement::default(),
            column_size: 0.0,
            row_size: 0.0,
            grid_template,
        }
    }
    pub fn placed_at(mut self, placement: Placement<LogicalContext>) -> Self {
        self.placement = placement;
        self.column_size =
            placement.section.area.width() / self.grid_template.cols as CoordinateUnit;
        self.row_size = placement.section.area.height() / self.grid_template.rows as CoordinateUnit;
        self
    }
    pub fn with_gap(mut self, gap: Gap) -> Self {
        self.gap = gap;
        self
    }
    pub fn place(
        &self,
        grid_placement: GridPlacement,
        config: LayoutConfiguration,
    ) -> Placement<LogicalContext> {
        let mut placement = Placement::default();
        let horizontal_range = grid_placement.horizontal(config);
        let vertical_range = grid_placement.vertical(config);
        placement.section.position = self.placement.section.position
            + Position::logical((
                self.column_size * horizontal_range.begin()
                    + grid_placement.padding.x
                    + self.gap.x * grid_placement.gap_ignore,
                self.row_size * vertical_range.begin()
                    + grid_placement.padding.y
                    + self.gap.y * grid_placement.gap_ignore,
            ));
        placement.section.area = Area::logical((
            self.column_size * horizontal_range.span()
                - grid_placement.padding.x * 2f32
                - self.gap.x * 2f32 * grid_placement.gap_ignore,
            self.row_size * vertical_range.span()
                - grid_placement.padding.y * 2f32
                - self.gap.y * 2f32 * grid_placement.gap_ignore,
        ));
        placement.layer = self.placement.layer + grid_placement.layer_offset;
        placement
    }
}
#[derive(Clone, Copy)]
pub struct Gap {
    x: CoordinateUnit,
    y: CoordinateUnit,
}
#[derive(Component, Clone)]
pub struct GridPlacement {
    horizontal: GridRange,
    vertical: GridRange,
    layer_offset: Layer,
    padding: Padding,
    gap_ignore: f32,
    exceptions: Vec<GridException>,
}
impl GridPlacement {
    pub fn horizontal(&self, config: LayoutConfiguration) -> GridRange {
        let mut range = self.horizontal;
        for except in self.exceptions.iter() {
            let filter = LayoutFilter::from(except.config);
            if filter.accepts(config) {
                range = except.horizontal;
            }
        }
        range
    }
    pub fn vertical(&self, config: LayoutConfiguration) -> GridRange {
        let mut range = self.vertical;
        for except in self.exceptions.iter() {
            let filter = LayoutFilter::from(except.config);
            if filter.accepts(config) {
                range = except.vertical;
            }
        }
        range
    }
    pub fn except(
        mut self,
        config: LayoutConfiguration,
        horizontal: GridRange,
        vertical: GridRange,
    ) -> Self {
        self.exceptions.push(GridException {
            config,
            horizontal,
            vertical,
        });
        self
    }
    pub fn ignore_gap(mut self) -> Self {
        self.gap_ignore = 0.0;
        self
    }
    pub fn new(h: GridRange, v: GridRange) -> Self {
        Self {
            horizontal: h,
            vertical: v,
            layer_offset: Default::default(),
            padding: Padding::default(),
            gap_ignore: 1.0,
            exceptions: vec![],
        }
    }
    pub fn offset_layer<L: Into<Layer>>(mut self, l: L) -> Self {
        self.layer_offset = l.into();
        self
    }
    pub fn padded(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }
}
pub trait GridCoordinate {
    fn span<I: Into<GridIndex>>(self, i: I) -> GridRange;
}
impl GridCoordinate for i32 {
    fn span<I: Into<GridIndex>>(self, i: I) -> GridRange {
        GridRange::new(self, i)
    }
}
#[derive(Clone, Copy)]
pub struct GridRange {
    start: GridIndex,
    span: GridIndex,
}
impl GridRange {
    pub fn new<IA: Into<GridIndex>, IB: Into<GridIndex>>(start: IA, span: IB) -> Self {
        Self {
            start: start.into(),
            span: span.into(),
        }
    }
    pub(crate) fn begin(&self) -> CoordinateUnit {
        (self.start.base - 1) as CoordinateUnit
    }
    pub(crate) fn span(&self) -> CoordinateUnit {
        self.span.base as CoordinateUnit
    }
}
#[derive(Clone, Copy)]
pub struct GridIndex {
    base: i32,
}
impl From<i32> for GridIndex {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}
impl GridIndex {
    pub fn new(base: i32) -> Self {
        Self { base }
    }
}
#[derive(Clone, Copy)]
pub struct GridException {
    config: LayoutConfiguration,
    vertical: GridRange,
    horizontal: GridRange,
}
pub(crate) fn place_on_grid(
    mut placed: Query<
        (
            &mut Position<LogicalContext>,
            &mut Area<LogicalContext>,
            &mut Layer,
            &GridPlacement,
        ),
        Changed<GridPlacement>,
    >,
    mut layout_changed: Query<(
        &mut Position<LogicalContext>,
        &mut Area<LogicalContext>,
        &mut Layer,
        &GridPlacement,
    )>,
    layout_config: Res<LayoutConfiguration>,
    layout: Res<Layout>,
) {
    for (mut pos, mut area, mut layer, grid_placement) in placed.iter_mut() {
        let placement = layout.grid.place(grid_placement.clone(), *layout_config);
        *pos = placement.section.position;
        *area = placement.section.area;
        *layer = placement.layer;
    }
    if layout.is_changed() {
        for (mut pos, mut area, mut layer, grid_placement) in layout_changed.iter_mut() {
            let placement = layout.grid.place(grid_placement.clone(), *layout_config);
            *pos = placement.section.position;
            *area = placement.section.area;
            *layer = placement.layer;
        }
    }
}
#[derive(Copy, Clone, Default)]
pub struct Padding {
    x: CoordinateUnit,
    y: CoordinateUnit,
}
#[derive(Copy, Clone)]
pub struct GridTemplate {
    cols: i32,
    rows: i32,
}
impl GridTemplate {
    pub fn new(cols: i32, rows: i32) -> Self {
        Self { cols, rows }
    }
}
pub(crate) fn viewport_changes_layout(
    viewport_handle: Res<ViewportHandle>,
    mut layout: ResMut<Layout>,
    mut config: ResMut<LayoutConfiguration>,
) {
    if viewport_handle.is_changed() {
        let (c, t) = Layout::configuration(viewport_handle.section().area.coordinates);
        *config = c;
        let placement = Placement::new(
            Section::new(
                viewport_handle.section().position.coordinates,
                viewport_handle.section().area.coordinates,
            ),
            0,
        );
        layout.grid = Grid::new(t).placed_at(placement).with_gap(layout.grid.gap);
    }
}
#[derive(Resource)]
pub struct Layout {
    grid: Grid,
}
impl Layout {
    pub(crate) const SMALL_HORIZONTAL_THRESHOLD: f32 = 640.0;
    pub(crate) const LARGE_HORIZONTAL_THRESHOLD: f32 = 880.0;
    pub(crate) const SMALL_VERTICAL_THRESHOLD: f32 = 440.0;
    pub(crate) const LARGE_VERTICAL_THRESHOLD: f32 = 640.0;
    pub(crate) fn configuration(coordinates: Coordinates) -> (LayoutConfiguration, GridTemplate) {
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
            LayoutConfiguration::FOUR_EIGHT
        } else {
            LayoutConfiguration::FOUR_FOUR
        };
        (orientation, GridTemplate::new(columns, rows))
    }
}
#[derive(Resource, Copy, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct LayoutConfiguration(u16);
// set of layouts this will signal at
#[derive(Component, Copy, Clone)]
pub struct LayoutFilter {
    pub(crate) config: LayoutConfiguration,
}

impl From<LayoutConfiguration> for LayoutFilter {
    fn from(value: LayoutConfiguration) -> Self {
        Self::new(value)
    }
}

impl LayoutFilter {
    pub fn new(config: LayoutConfiguration) -> Self {
        Self { config }
    }
    pub fn accepts(&self, current: LayoutConfiguration) -> bool {
        !(current & self.config).is_empty()
    }
}

bitflags! {
    impl LayoutConfiguration: u16 {
        const FOUR_FOUR = 1;
        const FOUR_EIGHT = 1 << 1;
        const FOUR_TWELVE = 1 << 2;
        const EIGHT_FOUR = 1 << 3;
        const EIGHT_EIGHT = 1 << 4;
        const EIGHT_TWELVE = 1 << 5;
        const TWELVE_FOUR = 1 << 6;
        const TWELVE_EIGHT = 1 << 7;
        const TWELVE_TWELVE = 1 << 8;
    }
}
