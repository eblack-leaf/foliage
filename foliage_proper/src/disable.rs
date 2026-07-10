use crate::enable::InheritEnable;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::entity::Entity;
use crate::EcsExtension;
use crate::interaction::listener::InteractionListener;
use crate::{Attachment, Branch, Event, Foliage, InteractionState, StackDeps, Tree, Write};
use crate::Trigger;
use bevy_ecs::system::Query;

#[derive(EntityEvent, Copy, Clone)]
pub struct Disable {
    entity: Entity,
}
impl Default for Disable {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(Disable);
impl Attachment for Disable {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Disable::interactions);
        foliage.define(Disable::user_signal);
        foliage.define(AutoDisable::interactions);
        foliage.define(AutoDisable::user_signal);
        foliage.define(InheritDisable::interactions);
        foliage.define(InheritDisable::user_signal);
    }
}
impl Disable {
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.event_target()) {
            listener.state.remove(InteractionState::ENABLED);
        }
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&StackDeps>,
    ) {
        tree.trigger_targets(Write::<Disable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable { entity: Entity::PLACEHOLDER },
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable { entity: Entity::PLACEHOLDER },
                    stack.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
    }
    pub fn new() -> Disable {
        Disable { entity: Entity::PLACEHOLDER }
    }
}
#[derive(EntityEvent, Copy, Clone)]
pub(crate) struct AutoDisable {
    entity: Entity,
}
impl Default for AutoDisable {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(AutoDisable);
impl AutoDisable {
    pub(crate) fn new() -> Self {
        Self { entity: Entity::PLACEHOLDER }
    }
    fn user_signal(trigger: Trigger<Self>, mut tree: Tree) {
        tree.trigger_targets(Write::<Disable>::new(), trigger.event_target());
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.event_target()) {
            listener.state.remove(InteractionState::AUTO_ENABLED);
        }
    }
}
#[derive(EntityEvent, Copy, Clone)]
pub(crate) struct InheritDisable {
    entity: Entity,
}
impl Default for InheritDisable {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(InheritDisable);
impl InheritDisable {
    pub(crate) fn new() -> Self {
        Self { entity: Entity::PLACEHOLDER }
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&StackDeps>,
    ) {
        tree.trigger_targets(Write::<Disable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable { entity: Entity::PLACEHOLDER },
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable { entity: Entity::PLACEHOLDER },
                    stack.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.event_target()) {
            listener.state.remove(InteractionState::INHERIT_ENABLED);
        }
    }
}
