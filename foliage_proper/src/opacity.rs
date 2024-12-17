use crate::{Component, RenderToken, Tree, Write};
use bevy_ecs::observer::Trigger;
use bevy_ecs::prelude::Query;
use bevy_ecs::system::Local;

#[derive(Component, Copy, Clone)]
pub struct Opacity {
    pub value: f32,
}
impl Opacity {
    pub fn new(value: f32) -> Opacity {
        Opacity { value }
    }
    pub fn token_push<R: Clone + Send + Sync + 'static>(
        trigger: Trigger<Write<Self>>,
        mut tree: Tree,
        opacities: Query<&Opacity>,
        mut cache: Local<Opacity>,
    ) {
        let opacity = *opacities.get(trigger.entity()).unwrap();
        // TODO cache check
        tree.trigger_targets(RenderToken::<R, _>::new(opacity), trigger.entity());
    }
}
impl Default for Opacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}
