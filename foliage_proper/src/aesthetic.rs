use crate::view::Aesthetics;

pub trait Aesthetic {
    fn pigment(self, aesthetics: &mut Aesthetics);
}