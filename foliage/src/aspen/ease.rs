//! How long a motion takes, and what shape it moves in.
//!
//! The two are independent. A duration says when the motion is over; an ease says where it is on
//! the way, and says nothing about how long that takes. Keeping them apart is what lets one
//! app-wide shape be stated beside any duration.

use core::time::Duration;

use crate::aspen::Sequence;

/// The shape of a motion: how its elapsed fraction maps onto its progress.
///
/// Every shape here is a cubic bezier through `(0, 0)` and `(1, 1)`, so all of them start where the
/// motion started and end exactly on its target. The named ones are the three a surface needs:
/// something arriving, something leaving, and something moving in place.
///
/// [`Linear`](Ease::Linear) is unshaped, and is what a [`Timing`] takes when nothing is said. It is
/// the honest default rather than the flattering one -- a shape an app did not ask for is a shape
/// it cannot see in the code.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum Ease {
    /// Constant rate.
    #[default]
    Linear,
    /// Quick to start, gentle to settle. For something arriving: the eye follows it in and is let
    /// down at the end.
    Decelerate,
    /// Gentle to start, quick to leave. For something going away, which needs no help being
    /// followed because it is already known.
    Accelerate,
    /// Eased at both ends with a quick middle. For a change in place that should be noticed
    /// without being waited on.
    Emphasis,
    /// A cubic bezier through two interior control points, each stated as a fraction: `x` along the
    /// motion's duration, `y` along its progress.
    ///
    /// The endpoints are fixed at `(0, 0)` and `(1, 1)` and are not stated. `x` is clamped to
    /// `0.0..=1.0`, because a control point outside the duration describes no ordering of the
    /// motion; `y` is not, so a curve may overshoot and come back.
    Curve {
        /// The first control point's position along the duration.
        x1: f32,
        /// The first control point's position along the progress.
        y1: f32,
        /// The second control point's position along the duration.
        x2: f32,
        /// The second control point's position along the progress.
        y2: f32,
    },
}

impl Ease {
    /// The two interior control points, as `(x1, y1, x2, y2)`.
    fn control(self) -> (f32, f32, f32, f32) {
        match self {
            // The cubic that is its own straight line: evenly spaced control points.
            Ease::Linear => (1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0),
            Ease::Decelerate => (0.0, 0.0, 0.2, 1.0),
            Ease::Accelerate => (0.4, 0.0, 1.0, 1.0),
            Ease::Emphasis => (0.4, 0.0, 0.2, 1.0),
            Ease::Curve { x1, y1, x2, y2 } => (x1.clamp(0.0, 1.0), y1, x2.clamp(0.0, 1.0), y2),
        }
    }

    /// The progress this shape is at, `fraction` of the way through the motion's duration.
    ///
    /// Exact at both ends: `0.0` and `1.0` map to themselves whatever the shape, so a motion always
    /// leaves where it was and lands on its target rather than near it.
    pub(crate) fn at(self, fraction: f32) -> f32 {
        let fraction = fraction.clamp(0.0, 1.0);
        if fraction == 0.0 || fraction == 1.0 || self == Ease::Linear {
            return fraction;
        }
        let (x1, y1, x2, y2) = self.control();
        cubic(y1, y2, parameter(x1, x2, fraction))
    }
}

/// A cubic bezier through `(0, 0)` and `(1, 1)` with the two interior points given, in its
/// polynomial form: `((a * u + b) * u + c) * u`.
fn coefficients(first: f32, second: f32) -> (f32, f32, f32) {
    let c = 3.0 * first;
    let b = 3.0 * (second - first) - c;
    (1.0 - c - b, b, c)
}

fn cubic(first: f32, second: f32, u: f32) -> f32 {
    let (a, b, c) = coefficients(first, second);
    ((a * u + b) * u + c) * u
}

fn slope(first: f32, second: f32, u: f32) -> f32 {
    let (a, b, c) = coefficients(first, second);
    (3.0 * a * u + 2.0 * b) * u + c
}

