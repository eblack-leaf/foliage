# Color

```rust
// foliage_proper/src/color.rs
#[derive(Component, Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub value: bevy_color::Srgba,
}
```

One component, wrapping `bevy_color`'s sRGBA directly -- no separate foliage-specific
color math, so conversions to `wgpu::Color` (`impl From<Color> for wgpu::Color`) and to
the GPU-packed `CReprColor` (`#[repr(C)] [f32; 4]`, for vertex/uniform buffers) are both
direct, lossless mappings rather than a reimplementation.

## Palette constructors, not raw RGB

Every color used across every example in this book (`Color::gray(900)`,
`Color::green(500)`) comes from one macro, instantiated once per Tailwind color family:

```rust
// foliage_proper/src/color.rs
macro_rules! color_fn {
    ($name:ident: $c50:expr, $c100:expr, ..., $c950:expr) => {
        pub fn $name<L: Into<Luminance>>(l: L) -> Self {
            Self { value: match l.into() { Luminance::Fifty => $c50, ..., Luminance::NineHundredFifty => $c950 } }
        }
    };
}
```

`Luminance` is the 50-950 Tailwind scale as an enum, with `impl From<i32> for Luminance`
rounding any integer to its nearest step -- which is what makes `Color::gray(900)`
(a plain `i32`) legal without an explicit `Luminance::NineHundred`. This is a genuine
constraint, not just convenience: authoring against a fixed, named palette (`gray`,
`green`, `orange`, ...) at fixed steps (50/100/.../950) instead of arbitrary RGB values
means two colors chosen for the same semantic role (say, two different composites both
reaching for "the danger color") are far more likely to actually match, and a whole
palette can be re-themed by swapping which Tailwind family a design system's roles point
to, not by hunting down hardcoded hex values.

## Animating color

`Color` implements [`Animate`](./anim.md) over four channels (r, g, b, a) -- see
[Animation](./anim.md) for the shared contract every animatable component follows;
`Color::attach` registers nothing else beyond `enable_animation::<Self>()`, since color
itself has no cascade/inheritance behavior the way [`Opacity`](./lifecycle.md) does.
