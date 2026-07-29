use crate::{Coordinates, Resource};
use bevy_ecs::prelude::Query;
use bevy_ecs::system::Res;
use bevy_ecs::component::Component;
use std::sync::Arc;

/// Which registered font a [`Text`](crate::Text) draws with. Put it on a text entity --
/// composites forward it to the text they own, exactly as they do [`FontSize`](crate::FontSize).
///
/// [`DEFAULT`](Self::DEFAULT) is the bundled JetBrains Mono, used by anything that never
/// sets one. Register others with [`Foliage::font`](crate::Foliage::font).
#[derive(Component, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FontId(pub(crate) u32);
impl Default for FontId {
    fn default() -> Self {
        Self::DEFAULT
    }
}
impl FontId {
    /// The bundled JetBrains Mono.
    pub const DEFAULT: Self = Self(0);
}

/// The registry plus the two per-entity components needed to measure a character cell:
/// which font, and at what size. Bundled as one `SystemParam` because a system reading all
/// three would otherwise spend three of its parameter slots on them -- which is what pushed
/// `Location::update` over the limit.
#[derive(bevy_ecs::system::SystemParam)]
pub(crate) struct FontContext<'w, 's> {
    pub(crate) fonts: Res<'w, MonospacedFont>,
    pub(crate) ids: Query<'w, 's, &'static FontId>,
    pub(crate) sizes: Query<'w, 's, &'static crate::FontSize>,
    /// Bundled so a caller resolving a size never has to remember to consult it separately
    /// -- a `FontSize::short` that only some call sites honoured would be worse than none.
    pub(crate) short: Res<'w, crate::Short>,
}
impl FontContext<'_, '_> {
    /// The registry paired with whichever font `entity` draws in, for handing to helpers
    /// that measure against it.
    pub(crate) fn font_ref(&self, entity: bevy_ecs::entity::Entity) -> FontRef<'_> {
        FontRef {
            fonts: &self.fonts,
            id: self.ids.get(entity).copied().unwrap_or_default(),
        }
    }
    /// `entity`'s character cell at `layout`'s breakpoint, or `None` if it carries no
    /// [`FontSize`](crate::FontSize) -- i.e. is not text-metric sized at all.
    pub(crate) fn character_block(
        &self,
        entity: bevy_ecs::entity::Entity,
        layout: crate::Layout,
    ) -> Option<Coordinates> {
        Some(self.fonts.character_block(
            self.ids.get(entity).copied().unwrap_or_default(),
            self.size(entity, layout)?,
        ))
    }
    /// `entity`'s resolved font size, honouring `short`. The one place that decision is
    /// made, so no caller can forget it.
    pub(crate) fn size(&self, entity: bevy_ecs::entity::Entity, layout: crate::Layout) -> Option<u32> {
        Some(self.sizes.get(entity).ok()?.resolve(layout, *self.short))
    }
}

/// A registry paired with the one font being measured against, so a helper can take a
/// single parameter instead of threading the id alongside every `&MonospacedFont`. `Copy`,
/// so passing it down a call chain costs nothing.
#[derive(Copy, Clone)]
pub(crate) struct FontRef<'a> {
    pub(crate) fonts: &'a MonospacedFont,
    pub(crate) id: FontId,
}
impl FontRef<'_> {
    pub(crate) fn character_block(&self, font_size: u32) -> Coordinates {
        self.fonts.character_block(self.id, font_size)
    }
}

/// Every registered font, indexed by [`FontId`]. Entry 0 is always the bundled one.
///
/// Held as `Arc`s so the render pipeline can lift one out of the world and drop the
/// resource borrow before it drains the render queues, which need `&mut World`.
#[derive(Resource)]
pub(crate) struct MonospacedFont(pub(crate) Vec<Arc<fontdue::Font>>);
impl MonospacedFont {
    /// Characters spanning the narrowest and widest shapes a Latin font has -- if these
    /// agree on advance width, it is monospaced.
    const PROBE: [char; 6] = ['i', 'W', 'm', '.', '1', 'g'];

