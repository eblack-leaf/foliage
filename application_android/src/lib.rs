//! Android's actual process entry point, kept in its own `crate-type = ["cdylib"]` crate
//! rather than in `application` itself -- a cdylib target also produces a competing
//! `wasm32-unknown-unknown` artifact sharing `application`'s own binary name, which broke
//! trunk's artifact selection for the real wasm build. This crate is only ever built via
//! `cargo ndk ... -p application_android` (see `application/android/README.md`); nothing
//! else in the workspace depends on it.

/// The Java/Kotlin GameActivity shim loads this crate as a `.so` and calls this via JNI,
/// handing over the one thing `Foliage::android` needs and nothing else on any other
/// platform has: a live `AndroidApp`.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: foliage::AndroidApp) {
    application::run(foliage::Foliage::android(app));
}
