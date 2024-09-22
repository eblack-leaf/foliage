use crate::anim::{Animate, Interpolations};
use crate::coordinate::area::Area;
use crate::coordinate::points::Points;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, Coordinates, LogicalContext};
use crate::ginkgo::viewport::ViewportHandle;
use crate::layout::{Layout, LayoutGrid};
use crate::leaf::{IdTable, LeafHandle, Stem};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, DetectChanges};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{ParamSet, Query, Res, ResMut};
use std::cmp::{Ordering, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Mul, Sub};

pub trait TokenUnit {
    fn px(self) -> LocationAspectToken;
    fn percent(self) -> PercentDescriptor;
    fn column(self) -> ColumnDescriptor;
    fn row(self) -> RowDescriptor;
}
pub struct RowDescriptor {
    value: i32,
    is_end: bool,
}
impl RowDescriptor {
    pub fn begin(mut self) -> Self {
        self.is_end = false;
        self
    }
    pub fn end(mut self) -> Self {
        self.is_end = true;
        self
    }
    pub fn of<GC: Into<GridContext>>(self, gc: GC) -> LocationAspectToken {
        LocationAspectToken::new(
            LocationAspectTokenOp::Add,
            gc.into(),
            LocationAspectTokenValue::Relative(RelativeUnit::Row(self.value, self.is_end)),
        )
    }
}
pub struct ColumnDescriptor {
    value: i32,
    is_end: bool,
}
impl ColumnDescriptor {
    pub fn begin(mut self) -> Self {
        self.is_end = false;
        self
    }
    pub fn end(mut self) -> Self {
        self.is_end = true;
        self
    }
    pub fn of<GC: Into<GridContext>>(self, gc: GC) -> LocationAspectToken {
        LocationAspectToken::new(
            LocationAspectTokenOp::Add,
            gc.into(),
            LocationAspectTokenValue::Relative(RelativeUnit::Column(self.value, self.is_end)),
        )
    }
}
pub struct PercentDescriptor {
    value: CoordinateUnit,
    use_width: bool,
}
impl PercentDescriptor {
    pub fn from<GC: Into<GridContext>>(mut self, gc: GC) -> LocationAspectToken {
        LocationAspectToken::new(
            LocationAspectTokenOp::Add,
            gc.into(),
            LocationAspectTokenValue::Relative(RelativeUnit::Percent(
                self.value,
                self.use_width,
                true,
            )),
        )
    }
    pub fn width(mut self) -> Self {
        self.use_width = true;
        self
    }
    pub fn height(mut self) -> Self {
        self.use_width = false;
        self
    }
    pub fn of<GC: Into<GridContext>>(self, gc: GC) -> LocationAspectToken {
        LocationAspectToken::new(
            LocationAspectTokenOp::Add,
            gc.into(),
            LocationAspectTokenValue::Relative(RelativeUnit::Percent(
                self.value,
                self.use_width,
                false,
            )),
        )
    }
}
impl TokenUnit for i32 {
    fn px(self) -> LocationAspectToken {
        LocationAspectToken::new(
            LocationAspectTokenOp::Add,
            GridContext::Absolute,
            LocationAspectTokenValue::Absolute(self as CoordinateUnit),
        )
    }
    fn percent(self) -> PercentDescriptor {
        PercentDescriptor {
            value: self as CoordinateUnit / 100.0,
            use_width: false,
        }
    }
    fn column(self) -> ColumnDescriptor {
        ColumnDescriptor {
            value: self,
            is_end: false,
        }
    }
    fn row(self) -> RowDescriptor {
        RowDescriptor {
            value: self,
            is_end: false,
        }
    }
}
impl From<LocationAspectToken> for SpecifiedDescriptorValue {
    fn from(value: LocationAspectToken) -> Self {
        SpecifiedDescriptorValue::new(value)
    }
}
impl Add for LocationAspectToken {
    type Output = SpecifiedDescriptorValue;

    fn add(self, rhs: Self) -> Self::Output {
        SpecifiedDescriptorValue::new(self).plus(rhs)
    }
}
impl Add<LocationAspectToken> for SpecifiedDescriptorValue {
    type Output = SpecifiedDescriptorValue;

    fn add(self, rhs: LocationAspectToken) -> Self::Output {
        self.plus(rhs)
    }
}

impl Sub for LocationAspectToken {
    type Output = SpecifiedDescriptorValue;

    fn sub(self, rhs: Self) -> Self::Output {
        SpecifiedDescriptorValue::new(self).minus(rhs)
    }
}

impl Sub<LocationAspectToken> for SpecifiedDescriptorValue {
    type Output = SpecifiedDescriptorValue;

