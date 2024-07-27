use crate::action::{ElementHandle, ElmHandle};
use crate::coordinate::elevation::Elevation;
use crate::element::TargetHandle;
use crate::grid::{Grid, GridPlacement};

pub mod button;

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
    pub fn bind<TH: Into<TargetHandle>, BFN: FnOnce(&mut ElementHandle<'a>), E: Into<Elevation>>(
        &mut self,
        th: TH,
        grid_placement: GridPlacement,
        elevation: E,
        grid: Option<Grid>,
        b_fn: BFN,
    ) {
        let handle = self.target_handle.extend(th.into().0);
        self.elm_handle
            .add_element(handle.clone(), grid_placement, elevation, grid, b_fn);
        self.elm_handle
            .update_element(handle, |e| e.dependent_of(self.target_handle.clone()));
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
