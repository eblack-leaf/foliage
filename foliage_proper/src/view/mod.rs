use std::marker::PhantomData;
use crate::action::ElementHandle;
use crate::element::TargetHandle;
use crate::grid::{Grid, GridPlacement};
use bevy_ecs::system::Command;
use bevy_ecs::world::World;

mod button;

pub trait Viewable
where
    Self: Sized + Send + Sync + 'static,
{
    fn build(self, view: &mut View);
}
pub struct View<'a> {
    pub target_handle: TargetHandle,
    pub(crate) world_handle: Option<&'a mut World>,
}
impl<'a> View<'a> {
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
        target_handle: TargetHandle,
        world_handle: Option<&'a mut World>,
    ) -> Self {
        Self {
            target_handle,
            world_handle,
        }
    }
}
