# Compositor

The `Compositor` is a resource to manage the creation/deletion of entities bound to a `View`.
When a `View` switches it spawns/despawns accordingly and updates the `ResponsiveSegment`s coordinate
when the `Viewport` resizes or moves. This composes all the elements active in your engine to 
the screen. When a `ResponsiveSegment` has excluded a particular `Layout`, it will `Disable` the 
entity to render it invisible and inactive.

A `Compositor` can be used to `.add_view(...)` and to `.add_to_view(...)`. These are what `elm.add_view_scene_binding(...)`
and `elm.add_view_binding(...)` use to control the entities grouped in a view. 