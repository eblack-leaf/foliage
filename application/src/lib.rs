use foliage::Foliage;

mod chapters;
mod chrome;
mod entry;
mod home;
#[path = "assets/icons/gen/generated.rs"]
mod icons;
mod navigator;
mod routes;
mod toc;
mod type_in;

/// This app's own hosting convention -- `foliage_proper` makes no assumption about it, so it
/// lives here, the one place that actually knows where these assets are served from.
#[cfg(target_family = "wasm")]
fn asset_url(path: &str) -> String {
    format!("{}/foliage/{path}", Foliage::window_origin())
}

/// Shared by every platform's entry point -- desktop's `main`, wasm's `main` (compiled to
/// `wasm32-unknown-unknown` and invoked by the generated JS glue), and `application_android`'s
/// `android_main`. Only *how a `Foliage` gets constructed* differs per platform (see
/// `Foliage::new` vs `Foliage::android`); everything after that is identical.
///
/// Android's own entry point deliberately isn't in this crate -- it needs `crate-type =
/// ["cdylib"]`, and a cdylib target also produces a competing `wasm32-unknown-unknown`
/// artifact sharing this crate's own binary name, which broke trunk's artifact selection
/// for the actual wasm build. `application_android` is the separate, cdylib-only crate
/// that calls this from its own `android_main`.
pub fn run(mut foliage: Foliage) {
    foliage.desktop_size((360, 800));
    icons::register(&mut foliage);
    entry::build(&mut foliage);
    foliage.photosynthesize();
}
