# The backend

Two subsystems sit below extraction. `Ash`
([`ash/`](https://github.com/eblack-leaf/foliage/tree/main/foliage/src/ash)) is the six renderers
and the one stack across them. `Ginkgo`
([`ginkgo/`](https://github.com/eblack-leaf/foliage/tree/main/foliage/src/ginkgo)) is the GPU: the
device, the surface, and everything sized against it. Both are built when the platform hands over a
window, dropped when it takes the window back, and invisible to everything above them.

## Six renderers, one quad

Six kinds of element own a pipeline and an instance buffer: panels, polygons, lines, icons, images
and text. Everything else, the fields included, is assembled from those. Every one of them draws the
same geometry, a unit quad of two triangles:

```rust,ignore
// ash/mod.rs
pub(crate) const CORNERS: [[f32; 2]; 6] = [
    [0.0, 0.0], [1.0, 0.0], [0.0, 1.0],
    [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
];
```

What differs is what the fragment shader does inside it: a rounded rectangle is a signed distance
field, a regular polygon is another, a line is the distance to a segment, an icon is a multi-channel
distance field sampled from a sheet, a picture is a texture sampled through a crop, and a glyph is a
coverage texture sampled from the atlas. Nothing is carved into a mesh. So five of the renderers are
one generic pipeline (`Quads<I>` in `ash/quad.rs`) given three things of their own: a shader, the
shape of an instance, and whether it samples a texture. Text is the sixth, and differs structurally
rather than by shader, because a run is one entry in the stack holding many quads
([Text](text.md#one-entry-in-the-stack)).

Edges are antialiased by the shapes themselves. A distance field's coverage is ramped over the
screen-space derivative of the distance (`fwidth`), so a rounded corner is smooth at any scale with
one sample per pixel, and there is no multisampling.

## Absorbing a batch

`Ash::absorb` runs once for every extraction, immediately after it, and applies the batch to each
renderer. This is where everything that depends on the display's density is derived:

- a **line**'s segment has its axis-aligned edges snapped to whole device pixels, and its coverage
  ramp sized for the density;
- an **icon**'s mark is packed onto the sheet the first time any instance draws it, and its distance
  range is converted into one on screen;
- an **image**'s picture is uploaded the first time an instance draws it, as a texture of its own,
  and uploaded again whenever the batch says its name was filled anew while it is held (see below);
- a **run**'s glyphs are cut through the atlas at the density, and snapped to the device grid.

Then each renderer flushes to the GPU. An instance buffer is kept in rank order, so what a flush costs
depends on what changed:

- **Values changed, order did not** (a move, a recolor): only the changed slots are written, as runs
  of neighbouring slots, one write per run. Slots up to sixteen apart are bridged into one write,
  because rewriting an unchanged slot is harmless and a write has a cost of its own.
- **Something was added, removed or re-ranked**: every slot after the change has moved, so the whole
  buffer is rebuilt in order and written once.

Each renderer keeps two buffers per instance: its own data, and a depth. They are separate because
they change for different reasons. A depth comes from an instance's *position* in the whole stack, so
it is untouched while the stack is; holding it inside the instance would make every recolor a
candidate for a full rewrite.

## One stack, many renderers

Everything draws back to front with alpha blending, so there is one stack across all six renderers,
ordered by rank. Each renderer keeps its own slots sorted by rank, and `Ash::restack` merges them:

```rust,ignore
// ash/mod.rs, shortened
fn restack(&mut self, ginkgo: &Ginkgo) {
    self.stack.clear();
    // every renderer's slots as (rank, renderer, slot); a run of text is one slot
    gather(self.panels.instances.ranks(), Renderer::Panel, &mut self.stack);
    // ... the other five
    self.stack.sort_unstable();
    let total = self.stack.len();
    self.spans.clear();
    for (position, entry) in self.stack.iter().enumerate() {
        let depth = Depth::of(position, total);
        // write the depth into the renderer's slot, read its clip and binding,
        // and extend the current span or start a new one
    }
    // flush every renderer's depths
}
```

Two things come out of one walk, and they have to come out of the same one.

**A depth for every instance, from its position.** Depths are spread evenly and strictly inside the
range, front-most smallest:

```rust,ignore
// ginkgo/depth.rs
pub(crate) fn of(index: usize, total: usize) -> f32 {
    let step = (index + 1) as f32 / (total + 1) as f32;
    Self::FAR - step * (Self::FAR - Self::NEAR)
}
```

A rank is a pair (an accumulated elevation and an allocation number) with an order but no meaningful
magnitude, so depth is taken from the position rather than scaled from the rank. The range subdivides
rather than fills, so there is no budget to run out of; the only ceiling is the depth format's
precision, somewhere around eight million elements. The depth test (`LessEqual`, with writes on)
then holds the order for fragments the draw order alone would not, and a repaint of the same content
lands on the same result.

**Spans.** The walk is cut into maximal runs that share a renderer, a clip and a binding, and each
span is one draw of one contiguous range of one renderer's buffer (contiguous because the merge meets
each renderer's slots in slot order). A binding only ever differs for images, which are a texture each;
everything else binds one thing or nothing.

The walk only runs when something it reads changed: a slot appeared, went, was re-ranked, or changed
its clip or binding. A frame that only recolored what was already there leaves the stack as it was.

## Drawing

```rust,ignore
// ash/mod.rs, shortened
pub(crate) fn draw(&self, ginkgo: &Ginkgo, clear: Color) {
    ginkgo.draw(clear, |pass| {
        for span in &self.spans {
            let (left, top, width, height) = ginkgo.scissor(span.clip);
            if width == 0 || height == 0 {
                continue;
            }
            pass.set_scissor_rect(left, top, width, height);
            // draw span.from..span.to with the span's renderer (and binding, for images)
        }
    });
}
```

**Clipping is a scissor.** A clip is applied to the pass rather than carried in an instance and
tested per fragment, so it costs nothing per element and every renderer has it without a line of
shader. The scissor is the clip in device pixels, expanded outward to whole pixels (a fractional scale
factor would otherwise shave the far edge off every clipped region) and clamped to the surface.

The surface is cleared to whatever `Palette::Surface` resolves to in the current scheme: nothing of
the engine's own sits behind the tree, so the ground is the app's.

**Drawing never changes anything.** What is drawn is held by the renderers rather than consumed by the
draw, so a draw can be repeated at any time (a window exposed, a compositor asking) and produce the
same picture. A surface that cannot be acquired (occluded, or mid-resize) skips the paint and loses a
picture, never a change; one that went stale is reconfigured once and tried again.

## Ginkgo: where the scale factor stops

The engine is written in logical pixels throughout, and the display's ratio of device pixels to
logical ones is applied in exactly two places below this line: the surface and its depth attachment
are sized in physical pixels, and the projection every renderer binds at group 0 is built from the
logical area. The rasteriser does the conversion between them, so no instance, radius or section is
ever carried in device pixels. The scissor and the density a glyph is cut at are the only other places
that read the scale.

A few decisions in `Ginkgo::acquire` are worth knowing about:

- **Backends**: Vulkan, Metal, DirectX 12 and GL (which is WebGL2 in a browser), whichever the
  adapter supports.
- **Limits**: on the web and on Android, the device is requested with WebGL2's downlevel limits, so
  asking for something the device lacks fails at startup rather than at the draw that needed it.
- **Color**: a `Color` holds sRGB channels, which is how a scheme is stated and how a shader reads
  them, so the surface's non-sRGB view is taken and the channels are written through unchanged.
- **No debug names on Android.** A debug build normally names every GPU object, and the Android
  emulator's Vulkan driver crashes inside the naming call, so naming is turned off there.
- **Presentation** is FIFO, so frames are paced by the display.

## What the backend holds, and for how long

| | Held on | Freed |
|---|---|---|
| glyphs | one 2048 by 2048 single-channel atlas | never ([Text](text.md#the-cut)) |
| marks | one sheet of distance fields, three channels | never: a set an app names once |
| pictures | a texture each | not today |
| instances | one pair of buffers per renderer, grown to the next power of two | with the backend |

Marks and glyphs cannot share a texture because one holds coverage in one channel and the other a
distance in three. Pictures get a texture each because they are arbitrarily large, arrive whenever a
decode finishes, and come in any number: packing them onto a shared texture would need eviction, and
evicting one would orphan every instance already pointing at it.

All of it is dropped when the platform takes the surface away, and rebuilt when it gives one back,
with extraction's cache dropped alongside so the next extraction states everything again
([The loop](loop.md#when-the-surface-goes-away)).
