use crate::EcsExtension;
use bevy_ecs::lifecycle::HookContext;
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
use crate::{Anchor, AnchorDeps, Branch, Location, Resolve, Resolved};

#[derive(Copy, Clone, Default, Component, PartialEq, PartialOrd)]
#[component(on_insert = Section::<Logical>::on_insert)]
/// A rectangle: a top-left [`Position`] plus an [`Area`], in one coordinate space.
///
/// This is the resolved geometry of an entity -- what a [`Location`](crate::Location)
/// produces, what the renderer draws into, and what hit-testing and clipping compare
/// against. Authors read it; the layout pass writes it.
///
/// Construct via [`Section::logical`]/[`physical`](Section::physical) or from a
/// `(position, area)` tuple. Conversions between spaces are explicit and take the current
/// [`ScaleFactor`](crate::ScaleFactor).
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
// See `Coordinates`' `Debug` impl -- delegate to the already-compact `Display` instead of
// the derived form, which spelled out `position`/`area`'s full nested struct names.
impl<Context: CoordinateContext> std::fmt::Debug for Section<Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Default, Component, PartialEq, Debug)]
/// A [`Section`] in GPU-upload layout.
pub struct CReprSection {
    pub pos: CReprPosition,
    pub area: CReprArea,
}
impl CReprSection {
    /// A GPU-layout rectangle from an already-converted position and area.
    pub fn new(p: CReprPosition, a: CReprArea) -> Self {
        Self { pos: p, area: a }
    }
    /// Rounds every component to the nearest whole device pixel.
    pub fn rounded(self) -> Self {
        Self::new(
            CReprPosition(self.pos.0.rounded()),
            CReprArea(self.area.0.rounded()),
        )
    }
}
impl Section<Numerical> {
    /// A rectangle in device pixels.
    pub fn physical<P: Into<Position<Physical>>, A: Into<Area<Physical>>>(
        p: P,
        a: A,
    ) -> Section<Physical> {
        Section::new(p, a)
    }
    /// A rectangle in scale-factor-independent pixels -- the space layout is authored in.
    pub fn logical<P: Into<Position<Logical>>, A: Into<Area<Logical>>>(
        p: P,
        a: A,
    ) -> Section<Logical> {
        Section::new(p, a)
    }
    /// A rectangle in no particular space, for plain number pairs.
    pub fn numerical<P: Into<Position<Numerical>>, A: Into<Area<Numerical>>>(
        p: P,
        a: A,
    ) -> Section<Numerical> {
        Section::new(p, a)
    }
}
impl Section<Physical> {
    /// This rectangle in its GPU-upload layout.
    pub fn c_repr(self) -> CReprSection {
        CReprSection::new(self.position.c_repr(), self.area.c_repr())
    }
    /// Divides out `scale_factor` to get back to logical pixels.
    pub fn to_logical(self, scale_factor: f32) -> Section<Logical> {
        Section::new(
            self.position.to_logical(scale_factor),
            self.area.to_logical(scale_factor),
        )
    }
}
impl Section<Logical> {
    /// Multiplies by `factor` to get device pixels for rendering.
    pub fn to_physical(self, factor: f32) -> Section<Physical> {
        Section::new(
            self.position.to_physical(factor),
            self.area.to_physical(factor),
        )
    }
}
impl<Context: CoordinateContext> Section<Context> {
    /// A rectangle from a top-left position and an area.
    pub fn new<P: Into<Position<Context>>, A: Into<Area<Context>>>(p: P, a: A) -> Self {
        Self {
            position: p.into(),
            area: a.into(),
        }
    }
    /// Left edge.
    pub fn left(&self) -> CoordinateUnit {
        self.position.left()
    }
    /// Moves the left edge, keeping the width.
    pub fn set_left(&mut self, x: CoordinateUnit) {
        self.position.set_left(x);
    }
    /// Top edge.
    pub fn top(&self) -> CoordinateUnit {
        self.position.top()
    }
    /// Moves the top edge, keeping the height.
    pub fn set_top(&mut self, y: CoordinateUnit) {
        self.position.set_top(y);
    }
    /// Width.
    pub fn width(&self) -> CoordinateUnit {
        self.area.width()
    }
    /// Resizes horizontally, keeping the left edge.
    pub fn set_width(&mut self, w: CoordinateUnit) {
        self.area.set_width(w);
    }
    /// Height.
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
    /// Resizes vertically, keeping the top edge.
    pub fn set_height(&mut self, h: CoordinateUnit) {
        self.area.set_height(h);
    }
    /// Moves the rectangle without resizing it.
    pub fn set_position<P: Into<Position<Context>>>(&mut self, p: P) {
        self.position = p.into();
    }
    /// Resizes the rectangle, keeping its top-left corner.
    pub fn set_area<A: Into<Area<Context>>>(&mut self, a: A) {
        self.area = a.into();
    }
    /// Right edge: `left + width`.
    pub fn right(&self) -> CoordinateUnit {
        self.left() + self.width()
    }
    /// Bottom edge: `top + height`.
    pub fn bottom(&self) -> CoordinateUnit {
        self.top() + self.height()
    }
    /// Midpoint of the rectangle.
    pub fn center(&self) -> Position<Context> {
        Position::new((
            self.left() + self.width() / 2f32,
            self.top() + self.height() / 2f32,
        ))
    }
    /// The overlapping rectangle, or `None` when the two do not overlap -- how a clip
    /// region is narrowed against its parent's.
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
    /// Whether the two rectangles overlap at all.
    pub fn contacts(&self, o: Self) -> bool {
        self.intersection(o).is_some()
    }
    /// Whether `p` falls inside this rectangle -- the rectangular hit test.
    pub fn contains(&self, p: Position<Context>) -> bool {
        p.left() <= self.right()
            && p.left() >= self.left()
            && p.top() <= self.bottom()
            && p.top() >= self.top()
    }
    /// This rectangle as a fraction of `c`, for expressing it in another's own terms.
    pub fn normalized<C: Into<Coordinates>>(&self, c: C) -> Self {
        let c = c.into();
        Self::new(
            self.position.coordinates.normalized(c),
            self.area.coordinates.normalized(c),
        )
    }
    /// Component-wise minimum of position and area.
    pub fn min(self, o: Self) -> Self {
        Self::new(
            self.position.min(o.position).coordinates,
            self.area.min(o.area).coordinates,
        )
    }
    /// Component-wise maximum of position and area.
    pub fn max(self, o: Self) -> Self {
        Self::new(
            self.position.max(o.position).coordinates,
            self.area.max(o.area).coordinates,
        )
    }
    /// Drops the space marker, keeping the numbers.
    pub fn to_numerical(self) -> Section<Numerical> {
        Section::new(self.position.to_numerical(), self.area.to_numerical())
    }
    /// Rounds every component to the nearest whole unit -- applied before rasterizing so
    /// edges land on pixel boundaries instead of being resampled.
    pub fn rounded(self) -> Self {
        Self::new(self.position.rounded(), self.area.rounded())
    }
    /// Rounds every component down.
    pub fn floored(self) -> Self {
        Self::new(self.position.floored(), self.area.floored())
    }
    /// Absolute value of every component, normalizing a rectangle built with a negative
    /// extent.
    pub fn abs(self) -> Self {
        Self::new(self.position.abs(), self.area.abs())
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        if TypeId::of::<Self>() != TypeId::of::<Section<Logical>>() {
            return;
        }
        world.trigger_targets(Resolved::<Self>::new(), this);
        let mut deps = world.get::<Branch>(this).unwrap().ids.clone();
        for d in deps.clone().iter() {
            if let Some(stack) = world.get::<Anchor>(*d) {
                if stack.id.is_some() {
                    deps.remove(d);
                }
            }
        }
        if let Some(d) = world.get::<AnchorDeps>(this) {
            deps.extend(d.ids.clone());
        }
        if deps.is_empty() {
            return;
        }
        let dep_vec = deps.iter().copied().collect::<Vec<_>>();
        tracing::trace!(entity = ?this, deps = ?dep_vec, "coordinate::section: Section<Logical> on_insert cascading Resolve<Location>");
        world
            .commands()
            .trigger_targets(Resolve::<Location>::new(), dep_vec);
    }
}
impl Section<Numerical> {
    /// Reinterprets these numbers as logical pixels, without scaling.
    pub fn as_logical(self) -> Section<Logical> {
        Section::new(self.position.as_logical(), self.area.as_logical())
    }
    /// Reinterprets these numbers as device pixels, without scaling.
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
