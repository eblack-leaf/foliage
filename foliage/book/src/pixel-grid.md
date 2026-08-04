# The Pixel Grid: Rounding, Scroll, and Rasterization

Everything an app states is in logical pixels. Everything the GPU draws lands on a device
pixel. The conversion between them is one multiply by the scale factor and one rounding step,
and almost every subtle geometry problem in the engine lives in that rounding step.

This page is the cross-cutting view. The mechanisms are documented where they are implemented
-- `Section::rounded`, `View::snapped_offset`, `rasterize_supersampled` each carry their own
reasoning -- but the tradeoffs between them span `coordinate`, `grid::view`, `text`, and the
pipelines, so no single one of those can hold the argument.

## The path a box takes

```
Location resolves ->  LayoutSection      (logical, fractional)
                   -  accumulated_offset (logical)
                   =  Section            (logical, what the app can sample)
                   *  scale_factor
                   -> Section::rounded() (physical, whole pixels)
                   -> instance buffer
```

Every pipeline does the last two steps itself (`panel/pipeline.rs`, `text/pipeline.rs`, and so
on), which is why `Section::rounded` is the single place the whole engine agrees on what
"snapped" means.

## Why `rounded` derives the extent from the edges

```rust
// foliage_proper/src/coordinate/section.rs
pub fn rounded(self) -> Self {
    let left = self.left().round();
    let top = self.top().round();
    Self::new(
        (left, top),
        (self.right().round() - left, self.bottom().round() - top),
    )
}
```

The obvious alternative is to round the position and the area independently. That puts an
element's right edge at `round(left) + round(width)`, which is not `round(left + width)` -- so
two boxes that agreed on a coordinate can snap a pixel apart and leave a seam between them.
Deriving both edges from the same rounding means any two shapes agreeing on a coordinate still
agree afterwards, by construction.

The cost is that the snapped *size* now depends on the position's fractional part:

```
width = round(x + w) - round(x)
```

As `x` slides continuously, that flips between `floor(w)` and `floor(w) + 1`. Under a
fractional scroll offset it produces two visible artifacts at once:

- **Breathing.** Every shape's rendered size oscillates by a pixel as the view moves.
- **Shimmer.** Each element has its own `frac(x)`, so neighbours cross pixel boundaries at
  different offsets and the gaps between them change by a pixel. Everything drifts against
  everything else.

## The fix: snap the offset, not the box

`View` carries `offset` (fractional, the accumulator) and `snapped_offset` (the same value
rounded to whole device pixels). `extent_check` computes the second, and all three sites that
build `accumulated_offset` sum the snapped one.

The reason that works is exact rather than approximate. For integral `n`, `round(a - n) ==
round(a) - n`, so with `offset * scale_factor = n`:

```
left  = round(layout_x*s) - n
width = round(layout_x*s + W) - round(layout_x*s)      <- n cancels
```

The offset drops out of the width entirely, leaving a pure function of the layout that does
not change while scrolling. It survives in the position as a single integer that *every*
element shifts by together, so a subtree moves as one rigid piece. Breathing and shimmer both
disappear, and edges still derive from shared coordinates so seams stay fixed.

`offset` itself deliberately stays fractional. Rounding it in place would discard every
sub-device-pixel adjustment, and a coast's decaying tail delivers exactly those -- it would
stall short of a stop instead of creeping to one.

## What remains, and why it cannot be removed here

The view now steps in whole device pixels. Below roughly one device pixel per frame it holds
still for several frames and then jumps, which reads as stutter. This is the irreducible cost,
and it is worth understanding why no rearrangement on the CPU side removes it.

`panel.wgsl` uses a hard `step()` for straight edges -- only the corners use `smoothstep` --
and MSAA is requested at 1. With no antialiasing anywhere in the geometry path, **the renderer
cannot represent a position between two pixels.** A fractional translation applied in the
shader, or held in instance data and added late, is quantized right back by the rasterizer. It
buys nothing that snapping does not already give.

So there are exactly three positions, and the choice between them is a rendering decision, not
a layout one:

| | static | moving | blocked by |
|---|---|---|---|
| snap everything (today) | crisp | quantized steps | -- |
| fractional geometry + AA | softer edges | smooth | text |
| offscreen composite | crisp | smooth but blurred | complexity |

**Fractional + AA** means enabling MSAA (or extending the analytic coverage the corners already
use to the straight edges) and then dropping the rounding entirely -- if nothing rounds, the
oscillation cannot exist. It is *less* machinery than today, not more. Text is what blocks it:
glyphs are a bitmap atlas sampled `Linear`, so fractional glyph positions resample every frame
and go permanently soft. Snapping text while panels glide fractionally is worse still, because
the two visibly desync during a scroll. Everything has to move together, and text sets the
floor.

