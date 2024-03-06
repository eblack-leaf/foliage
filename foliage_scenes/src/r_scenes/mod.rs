use foliage_proper::bevy_ecs::component::Component;
use foliage_proper::color::Color;

pub mod button;
pub mod circle_button;
pub mod circle_progress_bar;
pub mod dropdown;
pub mod icon_button;
pub mod icon_text;
pub mod progress_bar;
pub mod text_button;

use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::color::monochromatic::Monochromatic;
use foliage_proper::elm::Style;

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
pub struct Colors {
    foreground: ForegroundColor,
    background: BackgroundColor,
}
impl Colors {
    pub fn new<C: Into<Color>>(fc: C, bc: C) -> Self {
        Self {
            foreground: fc.into().into(),
            background: bc.into().into(),
        }
    }
    pub fn with_foreground<C: Into<Color>>(mut self, c: C) -> Self {
        self.foreground.0 = c.into();
        self
    }
    pub fn with_background<C: Into<Color>>(mut self, c: C) -> Self {
        self.background.0 = c.into();
        self
    }
}
#[derive(Bundle, Clone)]
pub struct Aesthetics {
    pub colors: Colors,
    pub style: Style,
}
impl Aesthetics {
    pub fn themed<M: Monochromatic>() -> Self {
        Self {
            colors: Colors::new(M::BASE, Color::DARK_GREY),
            style: Style::default(),
        }
    }
    pub fn with_foreground<C: Into<Color>>(mut self, c: C) -> Self {
        self.colors.foreground.0 = c.into();
        self
    }
    pub fn with_background<C: Into<Color>>(mut self, c: C) -> Self {
        self.colors.background.0 = c.into();
        self
    }
    pub fn with_style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
}
