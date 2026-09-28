# Proving it

foliage is proven by a headless suite of a little under six hundred tests in
[`foliage/src/tests/`](https://github.com/eblack-leaf/foliage/tree/main/foliage/src/tests), plus the
crate's doctests. This chapter is about what the suite is, why its evidence counts, and what it
cannot reach.

## The frame, minus the rasteriser

The suite does not simulate the engine. It runs the engine: the same `fern::run` the platform loop
calls, against a `Grove` with no surface, with the clock moved by hand.

```rust,ignore
// tests/mod.rs
fn grove() -> Grove {
    Grove::new(Area::new(400.0, 300.0))
}

/// One whole frame, with no app to run.
fn tick(grove: &mut Grove) {
    fern::run(grove, None);
}

/// One whole frame, with `app` taking its turn in it.
fn tick_with(grove: &mut Grove, app: &mut dyn Rooted) {
    fern::run(grove, Some(app));
}

/// Moves the clock forward, to be taken up by the next frame.
fn advance(grove: &mut Grove, millis: u64) {
    grove.clock.advance(Duration::from_millis(millis));
}
```

A grove with no window is still a grove: the same type, the same verbs, the same reads, built the
same way. That is not a convenience; it is what makes a test's evidence worth anything. Every law in
[The frame](frame.md) holds under the suite for the simple reason that the suite runs the frame.

Steps 1 to 8 all run. Step 9 does not, because there is no surface to draw to. Everything up to and
including extraction's batch (what would be sent to the GPU) is observable, and the renderer tests
assert on that batch directly.

## Reaching in the way the loop does

The suite reaches the engine through the same doors the loop uses, rather than through anything a
test gets that a platform does not:

- **Input** is written as the same `Input` values the loop translates window events into, pushed
  onto the same queue. Past that point a scripted press and a real one take one path. The harness
  gives them names: `press`, `drag`, `release`, `wheel`, `key`, `typing`, `controlled`,
  `past_the_hold`.
- **Time** enters through `Clock::advance`, which is the only way time enters the engine at all. The
  loop advances it by wall time (capped); the suite advances it exactly, so motion tests are
  arithmetic.
- **A resize** is the same `pending_resize` the loop's `reconfigure` sets.
- **The app** is anything implementing the crate-private `Rooted` trait. `Observer` is one that only
  keeps each frame's `Pollen`, so a test can ask what a frame reported.

What the suite never opens are the platform edges. `Grove::attach`, which connects the clipboard, the
soft keyboard and the URL handler, is called only by `photosynthesize`, so no test ever touches the
clipboard, the keyboard or the browser of whoever runs it; the edges are tested at the seam, where
what they would do is engine state.

## What a test looks like

Tests are named as the rule they hold, and most carry a comment saying why the rule exists:

```rust,ignore
// tests/interaction.rs
/// The mobile-correct default: a target holds a gesture only until it becomes a drag, and then
/// yields -- so the list scrolls and the button it began on gets nothing.
#[test]
fn a_drag_on_a_button_scrolls_the_region_and_the_button_gets_no_tap() {
    let mut grove = grove();
    let (region, _) = column(&mut grove);
    let button = grove.branch(
        region,
        Panel::new().at(at(0.0, 0.0, 200.0, 50.0)).interactive(),
    );
    tick(&mut grove);

    press(&mut grove, 50.0, 25.0);
    let pressed = frame(&mut grove);
    assert!(pressed.engaged(button));

    drag(&mut grove, 50.0, -15.0);
    release(&mut grove, 50.0, -15.0);
    let dragged = frame(&mut grove);

    assert_eq!(offset(&grove, region).y, 40.0);
    assert!(!dragged.clicked(button));
    assert!(dragged.disengaged(button));
    assert!(!dragged.drag_started(button));
}
```

Read down a test file and the list of names is a specification of that subsystem:
`the_claim_threshold_is_per_axis`, `a_drag_over_a_disabled_element_scrolls_its_region`,
`asking_for_another_frame_lasts_exactly_one_frame`, `a_paste_is_answered_in_a_later_frame`,
`a_read_only_cut_is_a_copy`.

| File | Tests | Covers |
|---|---|---|
| `frame.rs`, `root.rs`, `lifecycle.rs` | 41 | the frame laws, taking root, planting and withering |
| `placement.rs`, `rowan.rs`, `elevation.rs` | 89 | the placement algebra, resolution, the stack order |
| `text.rs`, `text_input.rs`, `text_area.rs` | 127 | the cell, wrapping, `content()`, and editing |
| `interaction.rs`, `focus.rs`, `keys.rs` | 61 | the box stack, claiming, holds, focus, keys |
| `views.rs` | 53 | extent, pinning, chaining, `ScrollTo`, momentum |
| `aspen.rs` | 58 | motion, channels, timers, sequences |
| `elm.rs`, `renderers.rs`, `palette.rs` | 78 | extraction, every renderer, the scheme |
| `sprig.rs`, `assets.rs`, `platform.rs` | 55 | off the frame, arriving bytes, the platform edges |
| `tracing.rs` | 3 | what a quiet frame reports |

## Pure parts are tested as arithmetic

Where the engine has a pure function at its core, the suite tests the function on its own, without a
frame:

- **The resolver.** `tests/placement.rs` tests the placement algebra "against the resolver alone": a
  `Config` and a `Context` in, a `Span` out. That is possible only because the resolver touches no
  world.
- **Editing.** `text_input::applied` takes a value, a caret, a key and a wrap, and says what the key
  does, so every editing rule is a one-line case in a table.
- **The scheme.** The ramps a scheme derives from its seeds are checked as color arithmetic.

## Compile-fail doctests

Some rules are held by the type system, and a type-system rule is only proven by code that fails to
compile. The placement grammar carries a set of `compile_fail` doctests, each **pinned to an error
code** rather than to the compiler's wording, which changes between releases:

```rust,ignore
/// ```compile_fail,E0277
/// use foliage::{Source, left};
/// left(0.px()).width(2.row());
/// ```
```

Each one holds a line of [the grammar's types](resolver.md#the-order-is-in-the-types): no second
extent on an axis, no vertical length in a horizontal role, no coordinate crossing axes, no adding two
positions.

## The quiet frame

`tests/tracing.rs` installs a subscriber and asserts that a frame which changes nothing, over a tree
of five hundred elements, emits nothing above `trace`, and the same for a watch at rest and a focused
field at rest. Spans are per frame and per phase and events are per thing that happened, so neither
scales with the size of the tree. That is what keeps `debug` usable on a real app: quiet at rest, and
loud only about what actually happened.

## Measuring is not testing

`tests/bench.rs` is a harness rather than a test, `#[ignore]`d unless asked for. It builds a field of
cells and runs loads that each write one kind of thing (layout, color, text, motion, churn, scroll,
resize and others) at two sizes, timing the frame on the processor with no surface.
[What a frame costs](performance.md) is built on it.

## CI

[`ci.yml`](https://github.com/eblack-leaf/foliage/blob/main/.github/workflows/ci.yml) runs on every
push and pull request to `main`:

| Job | Runs |
|---|---|
| native, on Linux, macOS and Windows | `cargo test --workspace` (the suite and every doctest), `cargo build --examples -p foliage`, `cargo check -p foliage --features origin-url`, `cargo check -p application` |
| web | `cargo check -p foliage -p application --target wasm32-unknown-unknown` |
| Android | `cargo ndk -t arm64-v8a check -p foliage -p application-android` |

`cargo check -p application` is a gate on the public API rather than on the site: `application` is
written against nothing but the public surfaces of foliage and lichen, so an API that cannot build a
real page fails CI.

## What is not proven here

The translation from platform events into `Input` (the whole of `photosynthesize.rs`'s
`window_event`), the surface, and the pixels. Those are answered for by the site running on the web,
by the examples building on three desktops, by the Android build being checked and run on an
emulator, and by the `stress` example under a real surface. None of that is a test in CI, which is
why the translation layer is kept as thin as it is: everything past it is on the one path the suite
does reach.
