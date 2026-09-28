# The resolver

Every placement an app writes (`left(16.px()).right(100.pct() - 16.px())`) becomes numbers in one
function: [`placement/resolve.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/placement/resolve.rs).
It takes one axis of one element's placement and everything that axis may read, and returns a span:
a near edge and a far edge.

```rust,ignore
// placement/resolve.rs
pub(crate) fn resolve(config: &Config, context: &Context) -> Span {
    let (near, far) = match &config.form {
        Form::NearExtent { near, extent } => {
            let near = coordinate(near, Role::Near, context);
            (near, near + clamped(extent, config, context))
        }
        Form::NearFar { near, far } => {
            let near = coordinate(near, Role::Near, context);
            let far = coordinate(far, Role::Far, context);
            let extent = clamp(far - near, config, context);
            (near, near + extent)
        }
        Form::FarExtent { far, extent } => {
            let far = coordinate(far, Role::Far, context);
            (far - clamped(extent, config, context), far)
        }
        Form::CenterExtent { center, extent } => {
            let center = coordinate(center, Role::Center, context);
            let extent = clamped(extent, config, context);
            (center - extent / 2.0, center + extent / 2.0)
        }
    };
    Span { near, far }
}
```

## Pure, on purpose

The resolver touches no world, no entity names and no mutable state. Given the same inputs it gives
the same answer, and it may be called as many times per element per frame as anything needs. Three
things depend on that:

- **Both axes are the same code.** R2a and R2b call it once per element each. With no accumulated
  state, the second call cannot be corrupted by the first.
- **Motion resolves both ends.** An element with a `Location` motion running is resolved twice per
  axis, once for where it is going and once for where it left, in the same context, and the two
  answers are blended. Nothing about the motion is remembered between frames, so nothing about it
  can go stale when the window resizes mid-slide.
- **The whole placement algebra is testable as arithmetic.** A `Config` and a `Context` in, a `Span`
  out, with no engine anywhere near it.

## What a placement is made of

A `Location` is one pointer to a set of placements, one per breakpoint (`xs`, `sm`, `md`, `lg`, `xl`,
and `short`), each an optional pair of axes. Looking one up is `Breakpoints::at`: a `short`
placement wins outright when the viewport is short; otherwise the breakpoint in force is taken if it
was stated, then each smaller one in turn, and finally `xs`, whose default is the whole of the
parent's box. That lookup is not part of resolving, so it happens in `rowan.rs` before the resolver
is called.

One axis of one breakpoint is a `Config` ([`placement/role.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/placement/role.rs)):
one of four **forms**, and an optional floor and ceiling on the extent.

| Written | Form |
|---|---|
| `left(a).width(b)`, `top(a).height(b)` | `NearExtent` |
| `left(a).right(b)`, `top(a).bottom(b)` | `NearFar` |
| `right(a).width(b)`, `bottom(a).height(b)` | `FarExtent` |
| `center_x(a).width(b)`, `center_y(a).height(b)` | `CenterExtent` |

Each value in a form is an **expression**: a sum of scaled **terms**, each of which reads one source.

```rust,ignore
// placement/source.rs, shortened
pub(crate) enum Expr {
    One(Term),        // almost every expression: a length, a percentage, an edge
    Sum(Vec<Term>),   // only a sum allocates
}

pub(crate) struct Term {
    pub(crate) scale: f32,
    pub(crate) kind: Kind,
}

pub(crate) enum Kind {
    Px(f32),
    Pct { fraction: f32, against: Against },
    Extent { axis: Axis, against: Against },
    Cell { index: i32, axis: Axis, against: Against },
    Letters { letters: f32, against: Against },
    Character { index: usize, against: Against },
    Span { from: usize, to: usize, end: SpanEnd, against: Against },
    Content { against: Against },
    Edge { edge: Edge, against: Against },
}
```

