use crate::anim::{Animate, Interpolations};
use crate::color::Color;
use crate::leaf::{Dependents, Stem};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Or, ParamSet, Query};

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

pub(crate) fn opacity(
    mut opaque: ParamSet<(
        Query<Entity, Or<(Changed<Color>, Changed<Opacity>, Changed<Dependents>)>>,
        Query<(&Opacity, &Dependents)>,
        Query<&mut Color>,
    )>,
    roots: Query<&Stem>,
) {
    let mut to_check = vec![];
    for entity in opaque.p0().iter() {
        to_check.push(entity);
    }
    for entity in to_check {
        let inherited = if let Ok(r) = roots.get(entity) {
            if let Some(rh) = r.0.as_ref() {
                let inherited = *opaque.p1().get(*rh).unwrap().0;
                Some(inherited.value)
            } else {
                None
            }
        } else {
            None
        };
        let changed = recursive_opacity(&opaque.p1(), entity, inherited);
        for (entity, o) in changed {
            if let Ok(mut color) = opaque.p2().get_mut(entity) {
                color.set_alpha(o);
            }
        }
    }
}

fn recursive_opacity(
    query: &Query<(&Opacity, &Dependents)>,
    current: Entity,
    inherited_opacity: Option<f32>,
) -> Vec<(Entity, f32)> {
    let mut changed = vec![];
    if let Ok((opacity, deps)) = query.get(current) {
        let blended = opacity.value * inherited_opacity.unwrap_or(1.0);
        changed.push((current, blended));
        for dep in deps.0.iter() {
            changed.extend(recursive_opacity(query, *dep, Some(blended)));
        }
    }
    changed
}
