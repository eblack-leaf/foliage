use bevy_ecs::prelude::Component;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Component, Copy, Clone, PartialEq, Default, Debug)]
pub struct Mips(pub f32);
