# Foliage

`Foliage` is a cross-platform UI written in `Rust`. It can target `Linux | Windows | MacOS`
natively, `Web` via `WebAssembly` and `Android` (via `cargo-ndk`). It leverages `wgpu.rs` and
`winit` for native rendering on (almost) every platform.

This book documents how to *use* a widget as an end user of the library, and how to *author*
one — the same tool builds both foliage's own widgets (`Button`, `TextInput`, ...) and your own.

- [The Contract](./contract.md) — the four verbs every widget speaks, end to end.
- [Authoring a Widget](./authoring.md) — the `Sprout` trait, and why `build` is static while
  everything data-dependent goes through `react`.
- [Reacting to Data](./reacting.md) — `react`, `react_any`, and `forward`.
- [A Complete Example](./example.md) — the real, shipped `Button` widget, walked through.
