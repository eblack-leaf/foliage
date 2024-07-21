use bevy_ecs::system::Command;

use crate::action::{ElementHandle, ElmHandle};
use crate::element::TargetHandle;
use crate::grid::{Grid, GridPlacement};

mod button;

pub trait Viewable
where
    Self: Sized + Send + Sync + 'static,
{
    fn build(self, view: &mut View);
}
pub struct View<'a> {
    pub target_handle: TargetHandle,
    pub elm_handle: ElmHandle<'a>,
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
        // then run w/ b_fn
        todo!()
    }
    pub fn config_grid(&mut self, grid: Grid) {
        self.elm_handle
            .update_element(self.target_handle.clone(), |e| e.give_attr(grid));
    }
    // TODO forward elm-handle functions
    pub(crate) fn new(target_handle: TargetHandle, elm_handle: ElmHandle<'a>) -> Self {
        Self {
            target_handle,
            elm_handle,
        }
    }
}
