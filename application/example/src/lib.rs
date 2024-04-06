mod generated;
mod overlay;
mod showcase;

use crate::generated::AssetGen;
use crate::overlay::Overlay;
use crate::showcase::icon::scene::IconDisplay;
use crate::showcase::icon::IconShowcase;
use crate::showcase::image::ImageShowcase;
use crate::showcase::progress::ProgressShowcase;
use crate::showcase::text::{TextShowcase, TextValueResource};
use foliage::color::monochromatic::Orange;
use foliage::coordinate::area::Area;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::image::{Image, ImageId, ImageStorage};
use foliage::view::ViewHandle;
use foliage::window::WindowDescriptor;
use foliage::workflow::Workflow;
use foliage::{AndroidInterface, Foliage};
use showcase::button::ButtonShowcase;
use std::sync::Arc;

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
        .with_leaf::<IconDisplay>()
        .with_leaf::<TextValueResource>()
        .with_android_interface(android_interface)
        .with_worker_path("./demo/worker.js")
        .run::<Engen>();
}
async fn tester() -> i32 {
    let a = 1 + 1;
    a
}
#[derive(Default)]
pub struct Engen {
    data: i32,
}
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl Workflow for Engen {
    type Action = u32;
    type Response = i32;

    async fn process(handle: Arc<Self>, action: Self::Action) -> Self::Response {
        tracing::trace!("received: {:?}", action);
        let b = tester().await;
        tracing::trace!("b: {}, data: {}", b, handle.data);
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
        elm.remove_web_element("loading");
        let assets = AssetGen::proc_gen_load(elm);
        elm.container().spawn(Image::storage(
            ImageId(0),
            ImageStorage::some(Area::from((651, 454))),
        ));
        elm.on_fetch(*assets.generated.get(0).unwrap(), |data, cmd| {
            cmd.spawn(Image::fill(ImageId(0), data));
        });
        elm.container().insert_resource(assets);
        elm.persistent_view::<Overlay>(ViewHandle(0));
        elm.add_view::<ButtonShowcase>(BUTTON);
        elm.add_view::<ProgressShowcase>(PROGRESS);
        elm.add_view::<IconShowcase>(ICON);
        elm.add_view::<TextShowcase>(TEXT);
        elm.add_view::<ImageShowcase>(IMAGE);
        elm.navigate_to(BUTTON);
    }
}
pub(crate) const BUTTON: ViewHandle = ViewHandle(1);
pub(crate) const PROGRESS: ViewHandle = ViewHandle(2);
pub(crate) const ICON: ViewHandle = ViewHandle(3);
pub(crate) const TEXT: ViewHandle = ViewHandle(4);
pub(crate) const IMAGE: ViewHandle = ViewHandle(5);
