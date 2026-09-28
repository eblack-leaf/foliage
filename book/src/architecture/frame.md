# The frame

`Fern` is the frame, and it is one function:
[`fern::run`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/fern.rs). It holds no
state (everything it touches is a field of the `Grove`) and it runs every phase of the engine in one
fixed order. The platform's loop calls it, and so does every test in the headless suite, so there is
one sequence and both answer with it.

```rust,ignore
// fern.rs
pub(crate) fn run(grove: &mut Grove, app: Option<&mut (dyn Rooted + '_)>) {
    grove.frames += 1;
    let _frame = trace_span!("frame", n = grove.frames).entered();
    grove.again = false;
    intake(grove);
    interaction::dispatch(grove);
    frond::gestured(grove);
    root(grove, app);
    drain(grove);
    aspen::run(grove);
    rowan::run(grove);
    sprig::publish(grove);
    elm::run(grove);
}
```

## The sequence

| Step | Called | Does |
|---|---|---|
| 1 intake | `fern::intake` | Samples the clock, answers any rouse, takes any keys the web's hidden input captured, and takes a pending resize: the viewport changes, the breakpoints are re-read, every element is marked to resolve again |
| 2 dispatch | `interaction::dispatch` | Takes each input in arrival order against the box stack the last frame built: opens gestures, claims drags, reports taps and holds, moves focus on a tap, moves a region under a drag or a wheel, turns each key into an op |
| | `frond::gestured` | Lets the elements that are several elements (the text fields) read what the gestures meant to them |
| 3 root | `fern::root` | Seals what was collected into a `Pollen`, hands it to a listening `Sprig`, and calls the app's `frame` (after `take_root`, the first time) |
| 4 drain | `fern::drain` | Applies every queued op, in arrival order, whatever queued it. Then lets focus go of anything that can no longer hold it, and lets the fields put their parts in step |
| 5 animate | `aspen::run` | Advances every motion, timer and sequence by the frame's time, writes back the blends that are the same kind as their declaration (opacity, shape), and records the rest for the phases that apply them |
| 6 resolve | `rowan::run`, R1 to R6 | Measures, places both axes, measures extents, applies scrolling, clips, and ranks |
| 7 settle | `rowan::run`, R7 and R8 | Computes the inherited off-states, and builds the box stack the next frame's dispatch reads |
| | `sprig::publish` | Publishes what the frame ended at to anything off the frame that is watching |
| 8 extract | `elm::run` | Turns what changed into render instances |
| 9 draw | the loop | Hands the batch to the backend and draws |

Every phase has a trace span of its own (`intake`, `dispatch`, `root`, `drain`, `animate`, `resolve`
and each of its passes, `settle`, `extract`), so a trace of one frame at `trace` level reads as this
table.

## The laws

Nine rules govern the order. They were written down before the engine was, and the code still cites
them by number (`F7's own terms`, `(F8)`), so they are the vocabulary for everything that follows.

### F1. One queue, one drain

