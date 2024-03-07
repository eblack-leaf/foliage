mod assets_gen;
mod overlay;
mod showcase;

use crate::assets_gen::AssetsGen;
use crate::overlay::Overlay;
use crate::showcase::icon::IconShowcase;
use crate::showcase::progress::ProgressShowcase;
use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::Resource;
use foliage::color::monochromatic::Orange;
use foliage::coordinate::area::Area;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::image::{Image, ImageId, ImageStorage};
use foliage::view::ViewHandle;
use foliage::window::WindowDescriptor;
use foliage::workflow::{EngenHandle, Workflow};
use foliage::{AndroidInterface, Foliage};
use showcase::button::ButtonShowcase;

pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((360, 800))
                .with_resizable(true),
        )
        .with_leaves::<foliage::SceneExtensions>()
        .with_leaf::<Main>()
        .with_android_interface(android_interface)
        .with_worker_path("./worker.js")
        .run::<Engen>();
}
#[derive(Default)]
pub struct Engen {}
impl Workflow for Engen {
    type Action = u32;
    type Response = i32;

    async fn process(_arc: EngenHandle<Self>, action: Self::Action) -> Self::Response {
        tracing::trace!("received: {:?}", action);
        (action + 1) as i32
    }

    fn react(_elm: &mut Elm, response: Self::Response) {
        tracing::trace!("got response: {:?}", response);
    }
}

pub(crate) struct Main {}

impl Leaf for Main {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        Elm::remove_web_element("loading");
        let assets = AssetsGen::proc_gen_load(elm);
        elm.on_fetch(*assets.generated.get(0).unwrap(), |data, cmd| {
            cmd.spawn(Image::fill(ImageId(0), data));
        });
        elm.container().spawn(Image::storage(
            ImageId(0),
            ImageStorage::some(Area::from((700, 700))),
        ));
        elm.container().insert_resource(assets);
        elm.persistent_view::<Overlay>(ViewHandle(0));
        elm.add_view::<ButtonShowcase>(BUTTON);
        elm.add_view::<ProgressShowcase>(PROGRESS);
        elm.add_view::<IconShowcase>(ICON);
        elm.navigate_to(BUTTON);
    }
}
pub(crate) const BUTTON: ViewHandle = ViewHandle(1);
pub(crate) const PROGRESS: ViewHandle = ViewHandle(2);
pub(crate) const ICON: ViewHandle = ViewHandle(3);
