use crate::view::Aesthetics;

pub trait Aesthetic {
    fn limn(self, aesthetics: &mut Aesthetics);
}
