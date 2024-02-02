# View

A `View` is a concept of screens offset by the viewport width and height.
This separates groups of entities into different screens that can be switched to
and viewed. On switching, each `ViewBinding` will be triggered to spawn if
it matches the `CurrentView`s `ViewHandle`. Changing the `CurrentView` resource will
despawn all the current views entities, and spawn the set views bindings.

```rust
elm.view_trigger::<TestHook>(InteractionHandlerTrigger::Active, |_, cv| {
    cv.change_view(ViewHandle::new(0, 1));
});
```
This example changes the `CurrentView` when any `InteractionListener` is in the state `Active` to
the passed `ViewHandle`.

This will result in an anchor position for the `Viewport` that is `ViewHandle.x * Viewport.width = 0 * 360px = 0px`,
and `ViewHandle.y * Viewport.height = 1 * 800px = 800px`.

The `ResponsiveSegment` will resolve their `SegmentUnit`s at their view to offset the elements created, and render
at the correct location. [`SegmentUnit`](segment-unit.md)