use crate::{Coordinates, Resource};

#[derive(Resource)]
pub(crate) struct MonospacedFont(pub(crate) fontdue::Font);
impl MonospacedFont {
    pub(crate) fn new(opt_scale: u32) -> Self {
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
    pub(crate) fn character_block(&self, font_size: u32) -> Coordinates {
        let metrics = self.0.metrics('a', font_size as f32);
        let line_metrics = self.0.horizontal_line_metrics(font_size as f32);
        Coordinates::new(
            metrics.advance_width.ceil(),
            line_metrics.unwrap().new_line_size.ceil(),
        )
    }
}
#[test]
fn block() {
    use crate::FontSize;
    let mut font = MonospacedFont::new(20);
    println!("block: {}", font.character_block(FontSize::DEFAULT_SIZE));
}
