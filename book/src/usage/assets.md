# Assets

A font, an icon and a picture are all bytes. Each is registered by handing over the bytes, or an
[`Origin`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Origin.html) to read them from,
and **the name comes back at once either way**. An element can be grown against the name in the
same frame, and draws the asset on whichever frame it arrives. That is what keeps "is it loaded
yet" out of an app's state.

## Registering

| Registers | At boot | At any frame | Hands back |
|---|---|---|---|
| a font | `Foliage::font` | `Grove::font` | `Font` |
| a mark (an icon's field) | `Foliage::icon` | `Grove::icon` | `Field` |
| a set of marks | `Foliage::marks` | `Grove::marks` | the app's own set |
| an encoded picture | `Foliage::image` | `Grove::image` | `Plate` |
| pixels the app made | `Foliage::pixels` | `Grow::pixels` | `Plate` |

Each takes `impl Into<Bytes>`, which is satisfied by bytes the app holds (a `Vec<u8>`, a slice, or
what `include_bytes!` produces) and by an `Origin`:

```rust
# use foliage::{Grove, Origin};
# fn f(grove: &mut Grove, bytes: &'static [u8]) {
// `bytes` is what `include_bytes!("assets/avatar.png")` produced.
let bundled = grove.image(bytes);
# #[cfg(not(target_family = "wasm"))]
let read = grove.image(Origin::path("assets/avatar.png"));
let fetched = grove.image(Origin::url("https://example.com/avatar.jpg"));
# let _ = (bundled, read, fetched);
# }
```

A registration from held bytes is done in the call. One from an `Origin` is read off the frame (on
a thread for a path, by the browser for a URL on the web) and lands as an op, in whichever frame
it finishes, ordered against everything else that frame.

- `Origin::path` exists wherever there is a filesystem. Naming one on the web is a compile error.
- `Origin::url` is fetched by the browser on the web. Everywhere else it needs the `origin-url`
  feature, which is the only thing in foliage that brings an http client and a TLS stack; without
  it a URL is accepted and reported `missing`, so an app that bundles its assets compiles neither.

## Arriving

An element drawing a mark or a picture that has not arrived yet **occupies its box and draws
nothing**, and appears on the frame the bytes do. There is nothing to undo when they land.

Two reports say what happened, for anything that wants to know:

```rust
# use foliage::{Grove, Grow, Leaf, Plate, Pollen};
# fn f(grove: &mut Grove, pollen: &Pollen, avatar: Plate, placeholder: Leaf) {
if pollen.loaded(avatar) {
    grove.visible(placeholder, false);
}
if pollen.missing(avatar) {
    // The name stays valid and unfilled; a second attempt is a second call.
}
# }
```

`loaded` and `missing` take a `Font`, a `Field` or a `Plate` alike. `loaded` is reported for bytes
that *arrived*: read from an `Origin`, or handed over through a [`Sprig`](off-frame.md). Bytes
handed to the grove directly are registered in the call itself, and there is nothing to wait for.
`missing` covers every way the bytes did not become an asset: a path that is not there, a fetch that answered with an error, a
picture in a format foliage does not decode, a font that turned out to be proportional. There is
one thing an app can do about any of them, so they are one report, and the reason is written to
the trace.

Bytes given outright that are wrong are a different matter. A proportional font, or an icon field
or [pixels](#pixels-an-app-makes) too small for the size they were said to be, stated in the
program's own source, panics at the call: that is a mistake in the program and worth stopping for
where it can be fixed. The same bytes read from a path or a URL are reported `missing` instead,
because what a file turned out to hold is not something the program stated, and so are bytes handed
over by a worker ([Off the frame](off-frame.md#pictures-and-fonts-from-a-worker)).

**A font is the one worth waiting for.** A run composed in a face that has not arrived is measured
in the bundled face, so a page laid out in `letters` is laid out sensibly from the first frame. When
the real face lands, every run is measured again and the page reflows on that frame. An app that
would rather not show the reflow keeps the text hidden until `loaded(font)`.

## Pictures

```rust
# use foliage::{Boxed, Fit, Image, Location, Plate, Rounding, Source, left, top};
# fn f(avatar: Plate) {
Image::new(avatar)
    .fit(Fit::Crop)
    .rounding(Rounding::Full)
    .at(Location::new().xs(left(16.px()).width(64.px()), top(16.px()).height(64.px())));
# }
```

PNG and JPEG are decoded, at whatever size the file says. How the picture's own proportions meet
its box is its [`Fit`](https://eblack-leaf.github.io/foliage/api/foliage/enum.Fit.html):

| `Fit` | Does |
|---|---|
| `Aspect` (default) | scaled to sit inside the box, centred, keeping its ratio; every pixel of the picture shows |
| `Crop` | scaled to fill the box, centred, keeping its ratio; the overflowing axis is cropped |
| `Stretch` | stretched to the box exactly, distorting where the ratios disagree |

Corners round exactly as a panel's do, through the same distance field, so a picture set flush in a
rounded card sits flush. An image has no fill: it carries its own color, and its opacity (and that
of everything above it) still applies.

`grove.depict(image, plate)` swaps the picture an image draws, keeping its fit and corners, and
`grove.fit(image, fit)` changes the fit.

### Pixels an app makes

`grove.pixels(rgba, size)` registers pixels the app computed: RGBA, one byte per channel,
row-major, `size` texels across.

```rust
# use foliage::{Area, Boxed, Fit, Grove, Grow, Image, Leaf, Location, Plate, Pollen, Source, left, top};
/// A one-pixel-tall strip of the scheme's first spectrum.
fn strip(grove: &Grove) -> (Vec<u8>, Area) {
    let stops = grove.scheme().stops(0);
    let width = 256;
    let mut rgba = Vec::with_capacity(width * 4);
    for x in 0..width {
        let at = x as f32 / (width - 1) as f32;
        let color = stops[0].toward(stops[stops.len() - 1], at);
        let channels = [color.red, color.green, color.blue, color.alpha];
        rgba.extend(channels.map(|channel| (channel * 255.0).round() as u8));
    }
    (rgba, Area::new(width as f32, 1.0))
}

# fn grow(grove: &mut Grove, page: Leaf) -> Plate {
let (rgba, size) = strip(grove);
let gradient = grove.pixels(rgba, size);
grove.branch(
    page,
    Image::new(gradient)
        .fit(Fit::Stretch)
        .at(Location::new().xs(left(16.px()).right(100.pct() - 16.px()), top(16.px()).height(8.px()))),
);
# gradient
# }
# fn frame(grove: &mut Grove, pollen: &Pollen, gradient: Plate) {
// The strip depends on the scheme, so a new scheme is new pixels under the same name.
if pollen.repainted() {
    let (rgba, size) = strip(grove);
    grove.load(gradient, rgba, size);
}
# }
```

`grove.load(plate, rgba, size)` fills a name, and writing one that already holds a picture replaces
it: every element drawing it follows, and none of them is written to. That is the right shape for a
picture that is redrawn, like the strip above, rather than a new name each time, because a picture
once drawn stays on the GPU for as long as the app runs.

`grove.plate()` hands out a name with nothing behind it yet, for `load` to fill in a later frame,
which is the shape of a picture decoded or rendered somewhere else: the element is grown against the
name now, occupies its box, and appears on the frame the pixels land.

## Marks

An icon is not a bitmap. A mark has no size of its own (the same artwork is a 16-pixel affordance
and a 96-pixel empty state), so it is stored as a **multi-channel signed distance field**: a small
square texture holding, per texel, the distance to the artwork's edge. It is reconstructed sharp at
whatever box a layout hands it, and the median of three channels keeps the corners a single
channel would round off.

[`foliage-icons`](https://github.com/eblack-leaf/foliage/tree/main/foliage-icons) bakes fields from
SVGs:

```sh
cargo install --git https://github.com/eblack-leaf/foliage foliage-icons
foliage-icons bake --svg assets/svg --out src/icons --marks Icons
foliage-icons preview --icon src/icons/check.icon --size 24 --out check.png
```

`bake` writes one `.icon` per SVG, and a module beside them that declares the set:

```rust,ignore
// AUTO-GENERATED by foliage-icons -- do not edit by hand.
use foliage::{Field, Grove, Marks};

const SIDE: u32 = 48;
const RANGE: f32 = 3.0;

pub struct Icons {
    /// The `arrow-up` mark.
    pub arrow_up: Field,
    /// The `check` mark.
    pub check: Field,
}

impl Marks for Icons {
    fn register(grove: &mut Grove) -> Self {
        Self {
            arrow_up: grove.icon(include_bytes!("arrow-up.icon"), SIDE, RANGE),
            check: grove.icon(include_bytes!("check.icon"), SIDE, RANGE),
        }
    }
}
```

Both are committed. An app registers the set by naming it, the way it names its `Root`, and
reaches a mark by the name it was given:

```rust
# use foliage::{Boxed, Field, Grove, Grow, Icon, Leaf, Location, Marks, Palette, Source, left, top};
# struct Icons { check: Field }
# impl Marks for Icons { fn register(grove: &mut Grove) -> Self { Icons { check: grove.icon(vec![0; 48 * 48 * 4], 48, 3.0) } } }
# fn f(grove: &mut Grove, bar: Leaf) {
let icons = grove.marks::<Icons>();
grove.branch(
    bar,
    Icon::new(icons.check)
        .color(Palette::Accent)
        .at(Location::new().xs(left(0.px()).width(24.px()), top(0.px()).height(24.px()))),
);
# }
```

Adding or removing a mark and baking again moves no callsite, because nothing addresses a mark by
its position in a list.

A mark is fitted into the largest square its box holds, so a box that is not square leaves room
around it rather than stretching it. It carries shape and no color, exactly as a glyph does, so an
icon is filled, repainted and animated by the same writes a panel takes. `grove.mark(icon, field)`
swaps the mark and keeps the fill, which is how a toggle trades one mark for another.

`preview` renders a baked field to a PNG, sampling exactly as the shader does, which is how to
judge a field without running an app. `--side` (texels across, 48 by default) and `--range` (how
many texels the distance spread covers, 3 by default) are facts about the bake, and are stated
once per set rather than per mark.
