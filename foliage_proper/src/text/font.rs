use crate::coordinate::area::Area;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
use crate::text::{FontSize, MaxCharacters, Text};
use crate::window::ScaleFactor;
use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct MonospacedFont(pub fontdue::Font);
impl MonospacedFont {
    pub const TEXT_HEIGHT_CORRECTION: f32 = 1.0;
    pub const MAX_CHECKED_FONT_SIZE: u32 = 500;
    pub fn character_dimensions(&self, px: CoordinateUnit) -> Area<DeviceContext> {
        (
            self.0.metrics('a', px).advance_width.ceil(),
            self.0
                .horizontal_line_metrics(px)
                .unwrap()
                .new_line_size
                .ceil(),
        )
            .into()
    }
    pub fn new(opt_scale: u32) -> Self {
        Self(
            fontdue::Font::from_bytes(
                include_bytes!("JetBrainsMono-Medium.ttf").as_slice(),
                fontdue::FontSettings {
                    scale: opt_scale as f32,
                    ..fontdue::FontSettings::default()
                },
            )
            .expect("font"),
        )
    }
    pub fn best_fit(
        &self,
        max_characters: MaxCharacters,
        extent: Area<InterfaceContext>,
        scale_factor: &ScaleFactor,
    ) -> (FontSize, Area<InterfaceContext>) {
        let mut calc_area = Area::default();
        let mut font_size = FontSize(0);
        while calc_area.height <= extent.height
            && calc_area.width <= extent.width
            && font_size.0 < Self::MAX_CHECKED_FONT_SIZE
        {
            let area_metrics = Text::area_metrics(font_size, max_characters, self, scale_factor);
            calc_area = area_metrics.0;
            font_size.0 += 1;
        }
        (font_size, calc_area)
    }
}
