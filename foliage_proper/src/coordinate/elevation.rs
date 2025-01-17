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
#[derive(Copy, Clone, PartialEq, PartialOrd, Component, Debug)]
#[require(ResolvedElevation)]
#[component(on_insert = Self::on_insert)]
pub struct Elevation {
    pub amount: f32,
    pub(crate) absolute: bool,
}
impl Default for Elevation {
    fn default() -> Self {
        Self::abs(0)
    }
}
impl Elevation {
    pub fn abs(e: i32) -> Self {
        Self {
            amount: 100f32 - e as f32,
            absolute: true,
        }
    }
    pub fn up(u: i32) -> Self {
        Self {
            amount: u as f32 * -1f32,
            absolute: false,
        }
    }
    pub fn down(d: i32) -> Self {
        Self {
            amount: d as f32,
            absolute: false,
        }
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
            .unwrap_or(ResolvedElevation(0f32));
        let elev = world.get::<Elevation>(this).unwrap();
        let resolved = if elev.absolute {
            ResolvedElevation(elev.amount)
        } else {
            ResolvedElevation(elev.amount + current.value())
        };
        println!(
            "elev {} current {} = res {} for {:?}",
            elev.amount, current.0, resolved.0, this
        );
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
        Interpolations::new().with(start.amount, end.amount)
    }
    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(e) = interpolations.read(0) {
            self.amount = e;
        }
    }
}
impl Attachment for Elevation {
    fn attach(foliage: &mut Foliage) {
        foliage.enable_animation::<Self>();
    }
}
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
