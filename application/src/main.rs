use crate::icon::IconHandles;
use crate::leaf_model::LeafModel;
use foliage::tree::EcsExtension;
use foliage::Foliage;
use home::{EventTest, Home};

mod home;
mod icon;
mod image;
mod leaf_model;

fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((360, 800));
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
    let model = foliage.ecs().branch(Home {});
    foliage.insert_resource(model);
    foliage.attach_root::<Home>();
    foliage.attach_root::<LeafModel>();
    foliage.photosynthesize();
}
