# Leaf

A leaf is an attachment to the main `Foliage` engine.

```rust
pub trait Leaf {
    type SetDescriptor: SystemSet + Hash + Eq + PartialEq + Copy + Clone + Debug;
    fn config(_elm_configuration: &mut ElmConfiguration) {}
    fn attach(elm: &mut Elm);
}
```

Anything in `Leaf::attach` will be added to the engine during initialization.