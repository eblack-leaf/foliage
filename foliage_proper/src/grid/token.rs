use crate::coordinate::CoordinateUnit;
use crate::grid::aspect::GridAspect;
use crate::grid::aspect::GridContext;
use crate::grid::resolve::ReferentialData;
use crate::grid::unit::RelativeUnit;
use smallvec::SmallVec;
use std::ops::{Add, Sub};

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

#[derive(Clone, Copy)]
pub(crate) enum LocationAspectTokenOp {
    Add,
    Minus,
    // ...
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
    tokens: SmallVec<[LocationAspectToken; 3]>,
}

impl SpecifiedDescriptorValue {
    pub(crate) fn resolve(&self, stem: ReferentialData, screen: ReferentialData) -> CoordinateUnit {
        let mut accumulator = 0.0;
        for t in self.tokens.iter() {
            let value = match t.value {
                LocationAspectTokenValue::ContextAspect(ca) => {
                    let data = match &t.context {
                        GridContext::Stem => stem,
                        _ => screen,
                    };
                    match ca {
                        GridAspect::Top => data.section.y(),
                        GridAspect::Height => data.section.height(),
                        GridAspect::CenterY => data.section.center().y(),
                        GridAspect::Bottom => data.section.bottom(),
                        GridAspect::Left => data.section.x(),
                        GridAspect::Width => data.section.width(),
                        GridAspect::CenterX => data.section.center().x(),
                        GridAspect::Right => data.section.right(),
                        GridAspect::PointAX => data
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[0].x()))
                            .unwrap_or_default(),
                        GridAspect::PointAY => data
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[0].y()))
                            .unwrap_or_default(),
                        GridAspect::PointBX => data
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[1].x()))
                            .unwrap_or_default(),
                        GridAspect::PointBY => data
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[1].y()))
                            .unwrap_or_default(),
                        GridAspect::PointCX => data
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[2].x()))
                            .unwrap_or_default(),
                        GridAspect::PointCY => data
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[2].y()))
                            .unwrap_or_default(),
                        GridAspect::PointDX => data
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[3].x()))
                            .unwrap_or_default(),
                        GridAspect::PointDY => data
                            .points
                            .as_ref()
                            .and_then(|p| Some(p.data[3].y()))
                            .unwrap_or_default(),
                    }
                }
                LocationAspectTokenValue::Relative(rel) => {
                    let data = match &t.context {
                        GridContext::Stem => stem,
                        _ => screen,
                    };
                    match rel {
                        RelativeUnit::Column(c, use_end) => {
                            data.section.x()
                                + (c as f32 - 1.0 * f32::from(!use_end))
                                    * (data.section.width() / data.grid.columns as f32)
                                + c as f32 * data.grid.gap.horizontal()
                        }
                        RelativeUnit::Row(r, use_end) => {
                            data.section.y()
                                + (r as f32 - 1.0 * f32::from(!use_end))
                                    * (data.section.height() / data.grid.rows as f32)
                                + r as f32 * data.grid.gap.vertical()
                        }
                        RelativeUnit::Percent(p, use_width, include_start) => {
                            data.section.x() * f32::from(include_start && use_width)
                                + data.section.width() * p * f32::from(use_width)
                                + data.section.y() * f32::from(include_start && !use_width)
                                + data.section.height() * p * f32::from(!use_width)
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
        let mut vec = SmallVec::new();
        vec.push(first);
        Self { tokens: vec }
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
    pub(crate) aspect: GridAspect,
    pub(crate) value: LocationAspectDescriptorValue,
}

impl LocationAspectDescriptor {
    pub(crate) fn new(aspect: GridAspect, value: LocationAspectDescriptorValue) -> Self {
        Self { aspect, value }
    }
}
