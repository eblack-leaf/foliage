# Text, shaped and cut

Text passes through four hands on its way to the screen. The font registry measures a character
cell; R1 shapes the run and R2m wraps it; extraction lays each character in its cell; and the
backend cuts each glyph onto a texture and draws it. This chapter follows a run through all four:
[`text/font.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/text/font.rs),
[`text/shape.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/text/shape.rs),
[`elm.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/elm.rs),
[`ash/atlas.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/ash/atlas.rs) and
[`ash/text.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/ash/text.rs).

## Why only monospace

Every text measurement foliage makes is a count of cells: `letters`, a letter-pitched grid track,
max-content width, wrapping, where a caret stands. In a monospaced face all of those are exact
arithmetic, and one of them is the key to the whole layout model: a run's widest unwrapped line is
its character count times the cell width, known before any layout runs. That is what lets
[width flow down and height flow up](resolution.md#r2-the-two-axes-and-the-measure-between-them)
in two passes instead of an iteration.

A proportional face would not degrade gracefully under that model. It would silently put every
column address somewhere it does not belong. So the registry checks, when a face is handed over,
that six characters spanning the narrowest and widest Latin shapes (`i`, `W`, `m`, `.`, `1`, `g`)
advance the same distance, and refuses the face otherwise. Bytes the program handed over directly
are a statement the program made, and a proportional face there panics at the call. Bytes read
from a path or a URL are refused instead and reported `missing`, since what a file turned out to
hold is not something the program stated.

## The cell

```rust,ignore
// text/font.rs
pub(crate) fn cell(&self, font: Font, size: u32) -> Area {
    let face = self.face(font);
    let px = size as f32;
    let advance = face.metrics(REFERENCE, px).advance_width;
    let line = face
        .horizontal_line_metrics(px)
        .map(|metrics| metrics.new_line_size)
        .unwrap_or(px);
    Area::new(advance.ceil(), line.ceil())
}
```

A cell is the face's advance and its line height at a size, **each rounded up to a whole logical
pixel**. A cell is the pitch a whole run is addressed on, so a fractional one would accumulate along
a line, and the hundredth column would not be a hundred cells in. The bundled face, JetBrains Mono
NL Medium, has a cell of 10 by 22 at 16 pixels and 9 by 19 at 14.

A font name is handed out before its face arrives (a font fetched from a URL, say). A name with no
face yet reads as the bundled face, so every element composed in it still has a cell and a page laid
out in `letters` is laid out sensibly from the first frame. When the real face lands, the drain drops
every run shaped in the stand-in and marks every element to be resolved again, and the page reflows
once.

## Shaping, and the one cache of content

R1 turns an element's text into a `Shaped` run:

```rust,ignore
// text/shape.rs
pub(crate) struct Shaped {
    characters: Vec<char>,
    cell: Area,     // the pitch it was shaped at
    widest: usize,  // the longest hard line, in cells
}
```

In a monospaced face, shaping is almost nothing: the characters, and the longest line between
newlines. `max_content` is `widest * cell.width`. Nothing about glyph shapes is known or needed at
this point.

Shaped runs are kept in the `Shaping` cache, keyed by font and size and then by the string, and
looked up by `&str`, so a frame in which every run has been seen before allocates nothing. It is the
one cache in the engine keyed by *content* rather than by element, and the reason is that shaping is
a function of the string alone: it does not change when the layout moves, and redoing it saves
nothing. Wrapping is deliberately **not** cached, because it depends on the width the layout
produced, which changes whenever the layout does, and walking an already-shaped run is cheap.

The cache is shared with the elements rather than copied into them: R1 hands each element the
`Arc<Shaped>` it states, and everything after R1 (wrapping, placing a caret, drawing) reads the
element's own. That makes eviction simple. On a frame where some run stopped being stated (text was
rewritten, or an element withered), the cache drops every run no element still holds:

```rust,ignore
// text/shape.rs
pub(crate) fn sweep(&mut self) {
    self.runs.retain(|_, runs| {
        runs.retain(|_, shaped| Arc::strong_count(shaped) > 1);
        !runs.is_empty()
    });
}
```

So the cache stays the size of what is on the page rather than the size of the session.

## One walk

How tall a run is, where each of its glyphs goes, and where a caret stands are the same question asked
three ways, and asking it three ways is how text comes to be measured at one height and drawn at
another. So there is one walk, `Shaped::walk`, and all three are answered by it.

It wraps greedily, on word boundaries, by three rules:

- a newline ends a line;
- a word that does not fit in what is left of the line starts the next one, and the spaces before
  it go with the break rather than trailing off the end;
- a word longer than a whole line fills what is left and breaks inside itself.

The walk hands every index in the value its cell, including spaces and newlines (which leave no
ink, and are handed the cell a caret before them would stand in) and one index past the end, where a
caret stands at the end of the run. What leaves ink is said alongside, so a caller that only draws
can skip the rest. An empty run takes no lines at all, which is why an empty element measures to
zero rather than to one line of nothing.

The column count has a tolerance. A width is solved, not stated, and a box that reaches its width
by subtracting two coordinates (every centred and every stretched one) can land a hair under the
width it asked for. Floored exactly, that hair is a whole column, and a run sized to its own content
would wrap at the one width it must not. So a line may be up to a sixty-fourth of a cell short and
still hold it.

`Wrap` answers the per-index questions against one width: `cell_of(index)` for where a caret
stands, `index_at(column, line)` for which character a pointer landed on, and `ends(line)` for what
`Home` and `End` go to. [The resolver's](resolver.md#how-each-source-reads) `anchor().character(n)`
reads `cell_of`, which is how a field's caret is an ordinary element placed against its run.

## From run to glyphs

Extraction lays the run out at the width it resolved to, which is the width it was measured at:

```rust,ignore
// elm.rs, shortened
shaped.place(painted.section.width(), |character, index, at| {
    glyphs.push(Glyph {
        cell: Section::new(origin.moved(at), cell),
        character,
        color: match tints.and_then(|tints| tints.over(index)) {
            Some(fill) => fill.color(&grove.scheme).faded(painted.opacity),
            None => color,
        },
    });
});
```

Each glyph is a cell on the surface, a character, and a color. The color is per glyph because a
`tint` is a fill over a range of the run's own index space, spaces and newlines included, which is
the same space a caret and a selection are addressed in. Counting drawn glyphs instead would make
every index after a space mean something other than what was written.

A run is compared against what the backend holds **whole**: its glyphs, face, size, rank and clip.
A run's glyphs move, refill and restack together, so finding which of them changed would cost more
than rewriting the run.

## The cut

The backend turns each glyph into a textured quad. What it rasterises is a `Cut`:

```rust,ignore
// ash/atlas.rs
pub(crate) struct Cut {
    pub(crate) font: Font,
    pub(crate) size: u32,
    pub(crate) density: u32,  // device pixels per logical pixel, in thousandths
    pub(crate) character: char,
}
```

The density is part of the key because a bitmap is made in device pixels: the same character at the
same size on a denser display is a different cut. It is stored in thousandths because a key has to
compare exactly, and a float from a window manager does not.

A cut is rasterised once, by [`fontdue`](https://crates.io/crates/fontdue), at `size × density`, and
packed onto **one 2048 by 2048 single-channel texture**. Three details are worth knowing:

- **Three samples, one channel.** The rasteriser's subpixel path produces three coverage samples per
  pixel, and they are averaged back into one. That is horizontal supersampling rather than subpixel
  antialiasing: keeping them as red, green and blue would need dual-source blending, which WebGL2
  lacks, and would assume a stripe order that is wrong on a rotated or pentile display. What it buys
  is the case single-sample coverage gets worst, a vertical stem narrower than a pixel: small type,
  and the bowls of round letters.
- **Snapped to the device grid.** A cut is an exact number of device pixels, and the quad it is drawn
  on is placed on a whole device pixel, so each sample lands on a texel centre and the glyph is the
  bitmap. Unsnapped, every sample would fall between texels and the glyph's outer rows would fade,
  which reads as a trimmed glyph rather than as a glyph half a pixel low.
- **Nothing is evicted.** The sheet is packed in shelves and never freed. A monospaced app addresses a
  small alphabet at a handful of sizes, and four megabytes holds thousands of cuts. If it does fill,
  an `error` is written once and further characters stop drawing; the
  [TODO](https://github.com/eblack-leaf/foliage/blob/main/TODO.md#glyph-atlas) records the plan to
  rebuild it from what the frame draws instead.

A character with no outline (a space, and more characters than a space) is remembered as nothing, so
asking again does not rasterise again.

## One entry in the stack

Every other renderer draws one instance per element. A run draws one quad per character, but it is
**one entry in the stack**: one rank, one clip, one depth. The text renderer keeps two numberings
to make that work. Its slots are runs, sorted by rank, and are what the backend's shared walk sees.
Its glyphs are its own, laid out in slot order with each run's contiguous, so a span of runs is a
range of glyphs and one draw call covers it. That keeps the stack the size of the tree rather than
the size of the text on it.

## What it costs

Rewriting a run's text costs the run: R1 shapes the new string (once, if it has not been seen), R2m
wraps it, and extraction lays out and compares every glyph. For a label that is nothing. For a text
area holding tens of thousands of characters it is noticeable, because a keystroke rewrites the whole
run, and the caret and selection are placed by reading character positions that each walk the run
from its start. [What a frame costs](performance.md) has the measurement and the planned fix.
