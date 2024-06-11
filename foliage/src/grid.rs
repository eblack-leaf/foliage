use crate::signal::LayoutConfig;
use bevy_ecs::system::Resource;

pub struct Grid {}
pub struct GridPlacement {
    // 1.span(2) ...
}
pub struct Padding {}
pub struct GridTemplate {}
#[derive(Resource)]
pub struct Layout {
    horizontal: i32,
    vertical: i32,
}
