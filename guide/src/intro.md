# Introduction

This is a UI library that focuses on lightweight abstraction over `bevy_ecs`, `wgpu`, and `winit`. This means that
features will sometimes interact closely with these crates as forwarding all of their functionality to
provide a cleaner API has not been done yet. This library is still under development and provides no promises about 
backwards compatibility until v1.0. 

This book will cover an overview of how the library works and your part in using the system. 

### Content Overview

#### Foliage

To start we will need to run an instance of [`Foliage`](foliage.md).

Foliage is the main engine, that collects leaves defined by an application and
runs with that configuration.

```rust
fn main() {
    Foliage::new().run::<Engen>();
}
```
See [`Entry`](entry.md) for more details on setting up a project using `Foliage`.
#### Engen 

You will need to define an `Engen` to handle `async` functions from the main loop.
```rust
#[derive(Default)]
pub struct Engen {}
impl Workflow for Engen {
    type Action = u32;
    type Response = i32;

    async fn process(_arc: EngenHandle<Self>, action: Self::Action) -> Self::Response {
        tracing::trace!("received: {:?}", action);
        (action + 1) as i32
    }

    fn react(_elm: &mut Elm, response: Self::Response) {
        tracing::trace!("got response: {:?}", response);
    }
}
```
Any data in the `Engen` that you define will be available
behind an `Arc<Mutex<...>>` through `EngenHandle`.
The `Workflow::Action` and `Workflow::Response` are entirely up to you. The main thread cannot run `future`s so if you
need to run `async` `fn`s then send an `Workflow::Action` with the input and configure your logic in
`Workflow::process`.

You can then read the `Workflow::Response` when it is finished and update any changes to `Elm` through 
`Workflow::react`. 

Now you can start adding [`Leaf`](leaf.md) attachments to `Foliage` to add functionality.
```rust
Foliage::new().with_leaf::<impl Leaf>()...
```