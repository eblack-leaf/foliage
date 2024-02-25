use crate::view::ViewBuilder;

pub trait Aesthetic {
    fn pigment(self, view_builder: &mut ViewBuilder);
}
