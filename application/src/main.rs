use foliage::{Foliage};
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

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((360, 800));
    icons::register(&mut foliage);
    entry::build(&mut foliage);
    foliage.photosynthesize(); // run
}
