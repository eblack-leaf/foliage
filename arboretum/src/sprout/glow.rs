//! A soft light that follows the hand.

use foliage::{
    Area, Boxed, Color, Ease, Elevation, Fit, Grove, Grow, Image, Leaf, Location, Motion, Place,
    Plate, Position, Sap, Source, Timing, Vein, left, top,
};

use crate::genart::sprite;

/// How many texels across the light's sprite is. Small: it is stretched, and the filtering is what
/// makes it soft.
const SPRITE: u32 = 96;

/// A soft light: one round falloff, stretched over a box `2 × radius` across, drawn wherever it
/// was last told to go and easing there.
///
/// foliage draws no blur and no glow, so the softness is a picture -- a small one, stretched --
/// and the light is moved by drawing it aside ([`Motion::Shift`]), which lays nothing out again.
/// A move re-aims the motion already running, from wherever the light has got to, so a light
/// asked to follow a cursor follows it smoothly however often it is asked.
pub struct Glow {
    under: Leaf,
    light: Leaf,
    plate: Plate,
    radius: f32,
    aimed: Option<Position>,
    shown: bool,
}

impl Glow {
    /// A light of `color` (its alpha is the light's strength) `radius` pixels round, grown under
    /// `under` and lifted by `lift`. Unseen until [`show`](Glow::show)n.
    pub fn grow(grove: &mut Grove, under: Leaf, radius: f32, color: Color, lift: Elevation) -> Self {
        let plate = grove.pixels(sprite::falloff(SPRITE, color), Area::new(SPRITE as f32, SPRITE as f32));
        let light = grove.branch(
            under,
            Image::new(plate)
                .fit(Fit::Stretch)
                .intangible()
                .elevate(lift)
                .opacity(0.0)
                .at(Location::new().xs(
                    left(0.px()).width((radius * 2.0).px()),
                    top(0.px()).height((radius * 2.0).px()),
                )),
        );
        Self {
            under,
            light,
            plate,
            radius,
            aimed: None,
            shown: false,
        }
    }

    /// What is drawn.
    pub fn leaf(&self) -> Leaf {
        self.light
    }

    /// Shows the light, or puts it out, over `ms`.
    pub fn show(&mut self, grove: &mut Grove, shown: bool, ms: u64) {
        if self.shown == shown {
            return;
        }
        self.shown = shown;
        let to = if shown { 1.0 } else { 0.0 };
        grove.animate(self.light, Motion::Opacity(to), Timing::ms(ms).ease(Ease::Decelerate));
    }

    /// Changes what colour the light is, and how strong.
    pub fn tint(&mut self, grove: &mut Grove, color: Color) {
        grove.load(
            self.plate,
            sprite::falloff(SPRITE, color),
            Area::new(SPRITE as f32, SPRITE as f32),
        );
    }

    /// Sends the light's middle toward `at`, a point on the surface, easing over `ms`.
    pub fn follow(&mut self, grove: &mut Grove, at: Position, ms: u64) {
        let Some(Sap::Section(origin)) = grove.tap(self.under, Vein::Drawn) else {
            return;
        };
        let to = Position::new(
            at.x - origin.left() - self.radius,
            at.y - origin.top() - self.radius,
        );
        if let Some(aimed) = self.aimed
            && (aimed.x - to.x).abs() + (aimed.y - to.y).abs() < 1.0
        {
            return;
        }
        match self.aimed {
            // The first aim is a placement, not a journey from the corner.
            None => grove.shift(self.light, to),
            Some(_) => {
                grove.animate(self.light, Motion::Shift(to), Timing::ms(ms).ease(Ease::Decelerate));
            }
        }
        self.aimed = Some(to);
    }
}
