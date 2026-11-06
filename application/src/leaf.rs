//! The leaf the site was sketched as: its outline, how it is tiled, and the light through it.

use foliage::Ease;
use lichen::{Mosaic, Ramp, Silhouette};

use crate::theme;

/// The outline, clockwise from the tip, in the leaf's own unit space, standing: its tip at the
/// top and its stem at the bottom. It was drawn lying on its side, stem out to the right, and
/// stood up a quarter turn.
///
/// Closed implicitly: the last vertex joins the first. The stem is part of it rather than a second
/// shape, so it is dashed, filled and measured with everything else.
#[rustfmt::skip]
const OUTLINE: [(f32, f32); 75] = [
    (1.000, 0.708), (0.948, 0.773), (0.875, 0.839), (0.789, 0.904),
    (0.733, 0.953), (0.709, 0.978), (0.666, 0.989), (0.703, 1.007),
    (0.706, 1.042), (0.647, 1.068), (0.592, 1.088), (0.531, 1.104),
    (0.475, 1.117), (0.432, 1.125), (0.412, 1.232), (0.380, 1.346),
    (0.334, 1.437), (0.279, 1.491), (0.242, 1.504), (0.230, 1.477),
    (0.273, 1.445), (0.322, 1.379), (0.359, 1.265), (0.383, 1.150),
    (0.387, 1.129), (0.328, 1.114), (0.267, 1.094), (0.208, 1.068),
    (0.166, 1.045), (0.154, 1.002), (0.174, 0.970), (0.129, 0.934),
    (0.107, 0.888), (0.066, 0.826), (0.031, 0.760), (0.006, 0.708),
    (0.000, 0.655), (0.011, 0.593), (0.018, 0.531), (0.031, 0.498),
    (0.088, 0.508), (0.141, 0.514), (0.187, 0.505), (0.230, 0.521),
    (0.259, 0.478), (0.273, 0.413), (0.291, 0.351), (0.313, 0.285),
    (0.338, 0.216), (0.362, 0.151), (0.389, 0.085), (0.410, 0.039),
    (0.436, 0.007), (0.485, 0.000), (0.531, 0.013), (0.559, 0.039),
    (0.584, 0.072), (0.617, 0.085), (0.657, 0.115), (0.684, 0.154),
    (0.709, 0.213), (0.742, 0.274), (0.775, 0.324), (0.797, 0.351),
    (0.785, 0.413), (0.768, 0.486), (0.742, 0.557), (0.807, 0.544),
    (0.862, 0.549), (0.918, 0.570), (0.903, 0.596), (0.942, 0.616),
    (0.924, 0.642), (0.961, 0.662), (0.979, 0.685),
];

/// How far apart tile centres sit, as a fraction of the leaf's width. Fine enough that the ramp
/// reads as a gradient down it and a tile coming loose is a fleck rather than a hole, coarse
/// enough that the whole leaf is a few hundred polygons -- which a browser draws as readily as a
/// desktop does.
const PITCH: f32 = 0.040;

/// Which way the ramp runs: mostly down the leaf, a little across it, so it is gold at the tip and
/// deepens toward the stem, and the across term keeps the bands from reading as stripes.
const ALONG: (f32, f32) = (0.25, 0.75);

/// The leaf, ungrown, its tiles coloured from `ramp`.
pub(crate) fn mosaic(ramp: Ramp) -> Mosaic {
    Mosaic::new(Silhouette::new(&OUTLINE), ramp)
        .along(ALONG)
        .edge(theme::EDGE)
        .pitch(PITCH)
}

/// How far the light reaches across the leaf, as a fraction of its width.
pub(crate) const REACH: f32 = 0.42;

/// How long the light takes to come up once the leaf has formed, to glide to where the leaf was
/// pressed, and to wander off again from there -- the last a time constant, so it never quite has
/// and never stops -- in seconds.
const RISE: f32 = 3.0;
const GLIDE: f32 = 1.8;
const RETURN: f32 = 14.0;

/// The light through the leaf: wandering slowly over the blade, as sun through a canopy moves,
/// drawn to wherever the leaf is pressed, and from there wandering on.
pub(crate) struct Sun {
    /// How far a press has drawn the light off its wander.
    drawn: (f32, f32),
    /// The draw under way.
    glide: Option<Glide>,
    /// When it rose, and when it was last moved, in seconds.
    rose: Option<f32>,
    last: f32,
}

