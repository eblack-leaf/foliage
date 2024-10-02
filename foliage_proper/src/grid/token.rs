use crate::coordinate::CoordinateUnit;
use crate::grid::aspect::GridAspect;
use crate::grid::aspect::GridContext;
use crate::grid::resolve::ReferentialData;
use crate::grid::unit::RelativeUnit;
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
    tokens: Vec<LocationAspectToken>,
}

impl SpecifiedDescriptorValue {
    pub(crate) fn resolve(
        &self,
        stem: Option<ReferentialData>,
        screen: ReferentialData,
    ) -> CoordinateUnit {
        let mut accumulator = 0.0;
        for t in self.tokens.iter() {
            let value = match t.value {
                LocationAspectTokenValue::ContextAspect(ca) => {
                    let data = match &t.context {
                        GridContext::Stem => stem.unwrap_or(screen),
                        _ => screen,
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
                        GridContext::Stem => stem.unwrap_or(screen),
                        _ => screen,
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
