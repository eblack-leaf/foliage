use crate::action::ElmHandle;
use crate::grid::Grid;
use crate::view::{View, Viewable};

pub struct Button {}
impl Viewable for Button {
    fn build(self, view: &mut View) {
        // view.config_grid(Grid::new(3, 1));
    }
}
