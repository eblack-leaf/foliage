use crate::icon::IconHandles;
use foliage::tree::EcsExtension;
use foliage::twig::Twig;
use foliage::Foliage;
use home::{EventTest, Home};

mod home;
mod icon;
mod image;
mod leaf_model;

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
    foliage.enable_event::<EventTest>();
    let model = foliage.ecs().branch(Twig::new(Home {}));
    foliage.insert_resource(model);
    foliage.attach_root::<Home>();
    foliage.photosynthesize();
}
