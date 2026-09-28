# The tree

Every element is one entity in a [`bevy_ecs`](https://crates.io/crates/bevy_ecs) `World`, and
`Tree` ([`tree.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/tree.rs)) owns that
world. It is the only place in the engine that touches `bevy_ecs` directly, and it touches very
little of it: the world as a store of components, the parent and child relationship, and the entity
allocator. There are no bevy systems and no schedules. The passes that run over the tree are plain
functions called in a fixed order by [the frame](frame.md), which is the order the engine needs and
not one a scheduler chose.

## Names

A `Leaf` is a bevy `Entity` and nothing else:

```rust,ignore
// leaf.rs
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Leaf(pub(crate) Entity);
```

It is handed out the moment `plant` or `branch` is called, before the element exists, and that is
the property the rest of the design leans on. The name comes from the world's own entity allocator,
reached through a `RemoteAllocator` ([`naming.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/naming.rs))
that can reserve an entity without touching the world. That is what lets a name be taken on a worker
thread through a `Sprig` as readily as inside `frame`. The drain later spawns the element *at* that
reserved entity.

So a name's state is read straight off the allocator:

| The entity is | `Presence` | Meaning |
|---|---|---|
| reserved, not yet spawned | `Planted` | the op that grows it has not been drained yet |
| spawned | `Live` | in the tree |
| neither | `Withered` | pruned, or taken down with an ancestor |

A `Planted` name stays planted forever if the op that would have grown it was dropped (its trunk
withered before the drain reached it). A withered entity's index does go back to the allocator, but
bevy's generation tells the old name from the new one, so a stale `Leaf` never comes to address
whatever grew after it.

Each element also gets a `Growth`: a number from one atomic counter, taken with the name. It is the
order `plant` and `branch` were *called* in, across every thread, rather than the order the drain
reached them, and it is what settles ties: which of two equal-elevation elements is in front, and
which of two elements in the same place comes first in focus order.

Everything else the engine names comes from atomic counters beside it (motions, sequences, pictures,
fonts, marks), each monotonic and never reused, so a stale handle of any kind is inert.

## What an element is made of

A seed (`Panel::new()...`) builds a `Bud` ([`op.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/op.rs)):
the element's components, formed before the element exists. The queue carries it boxed, and
`Tree::grow` inserts it. Every element gets the same core, so nothing that reads one of these has to
ask whether the element is the kind that has it:

| Component | Holds |
|---|---|
| `Grown` | marks an element the app grew, as against a part an element grew under itself |
| `SpawnedAt` | the source location of the `plant` or `branch` call, for error messages |
| `Growth` | allocation order |
| `Chlorophyll` | which renderer draws it: none, panel, text, polygon, line, icon or image |
| `Location` | where it sits, at every breakpoint |
| `Grid` | how it divides its box for its children |
| `Elevation` | how far in front of its trunk it sits |
| `Gestures` | whether it receives, whether it is intangible, which drags it takes, its hit shape |
| `Focusing` | its focus order and whether it is a focus scope |
| `Visible`, `Opacity`, `Disabled` | its own off-states, before its ancestry is taken into account |
| `Offset`, `Extent` | where a region is scrolled to, and how far its content reaches |

And, only where they apply: the renderer's **pigment** (`PanelPigment`, `TextPigment` and so on:
fill, rounding, shape, mark, plate), `Lettering` for a run of text, `Tints`, `Scrolls`, `Pinned`,
`Floats`, `Trace` and `Stroke` for a line, `Typeface` for an element that states a font or a size,
`Anchored` for one with an anchor, and bevy's `ChildOf` for one grown under another.

`Chlorophyll` deserves a word. It is set when the element is grown and there is no op that can
change it: what an element draws is part of what it is, and an element that is to draw something
else is a different element. It is also the *only* thing that says what an element is. Extraction
walks the elements once and routes each on its chlorophyll, so a further renderer costs a variant
rather than a pass, and a set of components that happens to look like a panel is not a panel.

## The hierarchy

`branch` records the trunk with bevy's `ChildOf`, and bevy keeps the reverse `Children` list. The
tree reads both ways: `trunk(leaf)` for what an element hangs off, `branched(leaf)` for what hangs
off it.

Some children are not the app's. A `TextInput` is one element to the app and seven in the tree:
the field itself, and under it a run, a placeholder, a caret, and three panels for a selection that
can span lines. [Fronds](fronds.md) grow those parts as ordinary children. They are not marked `Grown`, so `branches` (and the `Vein::Branches` read) lists only what
the app grew, while everything that resolves, draws and withers treats the parts like any other
element.

## Declarations and resolved values

The world holds **what was declared**: what the app said about each element, plus the few values
the engine keeps on its behalf and writes through the same setters (a region's offset and extent, a
field's caret). It does not hold what resolution makes of them. Boxes, clips, ranks and the
inherited off-states live in resolution's own arrays, `Elements`, described in
[Resolution](resolution.md), and are never written back into the world. The line is sharp: a value in
the world is something an op can write, and a value in `Elements` is something only a pass can.

`Grove::tap` reads across both, and always returns a copy:

```rust,ignore
// grove.rs, shortened
pub fn tap(&self, leaf: Leaf, vein: Vein) -> Option<Sap> {
    if !self.tree.is_live(leaf) {
        return None;
    }
    Some(match vein {
        Vein::Placed => Sap::Section(self.elements.placed(leaf)),   // resolved
        Vein::Drawn => Sap::Section(self.elements.drawn(leaf)),     // resolved
        Vein::Visible => Sap::Visible(self.tree.visible(leaf).0),   // declared
        Vein::Text => Sap::Text(/* the run's lettering */),        // declared
        // ...
    })
}
```

That is why `Vein::Visible` reads what the element itself declared rather than whether it is shown:
the product over its ancestry is resolution's, and the read answers the question the element can
be asked.

## Recording what was written

The most important thing `Tree` does is not store components. It is to **record every write, where
the write happens**, so that resolution can leave alone everything nothing wrote to:

```rust,ignore
// tree.rs
pub(crate) struct Tree {
    world: World,
    touched: Leaves,     // declarations resolution reads were written
    restyled: Leaves,    // something about how it is drawn changed, and nothing about where
    lettered: Leaves,    // its run was rewritten, or it was grown: R1 measures it again
    reached: Leaves,     // what was under it went away: it measures again
    restated: bool,      // some run stopped being stated: sweep the shaping cache
    restructured: bool,  // an element or an edge came or went: build the order again
    invalidated: bool,   // something every element reads changed: resolve everything
    repainted: bool,     // something every element is drawn against changed: draw everything
}
```

Every setter decides which of those it is, and **compares before it records**: writing an element
back to what it already holds marks nothing, because resolving it again would produce the same
answer and asking is the cost of the element and of everything resolved against it.

| Write | Recorded as |
|---|---|
| `at`, `trace`, `grid`, a caret moving | `touched` |
| `anchor` | `touched`, and `restructured`: an anchor is an edge of the graph |
| `text` | `touched`, `lettered`, and `restated` |
| `color`, `round`, `tint`, `reshape`, `mark`, `depict`, `fit` | `restyled` |
| `visible`, `opacity`, `disable`, `enable`, `elevate` | `restyled` |
| growing an element | `touched`, `lettered`, `restructured` |
| pruning an element | its trunk `reached`, `restructured`, `restated` |
| a resize, a crossed breakpoint, a font arriving, a density change | `invalidated` |
| `repaint`, a picture or a mark arriving | `repainted` |

The split between `touched` and `restyled` is where most of an ordinary frame's savings come from.
A fill, a rounding or a tint is read only by extraction. Whether an element is shown, how opaque it
is, whether it is disabled and how far forward it sits are read by passes that run over every element
anyway. None of them moves a box, so none of them is a reason to place anything again: such an
element is read again and drawn again, and its subtree is left exactly where it was.

Resolution borrows the record through `written()` and clears it with `taken()` **the moment it has
read it**, not at the end of the frame. A write made by anything that runs after that point belongs
to the next frame, which is the frame that would resolve it.

## Withering

`prune` is the one write that removes. The drain calls `Tree::wither`, which gathers the element and
everything under it (the app's children and any parts a frond grew), despawns the element, and
bevy despawns the rest through the parent relationship. Every name that went is reported three
places: to the running motion, which drops anything moving on those elements; to the coasting
scrolls, which drop any region that went; and to the drift, which becomes `withered` in the next
frame's `Pollen`.

The trunk is recorded as needing to measure again. An element sized to its content has one fewer
child to reach over, and the element that went is exactly the one that would have told it.

## Anchors, and the one thing that panics

An anchor is an edge in the dependency graph, and a cycle of anchors has no answer: A placed against
B placed against A. Rather than detect it during resolution and invent a box, the drain refuses the
anchor that would close the cycle, and refuses it loudly:

```text
anchor cycle: leaf 4294967290 cannot anchor to leaf 4294967292, which already reaches back to it
  leaf 4294967290 was planted at src/main.rs:40:21
  leaf 4294967292 was planted at src/main.rs:31:20
  the anchor was written at src/main.rs:58:15
```

The locations are real because `plant`, `branch`, `anchored` and `anchor` are `#[track_caller]`: each
records `core::panic::Location::caller()` into the bud or the op, and the element keeps it as
`SpawnedAt`. The check is a walk along anchors (`Tree::reaches`), which always terminates because the
tree it walks was acyclic before this write.

Refusing at the write is what lets resolution order everything by dependency with no cycle handling
of its own. Hiding the element instead would be absence with extra steps: a placement that cannot
resolve has no box to fall back to, and swallowing the mistake would leave nothing correct to
recover to.
