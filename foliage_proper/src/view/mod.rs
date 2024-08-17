use crate::branch::{Leaf, Tree};
use crate::coordinate::elevation::Elevation;
use crate::grid::{Grid, GridPlacement};
use crate::leaf::LeafHandle;

pub mod button;

pub trait Viewable
where
    Self: Sized + Send + Sync + 'static,
{
    fn build(self, view: &mut View);
}
pub struct View<'a> {
    pub target_handle: LeafHandle,
    pub elm_handle: Tree<'a>,
}
impl<'a> View<'a> {
    pub fn bind<TH: Into<LeafHandle>, BFN: FnOnce(&mut Leaf<'a>), E: Into<Elevation>>(
        &mut self,
        th: TH,
        grid_placement: GridPlacement,
        elevation: E,
        grid: Option<Grid>,
        b_fn: BFN,
    ) {
        let handle = self.target_handle.extend(th.into().0);
        self.elm_handle
            .add_leaf(handle.clone(), grid_placement, elevation, grid, b_fn);
        self.elm_handle
            .update_leaf(handle, |e| e.stem_from(self.target_handle.clone()));
    }
    pub fn config_grid(&mut self, grid: Grid) {
        self.elm_handle
            .update_leaf(self.target_handle.clone(), |e| e.give_attr(grid));
    }
    // TODO forward elm-handle functions
    pub(crate) fn new(target_handle: LeafHandle, elm_handle: Tree<'a>) -> Self {
        Self {
            target_handle,
            elm_handle,
        }
    }
}
