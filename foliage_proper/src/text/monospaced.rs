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
    use crate::{
        EcsExtension, Elevation, Foliage, FontSize, Grid, GridExt, Leaf, Location, Logical,
        Section, Sprout,
    };

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

    /// A second, distinct `.letters()` path: `Grid::new(N.letters(), ...)` on a *parent*,
    /// resolving a *child's* `.col()` through `stem_letters` (off the parent's own
    /// `FontSize`) rather than the entity's own. This is the exact mechanism
    /// `TextInput`'s field/cursor grid depends on -- and, until now, the only place in the
    /// whole crate this combination was ever exercised at all. `TextInput`'s own test
    /// suite never asserts a resolved pixel `Section` for its cursor/field, only the
    /// logical `Cursor { column, row }` state, so a zeroed-out `stem_letters` here could
    /// have passed every existing test silently.
    #[test]
    fn col_against_a_letters_grid_resolves_to_n_times_the_stem_s_real_font_metrics_width() {
        let mut foliage = Foliage::new();
        let block = foliage
            .world
            .resource::<MonospacedFont>()
            .character_block(FontSize::DEFAULT_SIZE);

        let field = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(40.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((Grid::new(1.letters(), 1.letters()), FontSize::default())),
        );
        let child = foliage.world.branch(
            field,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(5.col().as_width()),
                    0.px().as_top().with(20.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        foliage.world.flush();

        let section = *foliage.world.get::<Section<Logical>>(child).unwrap();
        assert_eq!(
            section.width(),
            block.a() * 5.0,
            "5 columns against a 1-letter-pitch grid should resolve to exactly 5x the \
             stem's real font advance width, not zero or a guessed constant"
        );
    }

    /// `LocationValue::gap` used to `debug_assert!` against `Letters`, even though
    /// `calc()`'s column-pitch resolution reads `grid.columns.gap.amount` unconditionally
    /// regardless of which value variant computed the pitch -- the assert disallowed a
    /// combination the resolver already supported. This both proves `N.letters().gap(g)`
    /// no longer panics and that the gap is actually honored: a column's left edge is
    /// `(index) * gap` further right than with no gap at all (see the `(Left, Width)`
    /// resolution math in `location.rs::calc`).
    #[test]
    fn letters_grid_gap_does_not_panic_and_shifts_column_position() {
        let mut foliage = Foliage::new();
        const GAP: i32 = 10;

        let field = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(40.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((
                    Grid::new(1.letters().gap(GAP), 1.letters()),
                    FontSize::default(),
                )),
        );
        let child = foliage.world.branch(
            field,
            Leaf::sprout()
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    0.px().as_top().with(20.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        foliage.world.flush();

        let section = *foliage.world.get::<Section<Logical>>(child).unwrap();
        assert_eq!(
            section.left(),
            GAP as f32,
            "the first cell's left edge should sit exactly one gap in from the field's own \
             left edge, not at zero (no gap applied) or panicked before ever resolving"
        );
    }

    /// Two SIBLING cells, same shape as `application`'s type-in effect (each character
    /// its own entity, `N.col().as_left().with(N.col().as_right())` for cell N) --
    /// checking that consecutive cells (1 and 2) actually sit side by side with no
    /// overlap, and that the gap between them is exactly the configured amount. Both
    /// single-cell tests above only ever spawned ONE child; this is the first test that
    /// resolves two at once against the same letters-grid parent.
    #[test]
    fn consecutive_letters_grid_cells_sit_flush_with_exactly_one_gap_between() {
        let mut foliage = Foliage::new();
        const GAP: i32 = 6;

        let field = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(400.px().as_width()),
                    0.px().as_top().with(40.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with((
                    Grid::new(1.letters().gap(GAP), 1.letters()),
                    FontSize::default(),
                )),
        );
        let cell = |n: i32| {
            Leaf::sprout()
                .at(Location::new().xs(
                    n.col().as_left().with(n.col().as_right()),
                    0.px().as_top().with(20.px().as_height()),
                ))
                .elevate(Elevation::up(1))
        };
        let first = foliage.world.branch(field, cell(1));
        let second = foliage.world.branch(field, cell(2));
        foliage.world.flush();

        let first_section = *foliage.world.get::<Section<Logical>>(first).unwrap();
        let second_section = *foliage.world.get::<Section<Logical>>(second).unwrap();

        assert!(
            second_section.left() >= first_section.right(),
            "cell 2 (left={}) should start at or after cell 1's right edge ({}) -- \
             adjacent letter cells must not overlap",
            second_section.left(),
            first_section.right()
        );
        assert_eq!(
            second_section.left() - first_section.right(),
            GAP as f32,
            "the gap between two adjacent cells should be exactly the configured GAP"
        );
    }
}
