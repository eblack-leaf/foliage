# The Contract

If you're using a widget — foliage's own, or one someone else authored the same way — there
are exactly four verbs, and they cover everything:

```rust
let s = tree.leaf(Slider::new().value(0.5).at(/* .. */).elevate(/* .. */)); // spawn
tree.write_to(s, SliderValue(0.25));                                       // update
tree.subscribe(s, |t: Trigger<ValueChanged>, ..| { /* .. */ });            // listen
tree.remove(s);
tree.enable(s);
tree.disable(s);                                                           // lifecycle
```

**A widget is one entity.** `tree.leaf(spec)` (a root, no parent) or `tree.branch(parent, spec)`
(a child) returns a single `Entity` — never a handle struct, a tuple of entities, or anything
that leaks knowledge of what the widget spawned internally. Whatever it built underneath is
private. `remove` cascades through every descendant automatically (stems form a tree; removing
a stem removes everything grown from it) — there's nothing to manually tear down.

**Update by writing a component the widget defined**, not by calling a method on it. A slider's
whole public surface is `SliderValue` — dragging it and calling `tree.write_to(s, SliderValue(v))`
from outside run through the exact same code path, so there's no "initial state" special case
separate from "later update."

**Listen via `tree.subscribe`**, watching for the widget's own event type (`ValueChanged`,
`SelectionChanged`, ...) — never a child entity's raw component writes, since children aren't
something a caller can name.

That's the whole end-user-facing surface. The next chapter covers the other half: how a widget
*is* built this way in the first place.
