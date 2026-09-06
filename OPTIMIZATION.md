# Optimisation

Candidates, not defects. Each is deliberate as built — the shapes below are what to look at when a
frame is too expensive, and what each one would cost to move. A candidate that has been measured
carries its measurement; for the rest, the first work is a measurement rather than a change.

## Measuring one

`cargo run -p foliage --release --example stress -- <load> <cells>` saturates one phase of the frame
at a chosen tree size and writes what each phase cost, taken from the engine's own spans. `idle`
writes nothing and is the floor for a tree of that size; every other load is that floor plus one kind
of write.

Every step of the frame states a span, so a phase's children sum to it and what is left unattributed
is nothing. A cost is reported per frame whatever a phase's call count, so the two runs of an axis
read as the one cost the frame paid for them.

`draw` opens by acquiring the surface, so what it reports is the wait for the display rather than
work. What a frame costs on the processor is `frame` plus `absorb`. A share is of all three, which
is the wall clock — so on a run held to the display's cadence a phase made cheaper moves time into
`draw` rather than shrinking the total. **A change shows in the millisecond column and not in the
share.**

## Rowan resolved the whole tree every frame

**Done.** The frame's cost was the tree's size rather than the change's. It is now the change's, for
the four passes that answer one element at a time; what is left of the tree's size is measured at the
end of this section.

The rest of this section is the reading that led there, kept because it is what any further work on
the same shape is measured against.

**Measured.** `layout`, which rewrites every element's placement, against `idle`, which writes none.
Resolution cost the same in both at every size, so all of it was the tree's and none of it was the
change's:

| elements | 512 | 1024 | 2048 | 4096 | 8192 | 16384 |
| --- | --- | --- | --- | --- | --- | --- |
| nothing written | 0.37 | 0.71 | 1.41 | 3.05 | 6.83 | 14.52 |
| every placement rewritten | 0.36 | 0.71 | 1.46 | 3.20 | 6.61 | 14.45 |

Milliseconds a frame, on a Radeon RX 7900 XTX under Vulkan. The cost is linear at 0.69µs an element,
reaching 0.89µs at the largest size. It is 69% of what a frame costs when nothing at all has
changed, and a 60Hz budget runs out at around 13000 elements that are not moving.

**Measured against the change's size.** The same load and the same tree, writing to less and less of
it — 4096 elements, milliseconds a frame, before any of it was skipped:

| written | all | 1 in 4 | 1 in 16 | 1 in 64 |
| --- | --- | --- | --- | --- |
| `drain` + `root` | 1.62 | 0.42 | 0.12 | 0.04 |
| `resolve` | 3.18 | 3.33 | 3.21 | 3.27 |
| `extract` | 0.89 | 0.91 | 0.89 | 0.92 |
| `frame` | 6.15 | 5.10 | 4.66 | 4.71 |

What the app writes falls with what it writes. Nothing else moves. At one element in sixty-four,
**4.6 of a 4.7ms frame is spent on the elements that did not change** — and extraction is in it as
much as resolution, because a difference it declines to send still costs the walk that found it.

### What skipping the unwritten was worth

The tree records a write where the write happens, and the order closes it over the dependencies:
an element resolves again if it was written to, or if what it hangs off or is anchored to does. R2m
closes the other way as well — up the trunks, because what an element reaches over is what the
elements under it resolved to — and a measure that moves under an element nothing wrote to makes that
element resolve again, which R2b is placed after R2m to pick up.

The constraint that made this look expensive was R2: an anchor may point anywhere, so a dirty set has
to close over everything anchored into it. It cost nothing to satisfy. The order is already built by
dependency and already holds each element's trunk and anchor as positions, so the closure is one walk
of it — the same walk that reads the columns — and the anchors close in it for free.

**Two kinds of change, spreading two ways.** What an element was *written to* travels to everything
resolved against it, which is down. What an element *measures to* travels toward its trunk, and no
further — unless the measure moves, which R2m finds out and says so, and R2b spreads from there.
Recording a wither as the first kind rather than the second was worth 39% of resolution under `churn`
on its own: a trunk marked as written to takes every one of its children with it, and a list adding a
row would have re-resolved the list.

