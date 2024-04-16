use bevy_ecs::system::Resource;

use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
use crate::coordinate::area::Area;
use crate::text::{CharacterDimension, FontSize, TextLineStructure, TextMetrics};
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
        structure: &TextLineStructure,
        area: Area<InterfaceContext>,
        scale_factor: &ScaleFactor,
    ) -> TextMetrics {
        let (fs, fa, d) = self.best_fit(
            structure.per_line,
            area / Area::new(1.0, structure.lines as f32),
            scale_factor,
        );
        TextMetrics::new(
            fs,
            fa * Area::new(1.0, structure.lines as f32),
            d,
            structure.max_chars(),
        )
    }
    fn area_metrics(
        font_size: FontSize,
        per_line: u32,
        font: &MonospacedFont,
        scale_factor: &ScaleFactor,
    ) -> (Area<InterfaceContext>, CharacterDimension) {
        let dim = CharacterDimension(
            font.character_dimensions(font_size.px(scale_factor.factor()))
                .to_interface(scale_factor.factor()),
        );
        let width = dim.dimensions().width * per_line as f32;
        let area = (width, dim.dimensions().height).into();
        (area, dim)
    }
    fn best_fit(
        &self,
        per_line: u32,
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
            let projected = font_size.0 + 1;
            let area_metrics =
                Self::area_metrics(FontSize(projected), per_line, self, scale_factor);
            if area_metrics.0 > extent {
                break;
            }
            font_size.0 += 1;
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
#[test]
fn best_fitting() {
    let font = MonospacedFont::new(40);
    for x in 0..100 {
        let fit = font.best_fit(x, Area::new(300.0, 100.0), &ScaleFactor::new(2.0));
        println!("per_line: {:?} {:?}, {}, {}", x, fit.0, fit.1, fit.2 .0);
    }
}