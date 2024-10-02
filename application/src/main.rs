use crate::icon::IconHandles;
use crate::leaf_model::{EventTest, LeafModel};
use foliage::tree::EcsExtension;
use foliage::twig::Twig;
use foliage::Foliage;

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
    foliage.enable_event::<EventTest>();
    let model = foliage.ecs().branch(Twig::new(LeafModel {}));
    foliage.insert_resource(model);
    foliage.attach_root::<LeafModel>();
    foliage.photosynthesize();
}
