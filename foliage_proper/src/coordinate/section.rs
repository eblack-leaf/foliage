use std::any::TypeId;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Mul, Sub};

use bevy_ecs::component::{Component, ComponentId};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;
use bytemuck::{Pod, Zeroable};

use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::{
    CoordinateContext, CoordinateUnit, Coordinates, Logical, Numerical, Physical,
};
use crate::{Branch, Location, Stack, StackDeps, Update, Write};

#[derive(Copy, Clone, Default, Component, Debug, PartialEq, PartialOrd)]
#[component(on_insert = Section::<Logical>::on_insert)]
pub struct Section<Context: CoordinateContext> {
    pub position: Position<Context>,
    pub area: Area<Context>,
}
impl<Context: CoordinateContext> Display for Section<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[{} + {} | {} + {}]",
            self.left(),
            self.width(),
            self.top(),
            self.height()
        ))
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, Component, PartialEq, Debug)]
pub struct CReprSection {
    pub pos: CReprPosition,
    pub area: CReprArea,
}
impl CReprSection {
    pub fn new(p: CReprPosition, a: CReprArea) -> Self {
        Self { pos: p, area: a }
    }
    pub fn rounded(self) -> Self {
        Self::new(
            CReprPosition(self.pos.0.rounded()),
            CReprArea(self.area.0.rounded()),
        )
    }
}
impl Section<Numerical> {
    pub fn physical<P: Into<Position<Physical>>, A: Into<Area<Physical>>>(
        p: P,
        a: A,
    ) -> Section<Physical> {
        Section::new(p, a)
    }
    pub fn logical<P: Into<Position<Logical>>, A: Into<Area<Logical>>>(
        p: P,
        a: A,
    ) -> Section<Logical> {
        Section::new(p, a)
    }
    pub fn numerical<P: Into<Position<Numerical>>, A: Into<Area<Numerical>>>(
        p: P,
        a: A,
    ) -> Section<Numerical> {
        Section::new(p, a)
    }
}
impl Section<Physical> {
    pub fn c_repr(self) -> CReprSection {
        CReprSection::new(self.position.c_repr(), self.area.c_repr())
    }
    pub fn to_logical(self, scale_factor: f32) -> Section<Logical> {
        Section::new(
            self.position.to_logical(scale_factor),
            self.area.to_logical(scale_factor),
        )
    }
}
impl Section<Logical> {
    pub fn to_physical(self, factor: f32) -> Section<Physical> {
        Section::new(
            self.position.to_physical(factor),
            self.area.to_physical(factor),
        )
    }
}
impl<Context: CoordinateContext> Section<Context> {
    pub fn new<P: Into<Position<Context>>, A: Into<Area<Context>>>(p: P, a: A) -> Self {
        Self {
            position: p.into(),
            area: a.into(),
        }
    }
    pub fn left(&self) -> CoordinateUnit {
        self.position.left()
    }
    pub fn set_left(&mut self, x: CoordinateUnit) {
        self.position.set_left(x);
    }
    pub fn top(&self) -> CoordinateUnit {
        self.position.top()
    }
    pub fn set_top(&mut self, y: CoordinateUnit) {
        self.position.set_top(y);
    }
    pub fn width(&self) -> CoordinateUnit {
        self.area.width()
    }
    pub fn set_width(&mut self, w: CoordinateUnit) {
        self.area.set_width(w);
    }
    pub fn height(&self) -> CoordinateUnit {
        self.area.height()
    }
    pub(crate) fn with_height(mut self, h: f32) -> Self {
        self.set_height(h);
        self
    }
    pub(crate) fn with_width(mut self, w: f32) -> Self {
        self.set_width(w);
        self
    }
    pub fn set_height(&mut self, h: CoordinateUnit) {
        self.area.set_height(h);
    }
    pub fn set_position<P: Into<Position<Context>>>(&mut self, p: P) {
        self.position = p.into();
    }
    pub fn set_area<A: Into<Area<Context>>>(&mut self, a: A) {
        self.area = a.into();
    }
    pub fn right(&self) -> CoordinateUnit {
        self.left() + self.width()
    }
    pub fn bottom(&self) -> CoordinateUnit {
        self.top() + self.height()
    }
    pub fn center(&self) -> Position<Context> {
        Position::new((
            self.left() + self.width() / 2f32,
            self.top() + self.height() / 2f32,
        ))
    }
    pub fn intersection(&self, o: Self) -> Option<Section<Context>> {
        let left = self.left().max(o.left());
        let top = self.top().max(o.top());
        let right = self.right().min(o.right());
        let bottom = self.bottom().min(o.bottom());
        let section = Section::new((left, top), (right - left, bottom - top));
        if right < left || bottom < top {
            return None;
        }
        Some(section)
    }
    pub fn contacts(&self, o: Self) -> bool {
        self.intersection(o).is_some()
    }
    pub fn contains(&self, p: Position<Context>) -> bool {
        p.left() <= self.right()
            && p.left() >= self.left()
            && p.top() <= self.bottom()
            && p.top() >= self.top()
    }
    pub fn normalized<C: Into<Coordinates>>(&self, c: C) -> Self {
        let c = c.into();
        Self::new(
            self.position.coordinates.normalized(c),
            self.area.coordinates.normalized(c),
        )
    }
    pub fn min(self, o: Self) -> Self {
        Self::new(
            self.position.min(o.position).coordinates,
            self.area.min(o.area).coordinates,
        )
    }
    pub fn max(self, o: Self) -> Self {
        Self::new(
            self.position.max(o.position).coordinates,
            self.area.max(o.area).coordinates,
        )
    }
    pub fn to_numerical(self) -> Section<Numerical> {
        Section::new(self.position.to_numerical(), self.area.to_numerical())
    }
    pub fn rounded(self) -> Self {
        Self::new(self.position.rounded(), self.area.rounded())
    }
    pub fn floored(self) -> Self {
        Self::new(self.position.floored(), self.area.floored())
    }
    pub fn abs(self) -> Self {
        Self::new(self.position.abs(), self.area.abs())
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        if TypeId::of::<Self>() != TypeId::of::<Section<Logical>>() {
            return;
        }
        world.trigger_targets(Write::<Self>::new(), this);
        let mut deps = world.get::<Branch>(this).unwrap().ids.clone();
        for d in deps.clone().iter() {
            if let Some(stack) = world.get::<Stack>(*d) {
                if stack.id.is_some() {
                    deps.remove(d);
                }
            }
        }
        if let Some(d) = world.get::<StackDeps>(this) {
            deps.extend(d.ids.clone());
        }
        if deps.is_empty() {
            return;
        }
        world.commands().trigger_targets(
            Update::<Location>::new(),
            deps.iter().copied().collect::<Vec<_>>(),
        );
    }
}
impl Section<Numerical> {
    pub fn as_logical(self) -> Section<Logical> {
        Section::new(self.position.as_logical(), self.area.as_logical())
    }
    pub fn as_physical(self) -> Section<Physical> {
        Section::new(self.position.as_physical(), self.area.as_physical())
    }
}
impl<Context: CoordinateContext, C: Into<Coordinates>, D: Into<Coordinates>> From<(C, D)>
    for Section<Context>
{
    fn from(value: (C, D)) -> Self {
        Self::new(value.0, value.1)
    }
}
impl<Context: CoordinateContext> Add for Section<Context> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.position + rhs.position, self.area + rhs.area)
    }
}
impl<Context: CoordinateContext> AddAssign for Section<Context> {
    fn add_assign(&mut self, rhs: Self) {
        self.position += rhs.position;
        self.area += rhs.area;
    }
}
impl<Context: CoordinateContext> Sub for Section<Context> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.position - rhs.position, self.area - rhs.area)
    }
}

impl<Context: CoordinateContext> Mul<f32> for Section<Context> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.position * rhs, self.area * rhs)
    }
}
