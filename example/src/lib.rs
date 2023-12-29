mod music_player;

use crate::music_player::controls::Controls;
use crate::music_player::MusicPlayer;

use foliage::window::WindowDescriptor;

use crate::music_player::track_progress::TrackProgress;
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
        .with_android_interface(android_interface)
        .run();
}
