# Foliage

Foliage is the main engine, that collects leaves defined by an application and 
runs with that configuration.

```rust
pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((360, 800))
                .with_resizable(true),
        )
        .with_leaf::<Showcase>()
        .with_android_interface(android_interface)
        .with_worker_path("./worker.js")
        .run::<Engen>();
}
```

Start by creating `Foliage::new()`. This attaches the default engine leaves and begins configuration.
Options can be applied such as `WindowDescriptor` to set title and window behavior. `Leaf`s can be added
with `.with_leaf::<L>()` and `.with_renderleaf::<L>()` with the latter adding 
a `Leaf + Render` implementor (see [`Leaf`](leaf.md) and [`Render`](render.md)).
The `AndroidInterface` is a unit struct, `()`, on platforms other than `Android` and is used to 
interact with the android lifecycle. The `.with_worker_path(<path>)` is to link with what you 
named the binary to start the web worker. Finally, we `.run::<Engen>()` the instance to invoke the
main loop. The `Engen` is an implementor of [`Workflow`](workflow.md).