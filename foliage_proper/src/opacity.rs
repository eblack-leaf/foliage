use crate::anim::{Animate, Interpolations};
use crate::color::Color;
use crate::leaf::{Dependents, Stem};
use crate::tree::Tree;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Query, Trigger};

impl Animate for Opacity {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new().with(start.value, end.value)
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(o) = interpolations.read(0) {
            self.value = o;
        }
    }
}

#[derive(Copy, Clone, Component)]
pub struct Opacity {
    value: f32,
}

impl Default for Opacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl Opacity {
    pub fn new(o: f32) -> Self {
        Self {
            value: o.clamp(0.0, 1.0),
        }
    }
}
#[derive(Copy, Clone, Event, Default)]
pub struct ResolveOpacity {}
pub(crate) fn triggered_opacity(
    trigger: Trigger<ResolveOpacity>,
    stems: Query<&Stem>,
    opaque: Query<&Opacity>,
    dependents: Query<&Dependents>,
    mut colors: Query<&mut Color>,
    mut tree: Tree,
) {
    let inherited = if let Ok(s) = stems.get(trigger.entity()) {
        if let Some(s) = s.0 {
            if let Ok(opacity) = opaque.get(s) {
                opacity.value
            } else {
                1.0
            }
        } else {
            1.0
        }
    } else {
        1.0
    };
    if let Ok(opacity) = opaque.get(trigger.entity()) {
        if let Ok(mut color) = colors.get_mut(trigger.entity()) {
            let blended = opacity.value * inherited;
            color.set_alpha(blended);
            if let Ok(deps) = dependents.get(trigger.entity()) {
                tree.trigger_targets(
                    ResolveOpacity {},
                    deps.0.iter().copied().collect::<Vec<Entity>>(),
                );
            }
        }
    }
}
