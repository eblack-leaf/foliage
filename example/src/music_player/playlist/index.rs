use foliage::bevy_ecs;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::prelude::Commands;
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding};
#[derive(Bundle)]
pub struct PlaylistIndex {}
pub enum PlaylistIndexBindings {
    Progress,
    Fraction,
}
impl From<PlaylistIndexBindings> for SceneBinding {
    fn from(value: PlaylistIndexBindings) -> Self {
        SceneBinding(value as i32)
    }
}
impl Scene for PlaylistIndex {
    type Bindings = PlaylistIndexBindings;
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