**Offscreen composite** is what browsers do. Rasterize the scrolled subtree into a texture in
*content space*, where geometry keeps its integer snapping because the content does not move
relative to its own texture origin, then put the fractional part only in the blit. Geometry
never goes fractional; only the sampling does. It is also cheaper per frame while coasting over
static content -- nothing re-renders, you re-blit at a new offset. The costs are a half-pixel
blur while moving, a re-rasterize when the scroll settles, and isolating the subtree into its
own render target.

Two platform facts constrain the first option: on web the backend is WebGL2 (`wgpu`'s `webgl`
feature; `webgpu` is not enabled), which has no dual-source blending. And `Msaa::new` already
does `requested.min(max_samples)` off `TextureFormatFeatureFlags`, so unsupported platforms
fall back to 1 sample and get today's behaviour -- availability is handled, only cost is open.

## Coasting reads the frame clock

`coast` scales its per-tick displacement by `Time::frame_diff()`, not a `Moment::now()` taken
inside the system. `update_time` sits in `MainMarkers::External`, chained ahead of the
`Process` set `coast` runs in, so that is the current tick's measurement rather than a stale
one.

The reason it matters is the clamp. `frame_diff` is bounded by
`Time::TIME_SKIP_RESISTANCE_FACTOR`; a raw reading is not. After a stall -- a backgrounded tab
is enough on web -- an unbounded `elapsed` turns `velocity * elapsed` into one enormous step
and teleports the view. It also keeps a coast on the same rhythm as everything else, since
every other piece of motion in the engine is scaled by the same number.

This does not remove ordinary frame-to-frame jitter, which is the same wall clock either way.

## Glyphs land on the same grid

The text path is already correct and worth not re-deriving. Glyph positions are rounded to
whole physical pixels, the quad area is exactly the rasterized bitmap, and the group origin is
rounded too, so the blit is 1:1 texel-to-pixel. The orthographic projection maps pixel edges to
NDC with no half-texel offset.

`fontdue` also steps the pen by a whole `ceil(advance)` already -- measured on the bundled
face, 9px at 14px, 13px at 21px, 15px at 25px, uniform within each. There is no fractional
advance to correct, and quantizing it in the engine only breaks things by discarding each
glyph's bearing.

What is left is a genuine limit. `Metrics::xmin` is an `i32`, so a glyph's true fractional left
bearing floors to a whole device pixel. On the bundled face, `h`'s bearing measures 1px at both
20 and 21px and only reaches 2px at 25px -- so below roughly 22px the gap before it loses about
a third of its width. It compounds with the fact that a wide glyph's ink width equals the pen
step exactly, which leaves the entire visual gap to the following glyph's bearing.

`fontdue` cannot fix this from outside: `metrics_raw` accepts a subpixel offset, but it is
private and every public entry point passes `0.0`, and `GlyphRasterConfig` carries no offset
field. Subpixel positioning would mean forking it or changing rasterizers.

**The practical consequence is that the lever is size, not layout.** Type below about 13 logical
pixels stops surviving the device grid before it stops being readable, and no amount of
positioning work recovers it.

The one thing that does help is coverage. `rasterize_supersampled` runs `fontdue`'s subpixel
path -- which rasterizes at 3x horizontal resolution -- and averages each pixel's three samples
into the single byte the atlas stores. That is not subpixel antialiasing: keeping the three as
R/G/B would need per-channel alpha, i.e. dual-source blending, which WebGL2 does not have, and
it would assume a physical RGB stripe order that is wrong on a rotated or pentile display.
Averaged, it is just 3x supersampling -- no format change, no shader change, no feature
detection -- and it measures coverage from three samples instead of one, which is exactly the
case a thin vertical stem falling between pixel centres defeats.

`Metrics` come back from the same call the plain path uses, so `width`, `height` and `xmin` are
unchanged and no glyph moves.

## Sizing in characters

Because the device grid sets a floor on type size, boxes stated in pixels against a particular
size stop fitting the moment that size moves. `.letters()` exists for this: it resolves against
the character cell rather than a pixel count, so a box tracks the type scale on its own.

It is axis-aware -- the advance width on horizontal designators, the line height on vertical --
so a separate `.lines()` would be a duplicate of what it already does on a vertical axis.

It measures against the entity's **own** `FontSize`, which a `Text` gets for free and nothing
else does. `Sprout::size` exists so a container that holds text without being text -- a `Bare`
region, a `Panel` divider whose offset is stated in characters -- can be handed the cell it
needs to measure against.
