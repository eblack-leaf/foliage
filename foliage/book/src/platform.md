# Platform Notes

## Linux, Windows, macOS

Native, via `winit` + `wgpu`. CI (`.github/workflows/ci.yml`, `native` job) runs `cargo
test --workspace`, `cargo build --examples -p foliage`, and `cargo check -p application`
on all three operating systems on every push/PR.

## Web (WASM)

Compiles against `wasm32-unknown-unknown`; CI checks `cargo check -p foliage_proper -p
foliage --target wasm32-unknown-unknown` build-only, since there's no headless browser
in CI to execute a WASM test binary against, and the crate's real test suite is already
native/headless (`Foliage::new()` never touches `wgpu`/`winit` either way, so the tests
that matter don't need a browser).

A handful of code paths are wasm-specific by necessity, not by choice: [`Willow::connect`](../willow.md)
appends the window's canvas into the DOM itself, since a `winit` window on the web *is*
a canvas element; [`Ginkgo::acquire_context`](../ginkgo.md) requests
`Limits::downlevel_webgl2_defaults()` instead of `Limits::default()`, a real
capability ceiling of WebGL2 relative to native backends; and device acquisition in
[Photosynthesis](../photosynthesis.md)'s `resumed` handler is async-via-channel rather
than `pollster::block_on`, since blocking isn't available on the web.

## Android

Real code exists for this target -- `AndroidConnection`, `cfg(target_os = "android")`
branches (including the same `downlevel_webgl2_defaults()` limits path WASM uses, since
mobile GPU drivers share that ceiling), and `winit`'s android-game-activity feature are
all already in the tree. What's missing is an actual app project wired up to build and
run against them end to end, and the Android SDK's own interactive license-acceptance
step (doable in CI via `android-actions/setup-android` + `yes | sdkmanager --licenses`,
but real setup work for a target that isn't finished being wired up locally either) --
so there's no Android job in CI yet. This is unfinished integration work, not an
unsolved technical problem.

## iOS

No toolchain is currently available to compile or run against an iOS target. The one
`cfg(not(target_os = "ios"))` branch in the crate (in [`Willow::connect`](../willow.md),
gating the desktop-only `.with_inner_size(..)` window attribute) has no iOS-specific
implementation behind it -- it's a single exclusion, not a maintained alternate path. A
native macOS build already compiles the same shared source an `aarch64-apple-ios`
cross-compile would exercise, so an iOS-specific CI check wouldn't currently catch
anything the macOS job doesn't already cover. In short: likely possible, given the
shared codebase already compiles cleanly for every other native target, but genuinely
unverified rather than confirmed working.

## `TextInput` gaps

See [TextInput](./composites/text-input.md) for the three specific, scoped-but-not-yet-implemented
selection/scroll gaps (Shift+Click range-extend, scroll-position stability across a
resize, and auto-scroll during an edge-adjacent drag).

## Router's no-URL-history design

See [Router](./composites/router.md) -- not a gap, a deliberate, documented rejection of
URL/browser-history sync after a full design was worked through.
