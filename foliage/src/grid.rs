use std::collections::HashSet;

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
        placement.section.position = self.placement.section.position
            + Position::logical((
                self.column_size * grid_placement.horizontal.begin(config)
                    + grid_placement.padding.x
                    + self.gap.x,
                self.row_size * grid_placement.vertical.begin(config)
                    + grid_placement.padding.y
                    + self.gap.y,
            ));
        placement.section.area = Area::logical((
            self.column_size * grid_placement.horizontal.span(config)
                - grid_placement.padding.x * 2f32
                - self.gap.x * 2f32,
            self.row_size * grid_placement.vertical.span(config)
                - grid_placement.padding.y * 2f32
                - self.gap.y * 2f32,
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
}
impl GridPlacement {
    pub fn new(h: GridRange, v: GridRange) -> Self {
        Self {
            horizontal: h,
            vertical: v,
            layer_offset: Default::default(),
            padding: Padding::default(),
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
    fn except(self, layout_configuration: LayoutConfiguration, i: i32) -> GridIndex;
}
impl GridCoordinate for i32 {
    fn span<I: Into<GridIndex>>(self, i: I) -> GridRange {
        GridRange {
            start: self.into(),
            span: i.into(),
        }
    }
    fn except(self, layout_configuration: LayoutConfiguration, i: i32) -> GridIndex {
        GridIndex::new(self).except(layout_configuration, i)
    }
}
impl GridCoordinate for GridIndex {
    fn span<I: Into<GridIndex>>(self, i: I) -> GridRange {
        GridRange {
            start: self,
            span: i.into(),
        }
    }

    fn except(self, layout_configuration: LayoutConfiguration, i: i32) -> GridIndex {
        self.except(layout_configuration, i)
    }
}
#[derive(Clone)]
pub struct GridRange {
    start: GridIndex,
    span: GridIndex,
}
impl GridRange {
    pub fn begin(&self, config: LayoutConfiguration) -> f32 {
        let mut index = self.start.base - 1;
        for except in self.start.exceptions.iter() {
            let filter = LayoutFilter::from(except.config);
            if filter.accepts(config) {
                index = except.index - 1;
            }
        }
        index as f32
    }
    pub fn span(&self, config: LayoutConfiguration) -> f32 {
        let mut index = self.span.base;
        for except in self.span.exceptions.iter() {
            let filter = LayoutFilter::from(except.config);
            if filter.accepts(config) {
                index = except.index;
            }
        }
        index as f32
    }
}
#[derive(Clone)]
pub struct GridIndex {
    base: i32,
    exceptions: HashSet<GridException>,
}
impl From<i32> for GridIndex {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}
impl GridIndex {
    pub fn new(base: i32) -> Self {
        Self {
            base,
            exceptions: Default::default(),
        }
    }
    pub fn except(mut self, config: LayoutConfiguration, index: i32) -> Self {
        self.exceptions.insert(GridException { config, index });
        self
    }
}
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct GridException {
    config: LayoutConfiguration,
    index: i32,
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
