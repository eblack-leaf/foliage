use crate::branch::{LeafPtr, Tree};
use crate::leaf::{Leaf, LeafHandle};
use crate::r_grid::Grid;

pub mod button;

pub trait TwigDef
where
    Self: Sized + Send + Sync + 'static,
{
    fn grow(self, twig_stem: &mut TwigStem);
}
pub struct TwigStem<'a> {
    pub target_handle: LeafHandle,
    pub tree: Tree<'a>,
}
impl<'a> TwigStem<'a> {
    pub fn bind<LFN: for<'b> FnOnce(&mut LeafPtr<'b>)>(&mut self, leaf: Leaf<LFN>) {
        let handle = leaf.handle.clone();
        self.tree.add_leaf(leaf);
        self.tree
            .update_leaf(handle, |e| e.stem_from(self.target_handle.clone()));
    }
    pub fn config_grid(&mut self, grid: Grid) {
        self.tree
            .update_leaf(self.target_handle.clone(), |e| e.give(grid));
    }
    // TODO forward elm-handle functions
    pub(crate) fn new(target_handle: LeafHandle, elm_handle: Tree<'a>) -> Self {
        Self {
            target_handle,
            tree: elm_handle,
        }
    }
}
