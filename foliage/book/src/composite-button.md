# Building Button From Scratch

Every earlier chapter exists to make this one possible. `Button` isn't special-cased
anywhere in the crate -- it's `Panel` + `Text` + an optional `Icon`, composed under one
root entity using exactly the tools covered so far: [`Sprout`](./spawning.md) for the
config API, [`tree.branch`](./spawning.md) to spawn its children,
[`react`/`react_any`/`forward`](./tree.md) to keep them in sync, and the
[`TextValue`/`IconValue`](./composites-overview.md) channels other composites use too.

## The public shape: one entity, config in, `OnClick` out

```rust
// foliage_proper/src/composite/button.rs
pub struct Button {}
pub struct ButtonStyle {
    pub foreground: Color, // content color at rest; fill color while engaged, or always when outlined
    pub background: Color, // fill color at rest when un-outlined; content color while engaged
    pub outline: Outline,
    pub rounding: Rounding,
}
pub struct Engagement(pub bool); // interaction state as a component, same reaction door as config
```

`ButtonStyle` is the whole appearance, poked at once (`tree.write_to(btn, ButtonStyle {
.. })`); `Engagement` is engaged/disengaged as a plain component rather than a special
case, so an engage/disengage restyle and a config restyle go through the *same*
reaction.

## `build()`: the static skeleton

```rust
// foliage_proper/src/composite/button.rs (abridged)
fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
    let panel = tree.branch(this, Panel::new().elevate(Elevation::up(1))
        .at(Location::new().xs(1.col().as_left().with(1.col().as_right()), 1.row().as_top().with(1.row().as_bottom())))
        .with((InteractionPropagation::pass_through(), FocusBehavior::ignore())));
    let text = tree.branch(this, Text::new("").elevate(Elevation::up(2))
        .with((HorizontalAlignment::Left, VerticalAlignment::Middle,
               InteractionPropagation::pass_through(), FocusBehavior::ignore())));
    ...
}
```

`panel` fills the button's whole box (`1.col()`/`1.row()` -- one full grid cell) one
elevation layer up; `text` sits two layers up, above the panel. Both pass interaction
through and ignore focus themselves -- only the button's own root should actually receive
clicks and focus, not its rendering children. Note this is `build()`, not `root()`: it's
the config-*independent* skeleton -- these two `tree.branch` calls run once, regardless
of what the button was configured with. The icon child is the one exception: it's
**not** spawned here at all when no icon was configured, only lazily inside the first
reaction below.

## The one reaction that drives every visual state

```rust
// foliage_proper/src/composite/button.rs (abridged)
tree.react_any::<(ButtonStyle, Engagement, IconValue), _>(this, move |trigger, styles, engagement, icon_values, has_icon, font_sizes, insets, mut tree| {
    let e = trigger.event_target();
    if icon.is_none() && has_icon.get(e).unwrap().0 {
        icon = Some(tree.branch(e, Icon::new(0).elevate(Elevation::up(2)).with((..))));
    }
    // ... compute icon_size, icon_inset ...
    restyle(&mut tree, e, panel, icon, text, *styles.get(e).unwrap(), engagement.get(e).unwrap().0, icon_size, icon_inset);
});
```

One `react_any` over `(ButtonStyle, Engagement, IconValue)` covers every input that can
change the button's appearance -- because `restyle` (a plain function, not a system)
reads all of them together and recomputes the whole look each time, there's no
combinatorial explosion of "which subset changed" cases to handle. `restyle` itself
decides: content color is foreground at rest, background when engaged; the panel's fill
inverts the same way when there's no outline, or stays foreground with just the outline
toggling when there is one. This runs once at spawn (via `react_any`'s build-time
re-fire, see [Tree and Graft](./tree.md)) and again on every later write -- there is no
separate "apply initial style" step.

Interaction events feed the same door, not a separate one:

```rust
tree.subscribe(this, |trigger: Trigger<Engaged>, mut tree: Tree| {
    tree.entity(trigger.event_target()).insert(Engagement(true));
});
```

An `Engaged` event doesn't restyle anything itself -- it just writes `Engagement(true)`,
which the `react_any` above picks up like any other config change.

## Text is a `react`, not a `forward`

```rust
// foliage_proper/src/composite/button.rs (abridged)
tree.react::<TextValue, _>(this, move |trigger, values, has_icon, font_sizes, mut tree| {
    let value = values.get(e).unwrap().0.clone();
    let width = value.len();
    let center_adjust = if has_icon.get(e).unwrap().0 { (icon_width + 8) / 2 } else { 0 };
    tree.entity(text).insert((TextValue(value), Location::new().xs(/* centered, width-dependent */)));
});
tree.forward::<FontSize>(this, text); // pure copy -- FontSize doesn't drive layout on its own
```

This is the concrete case [Tree and Graft](./tree.md) points to for why `forward` and
`react` stay separate mechanisms: `FontSize` really is a pure copy from root to `text`,
so `forward` is correct and simplest. `TextValue` is *not* a pure copy -- the text's
`Location` (centered, shifted to make room for an icon, sized to the string's length) is
computed from the value, not copied alongside it -- so it stays an explicit `react`. The
distinction is visible in the code on purpose rather than hidden behind one generic
"sync this field" mechanism.

## Using it

```rust
// foliage/examples/controls.rs
foliage.world.leaf(
    Button::new()
        .text("Button")
        .rounding(Rounding::Sm)
        .colors(Color::gray(900), Color::green(500))
        .at(Location::new().xs(
            8.px().as_left().with(160.px().as_width()),
            t.px().as_top().with(b.px().as_bottom()),
        ))
        .elevate(Elevation::up(1)),
);
```

Everything above -- the panel and text children, the icon spawned lazily if configured,
the restyle-on-any-change reaction, the width-aware text positioning -- runs
automatically the moment this spawns and again on every later `tree.write_to(button,
ButtonStyle { .. })`. From the outside, `Button` looks like any other widget in the
[Getting Started](https://github.com/eblack-leaf/foliage#getting-started) example --
because it is one. The only thing that changes composite to composite is what `build()`
spawns and what `react`/`react_any` do with it.
