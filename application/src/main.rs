use crate::icon::IconHandles;
use foliage::bevy_ecs::system::Resource;
use foliage::grid::unit::TokenUnit;
use foliage::tree::EcsExtension;
use foliage::twig::{Branch, Twig};
use foliage::Foliage;
use home::Home;

mod home;
mod icon;
mod image;

fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((800, 600));
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.set_base_url("foliage");
    foliage.load_icon(
        IconHandles::Concepts,
        include_bytes!("assets/icons/chevrons-left.icon"),
    );
    foliage.load_icon(
        IconHandles::Usage,
        include_bytes!("assets/icons/chevrons-right.icon"),
    );
    let id_table = foliage.ecs().branch(Twig::new(Home {}));
    foliage.insert_resource(id_table);
    foliage.photosynthesize();
}
