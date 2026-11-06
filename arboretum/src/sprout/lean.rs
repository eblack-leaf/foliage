//! A thing that leans toward the hand.

use foliage::{Ease, Grove, Grow, Leaf, Motion, Position, Sap, Timing, Vein};

/// How long a lean takes to let go when the pointer leaves.
const LET_GO_MS: u64 = 420;

/// A magnetic lean: while the pointer is over an element, the element is drawn a few pixels toward
/// it, and when the pointer leaves it eases back.
///
/// Drawn aside rather than placed ([`Grow::shift`]): nothing is laid out again, nothing placed
/// against it follows, and the element is pressed where it is drawn. It is a moment's
/// displacement, which is exactly what `shift` is for.
pub struct Lean {
    leaf: Leaf,
    reach: f32,
    leaning: bool,
    last: Position,
}

impl Lean {
    /// A lean on `leaf`, reaching at most `reach` pixels toward the pointer.
    pub fn new(leaf: Leaf, reach: f32) -> Self {
        Self {
            leaf,
            reach,
            leaning: false,
            last: Position::default(),
        }
    }

    /// What leans.
    pub fn leaf(&self) -> Leaf {
        self.leaf
    }

    /// Leans toward the pointer while it is `over` this element -- or over whatever the lean is
    /// read from, which need not be what leans -- and lets go when it leaves.
    pub fn frame(&mut self, grove: &mut Grove, over: bool) {
        if !over {
            if self.leaning {
                self.leaning = false;
                self.last = Position::default();
                grove.animate(
                    self.leaf,
                    Motion::Shift(Position::default()),
                    Timing::ms(LET_GO_MS).ease(Ease::Decelerate),
                );
            }
            return;
        }
        let (Some(cursor), Some(Sap::Section(drawn))) =
            (grove.cursor(), grove.tap(self.leaf, Vein::Drawn))
        else {
            return;
        };
        // Where it would be drawn unshifted, so the lean does not chase itself.
        let shifted = match grove.tap(self.leaf, Vein::Shift) {
            Some(Sap::Position(by)) => by,
            _ => Position::default(),
        };
        let centre = drawn.center();
        let rest = Position::new(centre.x - shifted.x, centre.y - shifted.y);
        let half = (drawn.width() / 2.0).max(1.0);
        let tall = (drawn.height() / 2.0).max(1.0);
        let toward = Position::new(
            ((cursor.x - rest.x) / half).clamp(-1.0, 1.0) * self.reach,
            ((cursor.y - rest.y) / tall).clamp(-1.0, 1.0) * self.reach,
        );
        if (toward.x - self.last.x).abs() + (toward.y - self.last.y).abs() < 0.25 {
            return;
        }
        self.leaning = true;
        self.last = toward;
        grove.animate(
            self.leaf,
            Motion::Shift(toward),
            Timing::ms(140).ease(Ease::Decelerate),
        );
    }
}