Every operator the grammar supports (adding, subtracting, scaling by a number) keeps an expression
linear, so a sum of scaled terms is the whole of what can be written rather than a simplification of
it. It is built once, where the placement is written, and resolving it allocates nothing.

`Against` says **whose** geometry a term reads: the element's own (`Own`), its trunk's (`Trunk`),
or its anchor's (`Anchor`). The bare sources on `Source` (`20.px()`, `50.pct()`, `2.col()`) read the
trunk. `trunk()` and `anchor()` open the same vocabulary against a named element, so nothing can be
said about a trunk that cannot be said about an anchor, and an element grown somewhere else (to
escape a clip, say) anchors back to where it came from and goes on addressing the same grid in the
same words.

A **coordinate** is an expression plus an origin: the near edge it is measured from. A bare length
used as a position is measured from the trunk's near edge, which is why `left(16.px())` is sixteen
pixels inside the trunk. An edge (`anchor().right()`) is already a position on the surface, so its
origin is the surface and nothing is added to it.

## What the element can see

```rust,ignore
// placement/resolve.rs
pub(crate) struct Context<'a> {
    pub(crate) axis: Axis,
    pub(crate) own: Basis<'a>,     // itself
    pub(crate) trunk: Basis<'a>,   // what it was grown under, or the viewport
    pub(crate) anchor: Basis<'a>,  // what it is anchored to, or a zero box
}

pub(crate) struct Basis<'a> {
    pub(crate) section: Section,        // its box
    pub(crate) intrinsic: Area,         // its measured content
    pub(crate) tracks: Tracks,          // its grid, at the breakpoint in force
    pub(crate) cell: Area,              // its character cell
    pub(crate) run: Option<&'a Shaped>, // its shaped run
}
```

Every element offers the same five readings, which is what keeps the grammar symmetric. What is
*in* them depends on where in the frame the resolver is called:

- **On the horizontal axis**, no height anywhere is known yet, so every box offered is flattened to
  its horizontal half. A top-level element's trunk is the viewport, which is complete.
- **The element's own box** is what is being solved for, so it offers only what is already settled:
  its content measure and its cell, and, on the vertical axis alone, its width, since R2a has run.
  That is what `aspect(16.0 / 9.0)` reads.
- **An anchor that was never set** offers a zero box. A placement reading it resolves against
  nothing, rather than failing.

## How each source reads

The role decides how a source reads, which is why a source says nothing about what it is for until
a role names it.

| Source | Reads |
|---|---|
| `px` | itself: the only source that reads no geometry |
| `pct` | a fraction of the basis's extent on the resolving axis |
| `col`, `row` | a track of the basis's grid: its near edge in a near role, its far edge in a far role, its middle in a centre role, and `n` tracks with the gaps between them in a size role |
| `letters` | a count of the basis's character cells on the resolving axis |
| `content()` | the basis's measured extent on the resolving axis: max-content width, or wrapped height |
| `anchor().width()`, `.height()` | the anchor's extent on the named axis, whichever axis is resolving |
| edges | a position on the surface |
| `anchor().character(n)` | where character `n` of the anchor's run stands, as it wrapped at the anchor's width: its column times the cell width, or its line times the cell height |

The track arithmetic is short enough to read whole:

```rust,ignore
// placement/resolve.rs
fn cell(index: i32, axis: Axis, role: Role, basis: &Basis) -> f32 {
    let track = basis.tracks.on(axis);
    let size = track.size(
        extent_of(basis.section, axis),
        extent_of_area(basis.cell, axis),
    );
    let index = index as f32;
    let inclusive = matches!(role, Role::Far | Role::Extent);
    let edge = (index - if inclusive { 0.0 } else { 1.0 }) * size + (index - 1.0) * track.gap;
    edge + if role == Role::Center { size / 2.0 } else { 0.0 }
}
```

A track's size is its pitch: a count divides the extent (gaps taken out first, and no outer margin),
a pixel pitch is fixed, and a letter pitch is that many cells *of the element the grid is on*, so a
child addressing a letter-pitched grid gets real column addresses in its parent's font whatever its
own is.

