use crate::{Component, RenderToken, Tree, Write};
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::{Local, Query};

#[derive(Component, Copy, Clone)]
pub struct Color {
    pub value: bevy_color::Srgba,
}
impl Default for Color {
    fn default() -> Self {
        Self {
            value: bevy_color::Srgba::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}
impl Color {
    pub fn token_push<R: Clone + Send + Sync + 'static>(
        trigger: Trigger<Write<Self>>,
        mut tree: Tree,
        colors: Query<&Color>,
        mut cache: Local<Color>,
    ) {
        let color = *colors.get(trigger.entity()).unwrap();
        tree.trigger_targets(RenderToken::<R, _>::new(color), trigger.entity());
    }
}
