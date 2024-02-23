# Elm

To interact with the core engine, we use `Elm` or the `Entity-Logic-Manager`.

Options are

```rust
elm.startup().add_systems(...);
```
which returns the startup schedule. [`ECS`](ecs.md)

```rust
elm.main().add_systems(...);
```
which returns the main schedule.
```rust
elm.teardown().add_systems(...);
```
which is for cleanup after the program exits.
```rust
elm.container().spawn(...);
elm.container().insert_resource(...);
```
returns a mutable reference to the storage of the `ecs`.
Here you can `elm.container().spawn(...)` to add entities and `elm.container().insert_resource(...)`
to add resources to the `ecs`.
```rust
elm.view_trigger::<Hook, View>();
```
can be used as a convenience method for setting an `interaction_handler` to change the `Current-Tree`.
```rust
elm.add_interaction_handler::<Hook, Resources>(
    |hook, res| { ...}
);
```
is used for adding handlers that run on the trigger condition and invoke your callback. [`Interaction`](interaction.md)
```rust
elm.on_fetch(...);
```
is used for adding a triggered effect when an `Asset` loads; see [`Asset`](asset.md)
```rust
elm.add_event::<E>(...);
elm.send_event::<E>(...);
```
are for configuring and interacting with
events that can be triggered on the `ecs`.
```rust
elm_configuration.configure_hook(
    ExternalSet::Configure, 
    Leaf::SetDescriptor::Label
);
```
`ElmConfiguration` can be used in a [`Leaf`](leaf.md) to add `SystemSet`s to the schedule to group
systems added. 

```rust
elm.enable_conditional::<C>();
elm.enable_conditional_scene::<S>();
```

can be used to enable runners to load conditional bundles|scenes.