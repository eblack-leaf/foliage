use foliage_proper::bevy_ecs::component::Component;
use foliage_proper::color::Color;

pub mod button;
pub mod circle_button;
pub mod dropdown;
pub mod icon_button;
pub mod icon_text;
pub(crate) mod progress_bar;
pub mod text_button;

use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;

#[derive(Component, Copy, Clone, Default)]
pub struct ForegroundColor(pub Color);

#[derive(Component, Copy, Clone, Default)]
pub struct BackgroundColor(pub Color);
impl From<Color> for ForegroundColor {
    fn from(value: Color) -> Self {
        ForegroundColor(value)
    }
}
impl From<Color> for BackgroundColor {
    fn from(value: Color) -> Self {
        BackgroundColor(value)
    }
}
#[derive(Bundle, Copy, Clone)]
pub struct UIColor {
    foreground: ForegroundColor,
    background: BackgroundColor,
}
impl UIColor {
    pub fn new<C: Into<Color>>(fc: C, bc: C) -> Self {
        Self {
            foreground: fc.into().into(),
            background: bc.into().into(),
        }
    }
}
