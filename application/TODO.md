# Chapter infographic ideas

Each `chapters/*.rs` page currently spawns just the shared `window_frame` and nothing
else. 8 chapters, not 9 -- Entity was cut as its own page (its whole point, "an entity is
just an id, existence and visibility are separate things," was too thin to carry a full
page on its own) and folded into Location's opening beat instead. One concept per page,
one simple animated visual per page -- rough ideas below, to be refined per-chapter as
each one actually gets built.

## 1. Location

Opens with Entity's old beat, now just the setup: a counter ticks up ("Entity #1... #2...
#3..."), small un-positioned tokens accumulate loosely, nothing else on screen -- existence
without placement. Then the actual payoff: pick up a single token and snap it into a
governed position relative to its parent. Resize/move the parent box and have the child
visibly slide/rescale in lockstep with it (still at the same relative percent). Point:
values are relative to the parent's resolved box, not absolute screen pixels -- and
nothing shows up there at all until it has one.

## 2. Grid

Draw the parent's own column/row lines faintly underneath. One panel morphs through a few
different spans -- 1x1, then 2 columns x 1 row, pause, then 1 column x 2 rows, pause, then
2x2 -- snapping cleanly to the grid lines at each transition. Point: children can span
multiple cells, not just address a single one.

## 3. Anchor

Two shapes: a "leader" wandering along a path, and a follower whose `Location` is
`Anchor`ed to the leader, tracking it live despite having a different structural parent.
A third, un-anchored shape sits still for contrast. Point: position can follow another
entity's live box, not just your structural parent.

## 4. Animate

One shape, one property (color or rotation -- most legible), tweening start -> end with a
visible scrub/progress indicator underneath. Point: a component is just a value, and
animating it is interpolating that value over time.

## 5. Sequence

2-3 shapes animate in a chain (not simultaneously), then a distinct final flash/checkmark
fires only once *all* of them finish. Point: `during` (chaining) and `sequence_end`
(reacting to completion) are separate mechanisms.

## 6. Interact

Two overlapping shapes, top one visibly labeled "pass-through," bottom one "listener."
Click lands on top, but the bottom one visibly reacts (pulse/color change) instead. Point:
propagation determines who actually receives a hit, not just who's visually on top.

## 7. Sprout

A visible "config" knob (slider/counter) driving a small generated structure (N shapes
appearing, laid out reactively) that rebuilds whenever the config changes. Point: config
in, structure out, and it's live -- not a one-time constructor.

## 8. Composite

Assemble a tiny real composite (e.g. a stripped-down toggle or slider) on screen, built via
a real `Sequence` -- each piece finishes appearing before the next starts. As each piece
lands, a small label fades in *next to that piece* naming its source ("Location",
"Anchor", "Interact"...), then fades out as the next piece's label takes over. Transient,
adjacent call-outs synced to the build sequence, not a permanent legend -- ends with all
pieces present and no labels left cluttering it. Point: this is what all seven previous
ideas look like combined.

## Open question

Build one chapter first (candidate: Grid, since its idea is the most concrete already) to
sanity-check the whole approach before doing all eight.
