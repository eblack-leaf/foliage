mod song_entry;

use foliage::bevy_ecs;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::prelude::Commands;
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::scene::{Anchor, Scene, SceneBinder};
#[derive(Bundle)]
pub struct Stream {}
pub enum StreamBindings {
    Last,
    Current,
    Next,
}
impl Scene for Stream {
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
