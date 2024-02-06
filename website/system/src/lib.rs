mod home;

use crate::home::Home;
use foliage::compositor::ViewHandle;
use foliage::elm::Elm;
use foliage::window::WindowDescriptor;
use foliage::workflow::{EngenHandle, Workflow};
use foliage::{AndroidInterface, Foliage};

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
pub(crate) const HOME: ViewHandle = ViewHandle::new(0, 0);
impl Workflow for Engen {
    type Action = ();
    type Response = ();

    async fn process(_arc: EngenHandle<Self>, _action: Self::Action) -> Self::Response {
        ()
    }

    fn react(_elm: &mut Elm, _response: Self::Response) {
        ()
    }
}
