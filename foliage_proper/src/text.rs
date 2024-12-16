use bevy_ecs::prelude::Component;

#[derive(Component, Clone)]
#[require(FontSize)]
pub struct Text {
    pub value: String,
}
impl Text {
    pub fn new<S: AsRef<str>>(value: S) -> Self {
        Self {
            value: value.as_ref().to_string(),
        }
    }
}
#[derive(Component, Clone, Copy)]
pub struct FontSize {
    pub value: u32,
}
impl FontSize {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}
impl Default for FontSize {
    fn default() -> Self {
        Self { value: 14 }
    }
}
