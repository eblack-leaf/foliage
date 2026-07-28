use crate::Coordinates;

/// The two interior control points of a cubic bezier easing curve, each clamped to the
/// unit square. The endpoints are always `(0,0)` and `(1,1)`.
#[derive(Copy, Clone)]
pub struct ControlPoints {
    a: Coordinates,
    b: Coordinates,
}

impl ControlPoints {
    /// Two control points, each `(x, y)` in `0..1`. Components outside that range are
    /// clamped, so an overshoot curve is not expressible here -- use one of [`Ease`]'s own
    /// constants for the shaped presets.
    pub fn new<A: Into<Coordinates>, B: Into<Coordinates>>(a: A, b: B) -> Self {
        Self {
            a: a.into().clamped(0.0, 1.0),
            b: b.into().clamped(0.0, 1.0),
        }
    }
}

/// An [`Ease`] in use by a running animation -- the curve plus the state of evaluating it.
pub struct Easement {
    behavior: Ease,
}

impl From<Ease> for Easement {
    fn from(value: Ease) -> Self {
        Easement::new(value)
    }
}

/// How an animation's progress maps onto its value: the shape of the motion, independent
/// of how long it lasts.
///
/// [`Linear`](Ease::Linear) is a constant rate. [`Bezier`](Ease::Bezier) shapes it with a
/// cubic curve -- the named constants below cover the usual cases, and are what most
/// callers should reach for.
#[derive(Clone)]
pub enum Ease {
    /// Constant rate, no shaping.
    Linear,
    /// Cubic bezier through the given interior control points.
    Bezier(ControlPoints),
}

impl Ease {
    /// Fast start, gentle settle. The default choice for something arriving on screen.
    pub const DECELERATE: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.05, 0.7),
        b: Coordinates::new(0.1, 1.0),
    });
    /// Slow start building to speed -- for something leaving.
    pub const ACCELERATE: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.3, 0.0),
        b: Coordinates::new(0.8, 0.15),
    });
    /// Pronounced ease at both ends, with a quick middle -- draws the eye to a change.
    pub const EMPHASIS: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.68, 0.0),
        b: Coordinates::new(0.0, 1.0),
    });
    /// Gentle at both ends, close to symmetric -- an unobtrusive move in place.
    pub const INWARD: Self = Self::Bezier(ControlPoints {
        a: Coordinates::new(0.29, 0.1),
        b: Coordinates::new(0.36, 0.92),
    });
}

impl Easement {
    /// Maps linear progress `d` (0..1) onto eased progress via the curve's y for that x.
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
