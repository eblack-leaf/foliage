mod showcase;

use crate::showcase::Showcase;
use foliage::asset::AssetKey;
use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::Resource;
use foliage::elm::Elm;
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
        .with_leaf::<Showcase>()
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
    #[asset(path = "something.dat")]
    #[group = f_asset]
    _something: AssetKey,
    #[asset(path = "img.png")]
    #[group = f_asset]
    image_id: AssetKey,
    #[icon(path = "icons/copy.gatl", Copy)]
    _copy_id: AssetKey,
    #[icon(path = "icons/command.gatl", Command)]
    _command_id: AssetKey,
}