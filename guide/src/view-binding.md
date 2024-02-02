# ViewBinding

A `ViewBinding` is created with `Elm` and binds a `Scene` or `Bundle` to a `View`.
This takes the form of 
```rust
pub fn add_view_scene_binding<S: Scene, Ext: Bundle + Clone>(
    &mut self,
    view_handle: ViewHandle,
    args: S::Args<'static>,
    rs: ResponsiveSegment,
    ext: Ext,
) {
    // bind scene
}
```
which can be called as
```rust
elm.add_view_scene_binding::<Button, ()>(
    handle,
    ButtonArgs::new(
        ButtonStyle::Ring,
        TextValue::new("ring"),
        MaxCharacters(4),
        FeatherIcon::Copy.id(),
        Color::CYAN_MEDIUM,
        Color::OFF_BLACK,
    ),
    ResponsiveSegment::new(
        0.075.relative(),
        0.05.relative(),
        0.4.relative(),
        40.fixed(),
    ),
    (),
);
```
in which you pass the `Scene` as `Button`, the `Extension` as `()`,the `ViewHandle` as `handle`,
the `Scene::Args` as `ButtonArgs`, a `ResponsiveSegment` and the `Extension` instance `()`.
The `Extension` adds a `Component` to the `Scene` to differentiate it from the base `Scene`. 
This can be useful to attach callbacks such as `elm.add_interaction_handler::<Extension, ...>(...)` using the `Extension`.

```rust
use foliage::bevy_ecs;
#[derive(Component)]
struct ToLoginPage();
impl Leaf for Example {
    // ... leaf requirements
    fn attach(elm: &mut Elm) {
        elm.add_view_scene_binding::<Button, ToLoginPage>(
            handle,
            ButtonArgs::new(
                ButtonStyle::Ring,
                TextValue::new("ring"),
                MaxCharacters(4),
                FeatherIcon::Copy.id(),
                Color::CYAN_MEDIUM,
                Color::OFF_BLACK,
            ),
            ResponsiveSegment::new(
                0.075.relative(),
                0.05.relative(),
                0.4.relative(),
                40.fixed(),
            ),
            (),
        );
        elm.add_interaction_handler::<ToLoginPage, ResMut<CurrentView>>(
            InteractionHandlerTrigger::Active,
            |_ih, ext_args| {
                ext_args.0 = ViewHandle::new(0, 1);
            },
        );
    }
}

```