# View

A `View` is a concept of screens offset by the viewport width and height.
This separates groups of entities into different screens that can be switched to
and viewed. On switching, each `ViewBinding` will be triggered to spawn if
it matches the `CurrentView`s `ViewHandle`. Changing the `CurrentView` resource will
despawn all the current views entities, and spawn the set views bindings.