# Virtual Keyboard

Touch platforms have no physical keyboard to intercept -- summoning the OS/browser's
soft keyboard requires giving *real* DOM/OS focus to some native text input, which
creates a real conflict: doing that moves keyboard focus away from the canvas `winit`
listens on, so control keys (Enter, Backspace, arrows) would be consumed by the
browser's own native editing behavior on that hidden trigger input instead of ever
reaching the app.

```rust
// foliage_proper/src/virtual_keyboard.rs
#[cfg(target_family = "wasm")]
pub(crate) enum PendingInput {
    Text(String),           // from the hidden input's native `input` event
    Key(crate::Key, crate::Modifiers), // from its `keydown` event, captured separately
}
```

The doc comment on `PendingInput` spells out exactly why committed text and control keys
need two separate capture paths rather than one: text composition (including IME) comes
through the hidden input's own `input` event, but control keys have to be caught on
`keydown` *before* the browser's native editing behavior consumes them on that same
hidden input.

## `VirtualInputQueue`: single-threaded on purpose

```rust
// foliage_proper/src/virtual_keyboard.rs
#[cfg(target_family = "wasm")]
pub(crate) struct VirtualInputQueue(Rc<RefCell<VecDeque<PendingInput>>>);
```

A `NonSend` resource, not a regular one -- its own doc comment explains why a plain
`Rc<RefCell<_>>` is correct here rather than `Arc<Mutex<_>>`: wasm32 in the browser is
single-threaded, so the DOM closures that push into this queue and the ECS system that
drains it never actually run concurrently, and reaching for thread-safe types would be
unearned ceremony for a platform that structurally can't race.

`map_control_key` mirrors the native `From<WinitKey> for Key` conversion
(`interaction/adapter.rs`) deliberately -- the same browser `KeyboardEvent.key` strings
map onto the same `crate::Key` variants a physical keyboard produces, so the rest of the
interaction pipeline ([Interaction](./interaction.md)) never needs to know whether a
given key event came from a real keyboard or this virtual path.
