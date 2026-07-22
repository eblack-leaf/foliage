# The Window: Willow

Before there's a GPU surface, there has to be a window to put it in. `Willow` owns that
one responsibility -- it doesn't know about rendering at all, only about the OS window
and its requested size/title/limits:

```rust
// foliage_proper/src/willow.rs
pub(crate) struct Willow {
    pub(crate) handle: WindowHandle,
    pub(crate) min_size: Option<Area<Physical>>,
    pub(crate) requested_size: Option<Area<Physical>>,
    pub(crate) title: Option<String>,
    pub(crate) max_size: Option<Area<Physical>>,
    pub(crate) resizable: Option<bool>,
    pub(crate) starting_position: Option<Position<Numerical>>,
    pub(crate) near_far: Option<NearFarDescriptor>,
}
```

## Connecting

`connect(event_loop)` is where the actual `winit::window::Window` gets created, once a
`winit::event_loop::ActiveEventLoop` exists to create it from (see
[Photosynthesis](./photosynthesis.md)'s `resumed`, the only place this is called). On
wasm, it does one extra thing native platforms don't need: appending the window's canvas
into the DOM itself, since a `winit` window on the web *is* a canvas element that has to
be attached somewhere:

```rust
// foliage_proper/src/willow.rs
#[cfg(target_family = "wasm")]
{
    use winit::platform::web::WindowExtWebSys;
    window.set_prevent_default(true);
    let canvas = window.canvas().expect("window-canvas");
    canvas.style().set_css_text("height: 100%; width: 100%;");
    web_sys::window().and_then(|win| win.document()).and_then(|doc| doc.body())
        .and_then(|body| body.append_child(&canvas).ok())
        .expect("append-canvas");
}
```

## `NearFarDescriptor`: internal depth headroom, not an author-facing budget

`Willow` also carries `NearFarDescriptor { near, far }` -- the `[near, far]`
`ResolvedElevation` range [`Ash::assign_elevations`](./ash.md) spaces entities across.
Its own doc comment is explicit about what it is and isn't:

> purely internal headroom for `ash::assign_elevations`'s gapped/fractional-index scheme
> (more room between adjacent entities before a gap needs renormalizing) -- not an
> author-facing budget. Nothing outside that scheme ever reads a specific
> `ResolvedElevation` value directly, so this is free to widen with no migration cost.

In other words: authors work with `Elevation::up(n)`/`abs(n)` (relative, symbolic
ordering), never with a raw depth number -- `NearFarDescriptor`'s actual `(0.0, 300.0)`
default is an implementation detail of how that symbolic ordering gets mapped onto real
floats, free to change without breaking anything an author wrote.

Willow feeds `Ginkgo` (surface creation, size queries) and is itself driven only by
[Photosynthesis](./photosynthesis.md) -- it has no update loop of its own.
