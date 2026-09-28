# Extraction

Step 8 turns resolved elements into render instances, and only where something changed. It is
`Elm`, in [`elm.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/elm.rs), and the
module's first line is its whole idea: **Rowan resolves. Elm decides what changed.**

## The cache is what the backend holds

For every renderer, `Elm` keeps a copy of what the backend is holding: each instance's value, its
rank, and its clip, keyed by element. Extraction states what each element should be drawn as now and
compares it against that copy. An element that has not changed costs one comparison and no upload.

What is cached is deliberately *what the backend holds*, and not what the element last was. That
distinction makes two guarantees:

- **A skipped frame cannot lose a change.** There is no dirty flag to miss, only a value that still
  differs at the next comparison. If the loop idles through a change, the next frame that runs finds
  it.
- **The backend must apply every batch.** A batch that was produced and dropped would leave the cache
  claiming something the GPU does not have, and nothing afterwards would disagree with it. That is why
  [the loop](loop.md#frames-happen-inside-paints) absorbs a batch immediately after the extraction that
  made it, in the same branch, whether or not the paint that follows succeeds.

## The walk

```rust,ignore
// elm.rs, shortened
pub(crate) fn run(grove: &mut Grove) {
    grove.elm.open();
    let elements = core::mem::take(&mut grove.elements);
    let recut = core::mem::take(&mut grove.elm.recut);
    // Whatever the backend held for an element that withered, it holds for nothing now.
    for &(leaf, chlorophyll) in &elements.gone {
        grove.elm.withdraw(chlorophyll, leaf);
    }
    let surface = Section::new(Position::default(), grove.viewport);
    for at in 0..elements.len() {
        let chlorophyll = elements.standing[at].chlorophyll;
        if chlorophyll == Chlorophyll::None {
            continue; // a stem draws nothing
        }
        if !(elements.moved[at] || recut) {
            continue; // nothing extraction reads about it moved
        }
        let stated = match painted(&elements, at, surface) {
            Some(painted) => state(grove, /* ... */),
            None => false,
        };
        if !stated {
            grove.elm.withdraw(chlorophyll, elements.order[at]);
        }
    }
    // ...
}
```

Extraction walks resolution's order once and routes each element on its `Chlorophyll`, so a further
renderer costs a match arm rather than another pass. It visits only elements resolution marked
`moved`. A frame that moves nothing does nothing here, whatever the size of the tree.

What the backend is told to let go of is said outright, and there are exactly two ways to earn it:
an element that withered (resolution lists those in `gone` when it rebuilds its order), and an element
that moved and is no longer painted. There is no per-frame sweep of everything held.

## Painted, or not

```rust,ignore
// elm.rs
fn painted(elements: &Elements, at: usize, surface: Section) -> Option<Painted> {
    let inherited = elements.inherited[at];
    let section = elements.drawn[at];
    let clip = elements.clip[at].intersect(surface);
    if !inherited.visible || section.intersect(clip).is_empty() {
        return None;
    }
    Some(Painted { section, clip, opacity: inherited.opacity })
}
```

An element is painted if it is visible over its whole ancestry and some part of its drawn box is
inside its clip, which is never wider than the surface. **Culling is decided here, from the clip
rectangle, and recorded nowhere.** An element scrolled out of its region is absent from the batch
and unchanged in every other respect, so scrolling back to it needs nothing undone, and nothing that
reads state (the extent, above all) can ever see "currently culled".

Two more things are not drawn yet rather than not drawn: an icon whose distance field has not arrived
and an image whose pixels have not. Each occupies its box, is absent from the batch rather than held
as blank, and appears on the frame its asset lands, with nothing to undo.

## What is stated

For each painted element, extraction builds what its renderer needs, entirely in logical pixels:

| Element | Stated as |
|---|---|
| panel | its drawn box, its color, its corner radii |
| polygon | its drawn box, its color, its shape (sides, rounding, rotation) |
| line | its two ends as resolved and scrolled, its color, its weight and cap |
| icon | the largest square its box holds, its color, which mark |
| image | the box and crop its `Fit` produces, its corner radii, its opacity, which picture |
| text | a run: every inked glyph's cell and character and color, its face and size |

**The color is resolved here**, because this is where a fill becomes a color:

```rust,ignore
// elm.rs
fn tint(grove: &Grove, leaf: Leaf, fill: Fill) -> Color {
    let target = fill.color(&grove.scheme);
    match grove.aspen.fill(leaf) {
        Some((Departed::Declared(fill), at)) => fill.color(&grove.scheme).blend(target, at),
        Some((Departed::Snapshot(color), at)) => color.blend(target, at),
        None => target,
    }
}
```

A role is looked up in the scheme in force; a color motion in progress is blended here, with both
ends read against the same scheme at the same instant; and the result is faded by the element's
inherited opacity. So a repaint is extraction's alone: the scheme is replaced in the drain, every
element is marked moved, and every fill is resolved again here, with nothing placed again.

## Compared, and handed on

Each renderer's instances go through the same small comparison:

```rust,ignore
// elm.rs
fn want(&mut self, key: impl Into<Key>, rank: ResolvedElevation, clip: Section, instance: I) {
    let key = key.into();
    match self.held.get_mut(&key) {
        Some(held) => {
            if held.instance == instance && held.rank == rank && held.clip == clip {
                return;
            }
            held.instance = instance;
            held.rank = rank;
            held.clip = clip;
        }
        None => {
            self.held.insert(key, Held { instance, rank, clip });
        }
    }
    self.written.push(Stacked { key, rank, clip, instance });
}
```

The rank and the clip travel beside the instance rather than inside it. Where an element sits in the
stack is a fact about the element, not about what it draws, and the backend needs it in a different
form from the one resolution produced; a clip says where the backend may paint, which it applies to
the pass rather than to the instance. Keeping both out is also what lets an instance be exactly the
bytes a vertex buffer takes.

A run of text is compared whole (its glyphs, face, size, rank and clip), because a run's glyphs move,
refill and restack together, and finding which of them changed would cost more than rewriting the
run.

The batch is the `written` and `withdrawn` lists of every renderer. The withdrawn lists are sorted
before they are handed on, so two identical runs of the engine extract identically whatever order the
elements that went were found in.

## Logical here, device pixels below

A panel and a polygon are described entirely in logical pixels, so what is compared here is also what
the vertex buffer holds. The other four are not, each for the same reason: turning what the element
declares into what the GPU draws needs the display's density, and the density stops at the backend.

| Element | Declares | The backend derives |
|---|---|---|
| text | cells and characters | the cut ink, snapped to device pixels |
| line | two ends and a weight | a segment, axis-aligned ones snapped to whole device pixels |
| icon | a box and a mark | the mark's place on the sheet, and its distance range on screen |
| image | a box and a picture | which texture to bind |

So extraction compares logical values, and the derivation happens once per written instance rather
than once per frame. The cost is one rule: when the density changes (a window dragged to another
display) the logical values do not, and would compare equal forever against instances derived for a
density that is gone. So a density change, and a backend rebuilt from nothing after an Android
suspend, both call `recut`, which empties every cache. The next extraction has nothing to compare
against and states every element.

## In the trace

The `extract` span records how many instances were written, how many withdrawn, how many glyphs the
written runs held, and how many elements were `kept`: visited by the walk and skipped because nothing
about them moved. On an idle frame everything is kept and nothing is written, which is the number
that says an unchanged frame really cost nothing.
