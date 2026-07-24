use foliage::Foliage;

#[path = "assets/icons/gen/generated.rs"]
mod icons;
mod entry;
mod home;
mod navigator;
mod next;
mod third;
mod type_in;

/// This app's own hosting convention -- `foliage_proper` makes no assumption about it, so it
/// lives here, the one place that actually knows where these assets are served from.
#[cfg(target_family = "wasm")]
fn asset_url(path: &str) -> String {
    format!("{}/foliage/{path}", Foliage::window_origin())
}

/// Shared by every platform's entry point -- desktop's `main`, wasm's `main` (compiled to
/// `wasm32-unknown-unknown` and invoked by the generated JS glue), and android's
/// `android_main` below. Only *how a `Foliage` gets constructed* differs per platform (see
/// `Foliage::new` vs `Foliage::android`); everything after that is identical.
pub fn run(mut foliage: Foliage) {
    foliage.desktop_size((360, 800));
    icons::register(&mut foliage);
    entry::build(&mut foliage);
    foliage.photosynthesize();
}

/// Android's actual process entry point -- the Java/Kotlin GameActivity shim loads this
/// crate as a `.so` and calls this via JNI, handing over the one thing `Foliage::android`
/// needs and nothing else on any other platform has: a live `AndroidApp`.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: foliage::AndroidApp) {
    run(Foliage::android(app));
}
