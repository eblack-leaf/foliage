# Web Extensions

A handful of things a real deployed web app needs that have no native equivalent at all
-- navigating the browser away, triggering a file download, showing a video/document
overlay. All web-only (`#[cfg(target_family = "wasm")]` bodies, no-ops elsewhere), and
all built the same way: synthesize a real DOM element, click it, remove it.

```rust
// foliage_proper/src/web_ext.rs
pub struct HrefLink { href: String }
impl HrefLink {
    pub fn navigate(&self) {
        // create a real <a href=...> element, .click() it, then remove it from the DOM
    }
}
```

This is the mechanism the live demo app actually uses to link back to the repository and
to this book from inside itself -- `HrefLink::new(url).navigate()` behind a button's
`on_click`. There's no `window.location` assignment anywhere in this file; going through
a real, momentarily-inserted anchor element is what lets the browser's own navigation
handling (new-tab modifiers, referrer policy, whatever the user's browser does with a
normal link click) apply exactly as it would to a hand-written `<a>` tag, instead of
this crate reimplementing that behavior.

## `Extensions`: download, video, and document overlays

```rust
// foliage_proper/src/web_ext.rs
impl Extensions {
    pub fn download(href: &str) { .. }      // <a download> click-and-remove, same pattern
    pub fn web_video(src: &str, ty: &str) { .. }    // <video> inside a full-screen overlay div
    pub fn web_document(src: &str) { .. }   // <iframe> inside the same overlay
    pub fn native_video(src: &str) { open::that(src) }     // native: hand off to the OS
    pub fn native_document(src: &str) { open::that(src) }  // native: hand off to the OS
}
```

The native/web split here is a real capability difference, not just a stylistic one:
native has no in-app video/document viewer at all, so `native_video`/`native_document`
hand the URL to the OS's own default application (`open::that`) instead of trying to
render it in-window; the web build can and does render it in-place via a synthesized
overlay `<div>`, dismissed by the same `Self::remove()` a close-button click calls.
Neither path is a placeholder for the other -- they're genuinely different strategies
suited to what each platform can actually do.
