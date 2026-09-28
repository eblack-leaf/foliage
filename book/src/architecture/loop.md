# The loop

`foliage.photosynthesize()` hands the engine to the platform and does not return. Everything in
this chapter lives in [`photosynthesize.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/photosynthesize.rs)
and [`willow.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/willow.rs): the one
part of the engine that knows a platform exists. Its jobs are to turn window events into input, to
decide whether a frame is owed, and to draw.

## Before the first frame

`Foliage::new()` touches no platform resource. It builds a `Grove` against a zero-sized viewport,
and every call made before `photosynthesize` (`title`, `desktop_size`, `root`, `font`, `tune`) writes
into the grove or into `Willow`'s description of the window it will open. That is what makes a
`Foliage` constructible in a test.

`photosynthesize` then:

1. builds winit's event loop (on Android, with the activity the process was started for, because
   the activity is what the platform's events arrive through);
2. sets it to `ControlFlow::Wait`, so the loop **sleeps until the platform has something to say**;
3. installs the **wake**, an `EventLoopProxy` that work finishing off the frame uses to rouse a
   sleeping loop;
4. opens the platform edges (the clipboard, the hidden input a soft keyboard is raised through, the
   road a URL takes to the host), once the wake they report through exists;
5. runs the app: `run_app` natively, `spawn_app` on the web, where the browser owns the loop.

Nothing is opened until the platform says the app has **resumed**. Only then does `Willow` open the
window, and the size the platform actually gave it is the first thing the tree is resolved
against, whatever `desktop_size` asked for. The window can never be dragged below 290 by 290
logical pixels, and a minimised window's zero extent is read as one pixel, because a surface cannot
be configured at zero and every placement would resolve against nothing.

The GPU device is acquired against that window. Natively that blocks, briefly, on `pollster`. A
browser hands over an adapter and a device only through promises, so on the web acquisition is
spawned and the device is collected by the first paint after it lands. Until there is a device,
nothing runs.

## Frames happen inside paints

A frame does not run on a timer. It runs inside `paint`, which is called when the platform delivers
`RedrawRequested`:

```rust,ignore
// photosynthesize.rs, shortened
fn paint(&mut self) {
    if self.owed() {
        self.advance();                       // move the clock by what elapsed
        fern::run(&mut self.grove, self.root.as_deref_mut());
        ash.absorb(&self.grove.elm, /* fonts, fields, plates */ ginkgo);
    }
    let clear = self.grove.scheme().color(Palette::Surface);
    ash.draw(ginkgo, clear);
    if self.grove.again {
        self.willow.repaint();
    }
}
```

Two questions are kept apart here. *Is a frame owed?* is the engine's: whether anything changed or
could change. *Should the surface be painted?* is the platform's, and it can ask for reasons of its
own: a window exposed, a compositor redrawing. Painting again from what the renderers already hold is
always possible and always correct, so a paint never forces a frame.

The batch extraction produced is absorbed immediately after the frame that made it, inside the same
branch. That is what makes it safe for [extraction](extraction.md) to keep a cache of what the
backend holds: every batch is applied exactly where it is made, and a paint that fails afterwards
costs one picture rather than leaving the cache and the GPU disagreeing.

Between events, `about_to_wait` asks the same question, and if a frame is owed it requests a paint.
That is the loop's only reason to keep running.

## Whether a frame is owed

The engine idles when there is nothing to do. The question is answered from state the engine
already holds, in one place:

```rust,ignore
// photosynthesize.rs
fn owed(&self) -> bool {
    self.grove.frames == 0                         // the tree has not been grown yet
        || self.grove.again                        // the app asked for one
        || self.grove.pending_resize.is_some()     // the window changed
        || self.grove.wake.pending()               // something arrived from off the frame
        || !self.grove.queue.is_empty()            // an op is waiting, from either side
        || !self.grove.incoming.pending.is_empty() // input is waiting
        || !self.grove.aspen.idle()                // a motion or timer is running
        || self.grove.incoming.awaiting_hold()     // a still press may yet become a hold
        || !self.grove.coasting.idle()             // a released scroll is still moving
        || self.grove.drift.pending()              // a report is waiting to be delivered
}
```

Each clause is there because something would otherwise stall:

- **A still press.** A press that does not move becomes a hold after half a second with nothing
  arriving to say so. Idling under a finger is how that half second would pass unnoticed, so frames
  are owed until the gesture is past being able to become one.
- **A coast.** A released scroll keeps moving, and it is not a tween and nothing on the app's side
  asks for the frames it needs.
- **An undelivered report.** Steps 4 to 7 of a frame emit into the drift, and it is step 3 of the
  *next* frame that hands the drift to the app. Without this clause the loop could go to sleep
  holding a report no frame would run to deliver.
- **A rouse.** Usually what arrives from off the frame is an op, which the queue clause sees. A key
  the web's hidden input captured is not: it waits in the keyboard until intake moves it. So a rouse
  is remembered as owing a frame on its own account.
- **The app asking.** An app animating something the engine cannot see (a value it moves itself
  from `frame_time`) has no clause of its own. `grove.again()` is that clause, and it lasts for
  exactly one frame, so the loop runs for as long as the app keeps asking and not a frame longer.

