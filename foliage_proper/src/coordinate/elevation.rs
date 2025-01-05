use crate::anim::interpolation::Interpolations;
use crate::{Animate, Attachment, Branch, Foliage, Stem};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::world::DeferredWorld;
use bytemuck::{Pod, Zeroable};
use std::fmt::Display;
use std::ops::{Add, Sub};

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, Pod, Zeroable, Component, Debug)]
pub struct ResolvedElevation(pub(crate) f32);
impl ResolvedElevation {
    pub fn value(&self) -> f32 {
        self.0
    }
}
impl Display for ResolvedElevation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
impl PartialOrd for ResolvedElevation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.0 < other.0 {
            Some(std::cmp::Ordering::Greater)
        } else if self.0 > other.0 {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Pod, Zeroable, Component, Debug)]
#[require(ResolvedElevation)]
#[component(on_insert = Self::on_insert)]
pub struct Elevation(pub f32);
impl Elevation {
    pub fn new(e: i32) -> Self {
        Self(e as f32)
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        if world.get::<Stem>(this).is_none() || world.get::<Branch>(this).is_none() {
            return;
        }
        let current = world
            .get::<Stem>(this)
            .unwrap()
            .id
            .and_then(|id| Some(*world.get::<ResolvedElevation>(id).unwrap()))
            .unwrap_or_default();
        let resolved = ResolvedElevation(world.get::<Elevation>(this).unwrap().0 + current.0);
        world.commands().entity(this).insert(resolved);
        for dep in world.get::<Branch>(this).unwrap().ids.clone() {
            if let Some(elev) = world.get::<Elevation>(dep).copied() {
                world.commands().entity(dep).insert(elev);
            }
        }
    }
}
impl Animate for Elevation {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        todo!()
    }
    fn apply(&mut self, interpolations: &mut Interpolations) {
        todo!()
    }
}
impl Attachment for Elevation {
    fn attach(foliage: &mut Foliage) {
        foliage.enable_animation::<Self>();
    }
}
macro_rules! elevation_conversion_implementation {
    ($i:ty) => {
        impl From<$i> for Elevation {
            fn from(value: $i) -> Self {
                Self::new(value as i32)
            }
        }
    };
}
elevation_conversion_implementation!(f32);
elevation_conversion_implementation!(i32);
elevation_conversion_implementation!(u32);
elevation_conversion_implementation!(usize);
elevation_conversion_implementation!(isize);
elevation_conversion_implementation!(f64);
elevation_conversion_implementation!(i64);
elevation_conversion_implementation!(u64);
impl Add for ResolvedElevation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}
impl Sub for ResolvedElevation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0 - rhs.0)
    }
}
impl ResolvedElevation {
    pub fn new(l: f32) -> Self {
        Self(l)
    }
}
macro_rules! elevation_conversion_implementation {
    ($i:ty) => {
        impl From<$i> for ResolvedElevation {
            fn from(value: $i) -> Self {
                Self::new(value as f32)
            }
        }
    };
}
elevation_conversion_implementation!(f32);
elevation_conversion_implementation!(i32);
elevation_conversion_implementation!(u32);
elevation_conversion_implementation!(usize);
elevation_conversion_implementation!(isize);
elevation_conversion_implementation!(f64);
elevation_conversion_implementation!(i64);
elevation_conversion_implementation!(u64);
