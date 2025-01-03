use crate::Coordinates;

#[derive(Copy, Clone)]
pub struct ControlPoints {
    a: Coordinates,
    b: Coordinates,
}

impl ControlPoints {
    pub fn new<A: Into<Coordinates>, B: Into<Coordinates>>(a: A, b: B) -> Self {
        Self {
            a: a.into().clamped(0.0, 1.0),
            b: b.into().clamped(0.0, 1.0),
        }
    }
}

pub struct Easement {
    behavior: Ease,
}

impl From<Ease> for Easement {
    fn from(value: Ease) -> Self {
        Easement::new(value)
    }
}

#[derive(Clone)]
pub enum Ease {
    Linear,
    Bezier(ControlPoints),
}

impl Ease {
    pub const DECELERATE: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.05, 0.7),
        b: Coordinates::new(0.1, 1.0),
    });
    pub const ACCELERATE: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.3, 0.0),
        b: Coordinates::new(0.8, 0.15),
    });
    pub const EMPHASIS: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.68, 0.0),
        b: Coordinates::new(0.0, 1.0),
    });
    pub const INWARD: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.29, 0.1),
        b: Coordinates::new(0.36, 0.92),
    });
}

impl Easement {
    pub fn percent_changed(&mut self, d: f32) -> f32 {
        match self.behavior {
            Ease::Linear => d,
            Ease::Bezier(points) => {
                let base = Coordinates::from((0, 0));
                let end = Coordinates::from((1, 1));
                (1f32 - d).powi(3) * base.b()
                    + 3f32 * (1f32 - d).powi(2) * d * points.a.b()
                    + 3f32 * (1f32 - d) * d.powi(2) * points.b.b()
                    + d.powi(3) * end.b()
            }
        }
    }
    pub(crate) fn new(behavior: Ease) -> Self {
        Self { behavior }
    }
}
