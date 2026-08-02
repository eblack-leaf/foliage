# Text

`Text` follows the same shape [Panel](./panel.md) does -- a rendering primitive with no
`#[require(Node)]` of its own, a pile of `Differential`-tracked attributes, and an
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

## `FontSize` is a `Location` input, not just a glyph input

```rust
// foliage_proper/src/text/mod.rs
fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
    let this = ctx.entity;
    if world.get::<Text>(this).is_some() {
        world.commands().trigger_targets(Resolve::<Text>::new(), this);
    }
    let layout = *world.get_resource::<Layout>().unwrap();
    if world.get::<Location>(this).is_some_and(|l| l.depends_on_own_font_size(layout)) {
        world.commands().trigger_targets(Resolve::<Location>::new(), this);
    }
}
```

`ResolvedFontSize::on_insert`, fired whenever `FontSize`'s own resolved value is
(re-)inserted. The `Resolve<Text>` half is the expected one -- new size, new glyph layout.
The second half is there because a [`.letters()`](./grid.md)-sized `Location` resolves its
numbers out of the entity's own `FontSize`, so a size change is a geometry change. Every
other resolve trigger in the engine is structural (`Location`/`Parent`/`Visibility`
written, or a parent's `Section` landing); this hook is the only one that fires for a bare
`FontSize` write, which makes it the place that dependency is honored.

Only the entity itself is triggered; its children follow from the existing cascade, in
that order. The new `Section<Logical>` fires `Resolve<Location>` for each child (see that
component's own `on_insert`), and each child's new `Section` re-fires its own
`Resolve<Text>` through `Resolved<Section<Logical>>`, re-cutting the `TextBounds` render
scissor against the child's new cell. A child resolving its column ahead of its parent
would read a box that hasn't moved yet.

`depends_on_own_font_size` keeps the trigger narrow. The hook runs for every entity
carrying a `FontSize` on every layout change, and `Letters` is the only `LocationValue`
that `calc` answers out of `letter_dims`; `Px`, `Percent`, `Column`/`Row`, `Anchor` and
`TextContent` all resolve without consulting the font size.

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
layout: a composite whose text width should shape its own box can't just forward
`TextValue` to its label unchanged, it has to `react` to it, since [`forward`](./tree.md)
is a pure copy and this needs to also recompute a size from the new string.
