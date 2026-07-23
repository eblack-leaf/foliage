# Image

Unlike every other primitive, `Image` doesn't own its own pixel data directly -- it
points at an [`AssetKey`](./asset.md), and the GPU texture identity is derived from that
key internally (the doc comment on the struct is explicit: "there is no separate memory
id to hand-assign or keep in sync with anything"):

```rust
// foliage_proper/src/image/mod.rs
#[derive(Component, Copy, Clone, PartialEq)]
#[require(ImageView, ImageMetrics)]
#[require(Differential<Image, Section<Logical>>)]
#[require(Differential<Image, BlendedOpacity>)]
#[require(Differential<Image, ResolvedElevation>)]
#[require(Differential<Image, ClipContext>)]
#[require(CropAdjustment, Differential<Image, CropAdjustment>)]
pub struct Image {
    pub key: AssetKey,
}
```

Loading is decoupled from spawning: `Image::new(key)` can be given a key before its
bytes have actually arrived (a network fetch on wasm still in flight, say) -- the entity
exists immediately and starts rendering once `AssetRetrieval`/`OnRetrieval` (see
[Assets](./asset.md)) resolve the key behind it.

## `ImageView`: how the image's box and its aspect ratio relate

```rust
// foliage_proper/src/image/mod.rs
pub enum ImageView {
    Aspect,   // box adopts the image's real aspect ratio
    Crop,     // box is authored freely; image is cropped to fill it
    Stretch,  // box is authored freely; image stretches to fill it, ignoring its own ratio
}
```

`Aspect` (the default) is what most authored images want: give it a box on one axis and
let the image's own width:height ratio determine the other, via
[`AspectRatio`](./grid.md) -- `ImageView::on_insert` computes and inserts that ratio
directly from `ImageMetrics.extent` (the image's real decoded pixel dimensions). `Crop`
and `Stretch` both hand full box control back to the author's own `Location`; the
difference is only what happens to content that doesn't match the box's own ratio --
`CropAdjustment` (a `Section<Numerical>` offset into the source texture) versus a
straight non-uniform stretch.

## Registration

```rust
// foliage_proper/src/image/mod.rs
impl Attachment for Image {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(RenderQueue::<Image, ImageWrite>::new());
        foliage.diff.add_systems(Image::update.in_set(DiffMarkers::Finalize));
        foliage.remove_queue::<Image>();
        foliage.differential::<Image, Section<Logical>>();
        // ClipContext, BlendedOpacity, ResolvedElevation, CropAdjustment
    }
}
```

`Image::update` runs in `Finalize` (see [The App](./app.md)'s `DiffMarkers` ordering),
before `Extract` drains the differentials -- the asset resolution and crop/stretch math
have to settle before anything downstream diffs the result, the same reasoning
[Text](./text.md)'s glyph-resolution ordering follows.
