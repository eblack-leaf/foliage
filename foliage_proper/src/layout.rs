use crate::coordinate::Coordinates;
use crate::r_grid::Grid;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Resource;
use bitflags::bitflags;

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
pub(crate) fn viewport_changes_layout() {}
