use foliage::Foliage;

use crate::home::Home;

mod home;
mod icon;
mod image;

fn main() {
    let mut foliage = Foliage::seed();
    foliage.set_desktop_size((800, 600));
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.set_base_url("foliage");
    // foliage.spawn(Image::memory(ImageHandles::Leaf, (1920, 1920)));
    // let leaf = load_asset!(foliage, "assets/leaf.png");
    // foliage.insert_resource(ImageKeys { leaf });
    foliage.grow_branch(Home {});
    foliage.plant();
}
