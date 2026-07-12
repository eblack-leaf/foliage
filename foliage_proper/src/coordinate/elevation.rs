use crate::anim::interpolation::Interpolations;
use crate::EcsExtension;
use crate::Trigger;
use crate::{Animate, Attachment, Branch, Foliage, Stem, Tree, Update};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::Component;
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
    fn stem_insert(trigger: Trigger<Insert, Stem>, mut tree: Tree) {
        tree.trigger_targets(Update::<Elevation>::new(), trigger.event_target());
    }
    fn update(
        trigger: Trigger<Update<Elevation>>,
        mut tree: Tree,
        resolved: Query<&ResolvedElevation>,
        elevation: Query<&Elevation>,
        stem: Query<&Stem>,
        branch: Query<&Branch>,
    ) {
        let this = trigger.event_target();
        if stem.get(this).ok().is_none() || branch.get(this).ok().is_none() {
            return;
        }
        // A stem-less entity has no parent to resolve `up`/`down` against — fall back to the
        // same front-most baseline `Elevation::default()` (`abs(0)`) already uses, not 0 (the
        // very floor of the valid range). Without this, a stem-less root using `up(0)` resolves
        // to exactly 0, and any child at `up(1)` goes negative — invisible, with no panic or
        // warning at all. Every existing stem-less root in this codebase already uses `abs()`
        // (never `up`/`down`), so this can't change any current behavior — it only prevents a
        // mistake that was previously silent instead of catching it after the fact.
        let current = stem
            .get(this)
            .unwrap()
            .id
            .and_then(|id| Some(*resolved.get(id).unwrap()))
            .unwrap_or(ResolvedElevation(100f32));
        let elev = elevation.get(this).unwrap();
        let resolved = if elev.absolute {
            ResolvedElevation(elev.amount)
        } else {
            ResolvedElevation(elev.amount + current.value())
        };
        tracing::trace!(entity = ?this, ?resolved, "elevation: computed");
        tree.entity(this).insert(resolved);
        for dep in branch.get(this).unwrap().ids.clone() {
            if let Some(elev) = elevation.get(dep).copied().ok() {
                tree.entity(dep).insert(elev);
            }
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
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
