use crate::view::ViewBuilder;

pub trait Aesthetic {
    fn pigment(self, aesthetics: &mut ViewBuilder);
}
