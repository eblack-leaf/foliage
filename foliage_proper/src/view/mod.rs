use crate::action::ElementHandle;
use crate::element::TargetHandle;
use crate::grid::{Grid, GridPlacement};
use bevy_ecs::system::Command;
use bevy_ecs::world::World;

mod button;

pub trait Viewable
where
    Self: Send + Sync + 'static,
{
    fn build(view: &mut View<Self>);
}
pub struct View<'a, V: Viewable> {
    pub v: V,
    pub target_handle: TargetHandle,
    pub(crate) world_handle: Option<&'a mut World>,
}
impl<'a, V: Viewable> View<'a, V> {
    pub fn bind<TH: Into<TargetHandle>, BFN: FnOnce(&'a mut ElementHandle)>(
        &mut self,
        th: TH,
        grid_placement: GridPlacement,
        grid: Option<Grid>,
        b_fn: BFN,
    ) {
        // this => self.target_handle.extend(th.into())
        // then elm-handle.add_element() + .dependent_of(self.target_handle)
        todo!()
    }
    // TODO forward elm-handle functions
    pub(crate) fn new(
        v: V,
        target_handle: TargetHandle,
        world_handle: Option<&'a mut World>,
    ) -> Self {
        Self {
            v,
            target_handle,
            world_handle,
        }
    }
}
