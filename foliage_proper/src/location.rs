use crate::{Component, RenderToken, Tree, Write};
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;

#[derive(Clone, Component)]
pub struct ResponsiveLocation {}

#[derive(Component, Copy, Clone, PartialEq, Default)]
pub struct Location {}
impl Location {
    pub fn token_push<R: Clone + Send + Sync + 'static>(
        trigger: Trigger<Write<Self>>,
        mut tree: Tree,
        locations: Query<&Location>,
    ) {
        let this = trigger.entity();
        let location = *locations.get(this).unwrap();
        tree.trigger_targets(RenderToken::<R, _>::new(location), this);
    }
}
