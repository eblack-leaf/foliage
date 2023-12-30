mod index;
mod stats;

use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::{Bundle, Commands};
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding};

#[derive(Bundle)]
pub struct Playlist {}
pub enum PlaylistBindings {
    Index,
    Name,
    Stats,
}
impl From<PlaylistBindings> for SceneBinding {
    fn from(value: PlaylistBindings) -> Self {
        SceneBinding(value as i32)
    }
}
impl Scene for Playlist {
    type Bindings = PlaylistBindings;
    type Args<'a> = ();
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        binder: SceneBinder<'_>,
    ) -> Self {
        todo!()
    }
}