`character` is how a caret and a selection are placed against a run that wraps: the run R1 shaped is
wrapped at the width the anchor resolved to this frame, and the character's line and column are read
off that wrap. The same wrap answers both axes, because the horizontal pass settled the width before
anything could ask. [Fronds](fronds.md) shows it in use.

## Clamps

`at_most` and `at_least` bound the extent, and the order is fixed: the ceiling first, then the floor,
then never below zero.

```rust,ignore
// placement/resolve.rs
fn clamp(extent: f32, config: &Config, context: &Context) -> f32 {
    let mut extent = extent;
    if let Some(most) = &config.most {
        extent = extent.min(value(most, Role::Extent, context));
    }
    if let Some(least) = &config.least {
        extent = extent.max(value(least, Role::Extent, context));
    }
    extent.max(0.0)
}
```

So a floor always wins over a ceiling, and `width(content()).at_most(240.px())` is fit-content: the
smaller of what the text wants and what the ceiling allows. In the `NearFar` form the clamp applies
to the distance between the two edges and keeps the near edge where it is.

## The order is in the types

The horizontal axis resolves before the vertical one, so a horizontal role can never read a height.
Rather than check that at run time, the grammar makes it unwritable. Four types carry the model:

| Type | Is | Legal in |
|---|---|---|
| `Length` | an extent that reads nothing vertical | any role, as a size or as a position from the trunk's near edge |
| `VerticalLength` | an extent only the vertical pass can answer: `row`, `anchor().height()`, `aspect` | vertical roles only |
| `HorizontalCoordinate` | a position on the horizontal axis: an edge, a track of another element's grid | horizontal position roles |
| `VerticalCoordinate` | the same, vertically | vertical position roles |

A `Length` converts into a `VerticalLength`, and never the reverse. Two coordinates subtract to the
length between them, and do not add. So `height(2.col())` is a two-column span used as a height,
while `width(2.row())` does not compile, and neither does `left(anchor().bottom())`.

The roles themselves are typed the same way. `left(..)` returns a `Left`, which has exactly two
methods, `width` and `right`, and each returns a complete `Horizontal`, which has no method that
could state a third value. An illegal pairing is not rejected; there is nothing to call. Each of
these is a `compile_fail` doctest in the crate, pinned to its error code rather than to the
compiler's wording:

```rust,compile_fail
use foliage::{Source, left};
left(0.px()).center_x(50.pct());
```

```rust,compile_fail
use foliage::{Source, left};
left(0.px()).width(2.row());
```

## Measurable, stated once

R2m needs to know which children describe their own height (see
[Resolution](resolution.md#r2-the-two-axes-and-the-measure-between-them)). That question is answered
next to the grammar, term by term:

```rust,ignore
// placement/role.rs
pub(crate) fn known(kind: Kind) -> bool {
    match kind {
        Kind::Px(_) => true,
        Kind::Letters { .. } => true,
        Kind::Character { .. } | Kind::Span { .. } => true,
        Kind::Content { against } => against == Against::Own,
        Kind::Extent { axis, .. } | Kind::Cell { axis, .. } => axis == Axis::Horizontal,
        Kind::Pct { .. } => false,
        Kind::Edge { .. } => false,
    }
}
```

A vertical placement is measurable if every term in it is known and it is not measured from an
anchor's edge. Everything else waits for R2b.

## Lines are the same grammar

A `Line` has no box: it has two ends. Each end is a `Point`, an x coordinate and a y coordinate in
the same grammar, and `locate` resolves one axis of a point through the same `coordinate` function a
box's near edge uses. That is the whole of the point mode: it adds a caller to the resolver rather
than a path through it. R2 then turns the two ends into a box, the rectangle around them grown by
half the stroke's weight, so a line has a box for clipping, extents and the stack like everything
else, and keeps its ends beside it because the box cannot say which diagonal it runs along.
