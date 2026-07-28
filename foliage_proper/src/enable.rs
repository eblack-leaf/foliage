use crate::EcsExtension;
use crate::Trigger;
use crate::{
    AnchorDeps, Attachment, Branch, Foliage, InteractionListener, InteractionState, Tree, Resolved,
};
use bevy_ecs::system::Query;

#[foliage_macros::targeted_event]
#[derive(Copy)]
/// Turns interaction back on for an entity and its subtree, undoing
/// [`Disable`](crate::Disable). A child disabled in its own right stays disabled.
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
impl Enable {
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&AnchorDeps>,
    ) {
        tree.trigger_targets(Resolved::<Enable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable::new(),
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable::new(),
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
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub(crate) struct AutoEnable {}
impl AutoEnable {
    fn user_signal(trigger: Trigger<Self>, mut tree: Tree) {
        tree.trigger_targets(Resolved::<Enable>::new(), trigger.event_target());
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
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub(crate) struct InheritEnable {}
impl InheritEnable {
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&AnchorDeps>,
    ) {
        tree.trigger_targets(Resolved::<Enable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable::new(),
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritEnable::new(),
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
