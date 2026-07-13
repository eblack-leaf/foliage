# Reacting to Data

`react`, `react_any`, and `forward` are the one door in `Sprout::build` for everything that
depends on config or later writes — both *values* (a button's colors) and *structure* (a
dropdown's option rows). All three are on `EcsExtension`, called from inside `build`.

## `react`

```rust
fn react<C: Component + Clone, M>(&mut self, entity: Entity, observer: impl IntoEntityObserver<M>)
```

Registers `observer` — a plain bevy entity-observer watching `Trigger<Insert, C>`, with full
`SystemParam` freedom — on `entity`, then immediately re-inserts `entity`'s current value of `C`
so the observer fires once right away. **Initial state and every later `write_to` run through
the exact same code path** — there's no separate "apply the initial config" step to keep in
sync with the update path, because there is no separate update path.

From the real `Button` widget (`foliage_proper/src/composite/button.rs`):

```rust
// NOT a pure copy -- the text's width Location depends on the value -- so this stays an
// explicit react rather than a forward.
tree.react::<TextValue, _>(
    this,
    move |trigger: Trigger<Insert, TextValue>, values: Query<&TextValue>, mut tree: Tree| {
        let value = values.get(trigger.event_target()).unwrap().0.clone();
        let width = value.len();
        tree.entity(text).insert((
            TextValue(value),
            Location::new().xs(/* width-dependent position */),
        ));
    },
);
```

## `react_any`

```rust
fn react_any<CS: Refire, M>(&mut self, entity: Entity, observer: impl IntoEntityObserver<M>)
```

The same idea over a **tuple** of components (`Refire` is implemented for tuples of arity 1–4):
the observer watches `Trigger<Insert, (A, B)>` and fires when *either* member is written — one
registration, one body, for state that's derived from more than one input. `Button`'s appearance
depends on both its config (`ButtonStyle`) and its interaction state (`Engagement`), and either
one changing needs the same restyle:

```rust
tree.react_any::<(ButtonStyle, Engagement), _>(
    this,
    move |trigger: Trigger<Insert, (ButtonStyle, Engagement)>,
          styles: Query<&ButtonStyle>,
          engagement: Query<&Engagement>,
          mut tree: Tree| {
        let e = trigger.event_target();
        restyle(&mut tree, e, panel, icon, text, *styles.get(e).unwrap(), engagement.get(e).unwrap().0);
    },
);
```

The re-fire at registration time only needs to insert *one* present member to trigger the
observer once — reaction bodies always read the full current state of everything they depend
on, so firing on whichever component happens to exist first is sufficient.

`react`/`react_any` are two separate methods rather than one generic-over-`Component`-or-tuple
method because a blanket `Refire` impl over any `C: Component` would collide with the tuple
impls under Rust's coherence rules (a foreign crate could in principle implement `Component` for
a tuple type). The two-method split is the price of that safety.

## `forward`

```rust
fn forward<C: Component + Clone>(&mut self, source: Entity, target: Entity)
```

The pure-copy specialization: `source`'s current (and every future) value of `C` is copied
verbatim onto `target`, nothing else computed. `Button` uses it for its icon and font size,
which its child `Icon`/`Text` entities consume unchanged:

```rust
tree.forward::<IconValue>(this, icon);
tree.forward::<FontSize>(this, text);
```

The boundary between `forward` and an explicit `react` is deliberately visible: the moment a
value needs *any* extra computation on the way to a child (like `TextValue`'s width-dependent
`Location` above), it stops being a pure copy and becomes a plain `react` instead. `forward` only
ever exists for the case where "copy this component to that entity" is the entire behavior.
