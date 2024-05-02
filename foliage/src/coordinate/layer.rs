#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct Layer(pub f32);
impl Layer {
    pub fn new(l: f32) -> Self {
        Self(l)
    }
}
