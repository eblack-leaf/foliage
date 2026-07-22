# Panel

`Panel` is the filled/outlined/rounded rectangle primitive -- the thing most composites
(`Button`, `Checkbox`, `Modal`'s backdrop, ...) actually draw as their visible surface.
It's a normal rendering primitive in the sense [Leaf](./leaf.md) describes: it does
**not** `#[require(Leaf, ...)]` -- `Leaf` gets unioned in at spawn time regardless of
what's being spawned. What `Panel` *does* require is its own render-relevant state, plus
one `Differential` per attribute a change to it should be able to trigger a redraw for:

```rust
// foliage_proper/src/panel/mod.rs
#[derive(Component, Copy, Clone, Default, PartialEq)]
#[require(Rounding, Side, Color, Outline)]
#[require(Differential<Self, ResolvedElevation>)]
#[require(Differential<Self, Color>)]
#[require(Differential<Self, Panel>)]
#[require(Differential<Self, Outline>)]
#[require(Differential<Self, Section<Logical>>)]
#[require(Differential<Self, BlendedOpacity>)]
#[require(Differential<Self, ClipContext>)]
pub struct Panel {
    pub(crate) corner_i: Corner,
    pub(crate) corner_ii: Corner,
    pub(crate) corner_iii: Corner,
    pub(crate) corner_iv: Corner,
}
```

`Panel` itself just holds four `Corner`s (a signed-distance-style radius/inset per
corner) -- it doesn't store color, rounding amount, or outline width directly; those are
separate components (`Color`, `Rounding`, `Side`, `Outline`) it `#[require]`s, each
tracked by its own `Differential`.

## Registering with the backend: `Attachment`

```rust
// foliage_proper/src/panel/mod.rs
impl Attachment for Panel {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Panel::update);
        foliage.define(Outline::update_anim);
        foliage.remove_queue::<Self>();
        foliage.differential::<Self, Section<Logical>>();
        foliage.differential::<Self, BlendedOpacity>();
        foliage.differential::<Self, Panel>();
        foliage.differential::<Self, Color>();
        foliage.differential::<Self, Outline>();
        foliage.differential::<Self, ResolvedElevation>();
        foliage.differential::<Self, ClipContext>();
        foliage.enable_animation::<Outline>();
    }
}
```

This is called once from [`Foliage::new()`](./app.md), and it's the only place `Panel`'s
[differential](./differential.md) registrations happen -- nothing in `foliage.rs` itself
knows `Panel` has seven trackable attributes.

## `Corner` geometry: computed, not authored

Authors set `Rounding`/`Side`/`Outline`; `Panel::update` (an observer on
`Update<Panel>`, triggered whenever `Section`, `Rounding`, `Side`, or `Outline` changes)
computes the actual per-corner geometry the shader consumes:

```rust
// foliage_proper/src/panel/mod.rs (abridged)
let depth = match rounding {
    Rounding::None => 0.0,
    Rounding::Xs => 0.1 * min,
    Rounding::Sm => 0.3 * min,
    ...
    Rounding::Full => 1.0 * min,
};
```

`Side` composes independently of `Rounding`'s amount -- a segmented control's middle
segment wants `Side::none()` (square all around) while its first segment wants
`Side::left()`, sharing one `Rounding` value between them. `Rounding::Full` also drives
`InteractionShape` automatically (`Rounding::on_insert`): a fully-rounded panel becomes a
circular hit-test region, not a rectangular one clipped to look round.

## `PanelSprout`: the author-facing builder

```rust
// foliage_proper/src/panel/mod.rs
pub struct PanelSprout {
    leaf: crate::LeafSprout,
    color: Option<Color>,
    rounding: Option<Rounding>,
    side: Option<Side>,
    outline: Option<i32>,
}
impl crate::Sprout for PanelSprout {
    fn root(self) -> impl Bundle {
        (Panel::new_marker(), self.color.unwrap_or_default(), self.rounding.unwrap_or_default(),
         self.side.unwrap_or_default(), self.outline.map(Outline::new).unwrap_or_default())
    }
}
```

`Panel::new()` returns this; `.color(..)`/`.rounding(..)`/`.side(..)`/`.outline(..)` set
its optional fields, and `root()` folds them into one bundle at spawn time -- the same
`Sprout` shape every primitive and composite in the crate follows (see
[Spawning](./spawning.md)). `build()` is left at its default empty implementation:
`Panel` is a primitive, it has no children.
