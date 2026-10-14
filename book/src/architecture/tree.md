# The tree

Every element is one slot in `Tree` ([`tree.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/tree.rs)),
which is foliage's own store. An element's name is the index of its slot, and every field an element
declares is a column of those slots: one array per field, so a pass that asks something of every
element reads one array rather than gathering from wherever each element is. There are no systems and
no scheduler. The passes that run over the tree are plain functions called in a fixed order by
[the frame](frame.md), which is the order the engine needs.

## Names

A `Leaf` is an index and a generation:

```rust,ignore
// leaf.rs
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Leaf {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}
```

It is handed out the moment `plant` or `branch` is called, before the element exists, and that is
the property the rest of the design leans on. The name comes from `Naming`
([`naming.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/naming.rs)), one source
shared by the frame and by every `Sprig`, whose counters are all atomic. Taking a name never touches
the tree, which is what lets one be taken on a worker thread through a `Sprig` as readily as inside
`frame`. The index is one that went back when an element withered, the last to go back first, or
else a fresh one past every index named so far. The drain later grows the element into that slot,
and the columns grow to meet a name past their end.

So a name's state is read straight off its slot:

| The slot is | `Presence` | Meaning |
|---|---|---|
| past the end, or at the name's generation and empty | `Planted` | the op that grows it has not been drained yet |
| at the name's generation, holding an element | `Live` | in the tree |
| at another generation | `Withered` | pruned, or taken down with an ancestor |

A `Planted` name stays planted forever if the op that would have grown it was dropped (its trunk
withered before the drain reached it). A withered element's index does go back to `Naming`, but a
generation on, so a stale `Leaf` never comes to address whatever grew after it.

Each element also gets a `Growth`: a number from one atomic counter, taken with the name. It is the
order `plant` and `branch` were *called* in, across every thread, rather than the order the drain
reached them, and it is what settles ties: which of two equal-elevation elements is in front, and
which of two elements in the same place comes first in focus order.

Everything else the engine names comes from atomic counters beside it (motions, sequences, pictures,
fonts, marks), each monotonic and never reused, so a stale handle of any kind is inert.

## What an element is made of

A seed (`Panel::new()...`) builds a `Bud` ([`op.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/op.rs)):
the element's declarations, formed before the element exists. The queue carries it, and
`Tree::grow` writes it into the element's slot. Every element has a value in the same core columns,
so nothing that reads one of these has to ask whether the element is the kind that has it:

| Column | Holds |
|---|---|
| `generation` | which name the slot answers to |
| `flags` | whether the slot holds an element; whether the app grew it or a frond grew it as a part; what it declared that is one bit (visible, disabled, pinned, read-only, masked); and which side tables hold something for it |
| `spawned_at` | the source location of the `plant` or `branch` call, for error messages |
| `growth` | allocation order |
| `chlorophyll` | which renderer draws it: none, panel, text, polygon, line, icon or image |
| `placements` | how it is placed: a `Location` at every breakpoint, or a `Trace` and `Stroke` for a line |
| `elevation` | how far in front of its trunk it sits |
| `gestures` | whether it receives, whether it is intangible, which drags it takes, its hit shape |
| `opacity` | its own opacity, before its ancestry is taken into account |
| `shift` | how far it has been moved off where the layout put it |
| `scrolls` | which axes it scrolls on, if any |
| `trunk`, `first`, `last`, `next`, `previous` | [the hierarchy](#the-hierarchy) |

What a renderer was told, its **pigment** (`PanelPigment`, `TextPigment` and so on: fill,
rounding, shape, mark, plate), is held packed by renderer. Each renderer has one array as long as
what draws with it, and an element's `pigment` column says where in that array its own sits. When
one withers, the last item is moved into its place and its owner is told where it went.

What only some elements state is held in a **side table**, and only for those that state it: `Grid`,
`Offset` and `Extent` for a region, `Focusing`, `Typeface`, `Anchored`, `Floats`, `Lettering` and
`Tints` for a run, and a field's `Parts`, `Editing` and `Keypad`. Each table owns a bit of the slot's
flags, set exactly while it holds something there, so an element that states nothing is answered by
the bit rather than by a lookup.

`Chlorophyll` deserves a word. It is set when the element is grown and there is no op that can
change it: what an element draws is part of what it is, and an element that is to draw something
else is a different element. It is also the *only* thing that says what an element is. Extraction
walks the elements once and routes each on its chlorophyll, so a further renderer costs a variant
rather than a pass, and a set of declarations that happens to look like a panel is not a panel.

## The hierarchy

`branch` hangs an element off its trunk. `trunk` holds what each element hangs off, and what hangs
off an element is a list threaded through the slots: `first` and `last` on the trunk, `next` and
`previous` on each child, in the order they were grown. Letting go of one child of thousands is a
few writes rather than a search. The tree reads both ways: `trunk(leaf)` for what an element hangs
off, `branched(leaf)` for what hangs off it.

Some children are not the app's. A `TextInput` is one element to the app and seven in the tree:
the field itself, and under it a run, a placeholder, a caret, and three panels for a selection that
can span lines. [Fronds](fronds.md) grow those parts as ordinary children with `grow_part`, which
is `grow` without the `GROWN` flag. `branches` (and the `Vein::Branches` read) lists only children
with that flag, so it lists only what the app grew, while everything that resolves, draws and withers
treats the parts like any other element.

## Declarations and resolved values

The tree holds **what was declared**: what the app said about each element, plus the few values
the engine keeps on its behalf and writes through the same setters (a region's offset and extent, a
field's caret). It does not hold what resolution makes of them. Boxes, clips, ranks and the
inherited off-states live in resolution's own arrays, `Elements`, described in
[Resolution](resolution.md), and are never written back into the tree. The line is sharp: a value in
the tree is something an op can write, and a value in `Elements` is something only a pass can.

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

The most important thing `Tree` does is not store declarations. It is to **record every write, where
the write happens**, so that resolution can leave alone everything nothing wrote to:

```rust,ignore
// tree.rs
pub(crate) struct Tree {
    // ...the columns and side tables above, then:
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
everything under it (the app's children and any parts a frond grew), walking the child lists, and
empties each slot: its pigment and side-table entries go, and its generation moves on. The indices
go back to `Naming` together. Every name that went is reported three
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
