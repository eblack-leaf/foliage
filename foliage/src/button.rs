use bevy_ecs::bundle::Bundle;
use crate::elm::{Elm, Leaf};
use crate::icon::Icon;
use crate::panel::Panel;
use crate::scene::{Scene, SceneBindRequest};
use crate::text::Text;
#[derive(Bundle)]
pub struct Button {
    scene: Scene,
    panel_req: SceneBindRequest<Panel>,
    text_req: SceneBindRequest<Text>,
    icon_req: SceneBindRequest<Icon>,
}
impl Leaf for Button {
    fn attach(elm: &mut Elm) {
        elm.enable_scene_bind::<Panel>();
        elm.enable_scene_bind::<Text>();
        elm.enable_scene_bind::<Icon>();
    }
}