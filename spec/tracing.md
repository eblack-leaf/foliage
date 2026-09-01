# Tracing

What foliage reports about itself, and to whom.

## The shape

> **A trace of one frame reads as the frame law.**

One span per frame, and a span per step nested inside it, named for the steps in `frame.md`:

```
frame
  intake  dispatch  root  drain  animate  resolve  settle  extract
```

Step 3 is `root` rather than `frame`, so it is not confused with the frame span containing it.

A timing trace is therefore directly comparable to the sequence in `frame.md`. A slow step is
named rather than inferred, a step running out of order is visible rather than theoretical, and
the shape of the trace is a second statement of the same law the code implements.

This is a convention every slice follows from the first one. A subsystem instrumented on its own
terms later produces a trace that has to be interpreted; one instrumented against the frame law
produces a trace that can be read.

## Where it is load-bearing rather than useful

Most of the above is convenience. One case is not.

`frame.md`'s F2 makes a whole class of op **silent by design**: an op naming a withered `Leaf` is
dropped, not refused. That is what makes a stale handle inert, and it is right — but it means "I
called `branch` and nothing appeared" has no answer at the callsite, by construction.

The trace is that answer, and it is the only one. Every drop names its verb, its leaf and its
reason.

## Levels

| Level | Carries |
|---|---|
| `trace` | the frame span and its step spans. Per-frame, so it is what you turn on for one frame rather than for a session |
| `debug` | structural change — planted, branched, pruned, resized, focus moved, an op dropped. One event per thing that actually happened |
| `info` | once-per-run facts: boot, adapter and surface, font and asset registration |
| `warn` | something an app should change |
| `error` | something foliage could not do |

**A dropped op is `debug`, never `warn`.** Dropping is the designed behaviour of a stale handle,
so warning about it would train a reader to ignore warnings for something working exactly as
specified.

## Events are structured, not prose

One event name per kind, with the specifics as fields:

```rust
debug!(verb, leaf, reason, "op dropped")
```

rather than a sentence per case. A reader filters on `op dropped` and reads the fields; a sentence
has to be grepped for, and cannot be filtered at all.

## What the library does not do

**foliage never installs a subscriber.** It depends on `tracing` and nothing else. Where output
goes is the app's decision, and on the web and on Android it is a platform decision — so it
belongs to `photosynthesize`, which is already the platform's entry point, and to tests, which
install their own.

An app that installs no subscriber pays a relaxed atomic load and a branch per callsite. Nothing
inside a per-element loop is instrumented for that reason: **spans are per frame and per step,
events are per thing that happened, and neither scales with the size of the tree.** Rowan
recomputes everything every frame (`rowan.md`), so an event per element per pass would be the one
addition capable of making total recomputation expensive.

## What is not proven

**The wording of an event is not a contract, and is not asserted.** A test that pins a log message
is testing the message, and the behaviour behind every one of these events is already proven where
it belongs — a dropped op by the op not landing (`lifecycle.md`), the step order by what each step
produces.

One property does have teeth, because it is the one a later slice can quietly break:

- a frame that changes nothing emits nothing above `trace`, however large the tree
