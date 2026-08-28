# Icon

An icon needs to render crisply at any size an author gives it -- next to a small label
or blown up as a hero graphic, and a raster image would blur or pixelate at the wrong end
of that range. `Icon` renders MTSDF (multi-channel signed distance field) fields instead
of raster bitmaps, which is what makes it resolution-independent: one baked field serves
every on-screen size.

```rust
// foliage_proper/src/icon/mod.rs
pub type IconId = i32;
#[derive(Component, Copy, Clone, PartialEq, Default)]
#[require(Color, Differential<Icon, Color>)]
#[require(Differential<Icon, ClipContext>)]
#[require(Differential<Icon, Section<Logical>>)]
#[require(Differential<Icon, Icon>)]
#[require(Differential<Icon, ResolvedElevation>)]
#[require(Differential<Icon, BlendedOpacity>)]
pub struct Icon {
    pub id: IconId,
}
```

The same shape every primitive follows -- required color/differentials, no `Node`
mentioned (see [Inside the Engine](./tree.md) for why). `id` is a plain `i32`, not a string or enum:
icons are registered by id up front (typically generated code from `foliage_icons`, the
separate SVG-to-MTSDF CLI tool in this workspace), and referenced by that id afterward.

## Registering a field

```rust
// foliage_proper/src/icon/mod.rs
pub fn msdf<ID: Into<IconId>, M: AsRef<[u8]>>(mem: ID, bytes: M, field_size: u32, px_range: f32) -> IconMemory
pub fn msdf_from_asset<ID: Into<IconId>>(mem: ID, key: AssetKey, field_size: u32, px_range: f32) -> IconMemory
```

`Icon::msdf` takes the field bytes directly, already in hand (`IconSource::Ready`) --
the common case, typically `include_bytes!` from `foliage_icons`-generated code.
`msdf_from_asset` instead stores an [`AssetKey`](./asset.md) (`IconSource::Pending`), so
**icons genuinely can be dynamic assets**, resolved through the exact same
`AssetLoader`/`AssetRetrieval`/`OnRetrieval` machinery `Image` uses -- not a separate,
icon-specific loading path:

```rust
// foliage_proper/src/icon/mod.rs (IconMemory::on_add, abridged)
let ready = match &value.source {
    IconSource::Ready(bytes) => Some(bytes.clone()),
    IconSource::Pending(key) => world.get_resource::<AssetLoader>().unwrap().retrieve(*key).map(|a| a.data),
};
if let Some(bytes) = ready {
    // already resolved (e.g. Bytes-sourced and loaded before this ran) -- queue immediately
} else if let IconSource::Pending(key) = value.source {
    // not resolved yet -- register AssetRetrieval and wait for OnRetrieval, same as Image
    world.commands().entity(this).insert(AssetRetrieval::new(key)).observe(Self::on_retrieved);
}
```

Either way, only a *resolved* `IconMemory` (`IconSource::Ready`) ever reaches the render
queue -- `resolved_bytes()` treats hitting `Pending` there as
`unreachable!("IconMemory reaches the render queue only once bytes are resolved")`, a
real invariant enforced by construction, not just documented. `field_size` is the baked
field's resolution; `px_range` is the distance-field's falloff range in pixels, which
controls how sharp vs. soft the rendered edge looks regardless of on-screen size.

## The public value channel

An icon entity's actual glyph is driven by `IconValue`, the same shared value-channel
convention `TextValue` follows -- across the boundary this is
[`Grows::icon`](./forest.md):

```rust
// foliage_proper/src/icon/mod.rs
fn apply_icon_value(
    trigger: Trigger<Insert, crate::IconValue>,
    values: Query<&crate::IconValue>,
    icons: Query<(), With<Icon>>,
    mut tree: crate::Tree,
) {
    let this = trigger.event_target();
    if icons.contains(this) {
        // ...applies the write to Icon::id
    }
}
```

The `With<Icon>` filter matters: `IconValue` is also carried by entities that merely use
it as *config* rather than render it directly -- a composite root can hold `IconValue`
and forward it to its own lazily-spawned `Icon` child -- so this observer has to skip
those and only act where an `Icon` component is actually present to update.
