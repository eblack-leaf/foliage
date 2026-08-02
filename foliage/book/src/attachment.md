# Attachment

The smallest file in the crate, and one of the most load-bearing:

```rust
// foliage_proper/src/attachment.rs
pub(crate) trait Attachment {
    fn attach(foliage: &mut Foliage);
}
```

One method, no default, no associated types, and `pub(crate)` -- engine-internal setup,
not something an app or an external library implements. Every subsystem in the crate
implements it once -- `Panel::attach` ([Panel](./panel.md)), `Text::attach`
([Text](./text.md)), `Ash::attach` ([Ash](./ash.md)), `Disable::attach`/
`Visibility::attach` ([Lifecycle](./lifecycle.md)), `Asset::attach` ([Assets](./asset.md)),
and dozens more -- and [`Foliage::new()`](./app.md) calls every one of them, in sequence,
at startup:

```rust
// foliage_proper/src/foliage.rs
Disable::attach(&mut foliage);
Panel::attach(&mut foliage);
Ash::attach(&mut foliage);
Text::attach(&mut foliage);
Asset::attach(&mut foliage);
// ... every subsystem, one call each
```

This is the entire mechanism behind a claim made repeatedly throughout this book: that
adding a new [`Differential`](./differential.md) registration, a new system, or a new
resource to an existing widget never requires touching `foliage.rs` itself. `attach`
*is* the registration point -- each type owns exactly what it needs to set up about
itself (systems, resources, differentials), and `Foliage::new()`'s job is only to call
every one of them, not to know what any of them actually do. A brand new primitive or
composite becomes fully wired in the moment its own `Attachment` impl is added to that
one call list -- nothing elsewhere in the engine needs to change to accommodate it.

An app has no equivalent setup to register -- it configures a `Foliage` through the
public calls on the type itself (`desktop_size`, `font`, `icon`, `tune`, `asset_base`,
...), not through `Attachment`.
