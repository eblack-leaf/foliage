# A Complete Example: `Button`

The full source is `foliage_proper/src/composite/button.rs` — this walks through it in order.
It's the reference to diff your own widget against; nothing here is simplified for the book.

## The public face

```rust
#[derive(Component, Copy, Clone)]
pub struct Button {}

#[derive(Component, Copy, Clone, Default)]
pub struct ButtonStyle {
    pub foreground: Color,
    pub background: Color,
    pub outline: Outline,
    pub rounding: Rounding,
}

#[derive(Component, Copy, Clone, Default)]
pub struct Engagement(pub bool);
```

`Button {}` itself carries no data — it's a marker. The whole appearance pokes in one write:
`tree.write_to(btn, ButtonStyle { .. })`. `Engagement` is interaction state (pressed/not)
represented as an ordinary component, so engage/disengage restyling runs through the *same*
reaction door as a config change — not a separate hand-called code path.

## The builder and `root`

```rust
#[derive(Default)]
pub struct ButtonSprout {
    leaf: LeafSprout,
    icon: Option<IconId>,
    text: Option<String>,
    colors: Option<(Color, Color)>,
    rounding: Option<Rounding>,
    outline: Option<i32>,
}
impl Sprout for ButtonSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        let (foreground, background) = self.colors.unwrap_or_default();
        (
            Button {},
            ButtonStyle { foreground, background, outline: /* .. */, rounding: /* .. */ },
            TextValue(self.text.unwrap_or_default()),
            IconValue(self.icon.unwrap_or_default()),
            FontSize::default(),
            Engagement(false),
            InteractionListener::new(),
            Grid::new(1.col().gap(4), 1.row().gap(4)),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) { /* next section */ }
}
```

`ButtonSprout`'s own builder methods (`.icon(..)`, `.text(..)`, `.colors(..)`, `.rounding(..)`,
`.outline(..)`) just set these `Option` fields; `root()` folds everything into the one bundle
inserted at spawn. `TextValue`/`IconValue` are the *same* value components `Text`/`Icon`
primitives use — Button doesn't invent its own vocabulary for content it's just forwarding.

## `build`: the static skeleton

```rust
fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
    let panel = tree.branch(this, Panel::new().elevate(Elevation::up(1)).at(/* fills parent */)
        .with((InteractionPropagation::pass_through(), FocusBehavior::ignore())));
    let icon = tree.branch(this, Icon::new(0).elevate(Elevation::up(2))
        .with((InteractionPropagation::pass_through(), FocusBehavior::ignore())));
    let text = tree.branch(this, Text::new("").elevate(Elevation::up(2))
        .with((HorizontalAlignment::Left, VerticalAlignment::Middle,
               InteractionPropagation::pass_through(), FocusBehavior::ignore())));
    // .. reactions, next section
}
```

Every button, regardless of config, gets exactly one panel + one icon + one text child. None of
these spawns reference `self`'s config at all — notice `Icon::new(0)` and `Text::new("")`, both
placeholders. That's the point: **nothing here is config-dependent**, so it belongs in `build`.
Real icon/text/colors/rounding all arrive through the reactions below, which fire once
immediately with the real values, in the same command batch as these spawns.

## `build`: the reactions

```rust
// one restyle for every appearance input, engage-state included
tree.react_any::<(ButtonStyle, Engagement), _>(this, move |trigger, styles, engagement, mut tree| {
    let e = trigger.event_target();
    restyle(&mut tree, e, panel, icon, text, *styles.get(e).unwrap(), engagement.get(e).unwrap().0);
});

// event -> state bridges: interaction events funnel into the same reaction door
tree.subscribe(this, |trigger: Trigger<Engaged>, mut tree: Tree| {
    tree.entity(trigger.event_target()).insert(Engagement(true));
});
tree.subscribe(this, |trigger: Trigger<Disengaged>, mut tree: Tree| {
    tree.entity(trigger.event_target()).insert(Engagement(false));
});

// NOT a pure copy -- the text's width Location depends on the value
tree.react::<TextValue, _>(this, move |trigger, values, mut tree| { /* .. */ });

// pure copies
tree.forward::<IconValue>(this, icon);
tree.forward::<FontSize>(this, text);
```

Five reactions, zero hand-called "apply this at construction time" step — the `react_any` fires
once at registration with `ButtonStyle`'s real spawn-time value, so there's no separate initial
styling pass to keep in sync with later writes. `Engaged`/`Disengaged` (interaction events) don't
touch rendering directly; they write `Engagement`, which the same restyle reaction already
watches — one restyle function, reachable from every input that can change how a button looks.

See [Reacting to Data](./reacting.md) for what `react`/`react_any`/`forward` actually do
underneath, and [Authoring a Widget](./authoring.md) for why this split between a static `build`
and reactive `react` calls exists in the first place.
