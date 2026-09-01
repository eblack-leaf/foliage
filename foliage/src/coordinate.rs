/// A width and a height, in logical pixels.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Area {
    pub width: f32,
    pub height: f32,
}

impl Area {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
