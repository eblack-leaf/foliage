use foliage_proper::bevy_ecs::component::Component;
use foliage_proper::color::Color;

pub mod button;
pub mod circle_button;
mod icon_button;
pub mod icon_text;

use foliage_proper::bevy_ecs;
#[derive(Component, Copy, Clone, Default)]
pub struct ForegroundColor(pub Color);

#[derive(Component, Copy, Clone, Default)]
pub struct BackgroundColor(pub Color);