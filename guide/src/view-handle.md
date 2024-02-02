# ViewHandle

A `ViewHandle` is the coordinate for which `View` to render to.
It comprises an `x` and `y` which is multiplied by the `Viewport.width`/`Viewport.height` respectively to get
the actual anchor to position the `Viewport` at. Moving the `Viewport` position will show
the entities placed at that location.