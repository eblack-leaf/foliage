use crate::EcsExtension;
use crate::Trigger;
use crate::{
    AnchorDeps, Attachment, Branch, Foliage, InteractionListener, InteractionState, Tree, Write,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::EntityEvent;
use bevy_ecs::system::Query;

#[derive(EntityEvent, Copy, Clone)]
pub struct Enable {
    entity: Entity,
}
impl Default for Enable {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(Enable);
impl Attachment for Enable {
    fn attach(foliage: &mut Foliage) {
        foliage.define(AutoEnable::interactions);
        foliage.define(AutoEnable::user_signal);
        foliage.define(Enable::interactions);
        foliage.define(Enable::user_signal);
        foliage.define(InheritEnable::interactions);
        foliage.define(InheritEnable::user_signal);
    }
}
impl Enable {
    pub fn new() -> Enable {
        Enable {
            entity: Entity::PLACEHOLDER,
        }
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&AnchorDeps>,
    ) {
        tree.trigger_targets(Write::<Enable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable {
                        entity: Entity::PLACEHOLDER,
                    },
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable {
                        entity: Entity::PLACEHOLDER,
                    },
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
            listener.state.insert(InteractionState::ENABLED);
        }
    }
}
#[derive(EntityEvent, Copy, Clone)]
pub(crate) struct AutoEnable {
    entity: Entity,
}
impl Default for AutoEnable {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(AutoEnable);
impl AutoEnable {
    pub(crate) fn new() -> AutoEnable {
        AutoEnable {
            entity: Entity::PLACEHOLDER,
        }
    }
    fn user_signal(trigger: Trigger<Self>, mut tree: Tree) {
        tree.trigger_targets(Write::<Enable>::new(), trigger.event_target());
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.event_target()) {
            listener.state.insert(InteractionState::AUTO_ENABLED);
        }
    }
}
#[derive(EntityEvent, Copy, Clone)]
pub(crate) struct InheritEnable {
    entity: Entity,
}
impl Default for InheritEnable {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}
crate::targeted_event!(InheritEnable);
impl InheritEnable {
    pub(crate) fn new() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&AnchorDeps>,
    ) {
        tree.trigger_targets(Write::<Enable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable {
                        entity: Entity::PLACEHOLDER,
                    },
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable {
                        entity: Entity::PLACEHOLDER,
                    },
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
            listener.state.insert(InteractionState::INHERIT_ENABLED);
        }
    }
}
