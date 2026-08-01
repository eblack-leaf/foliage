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

An actual app project *is* now wired up end to end -- three pieces:

- **`Foliage::android(app)`** -- the Android-only counterpart to `Foliage::new()`.
  `winit`'s android backend can't build an event loop without the `AndroidApp` handle
  Android itself hands over at process start (`EventLoop::new()` alone panics there), so
  unlike every other platform there's no zero-arg `Default` to fall back on for this one.
  ```rust
  // foliage_proper/src/foliage.rs
  #[cfg(target_os = "android")]
  pub fn android(app: crate::AndroidApp) -> Foliage { .. }
  ```
- **`application_android`** -- a separate `crate-type = ["cdylib"]` crate (not folded into
  `application` itself: a cdylib target also produces a competing
  `wasm32-unknown-unknown` artifact sharing `application`'s own binary name, which broke
  trunk's artifact selection for the real wasm build). Its whole job is the JNI boundary:
  ```rust
  // application_android/src/lib.rs
  #[unsafe(no_mangle)]
  fn android_main(app: foliage::AndroidApp) {
      application::run(foliage::Foliage::android(app));
  }
  ```
  The Java/Kotlin `GameActivity` shim loads this crate as a `.so` and calls `android_main`
  via JNI -- everything after that is the same `application::run` every other platform
  calls.
- **`foliage_android`** -- a scaffolding CLI (`cargo run -p foliage_android -- gen
  --app-id .. --lib-name application_android --out application/android`) that generates
  the Gradle/`GameActivity` project around the compiled cdylib -- build files, manifest,
  `MainActivity`, Gradle wrapper. `androidx.games:games-activity` (Google's `GameActivity`,
  not the older `NativeActivity`) has a real Java/Kotlin AAR dependency, so a Gradle
  project is genuinely unavoidable here, not a workaround; generating it instead of
  hand-writing it keeps it a parameterized copy of
  `rust-mobile/android-activity`'s own `agdk-mainloop` example (the crate that actually
  implements the android-game-activity backend) rather than a maintained fork of it.
  `application/android/README.md` has the full one-time SDK/NDK setup (all doable
  non-interactively from a terminal, no Android Studio required).

Still no Android job in CI (`.github/workflows/ci.yml` has a comment on this, itself
written before this local wiring existed). Provisioning an SDK in Actions is the open
question there -- `sdkmanager` is deprecated in favor of the `android` CLI, and how much
setup that actually takes on a runner hasn't been tried.

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

See [TextInput](./composites/text-input.md) for the two specific, scoped-but-not-yet-implemented
selection gaps (Shift+Click range-extend and auto-scroll during an edge-adjacent drag).

## Router's no-URL-history design

See [Router](./composites/router.md) -- not a gap, a deliberate, documented rejection of
URL/browser-history sync after a full design was worked through.
