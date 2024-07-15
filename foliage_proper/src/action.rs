use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Changed, Commands, Query, World};
use bevy_ecs::system::Command;

pub struct ElmConnection<'a> {
    pub(crate) world_handle: &'a mut World,
}

pub trait Actionable
where
    Self: Clone + Send + Sync + 'static,
{
    fn apply(self, conn: ElmConnection);
}

#[derive(Clone)]
pub struct Action<A: Actionable> {
    data: A,
}

impl<A: Actionable> Command for Action<A> {
    fn apply(self, world: &mut World) {
        let connection = ElmConnection {
            world_handle: world,
        };
        self.data.apply(connection);
    }
}

#[derive(Component)]
pub struct Signal(pub bool);
impl Signal {
    pub fn active() -> Self {
        Self(true)
    }
    pub fn inactive() -> Self {
        Self(false)
    }
}
#[derive(Component)]
pub struct SignaledAction<A: Actionable> {
    a: Action<A>,
}

pub(crate) fn signal_action<A: Actionable>(
    mut signals: Query<(&Signal, &SignaledAction<A>)>,
    mut cmd: Commands,
) {
    for (signal, signaled_action) in signals.iter() {
        if signal.0 {
            // clone + add action
        }
    }
}

pub(crate) fn clear_signal(mut signals: Query<(&mut Signal), Changed<Signal>>) {
    for mut signal in signals.iter_mut() {
        signal.0 = false;
    }
}
