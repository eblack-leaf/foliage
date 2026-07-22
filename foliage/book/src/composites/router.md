# Router

`Router` is destructive, element-level scene switching -- the exact opposite of
[`Tabs`](../composites-overview.md)' content lifecycle. `Tabs` builds every page once and
toggles `Visibility`; `Router` keeps exactly **one** route's content alive at a time --
navigating tears the current scene's subtree down (`tree.remove`) and runs the next
route's builder against a fresh, full-size slot. Hidden scenes never exist under a
router: nothing ticks, nothing accumulates, and a route entered twice is built from
nothing both times, so it can never depend on residue from its last visit.

```rust
// foliage_proper/src/composite/router.rs
pub type RouteFn = fn(&mut Tree, Entity);
```

`RouteFn` is deliberately a bare `fn`, not a closure: a bare `fn` cannot capture, so a
scene can never accidentally close over another scene's live entities -- the mistake
that would make destructive switching unsafe becomes a compile error instead of a
runtime surprise. Everything a scene needs beyond its slot arrives through `Resource`s
(asset keyrings, app state), which by definition outlive any one scene.

One entity carries the whole widget: `RouterRoutes` + [`PageIndex`](../composites-overview.md)
in, [`PageCount`](../composites-overview.md) maintained, `PageChanged` out -- the same
paging vocabulary `Tabs`/`Carousel`/`Pagination` already share, so any of those (a bottom
nav bar, a stepper) can drive a `Router` by forwarding an index. Navigation is
`tree.write_to(router, PageIndex(i))`; there is no richer protocol than that. `Router`
renders nothing of its own and has no style vocabulary -- like `List`/`Carousel`/`Tabs`,
the author builds and owns every scene; the widget only owns which one currently exists.
Anything that must survive a switch (persistent chrome, shared state) has to live outside
the router's subtree or in a `Resource`, never inside a scene.

## No URL/browser-history integration -- decided, not pending

This is the one composite in the crate with a documented, deliberately-rejected design,
directly in its own source (`composite/router.rs:38-47`):

> A full design was worked through (opt-in sync, per-route slug blessing, typed URL
> params as the only scene-input channel) and rejected: a deep link makes any route
> someone's FIRST scene, entered by a user-generated URL rather than an authored call
> site, so a route whose scene reads a navigation-coupled `Resource` cold-boots against
> defaults -- and no declared contract survives indirect circumvention, it just relocates
> the footgun. In-session navigation has no such class: every `PageIndex` write is a call
> site the author wrote, orderings are enforced by where navigation UI exists, and
> mistakes render visibly in development. The invariant is "top-level browser entry only":
> the app always boots at its root.

The distinction that makes this a real design decision rather than a missing feature: an
in-session `tree.write_to(router, PageIndex(i))` is always a call site the author put
somewhere reachable only after whatever setup that route depends on has already run. A
URL-driven deep link has no such guarantee -- it can land a user on any route with none of
the app's own setup having happened first, and there's no way to declare "this route
requires X" that a raw URL can't simply bypass. If you're looking at this file wondering
whether to add URL sync (composite-integrated or app-side): this was already considered
and the conclusion was no, for the reason above, not because nobody got to it.
