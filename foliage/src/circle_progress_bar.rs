use bevy_ecs::prelude::{Bundle, Commands};
use bevy_ecs::system::SystemParamItem;
use crate::progress_bar::ProgressBarArgs;
use crate::scene::{Anchor, Scene, SceneBinder};
use crate::set_descriptor;

#[derive(Bundle)]
pub struct CircleProgressBar {}
set_descriptor!(
    pub enum CircleProgressBarSet {
        Area,
    }
);
impl Scene for CircleProgressBar {
    type Bindings = ();
    type Args<'a> = ProgressBarArgs;
    type ExternalArgs = ();

    fn bind_nodes(cmd: &mut Commands, anchor: Anchor, args: &Self::Args<'_>, external_args: &SystemParamItem<Self::ExternalArgs>, binder: SceneBinder<'_>) -> Self {
        todo!()
    }
}