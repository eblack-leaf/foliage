//! lichen's dashed line, around any box.

use foliage::{
    Area, Boxed, Cap, Ease, Elevation, Fill, Grove, Grow, Leaf, Line, Location, Motion,
    Place, Point, Source, Stem, Timing, anchor, left, top,
};

/// How long a dash takes to come or go, and how long the whole outline takes to be drawn round.
const DASH_MS: u64 = 200;
const SWEEP: f32 = 360.0;
/// How much of each step along the outline is dash rather than gap.
const DUTY: f32 = 0.62;

/// A dashed outline round a box, drawn dash by dash and taken away the same way, in the look of
/// lichen's section line.
///
/// It stands over whatever it is [`put over`](Dashes::over), by reading that element's box as its
/// anchor, so one outline serves a whole wall of frames and moves to whichever is wanted. Its
/// dashes are placed in fractions of its own box, so it fits any box it is moved to; the counts
/// along each side are set once, for the box it was grown for, and a very different box gets
/// longer or shorter dashes rather than more or fewer.
///
/// It must be grown under the same region as what it stands over -- the same scrolling content --
/// so that the anchor's box and its own are read in the same terms.
pub struct Dashes {
    root: Leaf,
    dashes: Vec<Leaf>,
    over: Option<Leaf>,
    drawn: bool,
}

impl Dashes {
    /// An outline under `under`, `pad` pixels outside whatever it is put over, of dashes `weight`
    /// thick and about `pitch` pixels apart round a box like `like`, in `fill`, `lift` in front.
    /// Unseen, and over nothing.
    #[allow(clippy::too_many_arguments)]
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        like: Area,
        pitch: f32,
        pad: f32,
        weight: f32,
        fill: impl Into<Fill>,
        lift: Elevation,
    ) -> Self {
        let fill = fill.into();
        let root = grove.branch(
            under,
            Stem::new()
                .intangible()
                .elevate(lift)
                .visible(false)
                .at(Location::new().xs(
                    left(anchor().left() - pad.px()).right(anchor().right() + pad.px()),
                    top(anchor().top() - pad.px()).bottom(anchor().bottom() + pad.px()),
                )),
        );
        let across = ((like.width + 2.0 * pad) / pitch).round().max(2.0) as usize;
        let down = ((like.height + 2.0 * pad) / pitch).round().max(2.0) as usize;
        let dashes = perimeter(across, down)
            .into_iter()
            .map(|((x0, y0), (x1, y1))| {
                grove.branch(
                    root,
                    Line::new()
                        .between(
                            Point::new((x0 * 100.0).pct(), (y0 * 100.0).pct()),
                            Point::new((x1 * 100.0).pct(), (y1 * 100.0).pct()),
                        )
                        .weight(weight)
                        .color(fill)
                        .cap(Cap::Round)
                        .intangible()
                        .opacity(0.0),
                )
            })
            .collect();
        Self {
            root,
            dashes,
            over: None,
            drawn: false,
        }
    }

    /// The outline's own box.
    pub fn leaf(&self) -> Leaf {
        self.root
    }

    /// What it stands over now.
    pub fn standing_over(&self) -> Option<Leaf> {
        self.over
    }

    /// Whether it is drawn, or being drawn.
    pub fn drawn(&self) -> bool {
        self.drawn
    }

    /// Moves the outline over `target`, at once. A drawn outline is drawn again from its start, so
    /// the move reads as a new outline rather than a jump.
    pub fn over(&mut self, grove: &mut Grove, target: Leaf) {
        if self.over == Some(target) {
            return;
        }
        self.over = Some(target);
        grove.anchor(self.root, target);
        grove.visible(self.root, true);
        if self.drawn {
            for &dash in &self.dashes {
                grove.opacity(dash, 0.0);
            }
            self.drawn = false;
            self.draw(grove, true);
        }
    }

    /// Draws the outline, dash by dash clockwise from its top left, or takes it away the same way.
    pub fn draw(&mut self, grove: &mut Grove, drawn: bool) {
        if self.drawn == drawn {
            return;
        }
        self.drawn = drawn;
        let count = self.dashes.len().max(1) as f32;
        for (n, &dash) in self.dashes.iter().enumerate() {
            let front = n as f32 / count;
            grove.animate(
                dash,
                Motion::Opacity(if drawn { 1.0 } else { 0.0 }),
                Timing::ms(DASH_MS)
                    .after((front * SWEEP) as u64)
                    .ease(Ease::Decelerate),
            );
        }
    }

    /// Recolours every dash, over `ms`.
    pub fn tint(&mut self, grove: &mut Grove, fill: impl Into<Fill>, ms: u64) {
        let fill = fill.into();
        for &dash in &self.dashes {
            grove.animate(dash, Motion::from(fill), Timing::ms(ms));
        }
    }
}

/// The dashes of a box's outline in fractions of it, clockwise from the top left: `across` along
/// the top and bottom, `down` along each side, each a [`DUTY`] of its step and centred in it.
pub fn perimeter(across: usize, down: usize) -> Vec<((f32, f32), (f32, f32))> {
    let mut dashes = Vec::with_capacity(2 * (across + down));
    let side = |count: usize, along: &mut dyn FnMut(f32, f32)| {
        let step = 1.0 / count as f32;
        for n in 0..count {
            let middle = (n as f32 + 0.5) * step;
            along(middle - step * DUTY / 2.0, middle + step * DUTY / 2.0);
        }
    };
    side(across, &mut |a, b| dashes.push(((a, 0.0), (b, 0.0))));
    side(down, &mut |a, b| dashes.push(((1.0, a), (1.0, b))));
    side(across, &mut |a, b| dashes.push(((1.0 - a, 1.0), (1.0 - b, 1.0))));
    side(down, &mut |a, b| dashes.push(((0.0, 1.0 - a), (0.0, 1.0 - b))));
    dashes
}

#[cfg(test)]
mod tests {
    use super::perimeter;

    #[test]
    fn the_outline_goes_round_once_and_stays_on_the_box() {
        let dashes = perimeter(4, 3);
        assert_eq!(dashes.len(), 14);
        for ((x0, y0), (x1, y1)) in &dashes {
            for v in [x0, y0, x1, y1] {
                assert!((0.0..=1.0).contains(v), "{v} is off the box");
            }
        }
        // Clockwise: along the top first, then down the right.
        assert_eq!(dashes[0].0.1, 0.0);
        assert_eq!(dashes[4].0.0, 1.0);
        assert!(dashes[4].1.1 > dashes[4].0.1);
    }
}
