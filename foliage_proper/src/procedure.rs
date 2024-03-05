use crate::view::ViewBuilder;

pub trait Procedure {
    fn steps(self, view_builder: &mut ViewBuilder);
}