/// A press drawing the light to it: how far off its wander the light was drawn from, how far it
/// is being drawn to, and when, in seconds.
#[derive(Copy, Clone)]
struct Glide {
    from: (f32, f32),
    to: (f32, f32),
    began: f32,
}

impl Sun {
    pub(crate) fn new() -> Self {
        Self {
            drawn: (0.0, 0.0),
            glide: None,
            rose: None,
            last: 0.0,
        }
    }

    /// Where the light stands `now` -- in seconds, from when the leaf was planted -- in the leaf's
    /// unit space, and how strong it is; drawn toward `pressed` where the leaf was just pressed
    /// there. Asked once a frame from when the leaf has formed, which is when it rises.
    pub(crate) fn frame(&mut self, now: f32, pressed: Option<(f32, f32)>) -> ((f32, f32), f32) {
        let rose = *self.rose.get_or_insert(now);
        let since = (now - self.last).max(0.0);
        self.last = now;
        let wandering = wander(now);
        if let Some(to) = pressed {
            let off = (to.0 - wandering.0, to.1 - wandering.1);
            self.glide = Some(Glide {
                from: self.drawn,
                to: off,
                began: now,
            });
        }
        match self.glide {
            Some(Glide { from, to, began }) => {
                let along = ((now - began) / GLIDE).min(1.0);
                let eased = Ease::Emphasis.at(along);
                self.drawn = (
                    from.0 + (to.0 - from.0) * eased,
                    from.1 + (to.1 - from.1) * eased,
                );
                if along >= 1.0 {
                    self.glide = None;
                }
            }
            None => {
                let kept = (-since / RETURN).exp();
                self.drawn = (self.drawn.0 * kept, self.drawn.1 * kept);
            }
        }
        let at = (wandering.0 + self.drawn.0, wandering.1 + self.drawn.1);
        let strength = Ease::Decelerate.at(((now - rose) / RISE).min(1.0));
        (at, strength)
    }
}

/// Where the light wanders to at `t` seconds: round the blade on two slow swings each way, at
/// rates that never line up, so it does not retrace itself -- and never so far out that its
/// middle leaves the leaf.
fn wander(t: f32) -> (f32, f32) {
    (
        0.52 + 0.16 * (0.13 * t + 0.4).sin() + 0.05 * (0.31 * t + 2.1).sin(),
        0.66 + 0.20 * (0.09 * t + 1.3).sin() + 0.05 * (0.23 * t + 0.2).sin(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether `(x, y)` is inside the outline: an odd number of its edges cross a ray from it.
    fn inside((x, y): (f32, f32)) -> bool {
        let mut inside = false;
        for (n, &(x0, y0)) in OUTLINE.iter().enumerate() {
            let (x1, y1) = OUTLINE[(n + 1) % OUTLINE.len()];
            if (y0 > y) != (y1 > y) && x < x0 + (y - y0) / (y1 - y0) * (x1 - x0) {
                inside = !inside;
            }
        }
        inside
    }

    /// However long the page is left open, the light's middle stays on the leaf.
    #[test]
    fn the_light_stays_on_the_leaf() {
        for step in 0..8000 {
            let at = wander(step as f32 * 0.25);
            assert!(inside(at), "{at:?} at {}s", step as f32 * 0.25);
        }
    }

    /// Pressed, the light goes to where the leaf was pressed, and then wanders on from there.
    #[test]
    fn the_light_goes_where_it_is_pressed() {
        let mut sun = Sun::new();
        sun.frame(10.0, None);
        sun.frame(10.0, Some((0.3, 0.9)));
        let (at, _) = sun.frame(10.0 + GLIDE, None);
        let wandered = wander(10.0 + GLIDE);
        let expected = (0.3 + wandered.0 - wander(10.0).0, 0.9 + wandered.1 - wander(10.0).1);
        assert!((at.0 - expected.0).abs() < 1e-4 && (at.1 - expected.1).abs() < 1e-4, "{at:?}");
        let (later, strength) = sun.frame(10.0 + GLIDE + 60.0, None);
        let back = wander(10.0 + GLIDE + 60.0);
        assert!((later.0 - back.0).abs() < 0.05 && (later.1 - back.1).abs() < 0.05, "{later:?}");
        assert_eq!(strength, 1.0);
    }

    /// Standing: half again as tall as it is wide.
    #[test]
    fn stands() {
        let aspect = Silhouette::new(&OUTLINE).aspect();
        assert!((1.45..1.55).contains(&aspect), "{aspect}");
    }
}
