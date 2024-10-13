use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::Event;

pub mod button;

#[derive(Event, Copy, Clone, Default)]
pub struct Configure {}
