use std::collections::HashSet;
use bevy_ecs::system::{Commands, Query};
use bevy_ecs::world::World;
pub struct Root(pub TargetHandle);
pub struct Dependents(pub HashSet<TargetHandle>);
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TargetHandle(pub i32);
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ActionHandle(pub i32);
pub struct ElmConnection<'a> {
    pub(crate) world_handle: &'a mut World,
}
pub struct OnEnd {
    delay_targets: (),
    actions_to_apply: HashSet<ActionHandle>
}
pub struct Action<A: for <'a> FnMut(A, &'a mut ElmConnection<'a>) -> OnEnd> {
    delay_targets: (),
    a: Box<A>,
}
pub struct Signal(pub bool);
pub struct SignaledAction<A> {
    a: Action<A>,
}
pub(crate) fn signal_action<A>(
    mut signals: Query<(&Signal, &SignaledAction<A>)>,
    mut cmd: Commands,
) {
    for (signal, signaled_action) in signals.iter() {
        if signal.0 {

        }
    }
}
