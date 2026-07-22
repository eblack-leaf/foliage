# TextInput

`TextInput` is the crate's most involved composite -- editable text needs a cursor, a
selection range, click-to-place, keyboard navigation, scrolling a viewport that's smaller
than the content, and IME composition support, on top of everything `Text` itself
already handles. Its config lives in `foliage_proper/src/composite/text_input/mod.rs`,
alongside two submodules: `action` (the `InputAction`/`TextInputAction` vocabulary) and
`keybindings` (the actual key-to-action table, `KeyBindings`).

All cursor and selection offsets are **byte offsets** into the underlying `String`
(matching `fontdue`'s own `byte_offset`), not character indices -- every mutation moves
between offsets via `prev_boundary`/`next_boundary` helpers rather than `± 1`, because a
plain `± 1` on a byte offset would panic `String::remove`/slicing the moment the content
contains a multi-byte character.

## Known gaps

Three specific, scoped-but-not-implemented gaps exist in the current selection/scroll
handling, each documented in place as a `TODO` rather than silently absent:

**Shift+Click doesn't extend an existing selection.** Every click unconditionally
restarts the selection at the click point:

```rust
// foliage_proper/src/composite/text_input/mod.rs:611-613
// TODO: every click unconditionally restarts the selection here, even Shift+Click --
// some editors instead extend the existing selection to the click point when Shift
// is held. Not implemented; scoping only, not clear this is wanted yet.
```

**Resizing the box while scrolled doesn't preserve scroll position.** Growing the box
after scrolling away from the cursor snaps the view back toward the cursor instead of
leaving the user's manual scroll position alone -- and the root cause isn't even
pinned down yet between two candidates:

```rust
// foliage_proper/src/composite/text_input/mod.rs:694-697
// TODO: scroll up away from the cursor (cursor sitting at/below the bottom of view), then
// resize the box bigger -- the resulting rewrap drops `view.offset` back toward the
// cursor instead of leaving the user's manual scroll position alone. Two candidate
// causes, not yet distinguished by an actual trace...
```

**Drag-selecting near the box's edges doesn't auto-scroll.** Drag-selection only ever
sees whatever's currently visible -- it calls `extend_range` directly rather than through
the `move_cursor`/`extend_and_reselect` path keyboard navigation uses, which is the path
that already has scroll-into-view logic:

```rust
// foliage_proper/src/composite/text_input/mod.rs:1044-1049
// TODO: no auto-scroll while dragging near the box's edges -- drag-selection is only
// ever in view of whatever's currently visible... Reusing that math but driven
// continuously while the pointer sits near the boundary (not just once per keystroke)
// is the likely shape of the fix, not built yet -- scoping only.
```

None of these are silent -- normal typing, arrow-key navigation, click-to-place, and
programmatic scroll-into-view on edit all work and already account for multi-byte
characters correctly. The gaps are specifically: extending (not just replacing) a
selection via Shift+Click, scroll-position stability across a resize, and continuous
auto-scroll during a drag that leaves the visible box.
