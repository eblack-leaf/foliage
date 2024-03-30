mod home;

use crate::home::Home;
use foliage::elm::Elm;
use foliage::window::WindowDescriptor;
use foliage::workflow::Workflow;
use foliage::{AndroidInterface, Foliage};
use std::sync::Arc;

#[derive(Default)]
pub struct Engen {}
pub fn entry(app: AndroidInterface) {
    Foliage::new()
        .with_leaves::<foliage::SceneExtensions>()
        .with_leaf::<Home>()
        .with_worker_path("./worker.js")
        .with_android_interface(app)
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_resizable(true)
                .with_title("website")
                .with_desktop_dimensions((800, 360)),
        )
        .run::<Engen>();
}
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl Workflow for Engen {
    type Action = ();
    type Response = ();

    async fn process(_handle: Arc<Self>, _action: Self::Action) -> Self::Response {
        ()
    }

    fn react(_elm: &mut Elm, _response: Self::Response) {
        ()
    }
}