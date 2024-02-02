mod home;

use crate::home::Home;
use foliage::compositor::ViewHandle;
use foliage::elm::Elm;
use foliage::workflow::{EngenHandle, Workflow};
use foliage::{AndroidInterface, Foliage};

pub struct Engen {}
pub fn entry(app: AndroidInterface) {
    Foliage::new()
        .with_leaf::<Home>()
        .with_worker_path("./worker.js")
        .with_android_interface(app)
        .run::<Engen>();
}
pub(crate) const HOME: ViewHandle = ViewHandle::new(0, 0);
impl Workflow for Engen {
    type Action = ();
    type Response = ();

    async fn process(arc: EngenHandle<Self>, action: Self::Action) -> Self::Response {
        ()
    }

    fn react(elm: &mut Elm, response: Self::Response) {
        ()
    }
}
