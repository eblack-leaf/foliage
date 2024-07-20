use crate::action::ElmHandle;
use crate::element::TargetHandle;
use crate::grid::GridPlacement;
use bevy_ecs::prelude::World;
use bevy_ecs::system::Command;

pub trait Viewable
where
    Self: Send + Sync + 'static,
{
    fn apply(view: View<Self>, elm_handle: ElmHandle);
}
impl<V: Viewable> Command for View<V> {
    fn apply(self, world: &mut World) {
        let handle = ElmHandle {
            world_handle: Some(world),
        };
        V::apply(self, handle);
    }
}

pub struct View<V: Viewable> {
    pub v: V,
    pub grid_placement: GridPlacement,
    pub target_handle: TargetHandle,
}
impl<V: Viewable> View<V> {
    pub(crate) fn new(v: V, grid_placement: GridPlacement, target_handle: TargetHandle) -> Self {
        Self {
            v,
            grid_placement,
            target_handle,
        }
    }
}
