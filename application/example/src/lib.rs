mod button_tree;
mod showcase;

use crate::button_tree::BranchingButtonShowcase;
use crate::showcase::ButtonShowcase;
use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::Resource;
use foliage::coordinate::area::Area;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::image::{Image, ImageId, ImageStorage};
use foliage::window::WindowDescriptor;
use foliage::workflow::{EngenHandle, Workflow};
use foliage::{AndroidInterface, Foliage};
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
        .with_leaf::<BranchingButtonShowcase>()
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

#[foliage::assets(Engen, "../assets/", "/foliage/demo/assets/")]
#[derive(Resource, Clone)]
pub(crate) struct Assets {
    #[bytes(path = "something.dat", group = f_asset)]
    _something: AssetKey,
    #[bytes(path = "img.png", group = f_asset)]
    image_id: AssetKey,
    #[icon(path = "icons/copy.gatl", opt = FeatherIcon::Copy)]
    _copy_id: AssetKey,
    #[icon(path = "icons/command.gatl", opt = FeatherIcon::Command)]
    _command_id: AssetKey,
    #[icon(path = "icons/menu.icon", opt = FeatherIcon::Menu)]
    _command_id: AssetKey,
    #[icon(path = "icons/tag.icon", opt = FeatherIcon::Tag)]
    _command_id: AssetKey,
}

pub(crate) struct Main {}

impl Leaf for Main {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        Elm::remove_web_element("loading");
        let assets = Assets::proc_gen_load(elm);
        elm.on_fetch(*assets.f_asset.get(1).unwrap(), |data, cmd| {
            cmd.spawn(Image::fill(ImageId(0), data));
        });
        elm.container().spawn(Image::storage(
            ImageId(0),
            ImageStorage::some(Area::from((700, 700))),
        ));
        elm.container().insert_resource(assets);
        elm.enable_view::<ButtonShowcase>();
        elm.navigate_to::<ButtonShowcase>();
    }
}