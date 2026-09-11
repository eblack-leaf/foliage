//! What things are coloured from.

use foliage::Color;

/// A colour as three channels in `0.0..=1.0`.
pub type Rgb = (f32, f32, f32);

/// A run of colours, read at any point between the first and the last.
#[derive(Clone, Debug)]
pub struct Ramp {
    stops: Vec<Rgb>,
}

impl Ramp {
    /// Two or more stops, evenly spaced.
    pub fn new(stops: &[Rgb]) -> Self {
        assert!(stops.len() >= 2, "a ramp is two or more stops");
        Self {
            stops: stops.to_vec(),
        }
    }

    /// The colour `at` along the ramp, in `0.0..=1.0`.
    pub fn at(&self, at: f32) -> Rgb {
        let at = at.clamp(0.0, 1.0);
        let steps = (self.stops.len() - 1) as f32;
        let step = (at * steps).floor().min(steps - 1.0);
        let into = at * steps - step;
        let (from, to) = (self.stops[step as usize], self.stops[step as usize + 1]);
        (
            from.0 + (to.0 - from.0) * into,
            from.1 + (to.1 - from.1) * into,
            from.2 + (to.2 - from.2) * into,
        )
    }
}

/// The same colour nearer white, or nearer black where `by` is negative.
pub fn shifted((red, green, blue): Rgb, by: f32) -> Rgb {
    (
        (red + by).clamp(0.0, 1.0),
        (green + by).clamp(0.0, 1.0),
        (blue + by).clamp(0.0, 1.0),
    )
}

/// A colour off a ramp, as what an element is filled with. Stated outright rather than as a palette
/// role: a ramp of this many steps is not a scheme.
pub fn fill((red, green, blue): Rgb) -> Color {
    Color::rgb(red, green, blue)
}
