use crate::coordinate::CoordinateUnit;
use crate::grid::aspect::GridAspect;
use crate::grid::aspect::GridContext;
use crate::grid::responsive::evaluate::ReferentialData;
use crate::grid::unit::RelativeUnit;
use smallvec::SmallVec;
use std::ops::{Add, Sub};

impl From<AspectToken> for AspectValue {
    fn from(value: AspectToken) -> Self {
        AspectValue::new(value)
    }
}

impl Add for AspectToken {
    type Output = AspectValue;

    fn add(self, rhs: Self) -> Self::Output {
        AspectValue::new(self).plus(rhs)
    }
}

impl Add<AspectToken> for AspectValue {
    type Output = AspectValue;

    fn add(self, rhs: AspectToken) -> Self::Output {
        self.plus(rhs)
    }
}

impl Sub for AspectToken {
    type Output = AspectValue;

    fn sub(self, rhs: Self) -> Self::Output {
        AspectValue::new(self).minus(rhs)
    }
}

impl Sub<AspectToken> for AspectValue {
    type Output = AspectValue;

    fn sub(self, rhs: AspectToken) -> Self::Output {
        self.minus(rhs)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TokenOp {
    Add,
    Minus,
    // ...
}

#[derive(Clone, Copy)]
pub enum AspectTokenUnit {
    ContextAspect(GridAspect),
    Relative(RelativeUnit),
    Absolute(CoordinateUnit),
}

#[derive(Clone)]
pub struct AspectToken {
    op: TokenOp,
    context: GridContext,
    value: AspectTokenUnit,
}

impl AspectToken {
    pub(crate) fn new(op: TokenOp, context: GridContext, value: AspectTokenUnit) -> Self {
        Self { op, context, value }
    }
}

#[derive(Clone)]
pub struct AspectValue {
    tokens: SmallVec<[AspectToken; 2]>,
}

impl AspectValue {
    pub(crate) fn resolve(&self, stem: ReferentialData, screen: ReferentialData) -> CoordinateUnit {
        let mut accumulator = 0.0;
        for t in self.tokens.iter() {
            let value = match t.value {
                AspectTokenUnit::ContextAspect(ca) => {
                    let data = match &t.context {
                        GridContext::Stem => stem,
                        _ => screen,
                    };
                    match ca {
                        GridAspect::Top => data.section.top(),
                        GridAspect::Height => data.section.height(),
                        GridAspect::CenterY => data.section.center().y(),
                        GridAspect::Bottom => data.section.bottom(),
                        GridAspect::Left => data.section.left(),
                        GridAspect::Width => data.section.width(),
                        GridAspect::CenterX => data.section.center().x(),
                        GridAspect::Right => data.section.right(),
                        GridAspect::PointAX => data.points.data[0].x(),
                        GridAspect::PointAY => data.points.data[0].y(),
                        GridAspect::PointBX => data.points.data[1].x(),
                        GridAspect::PointBY => data.points.data[1].y(),
                        GridAspect::PointCX => data.points.data[2].x(),
                        GridAspect::PointCY => data.points.data[2].y(),
                        GridAspect::PointDX => data.points.data[3].x(),
                        GridAspect::PointDY => data.points.data[3].y(),
                    }
                }
                AspectTokenUnit::Relative(rel) => {
                    let data = match &t.context {
                        GridContext::Stem => stem,
                        _ => screen,
                    };
                    match rel {
                        RelativeUnit::Column(c, use_end) => {
                            data.section.left()
                                + (c as f32 - 1.0 * f32::from(!use_end))
                                    * (data.section.width() / data.grid.columns as f32)
                                + c as f32 * data.grid.gap.horizontal()
                        }
                        RelativeUnit::Row(r, use_end) => {
                            data.section.top()
                                + (r as f32 - 1.0 * f32::from(!use_end))
                                    * (data.section.height() / data.grid.rows as f32)
                                + r as f32 * data.grid.gap.vertical()
                        }
                        RelativeUnit::Percent(p, use_width, include_start) => {
                            data.section.left() * f32::from(include_start && use_width)
                                + data.section.width() * p * f32::from(use_width)
                                + data.section.top() * f32::from(include_start && !use_width)
                                + data.section.height() * p * f32::from(!use_width)
                        }
                    }
                }
                AspectTokenUnit::Absolute(a) => a,
            };
            match t.op {
                TokenOp::Add => {
                    accumulator += value;
                }
                TokenOp::Minus => {
                    accumulator -= value;
                }
            }
        }
        accumulator
    }
    pub(crate) fn minus(mut self, mut other: AspectToken) -> Self {
        other.op = TokenOp::Minus;
        self.tokens.push(other);
        self
    }
    pub(crate) fn plus(mut self, other: AspectToken) -> Self {
        self.tokens.push(other);
        self
    }
    pub(crate) fn new(first: AspectToken) -> AspectValue {
        let mut vec = SmallVec::new();
        vec.push(first);
        Self { tokens: vec }
    }
}

#[derive(Default, Clone)]
pub(crate) enum AspectValueWrapper {
    #[default]
    Existing,
    Specified(AspectValue),
    Auto,
}
impl AspectValueWrapper {
    pub(crate) fn resolve(
        &self,
        stem: ReferentialData,
        screen: ReferentialData,
    ) -> (CoordinateUnit, bool) {
        match self {
            AspectValueWrapper::Existing => (0.0, false),
            AspectValueWrapper::Specified(spec) => (spec.resolve(stem, screen), false),
            AspectValueWrapper::Auto => (0.0, true),
        }
    }
}

#[derive(Default, Clone)]
pub(crate) struct AspectDescriptor {
    pub(crate) aspect: GridAspect,
    pub(crate) value: AspectValueWrapper,
}

impl AspectDescriptor {
    pub(crate) fn new(aspect: GridAspect, value: AspectValueWrapper) -> Self {
        Self { aspect, value }
    }
}