    fn sub(self, rhs: LocationAspectToken) -> Self::Output {
        self.minus(rhs)
    }
}
#[derive(Clone, Hash, PartialEq, Eq, Debug, PartialOrd)]
pub enum GridContext {
    Screen,
    Stem,
    Named(LeafHandle),
    Absolute,
}
impl<LH: Into<LeafHandle>> From<LH> for GridContext {
    fn from(value: LH) -> Self {
        GridContext::Named(value.into())
    }
}
impl GridContext {
    fn context_token(self, aspect: GridAspect) -> LocationAspectToken {
        LocationAspectToken::new(
            LocationAspectTokenOp::Add,
            self,
            LocationAspectTokenValue::ContextAspect(aspect),
        )
    }
    pub fn top(self) -> LocationAspectToken {
        self.context_token(GridAspect::Top)
    }
    pub fn bottom(self) -> LocationAspectToken {
        self.context_token(GridAspect::Bottom)
    }
    pub fn left(self) -> LocationAspectToken {
        self.context_token(GridAspect::Left)
    }
    pub fn right(self) -> LocationAspectToken {
        self.context_token(GridAspect::Right)
    }
    pub fn width(self) -> LocationAspectToken {
        self.context_token(GridAspect::Width)
    }
    pub fn height(self) -> LocationAspectToken {
        self.context_token(GridAspect::Height)
    }
    pub fn center_x(self) -> LocationAspectToken {
        self.context_token(GridAspect::CenterX)
    }
    pub fn center_y(self) -> LocationAspectToken {
        self.context_token(GridAspect::CenterY)
    }
    pub fn point_ax(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointAX)
    }
    pub fn point_ay(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointAY)
    }
    pub fn point_bx(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointBX)
    }
    pub fn point_by(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointBY)
    }
    pub fn point_cx(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointCX)
    }
    pub fn point_cy(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointCY)
    }
    pub fn point_dx(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointDX)
    }
    pub fn point_dy(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointDY)
    }
}
pub trait ContextUnit {
    fn top(self) -> LocationAspectToken;
    fn bottom(self) -> LocationAspectToken;
    fn left(self) -> LocationAspectToken;
    fn right(self) -> LocationAspectToken;
    fn width(self) -> LocationAspectToken;
    fn height(self) -> LocationAspectToken;
    fn center_x(self) -> LocationAspectToken;
    fn center_y(self) -> LocationAspectToken;
    fn point_ax(self) -> LocationAspectToken;
    fn point_ay(self) -> LocationAspectToken;
    fn point_bx(self) -> LocationAspectToken;
    fn point_by(self) -> LocationAspectToken;
    fn point_cx(self) -> LocationAspectToken;
    fn point_cy(self) -> LocationAspectToken;
    fn point_dx(self) -> LocationAspectToken;
    fn point_dy(self) -> LocationAspectToken;
}
fn named_token<LH: Into<LeafHandle>>(lh: LH, aspect: GridAspect) -> LocationAspectToken {
    LocationAspectToken::new(
        LocationAspectTokenOp::Add,
        GridContext::Named(lh.into()),
        LocationAspectTokenValue::ContextAspect(aspect),
    )
}
impl<LH: Into<LeafHandle>> ContextUnit for LH {
    fn top(self) -> LocationAspectToken {
        named_token(self, GridAspect::Top)
    }
    fn bottom(self) -> LocationAspectToken {
        named_token(self, GridAspect::Bottom)
    }
    fn left(self) -> LocationAspectToken {
        named_token(self, GridAspect::Left)
    }
    fn right(self) -> LocationAspectToken {
        named_token(self, GridAspect::Right)
    }
    fn width(self) -> LocationAspectToken {
        named_token(self, GridAspect::Width)
    }
    fn height(self) -> LocationAspectToken {
        named_token(self, GridAspect::Height)
    }
    fn center_x(self) -> LocationAspectToken {
        named_token(self, GridAspect::CenterX)
    }
    fn center_y(self) -> LocationAspectToken {
        named_token(self, GridAspect::CenterY)
    }
    fn point_ax(self) -> LocationAspectToken {
        named_token(self, GridAspect::PointAX)
    }
    fn point_ay(self) -> LocationAspectToken {
        named_token(self, GridAspect::PointAY)
    }
    fn point_bx(self) -> LocationAspectToken {
        named_token(self, GridAspect::PointBX)
    }
    fn point_by(self) -> LocationAspectToken {
        named_token(self, GridAspect::PointBY)
    }
    fn point_cx(self) -> LocationAspectToken {
        named_token(self, GridAspect::PointCX)
    }
    fn point_cy(self) -> LocationAspectToken {
        named_token(self, GridAspect::PointCY)
    }
    fn point_dx(self) -> LocationAspectToken {
        named_token(self, GridAspect::PointDX)
    }
    fn point_dy(self) -> LocationAspectToken {
        named_token(self, GridAspect::PointDY)
    }
}
pub fn screen() -> GridContext {
    GridContext::Screen
}
pub fn stem() -> GridContext {
    GridContext::Stem
}
#[derive(Clone, Copy)]
pub(crate) enum LocationAspectTokenOp {
    Add,
    Minus,
    // ...
}
#[derive(Clone, Copy)]
pub enum RelativeUnit {
    Column(i32, bool),
    Row(i32, bool),
    Percent(f32, bool, bool),
}
#[derive(Clone, Copy)]
pub enum LocationAspectTokenValue {
    ContextAspect(GridAspect),
    Relative(RelativeUnit),
    Absolute(CoordinateUnit),
}
#[derive(Clone)]
pub struct LocationAspectToken {
    op: LocationAspectTokenOp,
    context: GridContext,
    value: LocationAspectTokenValue,
}
impl LocationAspectToken {
    pub(crate) fn new(
        op: LocationAspectTokenOp,
        context: GridContext,
        value: LocationAspectTokenValue,
    ) -> Self {
        Self { op, context, value }
    }
}
#[derive(Clone)]
pub struct SpecifiedDescriptorValue {
    tokens: Vec<LocationAspectToken>,
}

