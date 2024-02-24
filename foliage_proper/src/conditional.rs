use crate::animate::trigger::Trigger;
use crate::differential::Despawn;
use crate::scene::{Binder, Bindings, Scene, SceneBinding, SceneComponents};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Commands, Component, Query};

#[derive(Component, Copy, Clone)]
pub struct ConditionSet(pub Entity);
#[derive(Copy, Clone)]
pub enum SpawnTarget {
    This(Entity),
    BindingOf(Entity, SceneBinding),
}
pub struct Conditional<C: Clone> {
    c: C,
    target: SpawnTarget,
    is_extension: bool,
}
impl<C: Clone> Conditional<C> {
    pub fn new(c: C, target: SpawnTarget, is_extension: bool) -> Self {
        Self {
            c,
            target,
            is_extension,
        }
    }
}
pub struct ConditionalScene<CS: Scene + Clone> {
    cs: CS,
    target: SpawnTarget,
    is_extension: bool,
}
impl<CS: Scene + Clone> ConditionalScene<CS> {
    pub fn new(cs: CS, target: SpawnTarget, is_extension: bool) -> Self {
        Self {
            cs,
            target,
            is_extension,
        }
    }
}
pub struct ConditionHandle {
    this: Entity,
    target: Entity,
}
impl ConditionHandle {
    pub fn this(&self) -> Entity {
        self.this
    }
    pub fn target(&self) -> Entity {
        self.target
    }
    fn new(this: Entity, target: Entity) -> Self {
        Self { this, target }
    }
}
#[derive(Bundle)]
pub struct Branch<T: Clone + Send + Sync + 'static> {
    conditional: Conditional<T>,
    trigger: Trigger,
}
impl<T: Clone + Send + Sync + 'static> Branch<T> {
    pub fn new(t: T, e: SpawnTarget, is_extension: bool) -> Self {
        Self {
            conditional: Conditional::<T>::new(e, t, is_extension),
            trigger: Trigger::default(),
        }
    }
}
#[derive(Bundle)]
pub struct SceneBranch<T: Clone + Scene + Send + Sync + 'static> {
    conditional: ConditionalScene<T>,
    trigger: Trigger,
}
impl<S: Scene + Clone> SceneBranch<S> {
    pub fn new(t: S, e: SpawnTarget, is_extension: bool) -> Self {
        Self {
            conditional: ConditionalScene::<S>::new(e, t, is_extension),
            trigger: Trigger::default(),
        }
    }
}
pub(crate) fn conditional_spawn<C: Bundle + Clone + Send + Sync + 'static>(
    query: Query<(&Trigger, &Conditional<C>), Changed<Trigger>>,
    mut cmd: Commands,
) {
    for (trigger, cond) in query.iter() {
        if cond.is_extension {
            continue;
        }
        if trigger.is_active() {
            match cond.target {
                SpawnTarget::This(entity) => {
                    cmd.entity(entity).insert(cond.c.clone());
                }
                SpawnTarget::BindingOf(_, _) => {}
            }
        } else if trigger.is_inverse() {
            match cond.target {
                SpawnTarget::This(entity) => {
                    cmd.entity(entity).remove::<C>();
                }
                SpawnTarget::BindingOf(_, _) => {}
            }
        }
    }
}
pub(crate) fn conditional_scene_spawn<CS: Scene + Clone>(
    query: Query<(&Trigger, &ConditionalScene<CS>), Changed<Trigger>>,
    bindings: Query<&Bindings>,
    mut cmd: Commands,
) {
    for (trigger, cond) in query.iter() {
        if cond.is_extension {
            panic!("scenes-are-not allowed as extensions")
        }
        if trigger.is_active() {
            match cond.target {
                SpawnTarget::This(entity) => {
                    let _scene_desc = cond.cs.clone().create(Binder::new(&mut cmd, Some(entity)));
                }
                SpawnTarget::BindingOf(_, _) => {}
            }
        } else if trigger.is_inverse() {
            match cond.target {
                SpawnTarget::This(entity) => {
                    if let Ok(binds) = bindings.get(entity) {
                        for (_, b) in binds.nodes().iter() {
                            cmd.entity(b.entity()).insert(Despawn::signal_despawn());
                        }
                    }
                    cmd.entity(entity).remove::<SceneComponents<CS>>();
                }
                SpawnTarget::BindingOf(_, _) => {}
            }
        }
    }
}
pub(crate) fn conditional_extension<C: Bundle + Clone + Send + Sync + 'static>(
    query: Query<(&Trigger, &Conditional<C>), Changed<Trigger>>,
    mut cmd: Commands,
    bindings: Query<&Bindings>,
) {
    for (trigger, cond) in query.iter() {
        if !cond.is_extension {
            continue;
        }
        if trigger.is_active() {
            match cond.target {
                SpawnTarget::This(entity) => {
                    cmd.entity(entity).insert(cond.c.clone());
                }
                SpawnTarget::BindingOf(parent, bind) => {
                    cmd.entity(bindings.get(parent).unwrap().get(bind))
                        .insert(cond.c.clone());
                }
            }
        } else if trigger.is_inverse() {
            match cond.target {
                SpawnTarget::This(entity) => {
                    cmd.entity(entity).remove::<C>();
                }
                SpawnTarget::BindingOf(parent, bind) => {
                    cmd.entity(bindings.get(parent).unwrap().get(bind))
                        .remove::<C>();
                }
            }
        }
    }
}