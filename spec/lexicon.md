# Lexicon

The vocabulary every other spec document is written in. Settled first because nothing else can be
written until it is.

This is not an exercise in botanical accuracy. The point is **distinct themed names that carry the
right connotation**, so any part of the system is easy to name and easy to find. Where the metaphor
earns its keep it should be true; where being exactly right would cost a good name, the good name
wins.

## The rule

> **Structure words name the tree. Species and stand words name the machinery.**

Leaf, branch, stem, trunk, root describe the thing an app builds. Ash, Willow, Ginkgo, Grove
describe the parts of foliage that build it.

The test is mechanical: read a name, and you know which register it belongs to before you know what
it does. A `Leaf` is something in the app's tree. A `Rowan` is a subsystem. There is never a
question of which.

Species names are **loose and associative, not strict backronyms** — a tree whose name sounds like
what it does. `Ash` for the Aesthetics System Handler works because it reads as a tree first and an
acronym second. Anything that has to be explained to be understood has failed.

### What this rules out

Tree *parts* as subsystem names. `Cambium` for the resolve pipeline and `Phloem` for the extract
path were proposed and are wrong for exactly this reason: they are parts, so they read as structure,
and a reader would look for them in the element vocabulary. They belong to neither register as
named — the subsystems get species names instead.

## Structure — what an app builds

| Name | Is | Notes |
|---|---|---|
| `Foliage` | The engine instance | All of it. The growth, and the thing that grows it |
| `Root` | The app trait | One root system beneath many trees — literally true of an aspen stand, so this is not a stretch |
| `Grove` | The per-frame surface | What you plant into and read from. See below |
| `Leaf` | A name for one element | Any element, not only a childless one. See *Known imprecision* |
| `Seed` | An element described before it exists | `Stem::new()` is a `Stem`, and a `Stem` is a `Seed`. Sealed. See below |
| `Stem` | An element that draws nothing | Holds children, takes hits, has a box. Replaces `Bare` |
| `Sprig` | A cutting you carry off-thread | Keeps its name |
| `Sap` | What a read returns | Was `Sample` |
| `Vein` | What you ask a read for | Was `Sap`, which had it backwards — you tap a vein *for* sap |
| `Pollen` | What the tree put out this frame | Was `Moss`. A set you interrogate, not a list you walk — see `pollen.md` |

### Verbs

| Verb | Does |
|---|---|
| `plant(seed) -> Leaf` | Grows a top-level element. Was `leaf()`, which collided with the `Leaf` type |
| `branch(under, seed) -> Leaf` | Grows an element under another |
| `prune(leaf)` | Takes it and its subtree down; each one `Withered`s |
| `tap(leaf, Vein::X) -> Option<Sap>` | Reads one property |

You plant a tree, it grows branches, branches carry leaves. The verbs follow the structure rather
than describing it from outside.

### `Grow`

The trait carrying every write, implemented by `Grove` and `Sprig` so a change reads identically
wherever it is issued.

Named `Grow`, not `Grows` — `Grove: Grows` reads backwards, and Rust's own capability traits are
bare verbs (`Read`, `Write`, `Display`). The verb was always right; only the conjugation was wrong.

Sealed: it can be called, never implemented, so the set of things an app can ask for stays closed
and reviewable.

### `Seed`

What `plant` and `branch` accept. `Stem::new()` is a `Stem`; that a `Stem` *is a* `Seed` is what
lets `plant` take it.

A noun rather than a verb, on the same reasoning that named `Grow`: `Stem: Sow` would read
backwards, because the app sows the stem and not the reverse. `Stem: Seed` reads forwards.

Sealed, so the set of things that can be grown stays closed. The private half is `Buds`, and "a
`Stem` buds" reads forwards too.

### `Grove`

The per-frame surface: what the `Root` is handed, what it reads, and what it plants into.

`Forest` was the wrong scale — `Foliage` is already the whole growth, so a second word for
"all of it" was redundant. A grove is a stand of trees you tend as one thing, which is what this is.

