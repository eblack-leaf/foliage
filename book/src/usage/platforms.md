# Platforms

The same app runs on Linux, Windows, macOS, the web and Android. **Only construction differs**:
everything after `Foliage::new()` (or `Foliage::android(..)`) is the same statements on every
platform. This chapter covers how to lay an app out so that it builds for all of them, and what is
particular to each.

| Platform | Built with | Status |
|---|---|---|
| Linux, Windows, macOS | `Foliage::new()` | supported; built and tested in CI |
| Web (`wasm32-unknown-unknown`) | `Foliage::new()` | supported; built in CI |
| Android | `Foliage::android(activity)` | supported; surface, rendering and touch verified on an emulator |
| iOS | `Foliage::new()` | untested: the source carries iOS arms, and there is no toolchain in CI to check them |

## One entry point, shared

Put the app in a library crate, with one function that takes a `Foliage` and runs it. Each
platform's entry point then only builds the engine and hands it over:

```rust,no_run
// src/lib.rs
use foliage::{Area, Foliage};
# use foliage::{Grove, Pollen, Root};
# struct App;
# impl Root for App {
#     fn take_root(_: &mut Grove) -> Self { App }
#     fn frame(&mut self, _: &mut Grove, _: Pollen) {}
# }

/// Shared by every platform's entry point.
pub fn run(mut foliage: Foliage) {
    foliage.title("app");
    foliage.app_id("app");
    foliage.desktop_size(Area::new(390.0, 844.0));
    foliage.root::<App>();
    foliage.photosynthesize();
}
# fn main() {}
```

```rust,no_run
// src/main.rs: desktop and web
# mod app { pub fn run(_: foliage::Foliage) {} }
fn main() {
    app::run(foliage::Foliage::new());
}
```

```rust,no_run
// app-android/src/lib.rs: Android, in a crate of its own
# mod app { pub fn run(_: foliage::Foliage) {} }
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(activity: foliage::AndroidApp) {
    app::run(foliage::Foliage::android(activity));
}
# fn main() {}
```

This is how the site is built: [`application/src/lib.rs`](https://github.com/eblack-leaf/foliage/blob/main/application/src/lib.rs)
holds `run`, and `main.rs` and
[`application-android`](https://github.com/eblack-leaf/foliage/tree/main/application-android) are
three lines each.

## Desktop

`Foliage::new()`, and nothing else. `desktop_size` is the size the window opens at; the window can
be made no smaller than 290 by 290 logical pixels, because a window dragged to nothing leaves every
extent resolving against zero.

`app_id` matters on Linux. It becomes the Wayland `app_id` and the X11 `WM_CLASS`, which is how the
desktop matches the window to the entry that launched it. Without one, some desktops draw a second,
generic icon beside the one that was clicked.

Building on Linux needs the windowing headers (see [A first app](first-app.md#setting-up)).
Nothing else needs anything beyond Rust.

## Web

The app compiles to `wasm32-unknown-unknown` unchanged and is served by
[`trunk`](https://trunkrs.dev). The page needs almost nothing:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <meta content="width=device-width, viewport-fit=cover, initial-scale=1.0" name="viewport"/>
    <style>
        html, body { margin: 0; border: 0; padding: 0; height: 100%; }
    </style>
    <link data-trunk rel="rust" href="./Cargo.toml" data-bin="app"/>
</head>
<body></body>
</html>
```

The engine puts its canvas in the body and styles it to fill its parent, and the surface follows
whatever size the page resolves that to. `desktop_size` is ignored. The site's
[`index.html`](https://github.com/eblack-leaf/foliage/blob/main/application/index.html) and
[`Trunk.toml`](https://github.com/eblack-leaf/foliage/blob/main/application/Trunk.toml) are a
complete, deployed example, served from a GitHub Pages project path with `public_url`.

What is different on the web:

- **The device arrives by promise.** A browser hands over its GPU adapter asynchronously, so the
  first frame runs once it lands rather than at startup.
- **No threads.** A `Sprig` is not `Send` on the web; it is used from a promise or a callback
  instead of a spawned thread.
- **No paths.** `Origin::path` does not exist on the web; `Origin::url` is fetched by the browser,
  with no feature needed.
- **The keyboard is the browser's.** While a field holds focus, the engine focuses a hidden input
  so that a phone raises its keyboard, and that input hands every key back. `Keypad` becomes the
  input's `inputmode`. While it holds the keys, the Command key counts as `control`, so a field's
  `Cmd+C` copies on a Mac.
- **The clipboard is permission-gated.** A browser usually refuses a clipboard read outside a user
  gesture; a field's `Ctrl+V` takes the text from the paste event instead.
- **`navigate` replaces the page**, and `download` saves through a link the engine clicks.
- **A resize paints immediately.** The canvas is cleared when its size changes, so the engine
  paints the new size in the same browser frame rather than showing a blank one.
- **Logging goes to the console**, through a subscriber such as `tracing-web`.

## Android

Android is the one platform a Rust program cannot reach on its own: the process belongs to a Java
activity, and `androidx.games:games-activity` (the library behind winit's GameActivity backend)
can only be linked by Gradle. [`foliage-android`](https://github.com/eblack-leaf/foliage/tree/main/foliage-android)
generates that project and then drives it:

```sh
cargo install --git https://github.com/eblack-leaf/foliage foliage-android
foliage-android init --app-id io.github.yourname.yourapp
foliage-android setup    # the SDK and NDK, into a directory beside the repo
foliage-android doctor   # what is missing, and the command that installs each piece
foliage-android run      # every configured ABI, built, installed and launched
```

`init` writes the Gradle project, a `cdylib` entry crate holding `android_main`, and
`foliage-android.toml` recording every choice, so no later command takes arguments and running
`init` again regenerates the project against an edited file. The entry crate is its own crate
rather than the app's, because a `cdylib` target in the app crate would also produce a wasm
artifact with the app's name and confuse the web build.

`Foliage::android` takes the activity the process was started for, re-exported as
`foliage::AndroidApp` so the app never names the glue crate itself (two versions of it in one
binary is a link error at launch).

What is different on Android:

- **The surface comes and goes.** When the app leaves the foreground, Android destroys its window,
  and the engine drops everything built against it: the GPU device, the renderers, the surface.
  When it comes back they are built again. **The tree is untouched**, so an app returns to exactly
  the frame it left, and the first frame back is not told about the time it was away.
- **Typing into fields is not wired up yet.** A focused field raises the soft keyboard, with the
  field's `Keypad`, but Android delivers what is typed on it through the input method (IME), and
  the engine does not handle IME commits yet. A hardware keyboard works.
- **The clipboard is the engine's own.** The system clipboard is not reached yet, so a copy
  round-trips inside the app and no further.
- **`navigate` opens the URL** with an `ACTION_VIEW` intent.

## iOS

The shared source carries iOS arms wherever a platform decision is forced, and `Foliage::new()` is
the constructor. It has not been built or run, so it is unverified rather than unsupported.

## The one feature

```toml
foliage = { git = "https://github.com/eblack-leaf/foliage", features = ["origin-url"] }
```

`origin-url` lets `Origin::url` fetch off the web. It is the only thing in foliage that brings an
http client and a TLS stack, so it is off by default: an app that bundles its assets, or reads them
from paths, compiles neither. Without it a URL is still accepted everywhere and reported `missing`
off the web. On the web it selects nothing, because the browser does the fetching.
