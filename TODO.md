# TODO

## Typing on Android's soft keyboard

A field on Android raises the soft keyboard and hears nothing typed on it. A hardware keyboard works.

The keyboard types through the input method, into the text state GameActivity keeps for it
(`GameTextInput`), and winit 0.30.13's Android backend never reports that state: its
`handle_input_event` (`platform_impl/android/mod.rs`) handles `MotionEvent` and `KeyEvent`, and logs
`InputEvent::TextEvent` as "Unknown android_activity input event" and drops it. It never sends
`WindowEvent::Ime`, so the desktop path (`Ime::Commit` in `photosynthesize.rs`) is never reached,
and its `set_ime_allowed` only shows and hides the keyboard, which `Keyboard::raise` already does.

- First check whether a newer winit's Android backend reports `Ime` events. If it does, the desktop
  path is the whole answer and this is a version bump.
- Otherwise, within winit 0.30: the activity handle `Keyboard` already holds (`keyboard.rs`) reads
  the state with `AndroidApp::text_input_state()` (text, selection, compose region) and writes it
  with `set_text_input_state`. A change still wakes the loop -- GameActivity's `onTextInputEvent`
  calls `notifyInput` -- and winit runs `about_to_wait` after it, so the state can be read there and
  diffed.
- Take only what has left the compose region (inline preedit is not planned), hand it to `Incoming`
  as `Key::Typed` the way `Ime::Commit` is, and reset the state so the next diff starts from empty.
  Check what Gboard does against an editor that always looks empty: whether backspace arrives as a
  `KEYCODE_DEL` key event (which already works), and whether suggestions still work.
- Needs a device or an emulator image to verify; `.android-sdk/` has neither the emulator nor a
  system image.
- iOS, while there: winit's `set_ime_allowed` is `becomeFirstResponder` there, and what its keyboard
  types arrives as `KeyboardInput`, so raising on focus may be all iOS needs. `Keyboard::raise`
  leaves iOS out today; unverified either way.

## The input method's candidate window

`Window::set_ime_cursor_area` is never called, so a desktop input method opens its candidate window
at the window's corner rather than at the caret. `Keyboard::raise` is called from `settled`
(`text_input.rs`), where the focused field's caret is to hand; its drawn box is a frame behind
there, which is close enough for a candidate window.
