use crate::{
    Attachment, Branch, Event, Foliage, InteractionListener, InteractionState, StackDeps, Tree,
    Write,
};
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;

#[derive(Event, Copy, Clone)]
pub struct Enable {}
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
impl Default for Enable {
    fn default() -> Self {
        Self::new()
    }
}

impl Enable {
    pub fn new() -> Enable {
        Enable {}
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&StackDeps>,
    ) {
        tree.trigger_targets(Write::<Enable>::new(), trigger.entity());
        if let Ok(branch) = branches.get(trigger.entity()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable {},
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.entity()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable {},
                    stack.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.insert(InteractionState::ENABLED);
        }
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct AutoEnable {}
impl AutoEnable {
    pub(crate) fn new() -> AutoEnable {
        AutoEnable {}
    }
    fn user_signal(trigger: Trigger<Self>, mut tree: Tree) {
        tree.trigger_targets(Write::<Enable>::new(), trigger.entity());
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.insert(InteractionState::AUTO_ENABLED);
        }
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct InheritEnable {}
impl InheritEnable {
    pub(crate) fn new() -> Self {
        Self {}
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&StackDeps>,
    ) {
        tree.trigger_targets(Write::<Enable>::new(), trigger.entity());
        if let Ok(branch) = branches.get(trigger.entity()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable {},
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.entity()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable {},
                    stack.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.insert(InteractionState::INHERIT_ENABLED);
        }
    }
}
