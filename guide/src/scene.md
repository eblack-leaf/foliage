# Scene

A `Scene` is a fixed arrangement of entities grouped into a logical controller.
The `Scene` binds entities and/or other `Scene`s to particular slots called `SceneBinding`s.
This is just a handle to access each entity uniquely. 

```rust
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Debug)]
pub struct SceneBinding(pub i32);
```

This can be converted to from `enum`s that are `#[repr(i32)]` given a trivial `From` implementation.
This can be accomplished with a `derive` macro `foliage::SceneBinding`. 
```rust
#[derive(foliage::SceneBinding)]
enum Bindings {
    One,
    Two,
    // ...
}
```
Currently only `enum`s are valid `derive` targets for this `proc-macro`.
The `SceneBinding` is used to access elements via a `SceneAccessChain`. This is used by selecting the
`target` element by navigating any sub-scene bindings to get to the desired target.

```rust
let handle = SceneHandle::new(...);
let immediate = handle.access_chain().target(...);
let nested = handle.access_chain().binding(...).target(...);
let sub_nested = handle.access_chain().binding(...).binding(...).target(...);
// ...
```
The `SceneAccessChain` is used in combination with `SceneCoordinator` to get the entity for making changes.
```rust
let entity = coordinator.binding_entity(&handle.access_chain().target(...));
```
To `bind` an element use `Scene::bind_nodes(...)`.

```rust
impl Scene for ProgressBar {
    type Bindings = ProgressBarBindings;
    type Args<'a> = ProgressBarArgs;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        let entity = binder.bind(
            ProgressBarBindings::Back,
            (0.near(), 0.near(), 1),
            Rectangle::new(anchor.0.section.area, args.back_color, Progress::full()),
            cmd,
        );
        tracing::trace!("binding-progress-back: {:?}", entity);
        let entity = binder.bind(
            ProgressBarBindings::Fill,
            (0.near(), 0.near(), 0),
            Rectangle::new(anchor.0.section.area, args.fill_color, args.progress),
            cmd,
        );
        tracing::trace!("binding-progress-fill: {:?}", entity);
        Self {
            tag: Tag::new(),
            progress: args.progress,
        }
    }
}
```

Here is the implementation of `Scene` for a `ProgressBar`.
This `bind`s two `Rectangle`s at the `SceneBinding`s of `ProgressBarBindings::Fill/Back`.
The `enum` matches to an `i32` with `0/1` respectively.

The second argument to `SceneBinder::bind` is the `SceneAligment`( see [`SceneAlignment`](alignment.md)).
This places the element in the scene relative to the `Anchor` point that is determined by the
`Compositor` and `ResponsiveSegment`. This uses the given `Area` to configure the elements
position. The `Scene::Args<'_>` are arguments to pass to the scene to control the look. These are
user defined to fit whatever is needed. The `ProgressBar` uses a `FillColor` and a `BackColor` to 
determine the style.
The `Scene::ExternalArgs` are to pull `SystemParam` (`Query`, `Res`, and `ResMut` for example) from the `ecs`.
This is useful to have access to system variables when configuring the `Scene`.
A `Commands` is sent down as well to allow inline extension to the normal bindings.