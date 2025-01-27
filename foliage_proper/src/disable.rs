use crate::enable::InheritEnable;
use crate::interaction::listener::InteractionListener;
use crate::{Attachment, Branch, Event, Foliage, InteractionState, StackDeps, Tree, Write};
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;

#[derive(Event, Copy, Clone)]
pub struct Disable {}
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
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.remove(InteractionState::ENABLED);
        }
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&StackDeps>,
    ) {
        tree.trigger_targets(Write::<Disable>::new(), trigger.entity());
        if let Ok(branch) = branches.get(trigger.entity()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable {},
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.entity()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable {},
                    stack.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
    }
    pub fn new() -> Disable {
        Disable {}
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct AutoDisable {}
impl AutoDisable {
    pub(crate) fn new() -> Self {
        Self {}
    }
    fn user_signal(trigger: Trigger<Self>, mut tree: Tree) {
        tree.trigger_targets(Write::<Disable>::new(), trigger.entity());
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.entity()) {
            listener.state.remove(InteractionState::AUTO_ENABLED);
        }
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct InheritDisable {}
impl InheritDisable {
    pub(crate) fn new() -> Self {
        Self {}
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&StackDeps>,
    ) {
        tree.trigger_targets(Write::<Disable>::new(), trigger.entity());
        if let Ok(branch) = branches.get(trigger.entity()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable {},
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.entity()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable {},
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
            listener.state.remove(InteractionState::INHERIT_ENABLED);
        }
    }
}
