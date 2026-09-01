use bevy_ecs::component::Component;

/// An element that draws nothing.
///
/// It holds children, takes hits, and has a box.
#[derive(Component, Copy, Clone, Default, Debug)]
pub struct Stem;

impl Stem {
    pub fn new() -> Self {
        Self
    }
}
