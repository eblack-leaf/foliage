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
Sets can be grouped in `ExternalSet`s to coordinate their timing in the `schedule`.
Sets are described with `SetDescriptor` to provide slots. `elm.main().add_systems(...in_set(SetDescriptor::Label));`
can be used to add systems to the corresponding labeled set. The set can be added with 
`elm_configuration.configure_hook::<Self>(ExternalSet::Configure, SetDescriptor::Label`.