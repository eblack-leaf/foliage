use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Commands, Component, IntoSystemConfigs, Query};
use bevy_ecs::query::With;
use bevy_ecs::system::Command;

use crate::animate::trigger::{Trigger, TriggerState};
use crate::differential::Despawn;
use crate::elm::config::CoreSet;
use crate::elm::leaf::{EmptySetDescriptor, Leaf, Tag};
use crate::elm::Elm;
use crate::scene::{Binder, Bindings, IsScene, Scene, SceneBinding, SceneComponents};

#[derive(Component, Copy, Clone)]
pub struct ConditionSet(pub Entity, pub TriggerState);
impl ConditionSet {
    pub fn new(entity: Entity, trigger_state: TriggerState) -> Self {
        Self(entity, trigger_state)
    }
}
fn set_condition(
    query: Query<(Entity, &ConditionSet)>,
    mut triggers: Query<&mut Trigger>,
    mut cmd: Commands,
) {
    for (e, cs) in query.iter() {
        if let Ok(mut t) = triggers.get_mut(cs.0) {
            t.set(cs.1);
        }
        cmd.entity(e).despawn();
    }
}
impl Leaf for ConditionSet {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.main()
            .add_systems(set_condition.in_set(CoreSet::ConditionPrepare));
    }
}
#[derive(Copy, Clone)]
pub enum SpawnTarget {
    This(Entity),
    BindingOf(Entity, SceneBinding),
}
#[derive(Component, Clone)]
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
#[derive(Component, Clone)]
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
#[derive(Copy, Clone, Debug)]
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
    pub fn new(this: Entity, target: Entity) -> Self {
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
            conditional: Conditional::<T>::new(t, e, is_extension),
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
            conditional: ConditionalScene::<S>::new(t, e, is_extension),
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
#[derive(Copy, Clone, Component, Default)]
pub struct SceneClean {}
pub(crate) fn clean_scene<CS: Scene + Clone>(
    query: Query<(Entity, &Bindings), (With<SceneClean>, With<Tag<IsScene>>)>,
    mut cmd: Commands,
) {
    for (entity, bindings) in query.iter() {
        cmd.entity(entity)
            .remove::<SceneClean>()
            .remove::<SceneComponents<CS>>();
        for b in bindings.nodes().iter() {
            cmd.entity(b.1.entity()).insert(Despawn::signal_despawn());
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
                    if let Ok(binds) = bindings.get(entity) {
                        for (_, b) in binds.nodes().iter() {
                            cmd.entity(b.entity()).insert(Despawn::signal_despawn());
                        }
                    }
                    cmd.entity(entity).remove::<SceneComponents<CS>>();
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
#[derive(Component, Clone)]
pub struct ConditionalCommand<COMM: Command + Clone + Send + Sync + 'static>(pub COMM);
pub(crate) fn conditional_command<COMM: Command + Clone + Send + Sync + 'static>(
    query: Query<(&Trigger, &ConditionalCommand<COMM>), Changed<Trigger>>,
    mut cmd: Commands,
) {
    for (trigger, comm) in query.iter() {
        if trigger.is_active() {
            cmd.add(comm.0.clone());
        }
    }
}
