# Debugging

Most of what goes wrong in a foliage app goes wrong quietly by design. An op naming something it
does not apply to is dropped rather than refused, a press eaten by decoration reaches nothing, and
a read taken in the frame that wrote a value answers with the old one. Each of those is the right
behaviour, and each has an answer somewhere to look. This chapter is where to look.

## Turning the trace on

foliage reports what it does through [`tracing`](https://docs.rs/tracing), and **installs no
subscriber of its own**. Where the output goes is the app's decision, so nothing is printed until
the app says where. On a desktop:

```rust
# use foliage::Foliage;
fn trace() {
    // `tracing-subscriber = { version = "0.3", features = ["env-filter"] }` in Cargo.toml.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn main() {
    trace();
    let mut foliage = Foliage::new();
    // ...
#   let _ = &mut foliage;
}
```

A browser discards stderr and has no system clock to stamp lines with, so on the web the site
writes to the console instead, through [`tracing-web`](https://crates.io/crates/tracing-web). Its
setup, for both, is in
[`application/src/lib.rs`](https://github.com/eblack-leaf/foliage/blob/main/application/src/lib.rs).

Then `RUST_LOG` chooses how much to see:

| Level | Carries | Turn on with |
|---|---|---|
| `info` | once per run: boot, the adapter and surface, each font, mark and picture registered | the default above |
| `debug` | one event per thing that happened: planted, branched, pruned, resized, focus moved, **an op dropped** | `RUST_LOG=foliage=debug` |
| `trace` | a span for every frame and every step of it | `RUST_LOG=foliage=trace` |
| `warn` | something an app should change: an asset missing, a clipboard write refused | always shown |
| `error` | something foliage could not do: the glyph atlas full | always shown |

A frame that changes nothing emits nothing above `trace`, however large the tree, so `debug` is
quiet on an idle app and loud only about what actually happened.

## Why did nothing happen?

An op that names something it does not apply to is **dropped**: a `text` written to a panel, a
`round` on a polygon, an `at` on a line. So is any op naming an element that has withered, which
is what makes a stale name harmless. Nothing panics and the call site cannot tell. The trace can:
every drop is one `debug` event naming the verb, the element and the reason.

```text
DEBUG frame{n=42}:drain{ops=3}: foliage::fern: op dropped verb="text" leaf=4294967290 reason="says nothing to rewrite"
```

The reasons are fixed strings, so they can be searched for:

| Reason | Means |
|---|---|
| `not live` | the element has withered, or has not been grown |
| `trunk is not live` | a `branch` under something that has withered |
| `target is not live` | an `anchor` to something that has withered |
| ``is placed by its ends; use `between` `` | `at` written to a line |
| ``is placed by a box; use `at` `` | `trace` written to anything but a line |
| `draws nothing to fill` | `color` written to a `Stem` |
| `says nothing to rewrite` | `text` written to something that is not text or a field |
| `says nothing to tint` | `tint` written to something that is not text |
| `draws nothing to round` | `round` on anything but a panel or an image |
| `has no shape to reshape` | `reshape` on anything but a polygon |
| `is not an icon`, `is not an image` | `mark`, or `depict` and `fit`, on the wrong element |
| `does not scroll` | `scroll`, or a scroll motion, on something that does not scroll |
| ``no axis to move; name one with `on` `` | a `ScrollTo::px` on a region that scrolls both ways |
| `shows an element not grown under it` | `ScrollTo::show` naming something outside the region |
| `has no such property to move` | `animate` with a motion the element cannot carry |

## Reading a frame's report

`Pollen` implements `Debug`, which prints everything it holds:

```rust
# use foliage::{Grove, Pollen};
# fn frame(grove: &mut Grove, pollen: Pollen) {
println!("{pollen:?}");
# }
```

That is a print for a frame being investigated, not an API: `Pollen` offers no way to walk what it
holds, on purpose. An app asks it about the elements it owns.

## Common surprises

**"no method named `plant`" (or `at`, or `px`).** The verbs are trait methods, and the traits have
to be in scope: `Grow` for every write, `Boxed` for `at`, `Place` for the rest of what a seed
declares, `Source` for `px`, `pct`, `col`, `row` and `letters`, and `Divide` for `columns` and
`rows`.

**A press on a label does nothing.** Something drawn over a receiving element is on top of it at
those pixels, and an element on top that did not declare `interactive` eats the press. Mark the
decoration `intangible`.

**A tap is not reported as `clicked`.** A gesture that travelled past the claim threshold became a
drag, and one held still for half a second became a hold. Neither is ever a click. `activated`
also counts `Enter` and space.

**A write did not happen.** It did, at the drain after `frame` returned. A read in the same frame
answers with the state the frame started with, by design, so the write is visible from the next
frame.

**A box sized `height(content())` came out short.** Only children that describe their own extent
count toward it; one placed against its trunk's height, a row of its trunk's grid, or an anchor's
edge is left out, and so is padding below the last child. See [Sizing a box to what it
holds](placement.md#sizing-a-box-to-what-it-holds).

**A box sized `width(content())` came out zero.** As a width, `content()` is only the element's
own text; a container does not take its width from its children. State the width in `letters` of
the same font size instead.

**`letters` reads zero.** The element states neither a font nor a size, so it has no cell. Give it a
`font_size`.

**Something the app animates by hand stops between frames.** The engine idles when nothing it can
see is running. Call `grove.again()` every frame the motion is running.

**Text jumps once, shortly after starting.** A font read from a path or a URL arrived: until then
runs were measured in the bundled face. Hide the text until `pollen.loaded(font)` if that matters.

**Characters stop drawing after a great many sizes.** Every glyph is cut once per face, size and
display density onto one texture, and nothing is ever evicted from it. When it fills, an `error`
is written once and new characters stop drawing. A monospaced app addresses a small alphabet at a
handful of sizes and is nowhere near this; animating a font size through every integer would be.

## Panics, and what they mean

foliage panics only where a program stated something that cannot be true. All but one of these
panic at the call that stated it. The anchor cycle is checked when the op is applied, in the drain,
because it takes the whole tree to see, and it names the calls involved for that reason:

| Panic | Cause |
|---|---|
| `anchor cycle: leaf … cannot anchor to leaf …` | an anchor that would close a loop; names both elements, where each was planted, and where the anchor was written |
| `font is not monospaced: …` | a proportional face handed over as bytes |
| `the font could not be parsed: …` | bytes that are not a font, handed over as bytes |
| `an icon field is WxW texels of RGBA …` | a field smaller than the side it was registered with |
| `a WxH picture is N bytes of RGBA …` | pixels smaller than the size they were given with |
| `aspect ratio must be positive …` | `aspect(0.0)` or a negative ratio |
| `a hue slot is numbered below Palette::SLOTS` | `Palette::hue(n)` with `n` of 4 or more |
| `a spectrum is …` | a spectrum numbered past `Scheme::SPECTRA`, or with fewer than two or more than eight stops |

The same bytes *read* from a path or a URL, or handed over through a `Sprig`, are reported as
`missing` instead: what a file turned out to hold is not something the program stated.

## Measuring

The engine states every step of the frame as a span, so a profiler, or a subscriber that times
spans, sees the frame broken into its phases without any instrumentation of the app's own. The
[`stress`](https://github.com/eblack-leaf/foliage/blob/main/foliage/examples/stress.rs) example
does that under load and prints a table of what each phase cost, and
[What a frame costs](../architecture/performance.md) explains how to read one.
