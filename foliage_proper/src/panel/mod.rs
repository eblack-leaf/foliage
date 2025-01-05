use bytemuck::{Pod, Zeroable};
use crate::{Attachment, Color, Component, Foliage, Logical, Opacity, ResolvedElevation, Section};

mod pipeline;
mod vertex;

#[derive(Component, Copy, Clone, Default)]
#[require(Rounding, Color, Outline)]
pub struct Panel {}
impl Panel {
    pub fn new() -> Panel {
        Panel {}
    }
}
impl Attachment for Panel {
    fn attach(foliage: &mut Foliage) {
        foliage.differential::<Self, Section<Logical>>();
        foliage.differential::<Self, Opacity>();
        foliage.differential::<Self, Color>();
        foliage.differential::<Self, ResolvedElevation>();
    }
}
#[derive(Component, Copy, Clone, Default)]
pub enum Rounding {
    #[default]
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}
#[derive(Component, Copy, Clone)]
pub struct Outline {
    pub value: i32,
}
impl Default for Outline {
    fn default() -> Self {
        Outline { value: -1 }
    }
}