impl SpecifiedDescriptorValue {
    pub(crate) fn resolve(
        &self,
        stem: &Stem,
        ref_context: &HashMap<GridContext, ReferentialData>,
    ) -> CoordinateUnit {
        let mut accumulator = 0.0;
        for t in self.tokens.iter() {
            let value = match t.value {
                LocationAspectTokenValue::ContextAspect(ca) => {
                    println!("resolving context: {:?}  {:?}", &t.context, ref_context);
                    let data = match &t.context {
                        GridContext::Stem => {
                            if stem.0.is_some() {
                                let stem_handle = stem.0.clone().unwrap();
                                println!("stem-handle: {:?}", stem_handle);
                                ref_context.get(&GridContext::Named(stem_handle)).unwrap()
                            } else {
                                ref_context.get(&GridContext::Screen).unwrap()
                            }
                        }
                        _ => ref_context.get(&t.context).unwrap()
                    };
                    match ca {
                        GridAspect::Top => data.resolved.section.y(),
                        GridAspect::Height => data.resolved.section.height(),
                        GridAspect::CenterY => data.resolved.section.center().y(),
                        GridAspect::Bottom => data.resolved.section.bottom(),
                        GridAspect::Left => data.resolved.section.x(),
                        GridAspect::Width => data.resolved.section.width(),
                        GridAspect::CenterX => data.resolved.section.center().x(),
                        GridAspect::Right => data.resolved.section.right(),
                        GridAspect::PointAX => data
                            .resolved
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[0].x()))
                            .unwrap_or_default(),
                        GridAspect::PointAY => data
                            .resolved
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[0].y()))
                            .unwrap_or_default(),
                        GridAspect::PointBX => data
                            .resolved
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[1].x()))
                            .unwrap_or_default(),
                        GridAspect::PointBY => data
                            .resolved
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[1].y()))
                            .unwrap_or_default(),
                        GridAspect::PointCX => data
                            .resolved
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[2].x()))
                            .unwrap_or_default(),
                        GridAspect::PointCY => data
                            .resolved
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[2].y()))
                            .unwrap_or_default(),
                        GridAspect::PointDX => data
                            .resolved
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[3].x()))
                            .unwrap_or_default(),
                        GridAspect::PointDY => data
                            .resolved
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[3].y()))
                            .unwrap_or_default(),
                    }
                }
                LocationAspectTokenValue::Relative(rel) => {
                    let data = match &t.context {
                        GridContext::Stem => {
                            if stem.0.is_some() {
                                ref_context.get(&GridContext::Named(stem.0.clone().unwrap())).unwrap()
                            } else {
                                ref_context.get(&GridContext::Screen).unwrap()
                            }
                        }
                        _ => ref_context.get(&t.context).unwrap()
                    };
                    match rel {
                        RelativeUnit::Column(c, use_end) => {
                            data.resolved.section.x()
                                + (c as f32 - 1.0 * f32::from(!use_end))
                                    * (data.resolved.section.width() / data.grid.columns as f32)
                                + c as f32 * data.grid.gap.horizontal()
                        }
                        RelativeUnit::Row(r, use_end) => {
                            data.resolved.section.y()
                                + (r as f32 - 1.0 * f32::from(!use_end))
                                    * (data.resolved.section.height() / data.grid.rows as f32)
                                + r as f32 * data.grid.gap.vertical()
                        }
                        RelativeUnit::Percent(p, use_width, include_start) => {
                            data.resolved.section.x() * f32::from(include_start && use_width)
                                + data.resolved.section.width() * p * f32::from(use_width)
                                + data.resolved.section.y() * f32::from(include_start && !use_width)
                                + data.resolved.section.height() * p * f32::from(!use_width)
                        }
                    }
                }
                LocationAspectTokenValue::Absolute(a) => a,
            };
            match t.op {
                LocationAspectTokenOp::Add => {
                    accumulator += value;
                }
                LocationAspectTokenOp::Minus => {
                    accumulator -= value;
                }
            }
        }
        accumulator
    }
    pub(crate) fn minus(mut self, mut other: LocationAspectToken) -> Self {
        other.op = LocationAspectTokenOp::Minus;
        self.tokens.push(other);
        self
    }
    pub(crate) fn plus(mut self, other: LocationAspectToken) -> Self {
        self.tokens.push(other);
        self
    }
    pub(crate) fn new(first: LocationAspectToken) -> SpecifiedDescriptorValue {
        Self {
            tokens: vec![first],
        }
    }
    pub(crate) fn dependencies(&self, this: Entity, stems: &Query<&Stem>) -> ReferentialDependencies {
        let mut set = HashSet::new();
        for token in &self.tokens {
            // if == Root => pull from Stem.unwrap_or(screen())
            let context = match &token.context {
                GridContext::Stem => {
                    if let Ok(s) = stems.get(this) {
                        if let Some(s) = &s.0 {
                            GridContext::Named(s.clone())
                        } else {
                            GridContext::Screen
                        }
                    } else {
                        GridContext::Screen
                    }
                }
                _ => token.context.clone()
            };
            set.insert(context);
        }
        ReferentialDependencies::new(set)
    }
}
#[derive(Default, Clone)]
pub(crate) enum LocationAspectDescriptorValue {
    #[default]
    Existing,
    Specified(SpecifiedDescriptorValue),
}
#[derive(Default, Clone)]
pub(crate) struct LocationAspectDescriptor {
    aspect: GridAspect,
    value: LocationAspectDescriptorValue,
}
impl LocationAspectDescriptor {
    pub(crate) fn new(aspect: GridAspect, value: LocationAspectDescriptorValue) -> Self {
        Self { aspect, value }
    }
}
#[derive(Default, Clone)]
pub(crate) struct LocationAspect {
    aspects: [LocationAspectDescriptor; 2],
    count: u32,
}

