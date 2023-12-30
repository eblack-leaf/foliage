use foliage::bevy_ecs;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::prelude::Commands;
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::scene::{Anchor, Scene, SceneBinder};
#[derive(Bundle)]
pub struct SongEntry {}
pub enum SongEntryBindings {
    Icon,
    ArtistName,
    SongName,
}
impl Scene for SongEntry {
    type Bindings = ();
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
