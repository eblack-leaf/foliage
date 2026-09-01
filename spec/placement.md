# Placement

How an element states where it sits, and how that becomes a box.

Two halves, settled separately: the **resolution model** (this document) and the **authoring
surface** (open — see the end). The model comes first because what a `Location` can usefully *say*
depends on what resolution can honour.

## The problem

Text wrapping makes height depend on width. Width comes from layout. And if a container sizes
itself to its contents, layout depends on height — which is a cycle in the general case, and the
reason CSS needs constraint machinery to answer it.

The previous engine sidestepped this entirely with `.letters()`: state the size ahead of time in
character cells. It works, and it is genuinely useful, but it is a declaration rather than a
measurement, so nothing could be sized to text that wraps. That gap broke real app ideas.

## The rule

> **Width flows down. Height flows up.**

Bounded at two passes with no iteration to convergence:

| | Pass | Does |
|---|---|---|
| R2a | horizontal | Resolve the horizontal axis for the whole tree, top-down in dependency order |
| R2m | wrap | Each text run wraps at its now-known width → line count → intrinsic height |
| R2b | vertical | Resolve the vertical axis; content-sized elements read intrinsic heights upward |

### Why monospace is what makes this work

An element's **max-content width** — the widest it would like to be, unwrapped — is normally the
expensive half of intrinsic sizing, because it needs shaping to answer.

In a monospace font it is `char_count × cell_width`. Free, exact, no measure pass, available before
any layout has happened at all.

So the down-pass has everything it needs, and only the up-pass requires real measurement — which by
then has a known width to measure against. The font restriction that forced `.letters()` into
existence is the same restriction that makes proper intrinsic sizing tractable.

### What becomes expressible

- a box as tall as its text turns out to be, after wrapping
- a container that grows to fit stacked children of unknown height
- text that wraps at a width the layout decided, not one declared in advance
- `fit-content`: width clamped to the smaller of max-content and what the parent allows

`.letters()` stays. It is the right tool whenever the count is genuinely known ahead of time, it
costs nothing to resolve, and it says what it means. It is no longer the only way to be sized by
text.

## Anchors

**Anchor cycles stay disallowed, and multipass does not change that.** A ↔ B is a contradiction,
not a scheduling problem; running more passes defers the contradiction rather than resolving it.

What the previous restriction got wrong was the *reason*. Cycles were forbidden because multipass
was unaffordable; they are forbidden because they have no answer. Forward references — anchoring to
something that resolves later in tree order — were never cycles, and dependency-ordered R2 handles
them without comment.

### A cycle is a hard error, refused where it is made

Detected at the op that would create it — `anchored()` at spawn, or a later `anchor()` — and
**panics**, naming both ends and the source location of each.

Two reasons it is refused there rather than discovered in the resolver:

- **The tree is never in a cyclic state**, so R2's dependency ordering is a valid DAG by
  construction and needs no cycle handling at all.
- **The panic points at the write that caused it**, which is actionable, rather than at two entity
  ids inside a resolve pass, which is not. `SpawnedAt` already carries the caller location needed
  to name both.

Hiding the element instead was considered and rejected as incoherent. An element whose `Location`
cannot resolve has no box, so "hidden" is not a state it enters — it is absence with extra steps,
and it would swallow a programming error that has no correct recovery. A cycle is deterministic:
the same tree produces it every run, so a panic is reproducible rather than a hazard, and the
headless harness catches it in a test rather than in an app.

**Anchoring and content-sizing are different tools, and both stay.** Anchoring reads one
already-resolved box: cheap, exact, and the right answer for "sit below that thing, wherever it
ended up" — which is why it holds up under stacking and wrapping where a fixed offset does not.
Content-sizing answers "be as large as what is inside me". Neither substitutes for the other.

The instinct that anchoring is cheaper than fit-content is correct — one dependency edge versus a
measure — but with the model above, content-sizing is bounded and cheap enough that the choice can
be made on meaning rather than cost.

## What this requires of Rowan

R2 splits into R2a / R2m / R2b. Both sub-passes call the same pure resolver, once per axis, which
is only safe because it *is* pure — there is no accumulated state for a second call to corrupt.

`Context` gains the intrinsic each pass can supply:

- R2a: `intrinsic.width` — max-content, free from the character count
- R2b: `intrinsic.height` — measured at the width R2a produced

Both are reached through the one `content()` source, which asks whichever question its axis is for.

An element animating its `Location` resolves both endpoints in **both** passes, so an animated
content-sized box is no more special than a static one.

## Proof obligations

Pure, against the resolver:

- max-content width from character count, across fonts and sizes
- wrap at a given width → exact line count, including at the boundary and with a single word wider
  than the box
- height from line count and font metrics
- `fit-content` clamping in both directions

Headless:

- a text box sized to wrapped content is correct on the frame it is spawned, with no settling frame
- a container sized to stacked children of differing wrapped heights
- rewriting a string reflows the container in the same frame
- crossing a breakpoint reflows and re-wraps
- an anchored element follows a wrapped element that changed height
- an animating content-sized box lands exactly on target (with `aspen.md`)

## The authoring surface

### Two vocabularies

A placement value is a **source** and a **role**, and they are independent — which is what lets any
source fill any coordinate.

| | |
|---|---|
| **source** | where a number comes from: `20.px()`, `50.pct()`, `2.col()`, `8.letters()`, `anchor().right()`, `content()`, and arithmetic on any of them |
| **role** | what it means for this element: left, right, top, bottom, width, height, centre |

