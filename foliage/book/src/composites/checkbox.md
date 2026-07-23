# Checkbox

`Checkbox` follows a different shape than [Button](../composite-button.md): there's no
[`Engagement`](../composite-button.md) (momentary, press-driven) here at all -- checked
state is *persistent*, so a click just flips one component and lets the same reaction
that draws initial state redraw the new one:

```rust
// foliage_proper/src/composite/checkbox.rs
pub struct Checkbox {}
pub struct CheckboxState(pub bool); // the public value channel
pub struct CheckboxStyle { pub outline: Color, pub fill: Color, pub check: Color, pub rounding: Rounding }
```

```rust
tree.on_click(this, |trigger, states: Query<&CheckboxState>, mut tree: Tree| {
    let e = trigger.event_target();
    tree.write_to(e, CheckboxState(!states.get(e).unwrap().0));
});
```

One write. Nothing about drawing happens in the click handler at all -- that's the
`react_any::<(CheckboxState, CheckboxStyle), _>` reaction's job, which runs identically
whether the state changed via this click, a programmatic `tree.write_to(cb, CheckboxState(true))`,
or the reaction's own build-time re-fire.

## State *is* the visual, via `Panel`'s own exclusivity

```rust
// foliage_proper/src/composite/checkbox.rs (the restyle reaction, abridged)
if on {
    tree.write_to(panel, (style.fill, Outline::default(), style.rounding));
} else {
    tree.write_to(panel, (style.outline, Outline::new(2), style.rounding));
}
```

`Outline::default()` is `-1` (solid fill, no stroke); a positive value is stroke-only.
Checked and unchecked aren't two different components or a conditional render path --
they're just two different values fed into [`Panel`](../panel.md)'s existing
fill-vs-outline switch. The doc comment on `Checkbox` is explicit that the optional
`.check_icon(..)` glyph is deliberately *not* load-bearing: the box's own fill/outline
state already carries checked-vs-unchecked on its own, so a caller who skips
`.check_icon(..)` gets a plain, still-legible filled/outlined box with no icon child
spawned at all -- not a defunct hidden one.

## Structural config vs. style, spawned lazily

`CheckboxConfig { check_icon: Option<IconId> }` is separate from `CheckboxStyle`
specifically so a pure color/rounding restyle never re-evaluates whether an icon needs
spawning. Its own `react::<CheckboxConfig, _>` runs once (registered *before* the style
reaction, so its `CheckboxHandle` write lands first) and spawns the icon child only if
one was configured -- the same "config reaction registered before the style reaction"
ordering [Toggle](./toggle.md) uses for its own knob-glyph.

`Checked` fires from its own `react::<CheckboxState, _>`, kept deliberately separate from
the restyle reaction -- so a `CheckboxStyle`-only write (recoloring, say) never
misannounces a state change that didn't happen.