impl LocationAspect {
    pub(crate) fn resolve_grid_aspect(
        &self,
        stem: &Stem,
        aspect: GridAspect,
        ref_context: &HashMap<GridContext, ReferentialData>,
    ) -> CoordinateUnit {
        if self.aspects.get(0).unwrap().aspect == aspect {
            if let LocationAspectDescriptorValue::Specified(spec) =
                &self.aspects.get(0).unwrap().value
            {
                spec.resolve(stem, ref_context)
            } else {
                panic!("no existing")
            }
        } else {
            if let LocationAspectDescriptorValue::Specified(spec) =
                &self.aspects.get(1).unwrap().value
            {
                spec.resolve(stem, ref_context)
            } else {
                panic!("no existing")
            }
        }
    }
    pub(crate) fn set<LAD: Into<LocationAspectDescriptorValue>>(
        &mut self,
        aspect: GridAspect,
        desc: LAD,
    ) {
        if self.count == 0 {
            self.aspects[0] = LocationAspectDescriptor::new(aspect, desc.into());
            self.count += 1;
        } else if self.count == 1 {
            self.aspects[1] = LocationAspectDescriptor::new(aspect, desc.into());
            self.count += 1;
        } else {
            panic!("too many dimensions");
        }
    }
    pub fn new() -> LocationAspect {
        LocationAspect {
            aspects: [
                LocationAspectDescriptor::default(),
                LocationAspectDescriptor::default(),
            ],
            count: 0,
        }
    }
    pub(crate) fn config(&self) -> AspectConfiguration {
        match self.aspects[0].aspect {
            GridAspect::Top | GridAspect::Height | GridAspect::Bottom | GridAspect::CenterY => {
                AspectConfiguration::Vertical
            }
            GridAspect::Left | GridAspect::Width | GridAspect::Right | GridAspect::CenterX => {
                AspectConfiguration::Horizontal
            }
            GridAspect::PointAX | GridAspect::PointAY => AspectConfiguration::PointA,
            GridAspect::PointBX | GridAspect::PointBY => AspectConfiguration::PointB,
            GridAspect::PointCX | GridAspect::PointCY => AspectConfiguration::PointC,
            GridAspect::PointDX | GridAspect::PointDY => AspectConfiguration::PointD,
        }
    }
    pub(crate) fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Top,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_top(mut self) -> Self {
        self.set(GridAspect::Top, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Bottom,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_bottom(mut self) -> Self {
        self.set(GridAspect::Bottom, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn left<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Left,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_left(mut self) -> Self {
        self.set(GridAspect::Left, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn right<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Right,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_right(mut self) -> Self {
        self.set(GridAspect::Right, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn width<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Width,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_width(mut self) -> Self {
        self.set(GridAspect::Width, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn height<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Height,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_height(mut self) -> Self {
        self.set(GridAspect::Height, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn center_x<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::CenterX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_center_x(mut self) -> Self {
        self.set(GridAspect::CenterX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn center_y<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::CenterY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_center_y(mut self) -> Self {
        self.set(GridAspect::CenterY, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_ax<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointAX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_ax(mut self) -> Self {
        self.set(GridAspect::PointAX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_ay<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointAY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_ay(mut self) -> Self {
        self.set(GridAspect::PointAY, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_bx<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointBX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_bx(mut self) -> Self {
        self.set(GridAspect::PointBX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_by<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointBY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_by(mut self) -> Self {
        self.set(GridAspect::PointBY, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_cx<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointCX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_cx(mut self) -> Self {
        self.set(GridAspect::PointCX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_cy<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointCY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_cy(mut self) -> Self {
        self.set(GridAspect::PointCY, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_dx<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointDX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_dx(mut self) -> Self {
        self.set(GridAspect::PointDX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_dy<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointDY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_dy(mut self) -> Self {
        self.set(GridAspect::PointDY, LocationAspectDescriptorValue::Existing);
        self
    }
}
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) enum AspectConfiguration {
    Horizontal,
    Vertical,
    PointA,
    PointB,
    PointC,
    PointD,
}
#[derive(Clone, Hash, Eq, PartialEq)]
pub(crate) struct GridLocationException {
    layout: Layout,
    config: AspectConfiguration,
}
impl GridLocationException {
    fn new(layout: Layout, config: AspectConfiguration) -> GridLocationException {
        Self { layout, config }
    }
}
#[derive(Clone, Default)]
pub(crate) struct AnimationHookContext {
    pub(crate) hook_percent: f32,
    pub(crate) last: Section<LogicalContext>,
    diff: Section<LogicalContext>,
    pub(crate) create_diff: bool,
    pub(crate) hook_changed: bool,
}
#[derive(Clone, Default)]
pub(crate) struct PointDrivenAnimationHook {
    pub(crate) point_a: AnimationHookContext,
    pub(crate) point_b: AnimationHookContext,
    pub(crate) point_c: AnimationHookContext,
    pub(crate) point_d: AnimationHookContext,
}
#[derive(Clone)]
pub(crate) enum GridLocationAnimationHook {
    SectionDriven(AnimationHookContext),
    PointDriven(PointDrivenAnimationHook),
}
impl Default for GridLocationAnimationHook {
    fn default() -> Self {
        Self::SectionDriven(AnimationHookContext::default())
    }
}
#[derive(Clone, Component)]
pub struct GridLocation {
    configurations: HashMap<AspectConfiguration, LocationAspect>,
    exceptions: HashMap<GridLocationException, LocationAspect>,
    pub(crate) animation_hook: GridLocationAnimationHook,
}
impl Animate for GridLocation {
    fn interpolations(start: &Self, _end: &Self) -> Interpolations {
        match &start.animation_hook {
            GridLocationAnimationHook::SectionDriven(_) => Interpolations::new().with(1.0, 0.0),
            GridLocationAnimationHook::PointDriven(_) => Interpolations::new()
                .with(1.0, 0.0)
                .with(1.0, 0.0)
                .with(1.0, 0.0)
                .with(1.0, 0.0),
        }
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        match &mut self.animation_hook {
            GridLocationAnimationHook::SectionDriven(hook) => {
                if let Some(p) = interpolations.read(0) {
                    hook.hook_percent = p;
                    hook.hook_changed = true;
                }
            }
            GridLocationAnimationHook::PointDriven(hook) => {
                if let Some(p) = interpolations.read(0) {
                    hook.point_a.hook_percent = p;
                    hook.point_a.hook_changed = true;
                }
                if let Some(p) = interpolations.read(1) {
                    hook.point_b.hook_percent = p;
                    hook.point_b.hook_changed = true;
                }
                if let Some(p) = interpolations.read(2) {
                    hook.point_c.hook_percent = p;
                    hook.point_c.hook_changed = true;
                }
                if let Some(p) = interpolations.read(3) {
                    hook.point_d.hook_percent = p;
                    hook.point_d.hook_changed = true;
                }
            }
        }
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, PartialOrd)]
pub enum GridAspect {
    #[default]
    Top,
    Height,
    CenterY, // Dependent => Top | Height | Bottom
    Bottom,  // Dependent => Height | Top | CenterY
    Left,
    Width,
    CenterX, // Dependent => Right | Width | Left
    Right,   // Dependent => Width | Left | CenterX
    PointAX,
    PointAY,
    PointBX,
    PointBY,
    PointCX,
    PointCY,
    PointDX,
    PointDY,
}

impl GridLocation {
    pub(crate) fn resolve(
        &self,
        stem: &Stem,
        context: &HashMap<GridContext, ReferentialData>,
        layout: Layout,
    ) -> Option<ResolvedLocation> {
        let mut resolution = ResolvedLocation::new();
        for (aspect_config, location_aspect) in self.configurations.iter() {
            let mut to_use = None;
            let base = location_aspect;
            for except in self.exceptions.iter() {
                if except.0.layout.contains(layout) && aspect_config == &except.0.config {
                    to_use = Some(except.1);
                }
            }
            let to_use = to_use.unwrap_or(base);
            let a = match &to_use.aspects[0].value {
                LocationAspectDescriptorValue::Existing => {
                    base.resolve_grid_aspect(stem, to_use.aspects[0].aspect, context)
                }
                LocationAspectDescriptorValue::Specified(spec) => spec.resolve(stem, context),
            };
            let b = match &to_use.aspects[1].value {
                LocationAspectDescriptorValue::Existing => {
                    base.resolve_grid_aspect(stem, to_use.aspects[1].aspect, context)
                }
                LocationAspectDescriptorValue::Specified(spec) => spec.resolve(stem, context),
            };
            let (pair_config, data) = if to_use.aspects[0].aspect < to_use.aspects[1].aspect {
                ((to_use.aspects[0].aspect, to_use.aspects[1].aspect), (a, b))
            } else {
                ((to_use.aspects[1].aspect, to_use.aspects[0].aspect), (b, a))
            };
            match aspect_config {
                AspectConfiguration::Horizontal => {
                    if pair_config == (GridAspect::Left, GridAspect::Right) {
                        resolution.section.position.set_x(data.0);
                        resolution.section.area.set_width(data.1 - data.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::CenterX) {
                        resolution.section.position.set_x(data.0);
                        resolution.section.area.set_width((data.1 - data.0) * 2.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::Width) {
                        resolution.section.position.set_x(data.0);
                        resolution.section.area.set_width(data.1);
                    } else if pair_config == (GridAspect::Width, GridAspect::CenterX) {
                        resolution.section.position.set_x(data.1 - data.0 / 2.0);
                        resolution.section.area.set_width(data.0);
                    } else if pair_config == (GridAspect::Width, GridAspect::Right) {
                        resolution.section.position.set_x(data.1 - data.0);
                        resolution.section.area.set_width(data.0);
                    } else if pair_config == (GridAspect::CenterX, GridAspect::Right) {
                        let diff = data.1 - data.0;
                        resolution.section.position.set_x(data.0 - diff);
                        resolution.section.area.set_width(diff * 2.0);
                    }
                }
                AspectConfiguration::Vertical => {
                    if pair_config == (GridAspect::Top, GridAspect::Bottom) {
                        resolution.section.position.set_y(data.0);
                        resolution.section.area.set_height(data.1 - data.0);
                    } else if pair_config == (GridAspect::Top, GridAspect::CenterY) {
                        resolution.section.position.set_y(data.0);
                        resolution.section.area.set_height((data.1 - data.0) * 2.0);
                    } else if pair_config == (GridAspect::Top, GridAspect::Height) {
                        resolution.section.position.set_y(data.0);
                        resolution.section.area.set_height(data.1);
                    } else if pair_config == (GridAspect::Height, GridAspect::CenterY) {
                        resolution.section.position.set_y(data.1 - data.0 / 2.0);
                        resolution.section.area.set_height(data.0);
                    } else if pair_config == (GridAspect::Height, GridAspect::Bottom) {
                        resolution.section.position.set_y(data.0 - data.1);
                        resolution.section.area.set_height(data.1);
                    } else if pair_config == (GridAspect::CenterY, GridAspect::Bottom) {
                        let diff = data.1 - data.0;
                        resolution.section.position.set_y(data.0 - diff);
                        resolution.section.area.set_height(diff * 2.0);
                    }
                }
                AspectConfiguration::PointA
                | AspectConfiguration::PointB
                | AspectConfiguration::PointC
                | AspectConfiguration::PointD => {
                    if resolution.points.is_none() {
                        resolution.points.replace(Points::default());
                    }
                    match aspect_config {
                        AspectConfiguration::PointA => {
                            if pair_config == (GridAspect::PointAX, GridAspect::PointAY) {
                                resolution.points.as_mut()?.data[0] = data.into();
                            } else {
                                panic!("invalid-configuration aspect")
                            }
                        }
                        AspectConfiguration::PointB => {
                            if pair_config == (GridAspect::PointBX, GridAspect::PointBY) {
                                resolution.points.as_mut()?.data[1] = data.into();
                            } else {
                                panic!("invalid-configuration aspect")
                            }
                        }
                        AspectConfiguration::PointC => {
                            if pair_config == (GridAspect::PointCX, GridAspect::PointCY) {
                                resolution.points.as_mut()?.data[2] = data.into();
                            } else {
                                panic!("invalid-configuration aspect")
                            }
                        }
                        AspectConfiguration::PointD => {
                            if pair_config == (GridAspect::PointDX, GridAspect::PointDY) {
                                resolution.points.as_mut()?.data[3] = data.into();
                            } else {
                                panic!("invalid-configuration aspect")
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        match &self.animation_hook {
            GridLocationAnimationHook::SectionDriven(s) => {
                if s.hook_changed {
                    if s.create_diff {
                        let diff = s.last - resolution.section;
                        let offset = diff * s.hook_percent;
                        resolution.section += offset;
                        resolution
                            .hook_update
                            .replace([Some(diff), None, None, None]);
                    } else {
                        resolution.section += s.diff * s.hook_percent;
                    }
                }
            }
            GridLocationAnimationHook::PointDriven(p) => {
                if p.point_a.create_diff
                    || p.point_b.create_diff
                    || p.point_c.create_diff
                    || p.point_d.create_diff
                {
                    resolution.hook_update.replace([None, None, None, None]);
                }
                if p.point_a.hook_changed {
                    if p.point_a.create_diff {
                        let diff = p.point_a.last
                            - Section::new(resolution.points.as_ref()?.data[0], Area::default());
                        let offset = diff * p.point_a.hook_percent;
                        resolution.points.as_mut()?.data[0] +=
                            Position::new(offset.position.coordinates);
                        resolution.hook_update?.get_mut(0)?.replace(offset);
                    } else {
                        let offset = p.point_a.diff * p.point_a.hook_percent;
                        resolution.points.as_mut()?.data[0] +=
                            Position::new(offset.position.coordinates);
                    }
                }
                if p.point_b.hook_changed {
                    if p.point_b.create_diff {
                        let diff = p.point_b.last
                            - Section::new(resolution.points.as_ref()?.data[1], Area::default());
                        let offset = diff * p.point_b.hook_percent;
                        resolution.points.as_mut()?.data[1] +=
                            Position::new(offset.position.coordinates);
                        resolution.hook_update?.get_mut(1)?.replace(offset);
                    } else {
                        let offset = p.point_b.diff * p.point_b.hook_percent;
                        resolution.points.as_mut()?.data[1] +=
                            Position::new(offset.position.coordinates);
                    }
                }
                if p.point_c.hook_changed {
                    if p.point_c.create_diff {
                        let diff = p.point_c.last
                            - Section::new(resolution.points.as_ref()?.data[2], Area::default());
                        let offset = diff * p.point_c.hook_percent;
                        resolution.points.as_mut()?.data[2] +=
                            Position::new(offset.position.coordinates);
                        resolution.hook_update?.get_mut(2)?.replace(offset);
                    } else {
                        let offset = p.point_c.diff * p.point_c.hook_percent;
                        resolution.points.as_mut()?.data[2] +=
                            Position::new(offset.position.coordinates);
                    }
                }
                if p.point_d.hook_changed {
                    if p.point_d.create_diff {
                        let diff = p.point_d.last
                            - Section::new(resolution.points.as_ref()?.data[3], Area::default());
                        let offset = diff * p.point_d.hook_percent;
                        resolution.points.as_mut()?.data[3] +=
                            Position::new(offset.position.coordinates);
                        resolution.hook_update?.get_mut(3)?.replace(offset);
                    } else {
                        let offset = p.point_d.diff * p.point_d.hook_percent;
                        resolution.points.as_mut()?.data[3] +=
                            Position::new(offset.position.coordinates);
                    }
                }
            }
        }
        if let Some(pts) = resolution.points.as_mut() {
            resolution.section = pts.bbox();
        }
        Some(resolution)
    }
    pub fn new() -> Self {
        Self {
            configurations: Default::default(),
            exceptions: Default::default(),
            animation_hook: Default::default(),
        }
    }
    pub(crate) fn deps(&self, this: Entity, stems: &Query<&Stem>) -> ReferentialDependencies {
        let mut set = HashSet::new();
        for (_config, aspect) in self.configurations.iter() {
            if let LocationAspectDescriptorValue::Specified(s) = &aspect.aspects[0].value {
                set.extend(s.dependencies(this, stems).deps);
            }
            if let LocationAspectDescriptorValue::Specified(s) = &aspect.aspects[1].value {
                set.extend(s.dependencies(this, stems).deps);
            }
        }
        ReferentialDependencies::new(set)
    }
    pub fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Top,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations
                .insert(AspectConfiguration::Vertical, LocationAspect::new().top(d));
        }
        self
    }
    pub fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Bottom,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().bottom(d),
            );
        }
        self
    }
    pub fn height<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Height,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().height(d),
            );
        }
        self
    }
    pub fn center_y<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::CenterY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().center_y(d),
            );
        }
        self
    }
    pub fn left<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Left,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().left(d),
            );
        }
        self
    }
    pub fn right<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Right,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().right(d),
            );
        }
        self
    }
    pub fn width<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Width,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().width(d),
            );
        }
        self
    }
    pub fn center_x<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::CenterX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().center_x(d),
            );
        }
        self
    }
    fn point_driven_check(&mut self) {
        if let GridLocationAnimationHook::SectionDriven(_) = self.animation_hook {
            self.animation_hook =
                GridLocationAnimationHook::PointDriven(PointDrivenAnimationHook::default());
        }
    }
    pub fn point_ax<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointA) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointAX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.point_driven_check();
            self.configurations.insert(
                AspectConfiguration::PointA,
                LocationAspect::new().point_ax(d),
            );
        }
        self
    }
    pub fn point_ay<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointA) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointAY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.point_driven_check();
            self.configurations.insert(
                AspectConfiguration::PointA,
                LocationAspect::new().point_ay(d),
            );
        }
        self
    }
    pub fn point_bx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointB) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointBX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.point_driven_check();
            self.configurations.insert(
                AspectConfiguration::PointB,
                LocationAspect::new().point_bx(d),
            );
        }
        self
    }
    pub fn point_by<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointB) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointBY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.point_driven_check();
            self.configurations.insert(
                AspectConfiguration::PointB,
                LocationAspect::new().point_by(d),
            );
        }
        self
    }
    pub fn point_cx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointC) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointCX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.point_driven_check();
            self.configurations.insert(
                AspectConfiguration::PointC,
                LocationAspect::new().point_cx(d),
            );
        }
        self
    }
    pub fn point_cy<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointC) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointCY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.point_driven_check();
            self.configurations.insert(
                AspectConfiguration::PointC,
                LocationAspect::new().point_cy(d),
            );
        }
        self
    }
    pub fn point_dx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointD) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointDX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.point_driven_check();
            self.configurations.insert(
                AspectConfiguration::PointD,
                LocationAspect::new().point_dx(d),
            );
        }
        self
    }
    pub fn point_dy<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointD) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointDY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.point_driven_check();
            self.configurations.insert(
                AspectConfiguration::PointD,
                LocationAspect::new().point_dy(d),
            );
        }
        self
    }
    pub fn except_at<LA: Into<LocationConfiguration>>(mut self, layout: Layout, la: LA) -> Self {
        let config = la.into();
        for c in config.configurations {
            self.exceptions
                .insert(GridLocationException::new(layout, c.0), c.1);
        }
        self
    }
}
pub struct LocationConfiguration {
    configurations: HashMap<AspectConfiguration, LocationAspect>,
}
impl LocationConfiguration {
    pub fn new() -> Self {
        Self {
            configurations: HashMap::new(),
        }
    }
    pub fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Top,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations
                .insert(AspectConfiguration::Vertical, LocationAspect::new().top(d));
        }
        self
    }
    pub fn existing_top(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            aspect.set(GridAspect::Top, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().existing_top(),
            );
        }
        self
    }
    pub fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Bottom,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().bottom(d),
            );
        }
        self
    }
    pub fn existing_bottom(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            aspect.set(GridAspect::Bottom, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().existing_bottom(),
            );
        }
        self
    }
    pub fn height<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Height,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().height(d),
            );
        }
        self
    }
    pub fn existing_height(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            aspect.set(GridAspect::Height, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().existing_height(),
            );
        }
        self
    }
    pub fn center_y<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::CenterY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().center_y(d),
            );
        }
        self
    }
    pub fn existing_center_y(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            aspect.set(GridAspect::CenterY, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().existing_center_y(),
            );
        }
        self
    }
    pub fn left<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Left,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().left(d),
            );
        }
        self
    }
    pub fn existing_left(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            aspect.set(GridAspect::Left, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().existing_left(),
            );
        }
        self
    }
    pub fn right<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Right,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().right(d),
            );
        }
        self
    }
    pub fn existing_right(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            aspect.set(GridAspect::Right, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().existing_right(),
            );
        }
        self
    }
    pub fn width<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::Width,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().width(d),
            );
        }
        self
    }
    pub fn existing_width(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            aspect.set(GridAspect::Width, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().existing_width(),
            );
        }
        self
    }
    pub fn center_x<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::CenterX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().center_x(d),
            );
        }
        self
    }
    pub fn existing_center_x(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(&AspectConfiguration::Horizontal)
        {
            aspect.set(GridAspect::CenterX, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::Horizontal,
                LocationAspect::new().existing_center_x(),
            );
        }
        self
    }
    pub fn point_ax<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointA) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointAX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::PointA,
                LocationAspect::new().point_ax(d),
            );
        }
        self
    }
    pub fn existing_point_ax(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::PointA) {
            aspect.set(GridAspect::PointAX, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::PointA,
                LocationAspect::new().existing_point_ax(),
            );
        }
        self
    }
    pub fn point_ay<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointA) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointAY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::PointA,
                LocationAspect::new().point_ay(d),
            );
        }
        self
    }
    pub fn existing_point_ay(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::PointA) {
            aspect.set(GridAspect::PointAY, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::PointA,
                LocationAspect::new().existing_point_ay(),
            );
        }
        self
    }
    pub fn point_bx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointB) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointBX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::PointB,
                LocationAspect::new().point_bx(d),
            );
        }
        self
    }
    pub fn existing_point_bx(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::PointB) {
            aspect.set(GridAspect::PointBX, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::PointB,
                LocationAspect::new().existing_point_bx(),
            );
        }
        self
    }
    pub fn point_by<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointB) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointBY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::PointB,
                LocationAspect::new().point_by(d),
            );
        }
        self
    }
    pub fn existing_point_by(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::PointB) {
            aspect.set(GridAspect::PointBY, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::PointB,
                LocationAspect::new().existing_point_by(),
            );
        }
        self
    }
    pub fn point_cx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointC) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointCX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::PointC,
                LocationAspect::new().point_cx(d),
            );
        }
        self
    }
    pub fn existing_point_cx(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::PointC) {
            aspect.set(GridAspect::PointCX, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::PointC,
                LocationAspect::new().existing_point_cx(),
            );
        }
        self
    }
    pub fn point_cy<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointC) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointCY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::PointC,
                LocationAspect::new().point_cy(d),
            );
        }
        self
    }
    pub fn existing_point_cy(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::PointC) {
            aspect.set(GridAspect::PointCY, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::PointC,
                LocationAspect::new().existing_point_cy(),
            );
        }
        self
    }
    pub fn point_dx<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointD) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointDX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::PointD,
                LocationAspect::new().point_dx(d),
            );
        }
        self
    }
    pub fn existing_point_dx(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::PointD) {
            aspect.set(GridAspect::PointDX, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::PointD,
                LocationAspect::new().existing_point_dx(),
            );
        }
        self
    }
    pub fn point_dy<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::PointD) {
            // sanitize that other is compatible
            aspect.set(
                GridAspect::PointDY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        } else {
            self.configurations.insert(
                AspectConfiguration::PointD,
                LocationAspect::new().point_dy(d),
            );
        }
        self
    }
    pub fn existing_point_dy(mut self) -> Self {
        if let Some(aspect) = self.configurations.get_mut(&AspectConfiguration::PointD) {
            aspect.set(GridAspect::PointDY, LocationAspectDescriptorValue::Existing);
        } else {
            self.configurations.insert(
                AspectConfiguration::PointD,
                LocationAspect::new().existing_point_dy(),
            );
        }
        self
    }
}
#[derive(Clone, Copy, Component, Debug)]
pub struct Grid {
    columns: u32,
    rows: u32,
    gap: Coordinates,
}
impl Grid {
    pub fn new(columns: u32, rows: u32) -> Grid {
        Self {
            columns,
            rows,
            gap: Coordinates::new(8.0, 8.0),
        }
    }
    pub fn columns(&self) -> f32 {
        self.columns as f32
    }
    pub fn rows(&self) -> f32 {
        self.rows as f32
    }
    pub fn gap<C: Into<Coordinates>>(mut self, g: C) -> Self {
        self.gap = g.into();
        self
    }
}
impl Default for Grid {
    fn default() -> Self {
        Self::new(1, 1)
    }
}
#[derive(Clone, Default, Component, Debug)]
pub(crate) struct ReferentialDependencies {
    pub(crate) deps: HashSet<GridContext>,
}
impl ReferentialDependencies {
    fn new(deps: HashSet<GridContext>) -> ReferentialDependencies {
        Self { deps }
    }
}
pub(crate) struct ReferentialOrderDeterminant<'a> {
    stem: &'a Stem,
    deps: &'a ReferentialDependencies,
    lh: &'a LeafHandle,
    location: &'a GridLocation,
    grid: Grid,
}
pub(crate) fn distill_location_deps(
    mut query: Query<(Entity, &GridLocation, &mut ReferentialDependencies), Or<(Changed<GridLocation>, Changed<Stem>)>>,
    stems: Query<&Stem>,
) {
    for (entity, location, mut dep) in query.iter_mut() {
        *dep = location.deps(entity, &stems);
        println!("deps: {:?}", dep);
    }
}
pub(crate) fn resolve_grid_locations(
    mut check_read_and_update: ParamSet<(
        Query<Entity, Or<(Changed<GridLocation>, Changed<Grid>)>>,
        Query<(&Stem, &LeafHandle, &GridLocation, &ReferentialDependencies, &Grid)>,
        Query<(
            &mut Position<LogicalContext>,
            &mut Area<LogicalContext>,
            &mut Points<LogicalContext>,
            &mut GridLocation,
        )>,
    )>,
    id_table: Res<IdTable>,
    viewport_handle: ResMut<ViewportHandle>,
    layout_grid: Res<LayoutGrid>,
    layout: Res<Layout>,
) {
    if check_read_and_update.p0().is_empty() && !layout_grid.is_changed() {
        return;
    }
    let mut ref_context = ReferentialContext::new(viewport_handle.section(), layout_grid.grid);
    let read = check_read_and_update.p1();
    for (stem, handle, location, deps, grid) in read.iter() {
        ref_context.queue_leaf(stem, handle, location, deps, *grid);
    }
    ref_context.resolve(*layout);
    let updates = ref_context.updates();
    drop(ref_context);
    drop(read);
    for (handle, resolved) in updates {
        let e = id_table.lookup_leaf(handle).unwrap();
        *check_read_and_update.p2().get_mut(e).unwrap().0 = resolved.section.position;
        *check_read_and_update.p2().get_mut(e).unwrap().1 = resolved.section.area;
        if let Some(p) = resolved.points {
            *check_read_and_update.p2().get_mut(e).unwrap().2 = p;
        }
        if let Some(hook) = resolved.hook_update {
            match &mut check_read_and_update
                .p2()
                .get_mut(e)
                .unwrap()
                .3
                .animation_hook
            {
                GridLocationAnimationHook::SectionDriven(s) => {
                    s.diff = hook[0].unwrap();
                }
                GridLocationAnimationHook::PointDriven(p) => {
                    if let Some(h) = hook[0] {
                        p.point_a.diff = h;
                    }
                    if let Some(h) = hook[1] {
                        p.point_b.diff = h;
                    }
                    if let Some(h) = hook[2] {
                        p.point_c.diff = h;
                    }
                    if let Some(h) = hook[3] {
                        p.point_d.diff = h;
                    }
                }
            }
        }
    }
}
pub(crate) struct ReferentialContext<'a> {
    context: HashMap<GridContext, ReferentialData>,
    order_queue: Vec<ReferentialOrderDeterminant<'a>>,
}
impl<'a> ReferentialContext<'a> {
    pub(crate) fn new(screen_section: Section<LogicalContext>, layout_grid: Grid) -> Self {
        Self {
            context: {
                let mut context = HashMap::new();
                context.insert(
                    GridContext::Screen,
                    ReferentialData::new(
                        ResolvedLocation::new().section(screen_section),
                        layout_grid,
                    ),
                );
                context
            },
            order_queue: vec![],
        }
    }
    pub(crate) fn queue_leaf(
        &mut self,
        stem: &'a Stem,
        lh: &'a LeafHandle,
        location: &'a GridLocation,
        deps: &'a ReferentialDependencies,
        grid: Grid,
    ) {
        println!("queueing leaf: {:?}", lh);
        self.order_queue.push(ReferentialOrderDeterminant {
            stem,
            lh,
            location,
            deps,
            grid,
        });
    }
    pub(crate) fn resolve(&mut self, layout: Layout) {
        let mut needs_ordering = true;
        while needs_ordering {
            for
        }
        self.order_queue.sort_by(|a, b| {
            println!("comparing: {:?} - {:?}", a.lh, b.lh);
            let b_depends_a = b.deps.deps.contains(&GridContext::Named(a.lh.clone()));
            let a_depends_b = a.deps.deps.contains(&GridContext::Named(b.lh.clone()));
            println!("a-dep-b: {}, b-dep-a: {}", a_depends_b, b_depends_a);
            if a_depends_b && b_depends_a {
                panic!("circular grid reference")
            }
            if a_depends_b {
                println!("greater");
                Ordering::Greater
            } else if b_depends_a {
                println!("less");
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        let order = self
            .order_queue
            .drain(..)
            .collect::<Vec<ReferentialOrderDeterminant>>();
        for determinant in order.iter() {
            println!("determinant: {:?}, {:?}", determinant.lh, determinant.deps);
        }
        for determinant in order {
            let resolved = determinant.location.resolve(&determinant.stem, &self.context, layout);
            if let Some(resolved) = resolved {
                self.context.insert(
                    GridContext::Named(determinant.lh.clone()),
                    ReferentialData::new(resolved, determinant.grid),
                );
            } else {
                panic!("invalid grid-location")
            }
        }
    }
    pub(crate) fn updates(&mut self) -> Vec<(LeafHandle, ResolvedLocation)> {
        let mut updates = vec![];
        for (k, v) in self.context.drain() {
            match k {
                GridContext::Screen => {
                    continue;
                }
                GridContext::Stem => {
                    continue;
                }
                GridContext::Named(lh) => {
                    updates.push((lh, v.resolved));
                }
                GridContext::Absolute => {
                    continue;
                }
            }
        }
        updates
    }
}
#[derive(Debug)]
pub(crate) struct ResolvedLocation {
    pub(crate) section: Section<LogicalContext>,
    pub(crate) points: Option<Points<LogicalContext>>,
    pub(crate) hook_update: Option<[Option<Section<LogicalContext>>; 4]>,
}

impl ResolvedLocation {
    pub(crate) fn new() -> Self {
        Self {
            section: Section::default(),
            points: None,
            hook_update: None,
        }
    }
    pub(crate) fn section(mut self, section: Section<LogicalContext>) -> Self {
        self.section = section;
        self
    }
}
#[derive(Debug)]
pub(crate) struct ReferentialData {
    pub(crate) resolved: ResolvedLocation,
    pub(crate) grid: Grid,
}

impl ReferentialData {
    pub(crate) fn new(resolved: ResolvedLocation, grid: Grid) -> Self {
        Self { resolved, grid }
    }
}
