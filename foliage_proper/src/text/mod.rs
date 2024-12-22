mod glyph;
mod monospaced;
mod pipeline;

use crate::ash::{Node, Parameters, Render, Renderer};
use crate::color::Color;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::ginkgo::{Ginkgo, ScaleFactor};
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::text::glyph::{Glyph, GlyphColors, GlyphKey, Glyphs, ResolvedColors, ResolvedGlyphs};
use crate::text::monospaced::MonospacedFont;
use crate::{Attachment, DeviceContext, Foliage, Layer, Tree, Update, Write};
use crate::{ClipContext, Differential};
use crate::{ClipSection, DiffMarkers};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Res, Trigger, World};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{ParamSet, Query};
use bevy_ecs::world::DeferredWorld;
use wgpu::RenderPass;

impl Attachment for Text {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(MonospacedFont::new(40));
        foliage.define(Text::update);
        foliage.diff.add_systems(
            (Text::resolve_glyphs, Text::resolve_colors)
                .chain()
                .in_set(DiffMarkers::Prepare),
        );
        foliage.remove_queue::<Text>();
        foliage.differential::<Text, FontSize>();
        foliage.differential::<Text, Color>();
        foliage.differential::<Text, BlendedOpacity>();
        foliage.differential::<Text, Section<LogicalContext>>();
        foliage.differential::<Text, Layer>();
        foliage.differential::<Text, ClipSection>();
        foliage.differential::<Text, ResolvedGlyphs>();
        foliage.differential::<Text, ResolvedColors>();
    }
}
impl Render for Text {
    type Group = ();
    type Resources = ();

    fn renderer(ginkgo: &Ginkgo) -> Renderer<Self> {
        todo!()
    }

    fn prepare(renderer: &mut Renderer<Self>, world: &mut World, ginkgo: &Ginkgo) -> Vec<Node> {
        // read-attrs
        // queue-writes @ instance-id (instance-coordinator generated w/ reuse pool)
        // sort instance-coordinator
        // submit-nodes to ash (only changed (order, layer, clip-section) / added)
        todo!()
    }

    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, ginkgo: &Ginkgo, parameters: Parameters) {
        todo!()
    }
}
#[derive(Component, Clone, PartialEq, Default)]
#[require(Color, FontSize, UpdateCache, ClipContext)]
#[require(HorizontalAlignment, VerticalAlignment, Glyphs)]
#[require(ResolvedGlyphs, ResolvedColors, GlyphColors)]
#[require(Differential<Text, FontSize>)]
#[require(Differential<Text, Color>)]
#[require(Differential<Text, BlendedOpacity>)]
#[require(Differential<Text, Section<LogicalContext>>)]
#[require(Differential<Text, Layer>)]
#[require(Differential<Text, ClipSection>)]
#[require(Differential<Text, ResolvedGlyphs>)]
#[require(Differential<Text, ResolvedColors>)]
#[component(on_add = Text::on_add)]
#[component(on_insert = Text::on_insert)]
pub struct Text {
    pub value: String,
}
impl Text {
    pub fn new<S: AsRef<str>>(value: S) -> Self {
        Self {
            value: value.as_ref().to_string(),
        }
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .entity(this)
            .observe(Remove::token_push::<Text>);
        world
            .commands()
            .entity(this)
            .observe(Self::update_from_location);
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .trigger_targets(Update::<Text>::new(), this);
    }
    fn update_from_location(trigger: Trigger<Write<Section<LogicalContext>>>, mut tree: Tree) {
        tree.trigger_targets(Update::<Text>::new(), trigger.entity());
    }
    fn resolve_colors(
        glyph_colors: ParamSet<(Query<&GlyphColors>, Query<Entity, Changed<GlyphColors>>)>,
        colors: ParamSet<(Query<&Color>, Query<Entity, Changed<Color>>)>,
        glyphs: Query<&Glyphs>,
        mut resolved: Query<&mut ResolvedColors>,
    ) {
        // TODO gather changed entities to set resolved-colors on
        // TODO set resolved colors from exceptions (glyph-colors) + base (colors)
    }
    fn update(
        trigger: Trigger<Update<Text>>,
        mut tree: Tree,
        texts: Query<&Text>,
        font_sizes: Query<&FontSize>,
        mut glyph_query: Query<&mut Glyphs>,
        horizontal_alignment: Query<&HorizontalAlignment>,
        vertical_alignment: Query<&VerticalAlignment>,
        mut sections: Query<&mut Section<LogicalContext>>,
        mut cache: Query<&mut UpdateCache>,
        font: Res<MonospacedFont>,
        scale_factor: Res<ScaleFactor>,
    ) {
        let this = trigger.entity();
        let mut current = UpdateCache::default();
        current.font_size = FontSize::new(
            (font_sizes.get(this).unwrap().value as f32 * scale_factor.value()) as u32,
        );
        current.text = texts.get(this).unwrap().clone();
        current.section = sections.get(this).unwrap().to_device(scale_factor.value());
        current.horizontal_alignment = *horizontal_alignment.get(this).unwrap();
        current.vertical_alignment = *vertical_alignment.get(this).unwrap();
        if cache.get(this).unwrap() != &current {
            let mut glyphs = glyph_query.get_mut(this).unwrap();
            glyphs.layout.reset(&fontdue::layout::LayoutSettings {
                horizontal_align: current.horizontal_alignment.into(),
                vertical_align: current.vertical_alignment.into(),
                max_width: Some(current.section.width()),
                max_height: Some(current.section.height()),
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
            let adjusted_section = current
                .section
                .with_height(glyphs.layout.height())
                .to_logical(scale_factor.value());
            tree.entity(this).insert(current).insert(adjusted_section);
        }
    }
    fn resolve_glyphs(
        mut glyph_query: Query<(Entity, &mut Glyphs), Changed<Glyphs>>,
        mut tree: Tree,
    ) {
        for (entity, mut glyphs) in glyph_query.iter_mut() {
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
                    section: Section::device((g.x, g.y), (g.width, g.height)),
                    parent: g.parent,
                    offset: i,
                })
                .collect::<Vec<Glyph>>();
            let mut resolved = ResolvedGlyphs::default();
            let len_last = glyphs.last.len();
            for (i, g) in glyphs.last.drain(..).collect::<Vec<_>>().iter().enumerate() {
                if let Some(n) = new.get(i) {
                    if g.key != n.key {
                        resolved.added.push(n.clone());
                    }
                } else {
                    resolved.removed.push(g.clone());
                }
            }
            let len_new = new.len();
            if len_new > len_last {
                for i in len_last..len_new {
                    resolved.added.push(new[i].clone());
                }
            }
            glyphs.last = glyphs.glyphs.clone();
            glyphs.glyphs = new;
            tree.entity(entity).insert(resolved);
        }
    }
}
#[derive(Component, Clone, Copy, PartialEq)]
#[component(on_insert = FontSize::on_insert)]
pub struct FontSize {
    pub value: u32,
}
impl FontSize {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .trigger_targets(Update::<Text>::new(), this);
    }
}
impl Default for FontSize {
    fn default() -> Self {
        Self { value: 14 }
    }
}
#[derive(Component, Clone, PartialEq, Default)]
pub(crate) struct UpdateCache {
    pub(crate) font_size: FontSize,
    pub(crate) text: Text,
    pub(crate) section: Section<DeviceContext>,
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
