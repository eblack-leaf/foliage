# The GPU Wrapper: Ginkgo

Every rendering primitive needs to create buffers, textures, pipelines, and bind groups
against a GPU device -- and needs to do it the same way regardless of which primitive is
asking. `Ginkgo` is that shared surface over `wgpu`, so pipeline code (`Panel`'s,
`Text`'s, ...) never touches `wgpu::Device`/`wgpu::Queue` directly.

```rust
// foliage_proper/src/ginkgo/mod.rs
pub(crate) struct Ginkgo {
    context: Option<GraphicContext>,
    configuration: Option<ViewConfiguration>,
    viewport: Option<Viewport>,
}
pub(crate) struct GraphicContext {
    pub(crate) surface: Option<wgpu::Surface<'static>>,
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface_format: TextureFormat,
}
```

Both `context` and `configuration` start `None` -- there's no GPU device until the
window actually exists (see [Photosynthesis](./photosynthesis.md)'s `resumed` handler,
which is what calls `acquire_context`).

## Acquiring the device

`acquire_context` (async, since requesting an adapter/device is) creates a `wgpu::Instance`
across `VULKAN | METAL | DX12 | GL`, requests an adapter compatible with the window's
surface, then requests a device. On Android and wasm it deliberately requests
`Limits::downlevel_webgl2_defaults()` instead of `Limits::default()` -- a real
constraint of those backends, not an oversight:

```rust
// foliage_proper/src/ginkgo/mod.rs
cfg_if::cfg_if! {
    if #[cfg(any(target_os = "android", target_family = "wasm"))] {
        let limits = Limits::downlevel_webgl2_defaults();
    } else {
        let limits = Limits::default();
    }
}
```

## Configuring the view

`configure_view` builds the `SurfaceConfiguration` (format, size, `PresentMode::Fifo`)
plus MSAA and depth targets, and re-runs on resize/scale-factor-change (see
`photosynthesis.rs`'s `WindowEvent::Resized`/`ScaleFactorChanged` handling). `create_texture`
is a representative example of Ginkgo defending against a real cross-platform gap rather
than trusting `wgpu` to fail helpfully:

```rust
// foliage_proper/src/ginkgo/mod.rs
let max_dimension = self.context().device.limits().max_texture_dimension_2d;
assert!(
    width <= max_dimension && height <= max_dimension,
    "texture {width}x{height} exceeds this device's max_texture_dimension_2d \
     ({max_dimension}) -- likely fine on native, too large on wasm/WebGL2"
);
```

wasm/WebGL2's texture-size ceiling is well below native's, so a texture that's fine on
desktop can silently be invalid in the browser; this turns that into a clear panic
message at the point of creation instead of a confusing low-level `wgpu` validation
error surfacing somewhere else.

## What pipelines actually call

The rest of `Ginkgo` is small, composable helpers pipeline code builds on: `create_vertex_buffer`,
`create_pipeline`, `create_bind_group`(`_layout`), `create_sampler`, `write_texture`,
`color_attachment`/`depth_stencil_attachment` (used by [`Ash::render`](./ash.md) to open
the actual render pass), and `alpha_color_target_state`/`msaa_state`/`depth_stencil_state`
for pipeline descriptors. None of this is widget-specific -- it's the same handful of
calls every one of the six render pipelines (`Text`, `Panel`, `Image`, `Icon`, `Line`,
`Polygon`) goes through.

`ScaleFactor` also lives here -- a small `Resource` (rounded DPI scale) that every
logical-to-physical coordinate conversion in the crate reads.
