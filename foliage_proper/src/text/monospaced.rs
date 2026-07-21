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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EcsExtension, Elevation, Foliage, FontSize, GridExt, Leaf, Location, Logical, Section, Sprout};

    #[test]
    fn character_block_reports_positive_real_font_metrics() {
        let font = MonospacedFont::new(crate::Text::OPT_SCALE);
        let block = font.character_block(FontSize::DEFAULT_SIZE);
        assert!(block.a() > 0.0, "advance width should be a real size from the bundled font, not a stub");
        assert!(block.b() > 0.0, "line height should be a real size from the bundled font, not a stub");
    }

    /// `.letters(n)` (`grid/location.rs`'s `GridExt::letters`) is how every text-sized
    /// composite in the framework sizes itself -- Button's label, TextInput's field, list
    /// rows -- resolving through `letter_dims.a() * l as f32` off the entity's own
    /// `FontSize`. Only px/pct/col anchors had ever been exercised through real `Location`
    /// resolution; this is the same class of coverage for the one that actually depends on
    /// font metrics instead of pure arithmetic.
    #[test]
    fn letters_resolves_through_location_to_n_times_the_real_font_metrics_width() {
        let mut foliage = Foliage::new();
        let block = foliage
            .world
            .resource::<MonospacedFont>()
            .character_block(FontSize::DEFAULT_SIZE);
        let leaf = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(5.letters().as_width()),
                    0.px().as_top().with(20.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(FontSize::default()),
        );
        foliage.world.flush();

        let section = *foliage.world.get::<Section<Logical>>(leaf).unwrap();
        assert_eq!(
            section.width(),
            block.a() * 5.0,
            "5 letters should resolve to exactly 5x the font's own real advance width, not a guessed constant"
        );
    }
}