    pub(crate) fn new(opt_scale: u32) -> Self {
        Self(vec![Arc::new(Self::parse(
            include_bytes!("JetBrainsMonoNL-Medium.ttf").as_slice(),
            opt_scale,
        ))])
    }
    /// Parses `bytes` and rejects anything that is not monospaced.
    ///
    /// Checked here rather than left to render: foliage's whole text layout is built on a
    /// fixed advance -- `.letters()` sizing, the caret's column addressing, `character_block`
    /// -- so a proportional font does not degrade, it silently mispositions everything. A
    /// panic at registration names the problem where it can still be fixed.
    pub(crate) fn parse(bytes: &[u8], opt_scale: u32) -> fontdue::Font {
        let font = fontdue::Font::from_bytes(
            bytes,
            fontdue::FontSettings {
                scale: opt_scale as f32,
                ..fontdue::FontSettings::default()
            },
        )
        .expect("font");
        let px = opt_scale as f32;
        let mut reference: Option<(char, f32)> = None;
        for c in Self::PROBE {
            // a glyph index of 0 is `.notdef` -- the font simply has no such character, and
            // measuring the fallback would compare it against itself
            if font.lookup_glyph_index(c) == 0 {
                continue;
            }
            let advance = font.metrics(c, px).advance_width;
            match reference {
                None => reference = Some((c, advance)),
                Some((rc, ra)) => assert!(
                    (advance - ra).abs() <= 0.01,
                    "font is not monospaced: '{rc}' advances {ra}px but '{c}' advances \
                     {advance}px at {px}px. foliage sizes and addresses text by a fixed \
                     character cell (`.letters()`, the text caret's columns), so a \
                     proportional font mispositions rather than merely looking different."
                ),
            }
        }
        font
    }
    pub(crate) fn add(&mut self, font: fontdue::Font) -> FontId {
        self.0.push(Arc::new(font));
        FontId(self.0.len() as u32 - 1)
    }
    /// The font `id` names, falling back to the bundled one if it was never registered.
    pub(crate) fn get(&self, id: FontId) -> &Arc<fontdue::Font> {
        self.0.get(id.0 as usize).unwrap_or(&self.0[0])
    }
    pub(crate) fn character_block(&self, id: FontId, font_size: u32) -> Coordinates {
        Self::block_of(self.get(id), font_size)
    }
    /// The character cell for an already-resolved font -- what the render pipeline uses,
    /// since it holds its group's font directly rather than an id into the registry.
    pub(crate) fn block_of(font: &fontdue::Font, font_size: u32) -> Coordinates {
        let metrics = font.metrics('a', font_size as f32);
        let line_metrics = font.horizontal_line_metrics(font_size as f32);
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

    /// Registration hands back a fresh id and leaves `DEFAULT` pointing at the bundled
    /// font -- the two must not alias, or setting a font on one `Text` would move every
    /// other one.
    #[test]
    fn registering_a_font_returns_a_new_id_without_disturbing_the_default() {
        let mut foliage = Foliage::new();
        // the bundled bytes again: a second *entry*, which is what is under test here --
        // there is no second monospace font in-tree to register
        let id = foliage.font(include_bytes!("JetBrainsMonoNL-Medium.ttf").as_slice());
        assert_ne!(id, FontId::DEFAULT, "a registered font gets its own id");

        let fonts = foliage.world.resource::<MonospacedFont>();
        assert_eq!(fonts.0.len(), 2, "the registry should hold both entries");
        assert!(
            !Arc::ptr_eq(fonts.get(id), fonts.get(FontId::DEFAULT)),
            "the new entry must be its own font, not a second handle to the default"
        );
    }

    /// The guarantee `Foliage::font` advertises. A proportional font does not merely look
    /// different here -- `.letters()` sizing and the caret's columns both assume a fixed
    /// advance, so it mispositions silently. `expected` pins the message too, since a bare
    /// panic would leave the author with no idea which font or why.
    #[test]
    #[should_panic(expected = "not monospaced")]
    fn registering_a_proportional_font_is_rejected() {
        let mut foliage = Foliage::new();
        foliage.font(include_bytes!("test_fonts/DejaVuSans.ttf").as_slice());
    }

    /// An id that was never registered resolves to the bundled font rather than panicking
    /// or indexing out of bounds -- a `Text` carrying a stale id still draws.
    #[test]
    fn an_unregistered_font_id_falls_back_to_the_bundled_font() {
        let foliage = Foliage::new();
        let fonts = foliage.world.resource::<MonospacedFont>();
        assert!(
            Arc::ptr_eq(fonts.get(FontId(99)), fonts.get(FontId::DEFAULT)),
            "an out-of-range id should fall back rather than panic"
        );
    }

    #[test]
    fn character_block_reports_positive_real_font_metrics() {
        let font = MonospacedFont::new(crate::Text::OPT_SCALE);
        let block = font.character_block(FontId::DEFAULT, FontSize::DEFAULT_SIZE);
        assert!(
            block.a() > 0.0,
            "advance width should be a real size from the bundled font, not a stub"
        );
        assert!(
            block.b() > 0.0,
            "line height should be a real size from the bundled font, not a stub"
        );
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
            .character_block(FontId::DEFAULT, FontSize::DEFAULT_SIZE);
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
            .character_block(FontId::DEFAULT, FontSize::DEFAULT_SIZE);

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

    /// `N.letters().gap(g)` is a legal combination: column-pitch resolution reads
    /// `grid.columns.gap.amount` whichever value variant computed the pitch. With only one
    /// column there is nothing for the gap to sit *between*
    /// (`grid/location.rs`'s own resolution now spends `(n - 1)` gaps across `n` columns,
    /// matching ordinary CSS grid, not `(n + 1)` -- a leading/trailing margin `gap`, on its
    /// own, was never supposed to add), so the single cell still spans the field's own full
    /// width, unaffected by `GAP`.
    #[test]
    fn letters_grid_gap_does_not_panic_and_leaves_a_single_column_unaffected() {
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
            0.0,
            "a single column has no other column to gap *from* -- gap shouldn't add a \
             leading margin before it, only space between multiple columns"
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
