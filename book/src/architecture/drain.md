# Ops and the drain

Every change to the tree takes the same road: a verb builds an `Op`, the op waits in the queue, and
the drain applies it. This chapter follows that road, from
[`verbs.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/verbs.rs) through
[`op.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/op.rs) to the drain in
[`fern.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/fern.rs).

## A verb is an op

`Grow` is the trait every write is a method of, and every method is the same three lines: take
whatever names the op needs, build it, push it.

```rust,ignore
// verbs.rs
fn at(&mut self, leaf: Leaf, location: Location) {
    self.queue(Op::Place { leaf, location });
}

#[track_caller]
fn branch(&mut self, under: Leaf, seed: impl Seed) -> Leaf {
    let (leaf, growth) = self.allocate();
    self.queue(Op::Branch {
        leaf,
        growth,
        under,
        bud: Box::new(seed.bud(core::panic::Location::caller())),
    });
    leaf
}
```

`Grow` is implemented for anything that implements a crate-private trait, `Queues`: something that
can take an op and hand out names. The `Grove` and the `Sprig` are the two things that do, which is
the whole of why a change reads identically wherever it is written. The trait is sealed, so it can
be called and never implemented, and the set of things an app can ask for stays closed.

No verb returns an error, and none can fail at the call. The only verbs that return anything return
a name (`plant`, `branch`, `animate`, `tween`, `timer`, `sequence`, `plate`, `pixels`), because a name
is the one thing a later write might need before the frame is over.

## What an op is

`Op` is one enum of thirty-seven variants: growing and pruning, every declaration an element has,
motion, focus, the scheme, the host's clipboard and links, and the watches a `Sprig` sets up. Most
name one element and carry one value.

Ops are moved around a great deal (pushed, drained, matched), so they are kept small. Everything
large an op can carry is behind a pointer: a whole element (`Box<Bud>`), a scheme (`Box<Scheme>`),
and a placement at every breakpoint, which is one pointer wide. A test holds the line:

```rust,ignore
// tests/frame.rs
#[test]
fn a_write_is_small_enough_to_carry() {
    assert!(size_of::<crate::op::Op>() <= 256, "{}", size_of::<crate::op::Op>());
    assert_eq!(size_of::<Location>(), size_of::<usize>());
    assert_eq!(size_of::<crate::Trace>(), size_of::<usize>());
}
```

An op is 152 bytes today; before the large payloads were boxed it was over three thousand.

Not every op comes from an app. Some are the engine's own, queued so that they take their place in
the same order as everything else:

| Op | Queued by | Why it is an op |
|---|---|---|
| `Keyed` | dispatch, for every key | so a key is applied behind anything queued before it and ahead of anything after |
| `Focus` | dispatch, for `Tab` and `Escape` | focus moves in the drain like the verb that moves it |
| `Pasted` | a clipboard read finishing, or the web's paste event | it lands at a moment nothing chose |
| `Arrived` | a file read or a fetch finishing | the same |

## The drain

Step 4 takes the whole queue at once and applies it in order:

```rust,ignore
// fern.rs, shortened
fn drain(grove: &mut Grove) {
    let ops = grove.queue.take();
    let _step = trace_span!("drain", ops = ops.len()).entered();
    for op in ops {
        match op {
            Op::Prune(leaf) => {
                if !grove.tree.is_live(leaf) {
                    dropped("prune", leaf, "not live");
                    continue;
                }
                let gone = grove.tree.wither(leaf);
                grove.aspen.wither(&gone);
                grove.coasting.wither(&gone);
                grove.drift.withered.extend(gone);
            }
            // ... one arm per op
        }
    }
    focus::sweep(grove);
    frond::settled(grove);
}
```

`take` swaps the queue's vector out under its lock, so an op a worker pushes while the drain is
running goes into the next frame's queue rather than into this one's half-read list.

Three properties follow from the shape.

**Order is only arrival.** The drain does not group ops by kind, sort them, or apply one kind before
another. A `grid` either side of an unrelated `at` lands in the order the two were written, and the
same `anchor` lands or is dropped depending only on which side of a `prune` it arrived. The suite has
a test for each; this is the smallest:

```rust,ignore
// tests/frame.rs
#[test]
fn the_drain_is_fifo_within_one_frame() {
    let mut grove = grove();
    let trunk = grove.plant(Stem::new());
    let first = grove.branch(trunk, Stem::new());
    grove.prune(trunk);
    let second = grove.branch(trunk, Stem::new());
    tick(&mut grove);

    assert_eq!(grove.presence(trunk), Presence::Withered);
    assert_eq!(grove.presence(first), Presence::Withered);
    assert_eq!(grove.presence(second), Presence::Planted);
}
```

`first` was grown and then taken down with its trunk. `second` arrived after the prune, found no
trunk, and was dropped, which leaves its name `Planted` for good.

**Every op is checked where it is applied.** Almost every arm begins with the same question, whether
the element it names is live, and many ask a second: whether the element has the thing being
written. A `text` on a panel, a `round` on a polygon, an `at` on a line placed by its ends. The tree's
setters answer that second question by returning whether they applied, so there is one place per
property that knows which elements have it.

**Nothing is refused.** An op that does not apply is dropped: the drain writes one `debug` event
naming the verb, the element and a fixed reason, and moves on. It never panics and never tells the
call site, because the call site was a frame ago and has nothing it could do. What makes this safe
rather than careless is that a dropped op changes nothing: the state is exactly what it would have
been had the op not been written. [Debugging](../usage/debugging.md#why-did-nothing-happen) lists the
reasons.

The exceptions are statements that cannot be true, and they panic rather than drop, because they
are mistakes in the program rather than circumstances of the run. Two of them reach the drain: an
anchor that would close a cycle ([The tree](tree.md#anchors-and-the-one-thing-that-panics)), and
pixels shorter than the size a `load` gave them. The rest (a proportional font handed over as
bytes, a negative aspect ratio, a hue slot out of range) panic where they are written, before any op
exists.

## One writer per property

A motion and a direct write can both want the same property. The drain settles it with one rule: a
direct write **cancels** any motion running on the property it writes, before it writes.

```rust,ignore
// fern.rs
Op::Fade { leaf, opacity } => {
    if !grove.tree.is_live(leaf) {
        dropped("opacity", leaf, "not live");
        continue;
    }
    cancel(grove, "opacity", leaf, Property::Opacity);
    grove.tree.set_opacity(leaf, opacity);
}
```

| Verb | Cancels a motion on |
|---|---|
| `at`, `trace` | where the element is: a box and two ends are two ways of saying it |
| `color` | its fill |
| `opacity` | its opacity |
| `reshape` | its shape |
| `scroll` | where the region is scrolled to, and any coast from the last release |

Because the drain is step 4 and motion is step 5, by the time a motion is advanced nothing has both
a pending write and a running motion. An app never has to remember whether a property is animated
somewhere before writing it: the write wins, and the element is at what was written.

An `animate` on a property that is already moving is not a cancellation but a replacement. The new
motion starts from wherever the element is at that moment, which is what [Motion](motion.md)
describes.

## Arms worth reading

Most arms are a liveness check and a setter. A few do more, and each is the engine keeping one of
its promises:

- **`Plant` and `Branch`** refuse an anchor cycle stated on the seed, grow the element at the name
  that was handed out, and then let a [frond](fronds.md) grow its parts underneath it in the same
  step, so a field is whole in the frame it was planted.
- **`Letter` and `Recolor`** check whether the element is a field. A field is one element to the app
  and seven in the tree, so text and fill written to it are redirected to the part that holds its
  value, and a `text` also moves the caret to the end of what was written.
- **`Scroll`** checks that the destination can move the region at all, then records it in
  `grove.sought` rather than moving anything. It is answered in R4, against the extent R3 measures
  in this same frame, so scrolling to the end of a list grown in this frame lands at its new end.
- **`Arrived`** fills a font, a mark or a picture the name for which was handed out earlier, and
  reports it `loaded` or `missing`. A font arriving is every run measured again, since every run
  composed in the fallback face while it was away.
- **`Repaint`** replaces the scheme and marks every element to be drawn again, and nothing to be
  placed again: every role is resolved against the scheme at extraction.
- **`Focus`** is answered here, against what has been declared so far in this drain. Whether an
  element can take focus is a walk up its ancestry asking whether anything is hidden or disabled,
  and that walk reads declarations rather than last frame's resolution, so an app can show a drawer
  and focus into it in the same frame.

## After the last op

Two things run when the queue is empty.

**Focus lets go of anything that can no longer hold it.** Any op may have hidden, disabled or pruned
whatever held focus, and none of them is obliged to think about focus. `focus::sweep` asks once, at
the end, and reports `unfocused` if it had to let go.

**The fronds put their parts in step.** A field's caret and selection are ordinary elements whose
visibility and placement follow the field's state. Once focus and every write are final, nothing
can change what they read for the rest of the frame, so each field writes its parts once, as
ordinary writes, and resolution places them like anything else. [Fronds](fronds.md) describes it.