What the passes that answer one element at a time cost, at 4096 elements and the same shares:
`measure` 0.47ms to 0.01, `axis` 0.73 to 0.04 across both of its passes. Where most of the tree was
written to, working out what to leave alone costs more than resolving it does — a lookup for every
element and a skip that almost never takes — so past a quarter written the closure is not built at
all, and that is what holds the cost of a frame that changes everything to a few percent.

### What a frame costs, end to end

4096 elements, milliseconds a frame, against the same tree resolving and extracting all of itself
every frame:

| load | resolving everything | resolving what was written | |
| --- | --- | --- | --- |
| nothing written | 4.57 | **1.49** | −67% |
| every placement rewritten | 6.17 | **5.73** | −7% |
| one placement in 64 rewritten | 4.88 | **1.98** | −60% |
| every cell in motion | 6.10 | **5.78** | −5% |
| one cell in 16 in motion | 4.86 | **2.05** | −58% |
| 32 of 2048 cells regrown | 5.68 | **3.44** | −39% |
| every run rewritten | 5.66 | **5.47** | −3% |

Nothing is slower. A frame that writes to all of it pays for a closure it cannot use, and gets that
back and a little more from the three fixed costs below, which fell for every frame rather than only
for the quiet ones:

- **The read is kept between frames.** Growing, withering and anchoring are the only things that
  change the order, so an ordinary frame repairs it rather than reading it: `elements` 0.48ms to
  0.02 with nothing written, and 0.56 to 0.10 with every cell in motion.
- **A run is shaped to say it is still wanted.** The shaping cache is swept of what nothing stated
  this frame, so skipping an element there evicts the glyphs extraction is still drawing — which is
  what the suite reported the first time it was tried. A frame where no run *stopped* being stated
  has nothing to sweep, so it shapes only what it is measuring: `wrap` 0.37ms to 0.02 with nothing
  written.
- **Extraction states what moved.** R4 through R7 report whether what they wrote differs from what
  was held, and an element none of them moved is said to be unchanged rather than rebuilt and
  compared. The same withdrawal contract as the sweep, answered the same way: what is not said is
  withdrawn, so an element left alone is still said. `extract` 1.01ms to 0.14 with nothing written.

### What the passes spend it on

Not arithmetic — reaching for one element at a time. Two costs of about the same size, each around
ten nanoseconds against a few for the sums between them: a component read, which was a lookup by
entity; and a step of an accumulation, which was a hash on a map keyed by element.

Both are read by position now. The tree is read once a frame into `Elements` — every live element in
dependency order, holding what it depends on as a position in that order and holding what it offers a
placement beside it — and every pass indexes into that. The accumulations R3 through R7 carry are
arrays the length of the order, and a trunk's contribution is `[..]` rather than a hash. A write that
used to go through `insert` — the bundle machinery, which is what a component arriving for the first
time needs and not what one being overwritten every frame does — is made in place, since `grow` puts
every one of them on the element when it is planted.

Together those took 42% off resolution. At 4096 elements: `clip` 0.49ms to 0.11, `scroll` 0.60 to
0.16, `rank` 0.39 to 0.13, `inherit` 0.44 to 0.16, `axis` 1.54 to 0.79 across its two passes, and
`extent` 0.40 to 0.25.

### The read itself

`elements` costs 0.45ms at 4096 elements, and is flat at 0.11µs an element across every size above.
The sort of every entity in the world that opens it is 0.03 of that, so what it spends is the reading
rather than the ordering.

It was 0.95 before it was one read. Two thirds of that was an ordering pass that asked each element
for its trunk and its anchor twice over, kept a map of how many dependencies each element was waiting
on and a vector per element of what waited on it, and ran **outside every span** — so none of it
appeared in a breakdown, and a pass that cost more than any other in resolution read as nothing at
all. Asking each element once, keying the sort by position among the leaves, and holding what waits
on what as one array rather than a vector per element took it to a single pass that shows.

### R2m shapes in one walk and reaches in another

R2m takes the greater of two measures, and only one of them has an order. What an element's own run
wraps to reads nothing but that element, so it is `shaped`, in any order; what reaches below it reads
the measures of everything under it, so it is `reach`, up the order after them. At 4096 elements
`wrap` is 0.76ms, and it divides 0.29 to the first and 0.47 to the second.

