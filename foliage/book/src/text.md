# Text

`Text` follows the same shape [Panel](./panel.md) does -- a rendering primitive with no
`#[require(Leaf)]` of its own, a pile of `Differential`-tracked attributes, and an
`Attachment` that registers them -- but it's noticeably larger, because rendering text
well involves more moving parts than filling a rounded rectangle: glyph layout, a
monospaced-font metric table, per-character color runs, and content-driven sizing.

```rust
// foliage_proper/src/text/mod.rs
#[derive(Component, Clone, PartialEq, Default, Debug)]
#[require(Color, FontSize, ResolvedFontSize, UpdateCache)]
#[require(HorizontalAlignment, VerticalAlignment, Glyphs)]
#[require(ResolvedGlyphs, ResolvedColors, GlyphColors, TextContentHeight, TextContentWidth)]
#[require(UniqueCharacters, Differential<Text, UniqueCharacters>)]
#[require(Differential<Text, ResolvedFontSize>)]
#[require(Differential<Text, BlendedOpacity>)]
#[require(Differential<Text, Section<Logical>>)]
#[require(Differential<Text, ResolvedElevation>)]
#[require(Differential<Text, ClipContext>)]
#[require(Differential<Text, ResolvedGlyphs>)]
#[require(Differential<Text, ResolvedColors>)]
#[require(TextBounds, Differential<Text, TextBounds>)]
pub struct Text {
    pub value: String,
}
```

`value` is the only field on `Text` itself; everything else that determines how it looks
-- font size, alignment, resolved glyph positions, resolved per-glyph colors, content
bounds -- is a separate required component, each with its own `Differential` if it's
something the renderer needs to know about when it changes.

## Registering: more systems than `Panel`, same idea

```rust
// foliage_proper/src/text/mod.rs
impl Attachment for Text {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(MonospacedFont::new(Text::OPT_SCALE));
        foliage.define(Text::update);
        foliage.define(Text::apply_text_value);
        foliage.define(Text::responsive_font_size);
        foliage.diff.add_systems(
            (Text::resolve_glyphs, Text::resolve_colors).chain().in_set(DiffMarkers::Finalize),
        );
        foliage.remove_queue::<Text>();
        foliage.differential::<Text, ResolvedFontSize>();
        foliage.differential::<Text, BlendedOpacity>();
        // ...
        foliage.differential::<Text, ResolvedGlyphs>();
        foliage.differential::<Text, ResolvedColors>();
        foliage.differential::<Text, UniqueCharacters>();
        foliage.differential::<Text, TextBounds>();
    }
}
```

Two things worth noticing against [The App](./app.md)'s `DiffMarkers::{Prepare,
Finalize, Extract}` ordering: `resolve_glyphs`/`resolve_colors` run in `Finalize`,
*before* the `Extract` phase where every `cached_differential` system runs -- glyph
layout has to be settled before anything downstream diffs it. And `ResolvedGlyphs`
doesn't use the generic `foliage.differential::<Text, ResolvedGlyphs>()` helper at all --
it gets its own dedicated queuing system (`glyph::glyph_differential`), because glyph
data has update semantics the generic `PartialEq`-cache comparison doesn't fit (see that
function's own doc comment in `text/glyph.rs` for the specific reason).

## `TextSprout`

Same builder shape as every other primitive:

```rust
// foliage_proper/src/text/mod.rs
pub struct TextSprout {
    leaf: crate::LeafSprout,
    value: String,
    size: Option<FontSize>,
    color: Option<Color>,
}
```

`Text::new("...")` seeds `value`; `.size(..)`/`.color(..)` are the optional builder
methods. Like `Panel`, `Text` has no `build()` of its own -- it's a primitive, not a
composite -- but unlike most other primitives, `Text`'s *content* itself can drive
layout (a `Button`'s text width depends on the string it holds, which is exactly why
[Button's own reaction](./composite-button.md) to `TextValue` stays an explicit `react`
rather than a `forward`).