**A grove is the whole of what an app can touch.** Every write and every read is here. Nothing that
*advances* the engine is: an app does not own the loop, so it holds no verb that runs a frame,
moves the clock, or resizes the surface. Those are the platform's, reached from inside the crate.

`Foliage` is the boot object rather than a second surface. It is constructed, told the desktop
window size, given the app's `Root` and its one-time fonts and assets, and then it runs. **Nothing
on `Foliage` names a `Leaf`.**

A grove with no window is still a grove — same type, same verbs, same reads, constructed
identically. That the harness and the real frame surface are one type is not a convenience; it is
what makes a test's evidence worth anything. See `harness.md`.

### `Pollen`

What the tree released this frame: clicks, keys, finished animations, resizes, witherings.

`Moss` had no sense to it — moss grows on a tree, it does not come off one and it reports nothing.
Pollen is released outward, drifts, and is collected somewhere else, which is what an emission is.

**It is a mass noun, and that is the point.** There is no `Vec<Pollen>` and no singular, because an
app never handles one emission at a time — it interrogates the whole drift: `pollen.clicked(leaf)`.
The countability objection that sank this name as an enum is what makes it right as a set.

See `pollen.md` for why the set is unordered and what that buys.

## Machinery — what foliage is made of

Species names, alliterating with their function.

| Name | Is | Reading |
|---|---|---|
| `Fern` | The frame | Frame. One sequence, and every other subsystem is ordered by it |
| `Ash` | Render backend | Aesthetics System Handler |
| `Ginkgo` | GPU layer | Gpu instructions |
| `Willow` | Window and event loop integration | Window |
| `Rowan` | The resolve pipeline | Resolve |
| `Elm` | Change extraction, feeding `Ash` | Extract. Was `Differential` |
| `Aspen` | Animation and timing | Animation |
| `Lichen` | The opinionated widget layer | Grows on foliage. Keeps its name |
| `Tree` | The internal ECS door | The one place raw `bevy_ecs` is touched |
| `Bud` | A queued element, formed and not yet open | What the queue carries until the drain grows it |

`photosynthesize()` keeps its name: light in, growth out, and it does not return.

`Fern` is a subsystem in name only — it has no state, because everything a frame touches lives in
the `Grove`. It is a module and one function, and becomes a type on the day it owns something.

`Tree` and `Bud` are the structure words on this side of the line, and they earn the exception:
neither is a subsystem. `Tree` is the tree itself, seen from the inside. A `Bud` is an element
formed and not yet open — which is what a queued op holds, and what `Presence::Planted` names from
the `Leaf`'s side. Nothing outside the crate can name either.

## Names that stop existing

`Author`, `Sprout`, `LeafSprout`, `Spec`, `WithExtras`, `Refire`, `IntoTargets`, `TargetedEvent`.

Six names and a trait hierarchy for one idea — *the thing you configure before it exists*. They
were the price of letting outside code register its own reactive rebuilds, which nothing has done
since the boundary landed. `Panel::new()` is a `Panel`, a `Panel` is a `Seed`, and `plant` and
`branch` consume it. See `internals.md`.

`Bare` and `Node` fold into `Stem`. `Sample` becomes `Sap`, and the old `Sap` becomes `Vein`.

## Known imprecision

**A `Leaf` names any element, including one with children.** Botanically a leaf is terminal, and an
element with children is closer to a branch. This is accepted rather than fixed: "leaf" reads as
"named node in a tree" in general use, `branch` is already spoken for as the verb that makes one,
and the alternative is a second handle type splitting the API in half for a distinction no caller
acts on.

Where the difference matters, the structure says it: a leaf is *branched off* another, or *planted*
on its own.

## Deliberately unnamed

Layout, interaction, text, and asset handling are modules named for what they do. Not everything
earns a species name — a name that has to be learned before code can be read is a cost, and it is
only worth paying at a real subsystem boundary. Four such boundaries exist (`Ash`, `Ginkgo`,
`Willow`, `Rowan`), plus `Elm` and `Aspen` which cross phases and so need naming to be discussed at
all.

If layout later earns one, it takes a species name under the same rule.