That is the answer to where R2m's cost is, and it is not where the deferral below assumed. `reach`
visits each element exactly once — as the one child of its own trunk — so there is no repeated
question in it to hoist out. What it spends is a `branched` walk and a `measurable` read per child,
against a resolve that is genuine arithmetic.

### R1 and `shaped` read the world element by element

Both reach for a typeface and a run, and the run is handed out by reference rather than copied. A
reference into the world cannot be held across a write to it.

**The write half is settled.** Neither pass writes as it walks any more: each states what it measures
into the order, and `scatter` writes `Cell` and `Intrinsic` back once, after both have run. That took
`wrap` from 0.91ms to 0.78 and `measure` from 0.53 to 0.44 at 4096 elements, against 0.11 for the
scatter — and it leaves a measure in one place for the length of a frame, where before the array and
the component both held it and R2m wrote both.

**The read half is not.** What remains is the reach for a typeface and a run, per element per pass.
Making those columns means reading them where the rest are read, which is a gather of what is
currently borrowed rather than copied: a run is a `&str`, and the shaping cache is keyed on it. That
cache is reached mutably from the middle of both passes, which is a different field of the grove than
the tree and so already a separate borrow — R1 destructures for it as it stands.

The same borrow is what leaves the one question resolution still asks four times over. A `Location`
is handed out by reference, so it is read once per axis, once by `measurable` and once again by the
resolve `reach` does of each child — four reads of one element's placement in a frame, where every
other thing a placement resolves against is now read once. It is the largest repeat left, and it is
in the same gather as the two above rather than in a pass of its own.

**What it is worth.** Less than it looked. The two passes are `measure` 0.44ms and `shaped` 0.29 at
4096 elements, and both spend a large part of that in the font and the shaping cache rather than in
reaching for the element — so what a gather could take is a fraction of 0.73, against 0.45 for the
read that would carry it. An estimate still, but a smaller one than the pair suggested before `wrap`
was split, and the candidate is weaker for it.

**What it costs.** `Tree` hands out one element's value at a time, and owning the world is what lets
it do that. A pass that reads a column asks instead for a read of the whole tree — so for these two
the boundary stops being an accessor per property. That is a decision about what `Tree` is for rather
than a change inside a pass, which is what separates it from the rewrite above.

## Extraction rewrites a changed run entire

The scratch is kept between frames, so a frame that changes nothing allocates nothing there. A run
that changed by one character is still rebuilt whole, and what a run costs is its length.

The diff is deliberately of the run and not of its glyphs: a run is one entry in the one stack, and
the renderer holds its glyphs under its own numbering. A finer diff has to stay on the renderer's
side of that line or it reopens R6's tie-break.

## R8 and extraction ask the same questions

`regions` reads an element's inherited product, its drawn box, its clip, its gestures and its rank,
over the order. Extraction reads the first three again and the rank, one step later in the same
frame, over the same order — and only for the elements something moved, which on a quiet frame is
none of them. What is left to move is the duplication between the two: both walks are the order's,
and neither reads what the other found.

## `Pollen` builds its sets each frame

The sets are built every frame and handed out behind an `Arc`, including on frames where nothing was
reported. The candidate is reusing the buffers across frames, or building only what a frame actually
has to report.

**Measured, and not a cost.** Step 3 — which seals the drift, delivers it, and hands the app its
`Pollen` — takes 0.004ms a frame with nothing reported, and reads the same at 4096 elements as at
8192, so it is not the tree's size either. There is nothing here to reclaim.

## The shared walk

It runs only when the stack moved, and when it runs it walks the whole stack — merging every
renderer's slots by rank, taking each instance's depth from its position, and cutting the walk into
spans on renderer, clip and binding. The gate is already there; the cost when it opens is the stack's
size.

## The sheet never reclaims

A failed pack draws blank and traces once, and nothing is ever evicted. Nothing fills the shared
sheet either: marks are a bounded set packed once, and pictures have a texture each.

Should one ever fill, the packer already implies the shape — **shelves are the reclaim unit**, and
reclaiming one orphans the runs still pointing at its texels, so it needs the character kept per
glyph to re-cut them.