**Skipping a frame cannot lose a change.** A frame that does not run leaves every write in the
queue and every report in the drift, and the next frame that runs takes them. Extraction compares
against what the backend holds rather than consulting a flag that could be missed, so a change is
deferred, never dropped.

## Time

The engine has one clock ([`clock.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/clock.rs)).
Time enters it one way: `advance(delta)` adds to a pending amount, and `sample()` at the top of the
next frame takes it. The loop advances it by the wall-clock time since the last frame; the test suite
advances it by hand. Neither the frame nor anything in it can tell which.

The loop caps what it advances by at **100 milliseconds** (`HITCH`). Real gaps between frames have
nothing to do with the app: the machine slept, a tab was in the background, a breakpoint sat in the
loop. Reported honestly, the frame after a two-second gap would hand every running motion two
seconds at once, and what was on screen would jump to wherever wall time said it should be. Capped,
the whole engine is deferred by the same amount instead: a motion resumes where it was and takes
longer in wall time, and never arrives at its end without having been drawn on the way. Playing late
is the better failure, and it is only available because the frame has one clock, so nothing can fall
out of step with anything else.

The cap belongs to the loop and not to the clock. The suite advances by hand and has to be exact:
told to advance five seconds, it advances five.

## Input

Everything the platform says about input is translated into one small enum in
[`interaction/input.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/interaction/input.rs)
and appended to `Incoming::pending`, in arrival order:

| Platform event | Becomes |
|---|---|
| left button down or up, a touch starting or ending | `Pressed(at)`, `Released(at)` |
| the pointer moving **while held** | `Moved(at)` |
| a touch cancelled, the pointer leaving, the window losing focus mid-press | `Cancelled` |
| a wheel | `Wheeled { at, delta }`, a notch counted as 48 logical pixels |
| a key producing text | one `Keyed(Typed(c))` per character |
| a key that means something | `Keyed(Left)`, `Keyed(Enter)`, `Keyed(Tab)` and the others |
| modifiers changing | `Modifiers { shift, control }` |

Positions are divided by the window's scale factor on the way in, so nothing above this point ever
sees a device pixel. A pointer moving with nothing held is not queued at all: the engine has no
hover, so such a move says nothing and owes no frame. Modifiers travel as their own event in the
same stream rather than as flags on each key, so what a key was pressed with is decided by the
order the two arrived in, exactly as it happened.

Nothing is decided here. Resolving a press against what is on screen is a frame phase (dispatch,
[Interaction](interaction.md)), and it happens in the frame the input arrived for. This translation
is the whole of what the headless suite cannot reach: it writes the same `Input` values directly,
and from there on a scripted press and a real one take one path.

## Resizes and density

A resize or a change of scale factor calls `reconfigure`. It resizes the surface, and records the
new logical size as `pending_resize`, which the next frame's intake takes: the viewport changes,
the breakpoints are re-read, and every element is marked to be resolved again.

A change of **density** needs one more thing. Extraction compares logical values, while the backend
holds instances derived from them in device pixels: a glyph cut at a density, a stroke snapped to
the device grid. When a window moves to a display with a different density, the logical values do
not change at all, so the comparison would find nothing to redo and the backend would go on drawing
for a density that is gone. So a changed scale factor also drops extraction's cache (`recut`) and
invalidates the tree, and everything is extracted again at the new density.

On the web, `reconfigure` paints before it returns. Configuring the surface sets the canvas's size,
which clears it, and the resize arrives from a `ResizeObserver` after the browser's animation
callbacks have already run. A paint requested in the normal way would land in the next browser
frame, and the one in between would be shown blank, once for every step of a dragged resize.

## When the surface goes away

On Android, an activity that leaves the foreground has its window destroyed, and everything built
against it (the surface, the device's swapchain, every renderer's buffers) stops being valid then,
not when the process ends. `suspended` drops all of it, innermost first, and `resumed` builds it
again against whatever window comes back.

**The tree is untouched.** Nothing above `Ash` knows a surface exists, which is what makes a suspend
cost a rebuild rather than a reload: the app comes back to the frame it left. Two things are reset
with the backend:

- **Extraction's cache**, because it records what the *backend* holds, and a backend built fresh
  holds nothing. Kept, the next extraction would find nothing changed and paint an empty surface.
- **When the last frame was.** Forgetting it makes the first frame back advance the clock by zero,
  which is the truth: no frame ran while the app was away. Without that, the cap would still hand
  the first frame back 100 milliseconds of motion for time the app was not running.

## The wake

Work that finishes off the frame (a file read on a thread, a fetch's promise, an op from a `Sprig`)
pushes onto the shared queue, but a loop asleep in `ControlFlow::Wait` would never run the frame
that drains it. So an arrival pushes first and then calls `Wake::rouse`
([`queue.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/queue.rs)), which records
that a frame is owed and then sends the loop a user event through the proxy.

The order is what makes it race-free. The record is written before the loop is woken, so a loop that
wakes and asks whether a frame is owed finds the answer already there; and whatever the caller put
down, it put down before rousing, so the frame that answers finds it. Intake clears the record at the
top of every frame, before anything a rouse could have announced is looked for, so a rouse after
that point is owed the next frame.

The wake is installed by `photosynthesize` and nowhere else. The test suite runs its frames by hand,
and a frame it did not ask for is not one it could observe; it can still see that a rouse was
recorded.
