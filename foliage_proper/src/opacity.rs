use crate::Component;

#[derive(Component, Copy, Clone, PartialEq)]
pub struct Opacity {
    pub value: f32,
}
impl Opacity {
    pub fn new(value: f32) -> Opacity {
        Opacity { value }
    }
}
impl Default for Opacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}

