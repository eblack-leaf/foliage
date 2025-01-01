mod glyph;
mod monospaced;
mod pipeline;

use crate::ash::Render;
use crate::color::Color;
use crate::coordinate::section::Section;
use crate::coordinate::Logical;
use crate::ginkgo::ScaleFactor;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::text::glyph::{Glyph, GlyphColor, GlyphKey, Glyphs, ResolvedColors, ResolvedGlyphs};
use crate::text::monospaced::MonospacedFont;
use crate::{
    Attachment, Foliage, Layout, Physical, ResolvedElevation, ResolvedVisibility, Tree, Update,
    Visibility, Write,
};
use crate::{ClipContext, Differential};
use crate::{ClipSection, DiffMarkers};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Res, Trigger};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{ParamSet, Query};
use bevy_ecs::world::DeferredWorld;
pub use glyph::GlyphColors;
use std::collections::HashSet;
impl Attachment for Text {
    fn attach(foliage: &mut Foliage) {
        foliage
            .world
            .insert_resource(MonospacedFont::new(Text::OPT_SCALE));
        foliage.define(Text::update);
        foliage.define(Text::responsive_font_size);
        foliage.diff.add_systems(
            (Text::resolve_glyphs, Text::resolve_colors)
                .chain()
                .in_set(DiffMarkers::Finalize),
        );
        foliage.remove_queue::<Text>();
        foliage.differential::<Text, ResolvedFontSize>();
        foliage.differential::<Text, BlendedOpacity>();
        foliage.differential::<Text, Section<Logical>>();
        foliage.differential::<Text, ResolvedElevation>();
        foliage.differential::<Text, ClipSection>();
        foliage.differential::<Text, ResolvedGlyphs>();
        foliage.differential::<Text, ResolvedColors>();
        foliage.differential::<Text, UniqueCharacters>();
        foliage.differential::<Text, TextBounds>();
    }
}
#[derive(Component, Clone, PartialEq, Default)]
#[require(Color, FontSize, ResolvedFontSize, UpdateCache, ClipContext)]
#[require(HorizontalAlignment, VerticalAlignment, Glyphs)]
#[require(ResolvedGlyphs, ResolvedColors, GlyphColors, AutoHeight)]
#[require(UniqueCharacters, Differential<Text, UniqueCharacters>)]
#[require(TextBounds, Differential<Text, TextBounds>)]
#[require(Differential<Text, ResolvedFontSize>)]
#[require(Differential<Text, BlendedOpacity>)]
#[require(Differential<Text, Section<Logical>>)]
#[require(Differential<Text, ResolvedElevation>)]
#[require(Differential<Text, ClipSection>)]
#[require(Differential<Text, ResolvedGlyphs>)]
#[require(Differential<Text, ResolvedColors>)]
#[component(on_add = Text::on_add)]
#[component(on_insert = Text::on_insert)]
pub struct Text {
    pub value: String,
}
impl Text {
    pub(crate) const OPT_SCALE: u32 = 20;
    pub fn new<S: AsRef<str>>(value: S) -> Self {
        Self {
            value: value.as_ref().to_string(),
        }
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .entity(this)
            .observe(Remove::push_remove_packet::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Visibility::push_remove_packet::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Self::update_from_section);
        world
            .commands()
            .entity(this)
            .observe(Self::clear_last_on_visibility);
    }
    fn responsive_font_size(
        trigger: Trigger<Write<Layout>>,
        mut font_sizes: Query<(&FontSize, &mut ResolvedFontSize)>,
        layout: Res<Layout>,
    ) {
        for (font_size, mut resolved_font_size) in font_sizes.iter_mut() {
            resolved_font_size.value = font_size.resolve(*layout).value;
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .trigger_targets(Update::<Text>::new(), this);
    }
    fn update_from_section(trigger: Trigger<Write<Section<Logical>>>, mut tree: Tree) {
        tree.trigger_targets(Update::<Text>::new(), trigger.entity());
    }
    fn resolve_colors(
        mut glyph_colors: ParamSet<(Query<&GlyphColors>, Query<Entity, Changed<GlyphColors>>)>,
        mut colors: ParamSet<(Query<&Color>, Query<Entity, Changed<Color>>)>,
        mut glyphs: ParamSet<(Query<&Glyphs>, Query<Entity, Changed<Glyphs>>)>,
        mut resolved: Query<&mut ResolvedColors>,
    ) {
        let mut changed = glyph_colors.p1().iter().collect::<Vec<_>>();
        changed.extend(colors.p1().iter().collect::<Vec<_>>());
        changed.extend(glyphs.p1().iter().collect::<Vec<_>>());
        for e in changed {
            let mut res = ResolvedColors::default();
            let color = *colors.p0().get(e).unwrap();
            let exceptions = glyph_colors.p0().get(e).unwrap().exceptions.clone();
            for g in glyphs.p0().get(e).unwrap().glyphs.iter() {
                let c = if let Some(gc) = exceptions.get(&g.offset) {
                    *gc
                } else {
                    color
                };
                res.colors.push(GlyphColor {
                    color: c,
                    offset: g.offset,
                });
            }
            *resolved.get_mut(e).unwrap() = res;
        }
    }
    fn update(
        trigger: Trigger<Update<Text>>,
        mut tree: Tree,
        texts: Query<&Text>,
        font_sizes: Query<&ResolvedFontSize>,
        mut glyph_query: Query<&mut Glyphs>,
        horizontal_alignment: Query<&HorizontalAlignment>,
        vertical_alignment: Query<&VerticalAlignment>,
        mut sections: Query<&mut Section<Logical>>,
        mut cache: Query<&mut UpdateCache>,
        font: Res<MonospacedFont>,
        scale_factor: Res<ScaleFactor>,
        auto_heights: Query<&AutoHeight>,
    ) {
        let this = trigger.entity();
        let mut current = UpdateCache::default();
        current.font_size = ResolvedFontSize::new(
            (font_sizes.get(this).unwrap().value as f32 * scale_factor.value()) as u32,
        );
        current.text = texts.get(this).unwrap().clone();
        current.section = sections
            .get(this)
            .unwrap()
            .to_physical(scale_factor.value());
        current.horizontal_alignment = *horizontal_alignment.get(this).unwrap();
        current.vertical_alignment = *vertical_alignment.get(this).unwrap();
        if cache.get(this).unwrap() != &current {
            let mut glyphs = glyph_query.get_mut(this).unwrap();
            glyphs.layout.reset(&fontdue::layout::LayoutSettings {
                horizontal_align: current.horizontal_alignment.into(),
                vertical_align: current.vertical_alignment.into(),
                max_width: Some(current.section.width()),
                // max_height: Some(current.section.height()),
                ..fontdue::layout::LayoutSettings::default()
            });
            glyphs.layout.append(
                &[&font.0],
                &fontdue::layout::TextStyle::new(
                    current.text.value.as_str(),
                    current.font_size.value as f32,
                    0,
                ),
            );
            tree.entity(this)
                .insert(UniqueCharacters::count(&current.text))
                .insert(current.clone());
            let auto_height = auto_heights.get(this).unwrap();
            if auto_height.0 {
                let adjusted_section = current
                    .section
                    .with_height(glyphs.layout.height())
                    .to_logical(scale_factor.value());
                current.section = adjusted_section.to_physical(scale_factor.value());
                tree.entity(this).insert(adjusted_section);
            }
            tree.entity(this).insert(TextBounds::new(current.section));
        }
    }
    fn clear_last_on_visibility(
        trigger: Trigger<Write<Visibility>>,
        mut glyphs: Query<&mut Glyphs>,
        vis: Query<&ResolvedVisibility>,
    ) {
        let value = vis.get(trigger.entity()).unwrap();
        if !value.visible() {
            glyphs.get_mut(trigger.entity()).unwrap().last.clear();
        }
    }
    fn resolve_glyphs(
        mut glyph_query: Query<(Entity, &mut Glyphs, &ResolvedVisibility), Changed<Glyphs>>,
        mut tree: Tree,
    ) {
        for (entity, mut glyphs, vis) in glyph_query.iter_mut() {
            if !vis.visible() {
                continue;
            }
            let new = glyphs
                .layout
                .glyphs()
                .iter()
                .enumerate()
                .map(|(i, g)| Glyph {
                    key: GlyphKey {
                        glyph_index: g.key.glyph_index,
                        px: g.key.px as u32,
                        font_hash: g.key.font_hash,
                    },
                    section: Section::physical((g.x, g.y), (g.width, g.height)),
                    parent: g.parent,
                    offset: i,
                })
                .collect::<Vec<Glyph>>();
            let mut resolved = ResolvedGlyphs::default();
            let len_last = glyphs.last.len();
            for (i, g) in glyphs.last.drain(..).collect::<Vec<_>>().iter().enumerate() {
                if let Some(n) = new.get(i) {
                    if g.key != n.key {
                        resolved.updated.push(n.clone());
                    }
                } else {
                    resolved.removed.push(g.clone());
                }
            }
            let len_new = new.len();
            if len_new > len_last {
                for i in len_last..len_new {
                    resolved.updated.push(new[i].clone());
                }
            }
            glyphs.last = glyphs.glyphs.clone();
            glyphs.glyphs = new;
            tree.entity(entity).insert(resolved);
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct AutoHeight(pub bool);
#[derive(Component, Copy, Clone, Default, PartialEq)]
pub(crate) struct TextBounds {
    pub(crate) bounds: Section<Physical>,
}
impl TextBounds {
    pub(crate) fn new(bounds: Section<Physical>) -> Self {
        Self { bounds }
    }
}
#[derive(Copy, Clone, Component, Default, PartialEq)]
pub(crate) struct UniqueCharacters(pub(crate) u32);
impl UniqueCharacters {
    pub(crate) fn count(text: &Text) -> Self {
        let mut set = HashSet::new();
        for ch in text.value.chars() {
            set.insert(ch);
        }
        Self(set.len() as u32)
    }
}
#[derive(Component, Clone, Copy, PartialEq)]
#[component(on_insert = ResolvedFontSize::on_insert)]
pub struct ResolvedFontSize {
    pub value: u32,
}
impl ResolvedFontSize {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .trigger_targets(Update::<Text>::new(), this);
    }
}
impl Default for ResolvedFontSize {
    fn default() -> Self {
        Self {
            value: FontSize::DEFAULT_SIZE,
        }
    }
}
#[derive(Component, Clone, Copy, PartialEq)]
#[component(on_insert = FontSize::on_insert)]
pub struct FontSize {
    pub xs: u32,
    pub sm: Option<u32>,
    pub md: Option<u32>,
    pub lg: Option<u32>,
    pub xl: Option<u32>,
}
impl FontSize {
    pub const DEFAULT_SIZE: u32 = 16;
    pub fn new(value: u32) -> Self {
        Self {
            xs: value,
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
    pub fn resolve(&self, layout: Layout) -> ResolvedFontSize {
        match layout {
            Layout::Xs => ResolvedFontSize::new(self.xs),
            Layout::Sm => {
                if let Some(sm) = self.sm {
                    ResolvedFontSize::new(sm)
                } else {
                    ResolvedFontSize::new(self.xs)
                }
            }
            Layout::Md => {
                if let Some(md) = self.md {
                    ResolvedFontSize::new(md)
                } else if let Some(sm) = self.sm {
                    ResolvedFontSize::new(sm)
                } else {
                    ResolvedFontSize::new(self.xs)
                }
            }
            Layout::Lg => {
                if let Some(lg) = self.lg {
                    ResolvedFontSize::new(lg)
                } else if let Some(md) = self.md {
                    ResolvedFontSize::new(md)
                } else if let Some(sm) = self.sm {
                    ResolvedFontSize::new(sm)
                } else {
                    ResolvedFontSize::new(self.xs)
                }
            }
            Layout::Xl => {
                if let Some(xl) = self.xl {
                    ResolvedFontSize::new(xl)
                } else if let Some(lg) = self.lg {
                    ResolvedFontSize::new(lg)
                } else if let Some(md) = self.md {
                    ResolvedFontSize::new(md)
                } else if let Some(sm) = self.sm {
                    ResolvedFontSize::new(sm)
                } else {
                    ResolvedFontSize::new(self.xs)
                }
            }
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let layout = *world.get_resource::<Layout>().unwrap();
        let comp = world.get::<FontSize>(this).unwrap();
        let resolved = comp.resolve(layout);
        world.commands().entity(this).insert(resolved);
    }
    pub fn sm(mut self, value: u32) -> Self {
        self.sm.replace(value);
        self
    }
    pub fn md(mut self, value: u32) -> Self {
        self.md.replace(value);
        self
    }
    pub fn lg(mut self, value: u32) -> Self {
        self.lg.replace(value);
        self
    }
    pub fn xl(mut self, value: u32) -> Self {
        self.xl.replace(value);
        self
    }
}
impl Default for FontSize {
    fn default() -> Self {
        Self {
            xs: FontSize::DEFAULT_SIZE,
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
}
#[derive(Component, Clone, PartialEq, Default)]
pub(crate) struct UpdateCache {
    pub(crate) font_size: ResolvedFontSize,
    pub(crate) text: Text,
    pub(crate) section: Section<Physical>,
    pub(crate) horizontal_alignment: HorizontalAlignment,
    pub(crate) vertical_alignment: VerticalAlignment,
}
#[derive(Component, Copy, Clone, Default, PartialEq)]
#[component(on_insert = HorizontalAlignment::on_insert)]
pub enum HorizontalAlignment {
    #[default]
    Left,
    Center,
    Right,
}
impl HorizontalAlignment {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Text>::new(), this);
    }
}
impl From<HorizontalAlignment> for fontdue::layout::HorizontalAlign {
    fn from(value: HorizontalAlignment) -> Self {
        match value {
            HorizontalAlignment::Left => fontdue::layout::HorizontalAlign::Left,
            HorizontalAlignment::Center => fontdue::layout::HorizontalAlign::Center,
            HorizontalAlignment::Right => fontdue::layout::HorizontalAlign::Right,
        }
    }
}
#[derive(Component, Copy, Clone, Default, PartialEq)]
#[component(on_insert = VerticalAlignment::on_insert)]
pub enum VerticalAlignment {
    #[default]
    Top,
    Middle,
    Bottom,
}
impl VerticalAlignment {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Text>::new(), this);
    }
}
impl From<VerticalAlignment> for fontdue::layout::VerticalAlign {
    fn from(value: VerticalAlignment) -> Self {
        match value {
            VerticalAlignment::Top => fontdue::layout::VerticalAlign::Top,
            VerticalAlignment::Middle => fontdue::layout::VerticalAlign::Middle,
            VerticalAlignment::Bottom => fontdue::layout::VerticalAlign::Bottom,
        }
    }
}
