# Pollen

What the tree put out this frame, and how an app reads it.

## The problem with a list

The obvious shape is `Vec<Event>`, handed to the frame to iterate. It is the wrong shape, and the
reason is not ergonomics.

The order of that list is an artifact. It falls out of which observer the engine registered first,
which entity the hit test reached first, which phase happened to push before which other phase —
none of which is a fact about the app's world. But a `Vec` does not merely permit a reader to
depend on that order, it *invites* it: the only thing you can do with a list is walk it, and having
walked it you have silently taken a dependency on the sequence.

That dependency then breaks in the direction that is hardest to see. An emission that should have
been handled before another — one that changes what a later one means — arrives after it, and the
app reads state that is one frame stale. Nothing errors. The bug is an off-by-one in time, and it
surfaces as a control that needs two clicks, or a value that lags by a frame.

The engine cannot fix this by choosing a better order, because there is no order that is right for
every app. The app knows which of its own concerns must settle first; the engine does not and
cannot.

**So the order is not exposed.** An app structures its own reads, in the order its own logic
requires, which is what it was going to have to do anyway.

## The rule

> Emissions are a **set you interrogate**, not a sequence you walk.
>
> Where an ordering is **real** rather than incidental, it is exposed as its own ordered sequence.

Exactly one ordering is real: **keys**. Typing `abc` must arrive as `a`, `b`, `c`; that sequence is
the content of the event, not a side effect of how the engine collected it. Every other ordering —
between two leaves' clicks, between a click and a timer, between an animation landing and a resize
— is incidental, and is not observable.

`Pollen` is a mass noun on purpose. You never name one grain of it; there is no `Vec<Pollen>` and
no singular. You ask the drift questions.

## The surface

```rust
fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
    if pollen.clicked(self.submit) { ... }
    if let Some(text) = pollen.text_changed(self.field) { ... }
    for key in pollen.keys() { ... }          // the one real ordering
}
```

Four shapes cover everything:

| Shape | Reads | Example |
|---|---|---|
| Per-leaf predicate | `bool` | `clicked`, `engaged`, `disengaged`, `drag_started`, `focused`, `unfocused`, `withered`, `sequence_finished`, `timer_finished` |
| Per-leaf value | `Option<T>` | `text_changed`, `dragged`, `text_action`, `scroll_refused` |
| Keyed | `Option<T>` / `bool` | `asset_loaded(key)`, `tween(tween)` |
| Frame-wide one-shot | `Option<T>` | `resized()`, `layout_changed()` |
| Ordered sequence | iterator | `keys()`, `physical_keys()` |

A predicate collapses repeats: a leaf crossed by a pass-through gesture may be clicked several
times in one frame, and `clicked(leaf)` answers that it was clicked. How many times is engine
bookkeeping, not app-visible fact.

## What this costs

**Exhaustive matching is gone.** An app can no longer `match` an event enum and have the compiler
point at every site when a variant is added. This is a real loss and it is accepted: apps do not
want to handle every emission, they want to answer about the few elements they own. The engine
keeps its own exhaustive handling internally, where the compiler check actually has teeth.

**Queries scale with what you ask about, not with what happened.** Fifty list items means fifty
predicate calls — each a lookup, and each written where the app already knows which leaf it means.
That is the shape apps were already writing by hand against the list.

**"Something was clicked, but I don't know what."** Not answerable by design. An app that needs it
puts a `Stem` behind its content and asks about that, which is also how it would have to define
"outside" anyway.

**Debugging.** `Pollen` carries a `Debug` impl that dumps everything it holds, for a frame you need
to inspect. That is a print, not an API — nothing structured is exposed for a reader to walk.

## Consequences elsewhere

- `Grove` and `Sprig` deliver the same `Pollen` value, so an off-thread listener and the frame
  answer the same questions the same way.
- The frame law (`frame.md`) states only that emissions are collected within the frame that
  produced them. It states nothing about their order, because nothing may depend on it.
- `Pollen` is owned and per-frame. It is not held across frames; a `Leaf` that withered is
  reported once, in the frame it withered.

## Open

- Whether `withered` needs a bulk form. An app holding a map of leaves wants to know which entries
  to drop, and asking per-key works but is O(map). A bulk answer would have to be a set, not a
  sequence, to stay inside the rule.
