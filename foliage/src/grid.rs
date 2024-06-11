use bevy_ecs::component::Component;
use bevy_ecs::system::Resource;
use bitflags::bitflags;

pub struct Grid {}
impl Grid {
    pub fn new() -> Self {
        Self {}
    }
}
#[derive(Component, Copy, Clone)]
pub struct GridPlacement {
    // 1.span(2) ...
}
pub struct Padding {}
pub struct GridTemplate {}
#[derive(Resource)]
pub struct Layout {
    grid: Grid,
}

#[derive(Resource, Copy, Clone)]
pub struct LayoutConfig(u16);

// set of layouts this will (not) signal at
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
