use crate::Trigger;
use crate::interaction::listener::InteractionListener;
use crate::{AnchorDeps, Attachment, Children, Foliage, InteractionState, Resolved, Tree};
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
        mut propagations: Query<&mut crate::InteractionPropagation>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.event_target()) {
            listener.state.remove(InteractionState::ENABLED);
        }
        // Every entity has one of these; only an `.interactive()` one has a listener. Written
        // to both, so "disabled" means the same thing whether or not the thing was ever a
        // target -- without this, disabling a plain container was silently a no-op while the
        // container carried on winning gestures over everything beneath it.
        if let Ok(mut propagation) = propagations.get_mut(trigger.event_target()) {
            propagation.set_disabled(true);
        }
    }
    fn user_signal(
        trigger: Trigger<Self>,
        mut tree: Tree,
        branches: Query<&Children>,
        stacks: Query<&AnchorDeps>,
    ) {
        tree.send_to(Resolved::<Disable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.send_to(
                    InheritDisable::new(),
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.send_to(
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
        tree.send_to(Resolved::<Disable>::new(), trigger.event_target());
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
        mut propagations: Query<&mut crate::InteractionPropagation>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.event_target()) {
            listener.state.remove(InteractionState::AUTO_ENABLED);
        }
        // Same reasoning as `Disable::interactions`: written to both, or a plain container
        // that auto-disabled (its `Location` failed to resolve) carries on winning gestures
        // over whatever is actually visible beneath it.
        if let Ok(mut propagation) = propagations.get_mut(trigger.event_target()) {
            propagation.set_disabled(true);
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
        branches: Query<&Children>,
        stacks: Query<&AnchorDeps>,
    ) {
        tree.send_to(Resolved::<Disable>::new(), trigger.event_target());
        if let Ok(branch) = branches.get(trigger.event_target()) {
            if !branch.ids.is_empty() {
                tree.send_to(
                    InheritDisable::new(),
                    branch.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
        if let Ok(stack) = stacks.get(trigger.event_target()) {
            if !stack.ids.is_empty() {
                tree.send_to(
                    InheritDisable::new(),
                    stack.ids.iter().copied().collect::<Vec<_>>(),
                );
            }
        }
    }
    pub(crate) fn interactions(
        trigger: Trigger<Self>,
        mut listeners: Query<&mut InteractionListener>,
        mut propagations: Query<&mut crate::InteractionPropagation>,
    ) {
        if let Ok(mut listener) = listeners.get_mut(trigger.event_target()) {
            listener.state.remove(InteractionState::INHERIT_ENABLED);
        }
        // Same reasoning as `Disable::interactions`: written to both, or a descendant that
        // was never itself a target carries on winning gestures over whatever the disabled
        // ancestor was meant to take out of the hit test.
        if let Ok(mut propagation) = propagations.get_mut(trigger.event_target()) {
            propagation.set_disabled(true);
        }
    }
}
