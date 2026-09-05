//! Android's process entry point, kept in its own `crate-type = ["cdylib"]` crate rather than in
//! the app crate itself -- a cdylib target also produces a competing `wasm32-unknown-unknown`
//! artifact sharing the app's own binary name, which breaks trunk's artifact selection for the real
//! wasm build.
//!
//! Built through `foliage-android build`, which is `cargo ndk ... -p` this crate. Nothing else
//! should depend on it.

/// The Java `GameActivity` shim loads this crate as a `.so` and calls this through JNI, handing
/// over the one thing [`Foliage::android`] needs and no other platform has: a live activity.
///
/// [`Foliage::android`]: foliage::Foliage::android
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(activity: foliage::AndroidApp) {
    application::run(foliage::Foliage::android(activity));
}