/// The curve parameter whose `x` is `fraction`.
///
/// A bezier is parametric, so the curve is not a function of the fraction directly -- the parameter
/// has to be recovered from it before the progress can be read off. Newton converges in a handful
/// of steps for every shape that is a usable ease, and bisection finishes the ones where the slope
/// is too flat for it. Both are bounded, so this cannot fail to return and cannot be slow.
fn parameter(x1: f32, x2: f32, fraction: f32) -> f32 {
    let mut u = fraction;
    for _ in 0..8 {
        let error = cubic(x1, x2, u) - fraction;
        if error.abs() < 1e-6 {
            return u;
        }
        let slope = slope(x1, x2, u);
        if slope.abs() < 1e-6 {
            break;
        }
        u -= error / slope;
    }
    let (mut low, mut high) = (0.0, 1.0);
    u = fraction;
    for _ in 0..32 {
        let at = cubic(x1, x2, u);
        if (at - fraction).abs() < 1e-6 {
            break;
        }
        if at < fraction {
            low = u;
        } else {
            high = u;
        }
        u = (low + high) / 2.0;
    }
    u
}

/// How long a motion takes, when it starts, what shape it moves in, and what it arrives as part of.
///
/// ```no_run
/// # use foliage::{Ease, Timing};
/// Timing::ms(240).ease(Ease::Decelerate).after(60);
/// ```
///
/// Milliseconds throughout, because that is the scale every motion on a surface is stated at, and
/// one unit at the callsite is what keeps two motions comparable by reading them.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Timing {
    duration: Duration,
    delay: Duration,
    ease: Ease,
    within: Option<Sequence>,
}

impl Timing {
    /// A motion lasting `millis`, beginning at once, unshaped, arriving on its own.
    ///
    /// A duration of zero is a motion that is over on the frame it starts, which is what makes a
    /// [`timer`](crate::Grow::timer) of zero the next frame's report rather than a special case.
    pub fn ms(millis: u64) -> Self {
        Self {
            duration: Duration::from_millis(millis),
            delay: Duration::ZERO,
            ease: Ease::Linear,
            within: None,
        }
    }

    /// Counts this into a [`Sequence`], so its end is reported as part of the group's rather than
    /// only as its own.
    ///
    /// Here rather than as an argument to [`animate`](crate::Grow::animate) because a sequence is
    /// about *when a group is over*, which is the one question this type exists to answer. What is
    /// moving stays out of it: a motion, a channel and a timer join a sequence in the same words,
    /// and none of them has to be near any of the others to do it.
    pub fn within(mut self, sequence: Sequence) -> Self {
        self.within = Some(sequence);
        self
    }

    /// The sequence this was counted into, if any.
    pub(crate) fn sequence(&self) -> Option<Sequence> {
        self.within
    }

    /// The shape it moves in.
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }

    /// How long to wait before it starts, in milliseconds.
    ///
    /// The motion holds where it was for that long rather than being queued: it is running from the
    /// frame it was asked for, and a write that cancels it cancels it whether or not it has begun
    /// to move.
    pub fn after(mut self, millis: u64) -> Self {
        self.delay = Duration::from_millis(millis);
        self
    }

    /// The progress a motion `elapsed` into this timing has made.
    pub(crate) fn at(&self, elapsed: Duration) -> f32 {
        let running = elapsed.saturating_sub(self.delay);
        // Nothing to be a fraction of. A zero-duration motion is at its end from its first instant.
        if self.duration.is_zero() {
            return 1.0;
        }
        let fraction = running.as_nanos() as f64 / self.duration.as_nanos() as f64;
        self.ease.at(fraction.clamp(0.0, 1.0) as f32)
    }

    /// Whether a motion `elapsed` into this timing has reached its end.
    pub(crate) fn done(&self, elapsed: Duration) -> bool {
        elapsed >= self.delay + self.duration
    }
}
