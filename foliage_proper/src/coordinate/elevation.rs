use crate::anim::interpolation::Interpolations;
use crate::{Animate, Attachment, Branch, Foliage, Stem, Tree, Update};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, OnInsert, Trigger};
use bevy_ecs::system::Query;
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
impl Attachment for Elevation {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Elevation::update);
        foliage.define(Elevation::stem_insert);
        foliage.enable_animation::<Self>();
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
    fn stem_insert(trigger: Trigger<OnInsert, Stem>, mut tree: Tree) {
        tree.trigger_targets(Update::<Elevation>::new(), trigger.entity());
    }
    fn update(
        trigger: Trigger<Update<Elevation>>,
        mut tree: Tree,
        resolved: Query<&ResolvedElevation>,
        elevation: Query<&Elevation>,
        stem: Query<&Stem>,
        branch: Query<&Branch>,
    ) {
        let this = trigger.entity();
        if stem.get(this).ok().is_none() || branch.get(this).ok().is_none() {
            return;
        }
        let current = stem
            .get(this)
            .unwrap()
            .id
            .and_then(|id| Some(*resolved.get(id).unwrap()))
            .unwrap_or(ResolvedElevation(0f32));
        let elev = elevation.get(this).unwrap();
        let resolved = if elev.absolute {
            ResolvedElevation(elev.amount)
        } else {
            ResolvedElevation(elev.amount + current.value())
        };
        println!(
            "elev {} current {} = res {} for {:?}",
            elev.amount, current.0, resolved.0, this
        );
        tree.entity(this).insert(resolved);
        for dep in branch.get(this).unwrap().ids.clone() {
            if let Some(elev) = elevation.get(dep).copied().ok() {
                tree.entity(dep).insert(elev);
            }
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .trigger_targets(Update::<Elevation>::new(), this);
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
