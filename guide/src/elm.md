# Elm

To interact with the core engine, we use `Elm` or the `Entity-Logic-Manager`.

Options are

- `elm.startup()` which returns the startup schedule. [`ECS`](ecs.md)
- `elm.main()` which returns the main schedule.
- `elm.teardown()` which is for cleanup after the program exits.

- `elm.container()` returns a mutable reference to the storage of the `ecs`.
Here you can `elm.container().spawn(...)` to add entities and `elm.container().insert_resource(...)` 
to add resources to the `ecs`.

- `elm.add_view_scene_binding::<impl Scene, Ext>(...)` is for adding `Scene`s to the target `View` via a
`ViewHandle` (see [Scene](scene.md) and [View](view.md)).

- `elm.add_interaction_handler::<Hook, Resources>(InteractionHandlerTrigger::Active, |hook, res| { ...})`
is used for adding handlers that run on the trigger condition and invoke your callback.

- `elm.on_fetch(...)` is used for adding a triggered effect when an `Asset` loads. [Asset](asset.md)

- `elm.add_event::<E>(...)` and `elm.send_event::<E>(...)` are for configuring and interacting with
events that can be triggered on the `ecs`.

- `ElmConfiguration` can be used in a `Leaf` to add `SystemSet`s to the schedule to group
systems added. `elm_configuration.configure_hook::<Self>(ExternalSet::Configure, Leaf::SetDescriptor::Label);`