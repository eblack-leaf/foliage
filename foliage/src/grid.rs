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
use crate::coordinate::{CoordinateUnit, LogicalContext};
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
        config: LayoutConfig,
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
            self.column_size * grid_placement.horizontal.end(config)
                - grid_placement.padding.x
                - self.gap.x,
            self.row_size * grid_placement.vertical.end(config)
                - grid_placement.padding.y
                - self.gap.y,
        ));
        placement.layer = self.placement.layer + grid_placement.layer_offset;
        placement
    }
}
#[derive(Clone)]
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
#[derive(Clone)]
pub struct GridRange {
    start: GridIndex,
    span: GridIndex,
}
impl GridRange {
    pub fn begin(&self, config: LayoutConfig) -> f32 {
        let mut index = self.start.base - 1;
        for except in self.start.exceptions.iter() {
            let filter = LayoutFilter::from(except.config);
            if filter.accepts(config) {
                index = except.index - 1;
            }
        }
        index as f32
    }
    pub fn end(&self, config: LayoutConfig) -> f32 {
        let mut index = self.begin(config) + self.span.base as f32;
        for except in self.span.exceptions.iter() {
            let filter = LayoutFilter::from(except.config);
            if filter.accepts(config) {
                index = self.begin(config) + except.index as f32;
            }
        }
        index
    }
}
#[derive(Clone)]
pub struct GridIndex {
    base: i32,
    exceptions: HashSet<GridException>,
}
#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct GridException {
    config: LayoutConfig,
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
    layout_config: Res<LayoutConfig>,
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
#[derive(Copy, Clone)]
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
    mut config: ResMut<LayoutConfig>,
) {
    if viewport_handle.is_changed() {
        // recalculate layout + config
    }
}
#[derive(Resource)]
pub struct Layout {
    grid: Grid,
}

#[derive(Resource, Copy, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct LayoutConfig(u16);

// set of layouts this will signal at
#[derive(Component, Copy, Clone)]
pub struct LayoutFilter {
    pub(crate) config: LayoutConfig,
}

impl From<LayoutConfig> for LayoutFilter {
    fn from(value: LayoutConfig) -> Self {
        Self::new(value)
    }
}

impl LayoutFilter {
    pub fn new(config: LayoutConfig) -> Self {
        Self { config }
    }
    pub fn accepts(&self, current: LayoutConfig) -> bool {
        !(current & self.config).is_empty()
    }
}

bitflags! {
    impl LayoutConfig: u16 {
        const BASE_MOBILE = 1;
        const PORTRAIT_MOBILE = 1 << 1;
        const LANDSCAPE_MOBILE = 1 << 2;
        const PORTRAIT_TABLET = 1 << 3;
        const LANDSCAPE_TABLET = 1 << 4;
        const PORTRAIT_DESKTOP = 1 << 5;
        const LANDSCAPE_DESKTOP = 1 << 6;
        const BASE_TABLET = 1 << 7;
        const BASE_DESKTOP = 1 << 8;
    }
}
