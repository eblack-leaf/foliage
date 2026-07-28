use crate::EcsExtension;
use crate::Trigger;
use crate::enable::InheritEnable;
use crate::interaction::listener::InteractionListener;
use crate::{AnchorDeps, Attachment, Branch, Event, Foliage, InteractionState, Tree, Resolved};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::system::Query;

#[foliage_macros::targeted_event]
#[derive(Copy)]
/// Turns off interaction for an entity and everything beneath it. It still draws; it
/// stops competing for input. Paired with [`Enable`](crate::Enable).
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
        if let Ok(mut listener) = listeners.get_mut(trigger.event_target()) {
            listener.state.remove(InteractionState::ENABLED);
        }
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&AnchorDeps>,
    ) {
        tree.trigger_targets(Resolved::<Disable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable::new(),
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable::new(),
                    stack.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
    }
}
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub(crate) struct AutoDisable {}
impl AutoDisable {
    fn user_signal(trigger: Trigger<Self>, mut tree: Tree) {
        tree.trigger_targets(Resolved::<Disable>::new(), trigger.event_target());
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
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub(crate) struct InheritDisable {}
impl InheritDisable {
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Branch>,
        stacks: Query<&AnchorDeps>,
    ) {
        tree.trigger_targets(Resolved::<Disable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable::new(),
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.trigger_targets(
                    InheritDisable::new(),
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
