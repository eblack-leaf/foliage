mod music_player;

use crate::music_player::controls::Controls;
use crate::music_player::MusicPlayer;
use foliage::window::WindowDescriptor;

use crate::music_player::track_progress::TrackProgress;
use crate::music_player::volume_control::VolumeControl;
use foliage::elm::Elm;
use foliage::workflow::{EngenHandle, Workflow};
use foliage::{AndroidInterface, Foliage};

pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((360, 800)),
        )
        .with_leaf::<MusicPlayer>()
        .with_leaf::<Controls>()
        .with_leaf::<TrackProgress>()
        .with_leaf::<VolumeControl>()
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
