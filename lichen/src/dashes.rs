//! The dashed line of a section, round any box.

use foliage::{
    Area, Boxed, Cap, Ease, Elevation, Fill, Grove, Grow, Leaf, Line, Location, Motion, Palette,
    Place, Point, Source, Stem, Timing, anchor, left, top,
};

/// How long a dash takes to come or go, and how long the whole outline takes to be drawn round.
const DASH_MS: u64 = 200;
const SWEEP: f32 = 360.0;
/// How much of each step along the outline is dash rather than gap.
const DUTY: f32 = 0.62;

/// A dashed outline round a box, drawn dash by dash and taken away the same way, in the look of a
/// [`Section`](crate::Section)'s line.
///
/// It stands over whatever it is [put over](Dashes::over), by reading that element's box as its
/// anchor, so one outline serves a whole wall of frames and moves to whichever is wanted. Its
/// dashes are placed in fractions of its own box, so it fits any box it is moved to; the counts
/// along each side are set once, for the box it was grown for, and a very different box gets
/// longer or shorter dashes rather than more or fewer.
///
/// Built ungrown, which costs nothing, dressed by its builders, and grown once with
/// [`grow`](Dashes::grow). It must be grown under the same region as what it stands over -- the
/// same scrolling content -- so that the anchor's box and its own are read in the same terms.
pub struct Dashes {
    like: Area,
    pitch: f32,
    pad: f32,
    weight: f32,
    fill: Fill,
    lift: Elevation,
    root: Option<Leaf>,
    dashes: Vec<Leaf>,
    over: Option<Leaf>,
    drawn: bool,
}

impl Dashes {
    /// An ungrown outline for boxes like `like`: dashes about 11 pixels apart, 1.25 thick, 4 pixels
    /// outside the box, in the scheme's muted ink, a step in front.
    pub fn new(like: Area) -> Self {
        Self {
            like,
            pitch: 11.0,
            pad: 4.0,
            weight: 1.25,
            fill: Fill::Role(Palette::Muted),
            lift: Elevation::up(1),
            root: None,
            dashes: Vec::new(),
            over: None,
            drawn: false,
        }
    }

    /// How far apart the dashes are, about, in pixels.
    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.max(1.0);
        self
    }

    /// How far outside the box it stands over the outline runs, in pixels.
    pub fn pad(mut self, pad: f32) -> Self {
        self.pad = pad;
        self
    }

    /// How thick each dash is, in pixels.
    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// What the dashes are inked in: a role, ideally.
    pub fn color(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();
        self
    }

    /// How far in front of what is under it the outline stands.
    pub fn elevate(mut self, lift: Elevation) -> Self {
        self.lift = lift;
        self
    }

    /// Grows the outline under `under`. Unseen, and over nothing. Hands back its own box, to hide
    /// with whatever it stands over. Nothing, if it is grown already.
    pub fn grow(&mut self, grove: &mut Grove, under: Leaf) -> Leaf {
        if let Some(root) = self.root {
            return root;
        }
        let pad = self.pad;
        let root = grove.branch(
            under,
            Stem::new()
                .intangible()
                .elevate(self.lift)
                .visible(false)
                .at(Location::new().xs(
                    left(anchor().left() - pad.px()).right(anchor().right() + pad.px()),
                    top(anchor().top() - pad.px()).bottom(anchor().bottom() + pad.px()),
                )),
        );
        let across = ((self.like.width + 2.0 * pad) / self.pitch).round().max(2.0) as usize;
        let down = ((self.like.height + 2.0 * pad) / self.pitch).round().max(2.0) as usize;
        self.dashes = perimeter(across, down)
            .into_iter()
            .map(|((x0, y0), (x1, y1))| {
                grove.branch(
                    root,
                    Line::new()
                        .between(
                            Point::new((x0 * 100.0).pct(), (y0 * 100.0).pct()),
                            Point::new((x1 * 100.0).pct(), (y1 * 100.0).pct()),
                        )
                        .weight(self.weight)
                        .color(self.fill)
                        .cap(Cap::Round)
                        .intangible()
                        .opacity(0.0),
                )
            })
            .collect();
        self.root = Some(root);
        root
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
    /// the move reads as a new outline rather than a jump. Nothing, until it is grown.
    pub fn over(&mut self, grove: &mut Grove, target: Leaf) {
        let Some(root) = self.root else {
            return;
        };
        if self.over == Some(target) {
            return;
        }
        self.over = Some(target);
        grove.anchor(root, target);
        grove.visible(root, true);
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
        self.fill = fill;
        for &dash in &self.dashes {
            grove.animate(dash, Motion::from(fill), Timing::ms(ms));
        }
    }
}

/// The dashes of a box's outline in fractions of it, clockwise from the top left: `across` along
/// the top and bottom, `down` along each side, each a [`DUTY`] of its step and centred in it.
fn perimeter(across: usize, down: usize) -> Vec<((f32, f32), (f32, f32))> {
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
