//! A soft shadow under a card.

use foliage::{
    Area, Boxed, Color, Ease, Elevation, Fit, Grove, Grow, Image, Leaf, Location, Motion, Place,
    Plate, Source, Timing, left, top,
};

use crate::genart::sprite;

/// The shadow sprite's size. Stretched, so small.
const SPRITE: (u32, u32) = (64, 64);

/// How long a lift takes, either way.
const LIFT_MS: u64 = 280;

/// A soft drop shadow, grown under a card and behind it, that spreads when the card lifts.
///
/// One blurred-rectangle picture, shared by every shadow on a page ([`Shade::plate`]) and
/// stretched under each card; the card's own box is what it is placed against.
pub struct Shade {
    leaf: Leaf,
    spread: f32,
    lifted: bool,
}

impl Shade {
    /// The picture every shadow is drawn from, in `color` (its alpha is how dark the shadow is at
    /// its densest). Make it once per page and hand it to every [`grow`](Shade::grow).
    pub fn plate(grove: &mut Grove, color: Color) -> Plate {
        grove.pixels(
            sprite::shadow(SPRITE.0, SPRITE.1, 0.35, color),
            Area::new(SPRITE.0 as f32, SPRITE.1 as f32),
        )
    }

    /// A shadow under `card`, `spread` pixels beyond its edges and a little below, drawn behind it.
    pub fn grow(grove: &mut Grove, card: Leaf, plate: Plate, spread: f32) -> Self {
        let leaf = grove.branch(
            card,
            Image::new(plate)
                .fit(Fit::Stretch)
                .intangible()
                .elevate(Elevation::down(1))
                .opacity(0.55)
                .at(rest(spread)),
        );
        Self {
            leaf,
            spread,
            lifted: false,
        }
    }

    /// What is drawn.
    pub fn leaf(&self) -> Leaf {
        self.leaf
    }

    /// Spreads the shadow and darkens it, as if the card had lifted toward the eye -- or settles it.
    pub fn lift(&mut self, grove: &mut Grove, lifted: bool) {
        if self.lifted == lifted {
            return;
        }
        self.lifted = lifted;
        let timing = Timing::ms(LIFT_MS).ease(Ease::Decelerate);
        let (at, opacity) = match lifted {
            true => (raised(self.spread), 0.9),
            false => (rest(self.spread), 0.55),
        };
        grove.animate(self.leaf, Motion::Location(at), timing);
        grove.animate(self.leaf, Motion::Opacity(opacity), timing);
    }
}

fn rest(spread: f32) -> Location {
    Location::new().xs(
        left((-spread).px()).right(100.pct() + spread.px()),
        top((-spread * 0.5).px()).bottom(100.pct() + (spread * 1.5).px()),
    )
}

fn raised(spread: f32) -> Location {
    let wide = spread * 1.8;
    Location::new().xs(
        left((-wide).px()).right(100.pct() + wide.px()),
        top((-wide * 0.2).px()).bottom(100.pct() + (wide * 1.8).px()),
    )
}
