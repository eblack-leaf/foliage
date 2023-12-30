use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::{Bundle, Commands};
use foliage::bevy_ecs::query::With;
use foliage::bevy_ecs::system::{Query, Res, ResMut, SystemParamItem};
use foliage::elm::leaf::Tag;
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use foliage::text::font::MonospacedFont;
use foliage::text::TextValue;
use foliage::window::ScaleFactor;
#[derive(Bundle)]
pub struct SongInfo {
    tag: Tag<Self>,
}
pub enum SongBindings {
    Artist,
    Divider,
    Song,
}
impl From<SongBindings> for SceneBinding {
    fn from(value: SongBindings) -> Self {
        SceneBinding(value as i32)
    }
}
pub struct SongInfoArgs {
    artist: TextValue,
    song: TextValue,
}
fn resize(
    scenes: Query<&SceneHandle, With<Tag<SongInfo>>>,
    mut coordinator: ResMut<SceneCoordinator>,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
) {
    for handle in scenes.iter() {
        // best-fit text and divider dims
    }
}
impl Scene for SongInfo {
    type Bindings = SongBindings;
    type Args<'a> = SongInfoArgs;
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
