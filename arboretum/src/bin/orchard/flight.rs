//! A drop of wax flying from where a good was added to the bag it went into: up off the press, and
//! over and down into the bag, in two legs.

use foliage::{
    Boxed, Ease, Elevation, Grove, Grow, Leaf, Location, Motion, Palette, Place, Pollen, Polygon,
    Position, Source, Timing, Tween, left, top,
};

/// How large the drop is.
const DROP: f32 = 14.0;
/// How long each leg takes.
const UP_MS: u64 = 300;
const DOWN_MS: u64 = 360;

enum Leg {
    Up(Tween, Position),
    Down(Tween),
}

pub struct Flight {
    drop: Leaf,
    leg: Option<Leg>,
}

impl Flight {
    /// A drop under `window`, which covers the whole surface, in front of everything.
    pub fn grow(grove: &mut Grove, window: Leaf) -> Self {
        let drop = grove.branch(
            window,
            Polygon::circle()
                .color(Palette::Accent)
                .intangible()
                .elevate(Elevation::up(90))
                .visible(false)
                .at(at(Position::default())),
        );
        Self { drop, leg: None }
    }

    /// Sends the drop from `from` to `to`, by way of a point above both.
    pub fn launch(&mut self, grove: &mut Grove, from: Position, to: Position) {
        let apex = Position::new(from.x + (to.x - from.x) * 0.35, from.y.min(to.y) - 70.0);
        grove.visible(self.drop, true);
        grove.at(self.drop, at(from));
        grove.opacity(self.drop, 1.0);
        let up = grove.animate(
            self.drop,
            Motion::Location(at(apex)),
            Timing::ms(UP_MS).ease(Ease::Decelerate),
        );
        self.leg = Some(Leg::Up(up, to));
    }

    /// Carries the drop. Whether it landed this frame.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> bool {
        match self.leg {
            Some(Leg::Up(up, to)) if pollen.finished(up) => {
                let down = grove.animate(
                    self.drop,
                    Motion::Location(at(to)),
                    Timing::ms(DOWN_MS).ease(Ease::Accelerate),
                );
                self.leg = Some(Leg::Down(down));
                false
            }
            Some(Leg::Down(down)) if pollen.finished(down) => {
                self.leg = None;
                grove.visible(self.drop, false);
                true
            }
            _ => false,
        }
    }
}

/// The drop's box, centred on `centre`.
fn at(centre: Position) -> Location {
    Location::new().xs(
        left((centre.x - DROP / 2.0).px()).width(DROP.px()),
        top((centre.y - DROP / 2.0).px()).height(DROP.px()),
    )
}