There is one op queue ([`queue.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/queue.rs)),
an `Arc<Mutex<Vec<Op>>>`. The `Grove` pushes to it and so does every `Sprig`, from any thread, and an
op's place in it is fixed by when it arrived and by nothing else.

An op from a worker and an op from `frame` are therefore indistinguishable: same ordering, same
application, same timing. The only difference left is the one concurrency makes unavoidable: an op
from another thread lands in this frame's drain if it arrived before step 4, and the next frame's
otherwise.

### F2. The drain is total, and in arrival order

The drain applies every op in the order it was queued, and does not group them by kind. A grow
followed by a write to the element it grew behaves the way it reads. An op naming an element that has
withered (or naming something the element does not have: a `text` on a panel) is **dropped**, never
refused: the drain writes one `debug` event and moves on. That is what makes a stale `Leaf` inert
rather than dangerous. [Ops and the drain](drain.md) is the whole of it.

### F3. Reads do not change inside `frame`

Nothing an app queues can land while it is still running, because the drain is step 4 and `frame`
is step 3. The state read at the top of `frame` is the state read at the bottom, so `frame` is a
function of what the tree was and what was reported, returning ops. An app cannot observe its own
write, and never has to wonder which side of it a read is on.

### F4. An app reads what is on screen, and writes what will be

The state `frame` reads was settled at step 7 of the previous frame, which is exactly what step 9
drew. The ops it writes are applied at step 4 and drawn at step 9 of the same frame. So a value
read in `frame` is what the person is looking at, and a write made in it is on screen when the frame
finishes. The two are one frame apart because they are about different instants, and each is the
right instant for its purpose.

### F5. Hit-testing runs against what was drawn

Dispatch is step 2, before the drain, and it reads the box stack R8 built at the end of the previous
frame. A press was made by a person looking at the screen, and the screen is the previous frame's
picture; testing it against geometry that has moved since would resolve it against something they
never saw. It also means input is handled in the frame it arrived for, with no frame of latency
added.

### F6. One clock

The instant is sampled once, at intake, and every phase sees that value. Motion, timers, holds and
`frame_time` all read it, so two things due at the same moment cannot disagree about when that moment
is. ([The loop](loop.md#time) covers how time gets into the clock.)

### F7. The collection window is "since your last frame"

What a `Pollen` holds is everything reported since the previous `frame` returned. That is one
continuous window, and it spans a frame boundary: what dispatch reported at step 2 of *this* frame,
and what the drain, the motion and resolution reported at steps 4 to 8 of the *last* one.

That is why `Pollen` is a set rather than a list. Across the boundary, any order would describe the
engine's phases rather than the app's world. It also decides which frame each report reaches an app
in:

| Reported | Recorded at | Reaches `frame` |
|---|---|---|
| `resized` | intake | the same frame |
| `engaged`, `disengaged`, `clicked`, `held`, `drag_started`, `dragged` | dispatch | the same frame |
| `focused`, `unfocused` from a tap | dispatch | the same frame |
| `focused`, `unfocused` from a verb, `Tab` or `Escape`, or a hidden holder | drain | the next frame |
| `keys`, `root_keys`, `edited`, `submitted`, `pasted` | drain | the next frame |
| `withered`, `loaded`, `missing`, `repainted` | drain | the next frame |
| `tween`, `finished`, `landed`, `sequence_finished` | animate | the next frame |

The pattern is simple once stated: dispatch decides and reports at once, while everything that
happens in or after the drain is reported to the frame after. A key is dispatched in the frame it
arrives for, but what it *does* is an op, applied in the drain behind anything the app wrote before
it, so a key is heard a frame after a tap would be. The
[notes walkthrough](../usage/notes.md#one-area-per-note) is built around that.

### F8. One writer per property

Every value has exactly one writer. Each resolution pass writes its own column and nothing else
(R3 writes extents, R4 offsets and drawn boxes, R5 clips, R6 ranks), and extraction writes nothing
but its own instance cache.

Declared values (what an app writes) have two potential writers: the drain and the motion that
might be running on the same property. The drain runs first, and **a direct write cancels any motion
on the property it writes**:

```rust,ignore
// fern.rs, in the drain
Op::Place { leaf, location } => {
    // ...
    cancel(grove, "at", leaf, Property::Location);
    grove.tree.set_location(leaf, location);
}
```

So by the time step 5 advances the motion, no property has both a pending write and a running
motion. The advisory rule this replaces ("if a property is animated anywhere, animate it
everywhere") is gone, because the conflict it warned about can no longer happen.

Dispatch writes two things of its own, and keeps to the same rule. A drag or a wheel moves a
region's offset where it is resolved, and cancels any scroll motion and any coast on that region
first; a tap moves focus, and an app writing focus elsewhere in `frame` is drained afterwards and
simply wins.

### F9. A frame runs only when one is owed

The loop idles otherwise. [The loop](loop.md#whether-a-frame-is-owed) has the list of what owes a
frame, and why skipping one cannot lose a change.

## Consequences worth stating

- **A `Leaf` is usable the moment it is handed out**, as a trunk, an anchor or the target of a write,
  though the element does not exist until the drain grows it. Before then it reads as
  `Presence::Planted`, and every `tap` of it answers `None`.
- **A zero-length motion or timer reports in the next frame, never the same one.** It is queued in
  step 3, started in step 4, finishes in step 5, and is delivered at step 3 of the following frame.
  That is not special-cased.
- **A `prune` in `frame` is reported as `withered` in the next one**, and the element is still there,
  and readable, for the rest of the frame that pruned it.
- **Off-thread ops never interleave with a partly applied frame.** They join the queue, and the
  queue is drained whole.

## How the laws are held

Each law is a test in the headless suite
([`tests/frame.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/tests/frame.rs)),
and the test names read as the laws:

- `the_drain_is_fifo_within_one_frame`, `a_write_lands_in_the_frame_that_planted_its_leaf`,
  `a_write_that_arrives_after_a_prune_is_dropped`,
  `ops_of_different_kinds_are_not_reordered_against_each_other` (F2);
- `a_move_is_not_visible_inside_the_frame_that_made_it`,
  `a_prune_is_not_visible_inside_the_frame_that_made_it`, `reads_do_not_change_inside_a_frame` (F3);
- `advance_is_exact`, `the_clock_does_not_move_without_being_advanced` (F6);
- `asking_for_another_frame_lasts_exactly_one_frame`,
  `a_rouse_is_owed_a_frame_and_answered_by_the_one_that_runs` (F9);
- `the_same_script_produces_the_same_state` and `idle_frames_change_nothing`, which run one script
  of writes on two groves, then again with idle frames between its steps, and require the same tree
  every time.

F1 is proven in [`tests/sprig.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/tests/sprig.rs)
by issuing the same writes through a `Grove` and through a `Sprig` and comparing the results.
[Proving it](testing.md) describes the suite.
