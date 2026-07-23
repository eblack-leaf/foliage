# Clipboard

A small `Resource`, and an honest one about where native and web genuinely differ rather
than papering over it:

```rust
// foliage_proper/src/clipboard.rs
pub struct Clipboard {
    local: String,
    #[cfg(not(target_family = "wasm"))]
    provider: Option<copypasta::ClipboardContext>,
}
```

Native reads and writes both go through the real system clipboard (`copypasta`). On the
web, `write` is real too (`navigator.clipboard.writeText`, inside a user gesture, which
`OnClick`/`Engaged` handlers already run within) -- but `read` can only ever return
`local`, the app's own in-session mirror, not the OS clipboard. The doc comment on the
struct explains why plainly: `navigator.clipboard.readText()` is async and
permission-gated, and cannot resolve inside a synchronous observer. Pasting content that
was copied *outside* the app is genuinely out of scope on the web build -- copying
within the app still round-trips correctly through `local`, since `write` updates it
too. The same doc comment notes a real future path if that gap ever needs closing: paste
events on the hidden virtual-keyboard inputs [Virtual Keyboard](./virtual-keyboard.md)
already maintains for IME/soft-keyboard support.

`tree_input.rs`/`TextInput` reads and writes through this `Resource` directly for
copy/cut/paste -- there's no widget-specific clipboard handling anywhere else in the
crate.
