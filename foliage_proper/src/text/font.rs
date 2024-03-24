use bevy_ecs::system::Resource;

use crate::coordinate::area::Area;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
use crate::text::{CharacterDimension, FontSize, MaxCharacters, TextLines, TextMetrics};
use crate::window::ScaleFactor;

#[derive(Resource)]
pub struct MonospacedFont(pub fontdue::Font);
impl MonospacedFont {
    pub const LINE_HEIGHT: f32 = 1.0;
    pub const MAX_CHECKED_FONT_SIZE: u32 = 500;
    pub fn character_dimensions(&self, px: CoordinateUnit) -> Area<DeviceContext> {
        let horizontal_metrics = self.0.horizontal_line_metrics(px).unwrap();
        (
            self.0.metrics('a', px).advance_width.ceil(),
            (horizontal_metrics.ascent - horizontal_metrics.descent).ceil(),
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
    pub fn line_metrics(
        &self,
        mc: &MaxCharacters,
        lines: &TextLines,
        area: Area<InterfaceContext>,
        scale_factor: &ScaleFactor,
    ) -> TextMetrics {
        let per_line = (mc.0 as f32 / lines.0 as f32).ceil() as u32;
        let (fs, fa, d) = self.best_fit(
            per_line.into(),
            area / Area::new(1.0, lines.0 as f32),
            scale_factor,
        );
        TextMetrics::new(fs, fa * Area::new(1.0, lines.0 as f32), d, per_line)
    }
    fn area_metrics(
        font_size: FontSize,
        max_characters: MaxCharacters,
        font: &MonospacedFont,
        scale_factor: &ScaleFactor,
    ) -> (Area<InterfaceContext>, CharacterDimension) {
        let dim = CharacterDimension(
            font.character_dimensions(font_size.px(scale_factor.factor()))
                .to_interface(scale_factor.factor()),
        );
        let width = dim.dimensions().width * max_characters.0 as f32;
        let area = (width, dim.dimensions().height).into();
        (area, dim)
    }
    fn best_fit(
        &self,
        max_characters: MaxCharacters,
        extent: Area<InterfaceContext>,
        scale_factor: &ScaleFactor,
    ) -> (FontSize, Area<InterfaceContext>, CharacterDimension) {
        let mut calc_area = Area::default();
        let mut font_size = FontSize(0);
        let mut dims = CharacterDimension(Area::default());
        while calc_area.height <= extent.height
            && calc_area.width <= extent.width
            && font_size.0 < Self::MAX_CHECKED_FONT_SIZE
        {
            font_size.0 += 1;
            let area_metrics = Self::area_metrics(font_size, max_characters, self, scale_factor);
            calc_area = area_metrics.0;
            dims = area_metrics.1;
        }
        (font_size, calc_area, dims)
    }
}

#[test]
fn chars() {
    let mono = MonospacedFont::new(40);
    for px in 13..200 {
        let dims = mono.character_dimensions(px as CoordinateUnit);
        println!("dims for {:?}: {:?}", px, dims);
    }
}
