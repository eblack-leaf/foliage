use crate::coordinate::area::Area;
use crate::coordinate::{CoordinateUnit, NumericalContext};
use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct MonospacedFont(pub fontdue::Font);
impl MonospacedFont {
    pub const TEXT_HEIGHT_CORRECTION: f32 = 0.85;
    pub fn character_dimensions(&self, px: CoordinateUnit) -> Area<NumericalContext> {
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
}
