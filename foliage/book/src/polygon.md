# Polygon

`Polygon` is the "expressive shape" primitive -- a regular N-sided shape with uniform
corner rounding, where the whole point is that `sides`/`rounding`/`rotation` are plain
animatable scalars:

```rust
// foliage_proper/src/polygon/mod.rs
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, PartialEq)]
#[require(Differential<Self, Section<Logical>>)]
#[require(Color, Differential<Self, Color>)]
#[require(Differential<Self, ResolvedElevation>)]
#[require(Differential<Self, BlendedOpacity>)]
#[require(Differential<Self, ClipContext>)]
#[require(Differential<Self, Self>)]
pub struct Polygon {
    pub sides: f32,
    pub rounding: f32,
    pub rotation: f32,
}
impl Default for Polygon {
    fn default() -> Self { Self { sides: 3.0, rounding: 0.0, rotation: 0.0 } }
}
```

Because those three fields are just `f32`s, morphing circle↔hexagon↔triangle or
sharp↔round corners is *exactly* interpolating them -- the same [`Animate`](./anim.md)
mechanism already driving `Color`/`Location`, with no bespoke shape-tweening system:

```rust
// foliage_proper/src/polygon/mod.rs
impl Animate for Polygon {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new().with(start.sides, end.sides).with(start.rounding, end.rounding).with(start.rotation, end.rotation)
    }
    // apply(..) writes each back symmetrically
}
```

Side-count changes are a distance-field blend in the shader (`polygon.wgsl`), not a
vertex-matched morph -- cheap, and every rounded endpoint already lacks the acute
unrounded corner that would make a blend look visually wrong mid-transition.

## Deliberately not `Panel`

The doc comment on `Polygon` is explicit about why this isn't folded into
[`Panel`](./panel.md): `Panel` owns arbitrary-aspect rectangles with independent
per-corner radii and borders, and is used everywhere already; a regular polygon's
rounded corners only stay circular if the shape stays roughly square, so `Polygon`
doesn't generalize to `Panel`'s job and isn't trying to. It's placed like `Panel`/`Icon`
-- a bounding box via `.at(Location::new().xs(..))` -- not like `Line`'s point-to-point
mode.
