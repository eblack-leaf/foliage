use bevy_ecs::prelude::{Component, Event, Trigger};
use bevy_ecs::system::Query;

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
#[derive(Event, Clone)]
pub struct TextValue {
    pub value: String,
}
impl TextValue {
    pub fn new<S: AsRef<str>>(value: S) -> Self {
        Self {
            value: value.as_ref().to_string(),
        }
    }
    pub fn obs(trigger: Trigger<Self>, mut texts: Query<&mut Text>) {
        let mut text = texts.get_mut(trigger.entity()).unwrap();
        text.value = trigger.event().value.clone();
        // glyph caching + renderer tokens
    }
}
