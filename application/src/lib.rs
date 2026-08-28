use foliage::{Moss, Forest, ClearColor, Foliage, Root};

mod entry;
#[path = "assets/icons/gen/generated.rs"]
mod icons;
mod site;

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
    // The app paints its background tone itself (the cutout badges' backdrops), so the tone the
    // surface is cleared to has to come from the same place those do -- not from whatever the
    // engine happens to default to.
    foliage.tune(ClearColor(site::background()));
    // Startup-only: artwork, faces and image bytes have to be registered before anything
    // drawn with them is grown, and none of the three is something the frame can change.
    icons::register(&mut foliage);
    site::register_fonts(&mut foliage);
    site::register_assets(&mut foliage);

    // Everything else is the app, and the app is a struct. Nothing here is handed to the
    // engine, and the engine has no way to reach it.
    foliage.root::<entry::Site>();
    foliage.photosynthesize();
}

impl Root for entry::Site {
    fn take_root(forest: &mut Forest) -> Self {
        entry::Site::grow(forest)
    }
    fn frame(&mut self, forest: &mut Forest, mosses: Vec<Moss>) {
        for moss in mosses {
            self.respond(forest, moss);
        }
        self.tick(forest);
    }
}
