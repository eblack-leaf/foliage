# Foliage

`Foliage` is a cross-platform UI written in `Rust`. It can target `Linux | Windows | MacOS` natively,
`Web` via `WebAssembly` and `Android` (via `cargo-ndk`). Capable of running on `iOS` but not ported
as of writing. It leverages `wgpu.rs` and `winit` for native-rendering on (almost) every platform.

## Overview

`Foliage` is the main class to interact with the library.

```rust
let mut foliage = Foliage::new();
```

Once everything is to your liking, you can run the system with

```rust
// run main-loop
foliage.photosynthesize();
```