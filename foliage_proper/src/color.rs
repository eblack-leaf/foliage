use crate::Component;

#[derive(Component, Copy, Clone, PartialEq)]
pub struct Color {
    pub value: bevy_color::Srgba,
}
impl Default for Color {
    fn default() -> Self {
        Self {
            value: bevy_color::Srgba::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}
impl Color {}
impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        wgpu::Color {
            r: color.value.red as f64,
            g: color.value.green as f64,
            b: color.value.blue as f64,
            a: color.value.alpha as f64,
        }
    }
}
