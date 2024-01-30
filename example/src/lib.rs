mod showcase;

use crate::showcase::Showcase;
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
        .with_leaf::<Showcase>()
        .with_android_interface(android_interface)
        .with_worker_path("foliage/worker.js")
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
