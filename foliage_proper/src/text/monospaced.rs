use crate::{Coordinates, Resource};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Query;
use bevy_ecs::system::Res;
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
    pub(crate) fn size(
        &self,
        entity: bevy_ecs::entity::Entity,
        layout: crate::Layout,
    ) -> Option<u32> {
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
