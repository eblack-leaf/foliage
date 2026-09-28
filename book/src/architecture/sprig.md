# Off the frame

Everything the engine does happens in a frame, on the thread the frame runs on. A `Sprig`
([`sprig.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/sprig.rs)) is how work
that is on neither (a worker decoding something large, a promise that resolved, a host callback)
reaches the tree anyway. This chapter is about what makes that safe without a second code path: a
write from a thread is not a different kind of write, it is the same op arriving from somewhere else.

## One handle

```rust,ignore
// sprig.rs
#[derive(Clone)]
pub struct Sprig(Arc<Cutting>);

struct Cutting {
    queue: Queue,                                   // the one op queue
    wake: Wake,                                     // how a sleeping loop is roused
    naming: Naming,                                 // the one source of names
    conditions: Mutex<Option<Conditions>>,          // last frame's ambient state
    readings: Mutex<HashMap<(Leaf, Vein), Sap>>,    // every watched property, as last read
    inbox: Mutex<Vec<Pollen>>,                      // reports not yet taken
    listening: AtomicBool,                          // whether anyone has asked for reports
}
```

The grove holds one `Sprig` from the moment it is built, and `grove.sprig()` hands out a clone of it.
Every clone is the same handle: one queue, one inbox of reports, one set of watches. Two workers
holding clones read a single stream between them rather than each being sent a copy, which keeps the
reports finite when nobody is listening and the watches finite when everybody is.

## Writing: the same queue, the same names

`Sprig` implements the crate-private `Queues` trait exactly as the `Grove` does, so it gets the whole
of `Grow` for free, and every verb builds the same op:

```rust,ignore
// sprig.rs
impl Queues for Sprig {
    fn queue(&mut self, op: Op) {
        self.0.queue.push(op);
        // The frame that will drain it. An op arriving from off the frame is the one kind nothing
        // else announces: the loop is asleep, and the platform has nothing to say about it.
        self.0.wake.rouse();
    }

    fn allocate(&self) -> (Leaf, Growth) {
        self.0.naming.leaf()
    }
    // ...
}
```

The two halves of that are what [F1](frame.md#f1-one-queue-one-drain) needs.

**The queue is the frame's own queue**, an `Arc<Mutex<Vec<Op>>>`, so an op from a worker takes its
place among the frame's ops by arrival and nothing else. It lands in the drain of the frame that was
running when it arrived, if it arrived before that frame's drain began, and in the next frame's
otherwise. Nothing about how it is applied depends on which side it came from, and the suite proves
it by running one script of writes through a `Grove` and through a `Sprig` and comparing the trees.

**Names come from the same place.** `Naming` holds the world's entity allocator in its remote form
and an atomic counter for everything else, so a worker can take a name for an element (or a motion,
a picture, a face, a mark) without the world. Names from both sides never collide, and the
allocation order that settles elevation ties is one order across every thread. A worker can grow an
element and branch another off it on the next line, exactly as `frame` can.

Then the **rouse**: the loop sleeps until the platform has something to say, and an op pushed from a
thread is not something the platform knows about. So every push from a `Sprig` also records that a
frame is owed and wakes the loop ([The loop](loop.md#the-wake)).

A leaf a worker holds may have withered by the time the worker writes to it. That is safe on the
same terms it is safe anywhere: the op is dropped in the drain.

## Registering assets

A worker can register a font, a mark or a picture, and the path is deliberately the one a file read
takes. A name is taken at once from the shared counters (every registry grows to meet a name it has
not seen), and the bytes arrive as an `Arrived` op:

```rust,ignore
// sprig.rs
fn supply(&mut self, destination: Destination, bytes: Bytes) {
    match bytes.0 {
        Supply::Held(bytes) => self.queue(Op::Arrived {
            destination,
            bytes: Ok(bytes),
        }),
        Supply::At(origin) => retrieve(&self.0.queue, &self.0.wake, destination, origin),
    }
}
```

`retrieve` is how *any* read happens, from the frame or not: a path is read on a spawned thread (a
font is megabytes, and the thread that asked is the one drawing), a URL is fetched on one (or by the
browser, on the web), and when the bytes are in hand the thread pushes `Arrived` and rouses the loop.
So a worker that *made* an asset and a thread that *read* one are the same case, and both are drained
in the frame they reached.

That is also why nothing a worker hands over panics. Bytes handed to the grove at a call site are a
statement the program made, and a proportional font there stops the program where it can be fixed.
Bytes a worker built or was sent are not, and a panic on a worker would end the thread rather than
stop the program, so they are refused and reported `missing` instead.

`load` and `pixels` are `Grow` verbs rather than registrations, shared by both sides, so the rule is
one method of the `Queues` trait both sides implement. The verb measures the pixels against their
size where it is called, and hands a mismatch to `misfit`: the grove's panics at the caller's line
(it is `#[track_caller]` all the way down), and the sprig's pushes the `Arrived` a failed decode would
have pushed, so the refusal is reported `missing` in its place among everything else the worker
wrote.

```rust,ignore
// sprig.rs
fn misfit(&mut self, plate: Plate, reason: String) {
    self.queue(Op::Arrived {
        destination: Destination::Picture(plate),
        bytes: Err(reason),
    });
}
```

## Reading: pushed, not sampled

A worker cannot sample the tree. A read needs the world, and the world belongs to the frame. So the
reads a worker needs are **pushed** to it, at one point in the frame, by `sprig::publish`:

```rust,ignore
// fern.rs
    rowan::run(grove);
    // What settled, for the side of the boundary that cannot ask. After resolution and before
    // extraction: what is published is what the frame ended at.
    sprig::publish(grove);
    elm::run(grove);
```

After settle, so what is published is what the frame ended at rather than what it passed through;
before extraction, which is the backend's business and not the boundary's.

**Conditions** are the frame-wide reads: the viewport, the breakpoint, the short reading, the scheme,
what holds focus, the frame time and the elapsed time. They are a handful of `Copy` values, written
every frame whether or not anyone reads them, all from one frame, so the viewport and the breakpoint a
worker holds always agree with each other.

**Watches** are per-element reads. `watch(leaf, vein)` is an op like any other, so it is drained in
order: watching an element in the same breath as growing it works for the reason writing to it does.
The frame keeps the set of watches on its own side, unshared and unlocked. At publish, it reads each
watched property with the same `tap` an app would use and writes the reading across only if it
changed. A watch on an element that withered is dropped, with its reading, because a name is never
handed out twice and nothing could ever answer it again. `unwatch` takes the reading with it, so
`tap` never answers with a value nothing is keeping current.

**Reports** are the same `Pollen` the app's `frame` is handed, delivered at the same point (step 3) so
the two can never differ. The inbox is a history rather than a state, because a report is about a
moment and nothing else will say it happened. And it is **armed by the first call** to
`sprig.pollen()`: until someone asks, the frame has nowhere to deliver and does not, so a handle that
only ever writes never fills an inbox nobody reads. The first call answers with nothing; every call
after it answers with every frame's report since the last one, oldest first.

A reading is never older than the last report and may be one frame newer, which is the ordinary
condition of reading a running engine from beside it.

## The locks

Every lock a `Sprig` takes is recovered if it was poisoned. What is behind each is a value written
whole (a vector of ops, a map of readings, a list of reports) rather than a structure kept in step
across calls, so it is well-formed even if another thread panicked while holding it, and carrying on
is better than propagating that panic into the event loop.

## On the web

There are no threads in a browser, so `Sprig` is not `Send` there, and the wake is not required to be
either. The same handle is used from a promise or a callback, anything `spawn_local` runs, and the
ops land the same way.