A source is **dimensionless until a role names it**. `20.px()` is neither a position nor a length
on its own; the role decides. This is the good part of the existing design and it is kept.

### Except that a position is not an extent

Two sources are not dimensionless, and pretending otherwise is what made
`width(anchor().left())` mean something absurd rather than nothing at all.

| | Is | Legal in |
|---|---|---|
| **length** | `px` `pct` `col` `letters` `content` `anchor().width()` | any role |
| **coordinate** | `anchor().right()` and the other anchor edges | position roles only |

A length in a position role is measured from the parent's near edge. An anchor's edge is already a
position and stays absolute -- no conversion, and no second coordinate space in the expression. The
one bit that decides it is the source's own type, not an inspection of its terms.

`coordinate - coordinate = length`, `coordinate ± length = coordinate`, and `coordinate + coordinate`
does not compile. So the algebra closes, and an anchor's edges reach a size the only way that means
anything: as the distance between two of them.

### And the axes are not symmetric

Width resolves before height, so a horizontal length is available to a vertical role and a vertical
one is not available to a horizontal role. That is the resolution model, and it is expressible:
`height(2.col())` is a two-column span used as a height, and `width(2.row())` does not compile.

Coordinates never cross at all -- a position on one axis has no reading on the other.

### Role first

```rust
left(anchor().right()).width(140.px())
left(20.px()).right(100.pct() - 16.px())
center_x(50.pct()).width(140.px())
top(0.px()).height(text())
```

Role-first rather than `source.as_role()` because **the role tells you what the value is describing
before you have to parse it**. "My left is my anchor's right" is read in the order it is understood;
"my anchor's right is my left" makes you hold a number in mind before learning what it is for.

### The pairing is a type, not a rule

Each opener returns a distinct type carrying only the completions that are legal with it:

| Opener | Completions |
|---|---|
| `left(..)` / `top(..)` | `.width(..)` / `.height(..)`, or the opposite edge |
| `right(..)` / `bottom(..)` | `.width(..)` / `.height(..)` |
| `center_x(..)` / `center_y(..)` | `.width(..)` / `.height(..)` |

So `left(..).center_x(..)` and `width(..).width(..)` are not rejected — they cannot be written,
because the method is not there. Each of the four legal forms has exactly one spelling.

This replaces `panic!("unsupported combination")` in the resolver (`location.rs:712`, `:783`). The
old surface had the right intent and enforced it a whole frame too late; this is the same
correction already made for `.elevate()` and `.grid()`.

Axis roles are distinct types, so a breakpoint takes horizontal and vertical in that order and
cannot take them the other way round.

### Coordinates, not insets

Every edge role is a coordinate in the parent's space, including `right` and `bottom`. So 16 in
from the right is `right(100.pct() - 16.px())`.

Insets would read more nicely for that one case, but anchor sources are absolute positions, so
insets would put two coordinate spaces in one expression. Uniform coordinates keep
`left(anchor().right())` meaning exactly what it says, and make the arithmetic explicit rather than
folded into a designator.

### `content()` — one intrinsic source

The resolution model makes intrinsic sizing a real value rather than a special case, and it needs
exactly one source, because **which question it asks is decided by the axis**:

```rust
left(0.px()).width(content())                      // as wide as the string wants, unwrapped
top(0.px()).height(content())                      // as tall as it wraps to, at the resolved width
left(0.px()).width(content()).at_most(300.px())    // fit-content: whichever is smaller
```

In a width role it resolves in R2a, where no wrapping has happened and max-content is free from the
character count. In a height role it resolves in R2b, after wrapping. One word, and the width-down /
height-up model supplies the meaning.

Separate `max_content()` and `text()` sources were drafted and collapsed: two names for "ask the
content" when the axis already says which question is being asked.

`.at_least(..)` and `.at_most(..)` clamp any resolved span. They are named that way rather than
`min`/`max` because `width(max_content()).max(300)` puts two unrelated senses of "max" in one
expression — one meaning *the content's natural size*, the other *a ceiling*.

### What is not a requirement

**Preventing "double-counting" is not one.** `right(100.pct() - 16.px()).width(140.px())` resolves
to `[100% - 156, 100% - 16]`: a 140-wide box sitting 16 in from the right, which is exactly what
those two terms declare. Nothing is counted twice.

The previous `USAGE.md` called this "the single most common layout bug", and it was wrong — two
independent declarations composed exactly as written. A grammar contorted to prevent it would be
defending against arithmetic that was already correct.

## Proof obligations — the grammar

Compile-fail doctests, which is where a type-level guarantee is actually demonstrated. Each pins
its **error code** rather than the diagnostic text, so the claim is "this does not compile, for this
reason" and not a snapshot of how one compiler release worded it. They sit on the type that refuses
the case, so the refusal is documented where it is met.

- every illegal pairing fails to compile, one case per combination
- horizontal roles in a vertical slot fail to compile
- a vertical length in a horizontal role fails to compile
- an anchor edge used as a size fails to compile, and two of them subtract to one that works
- two coordinates added fail to compile

`trybuild` was the obvious tool and is the wrong one here: its expected output is the compiler's
full diagnostic, `help:` suggestions included, so nine cases become nine files that a toolchain bump
rewrites, and `cargo test --workspace` starts failing for reasons that are not about foliage.

Pure, against the resolver:

- all four legal forms, per axis
- `anchor()` sources in every role they are legal in, and the length between two anchor edges used
  as a size
- arithmetic on sources, including negative results
- `.at_least` / `.at_most` clamping, and `content()` under an `at_most` behaving as fit-content
- `content()` in a width role and in a height role resolve their two different questions
