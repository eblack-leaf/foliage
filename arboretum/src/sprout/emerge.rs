//! Things coming in out of the dark and into focus, the way a generated picture's preview does.

use foliage::{
    Boxed, Ease, Fill, Font, FontSize, Grove, Grow, Leaf, Location, Motion, Palette, Place, Pollen,
    Position, Sap, Text, Timing, Tween, Vein, anchor, left, top,
};

/// How long a focus pull takes, from nothing to sharp.
const FOCUS_MS: f32 = 1100.0;
/// How long something with no words in it takes to come up out of the dark.
const RISE_MS: u64 = 760;
/// How many ghosts a run of words is blurred with.
const GHOSTS: usize = 8;
/// How bright a ghost is at its brightest: eight together read about as solid as the words.
const GHOST_PEAK: f32 = 0.24;

/// What a ghost's opacity follows, as a fraction of the little it ends at: up from nothing, a
/// broad swell through the middle of the pull, and back down to almost nothing as the words
/// themselves take over. One motion, so nothing has to be chained.
const SWELL: Ease = Ease::Curve {
    x1: 0.3,
    y1: SWELL_HEIGHT,
    x2: 0.55,
    y2: SWELL_HEIGHT,
};
const SWELL_HEIGHT: f32 = 20.0;
/// What a ghost's opacity ends at; the swell scales up from it.
const GHOST_REST: f32 = GHOST_PEAK / (0.75 * SWELL_HEIGHT);

/// Something to bring in: a run of words, which is blurred into focus, or anything else, which
/// rises out of the dark.
#[derive(Copy, Clone, Debug)]
pub enum Arrival {
    /// Anything: a press, a picture, a panel, a whole form. It comes up out of the dark.
    Plain(Leaf),
    /// A run of words, set in `font` at `size` as it was grown, blurred `reach` pixels each way at
    /// first.
    Words {
        leaf: Leaf,
        font: Option<Font>,
        size: FontSize,
        reach: f32,
    },
}

impl Arrival {
    /// A run of words, as it was grown: `font` and `size` are what it is set in, since a ghost has
    /// to be set the same to land on it.
    pub fn words(leaf: Leaf, font: Option<Font>, size: FontSize, reach: f32) -> Self {
        Arrival::Words {
            leaf,
            font,
            size,
            reach,
        }
    }

    /// What is arriving.
    pub fn leaf(&self) -> Leaf {
        match *self {
            Arrival::Plain(leaf) | Arrival::Words { leaf, .. } => leaf,
        }
    }
}

impl From<Leaf> for Arrival {
    fn from(leaf: Leaf) -> Self {
        Arrival::Plain(leaf)
    }
}

/// Elements that are not there until they are wanted, and then come in out of the dark and into
/// focus: words as a soft, dim blur that brightens and draws together until it is the words
/// themselves, and everything else rising up out of the dark. There is no noise in it anywhere --
/// the look is a picture coming into focus, not a signal coming out of static.
///
/// foliage draws no blur, so a run of words is blurred the old way: a few ghosts of it, set the same
/// and drawn a little apart, which swell into view while they draw together onto the words and then
/// give way to them. The ghosts are grown beside the words, in whatever region the words are in,
/// and taken down once the words are sharp: an element carries nothing extra for having arrived.
pub struct Emerge {
    waiting: Vec<(Arrival, Tween)>,
    running: Vec<Running>,
    pulled: u32,
}

struct Running {
    target: Leaf,
    ghosts: Vec<Leaf>,
    done: Tween,
}

impl Emerge {
    pub fn new() -> Self {
        Self {
            waiting: Vec::new(),
            running: Vec::new(),
            pulled: 0,
        }
    }

    /// Puts `leaves` in the dark: there, laid out, and unseen until they arrive.
    pub fn hide(&mut self, grove: &mut Grove, leaves: &[Leaf]) {
        for &leaf in leaves {
            self.forget(grove, leaf);
            grove.opacity(leaf, 0.0);
        }
    }

    /// Brings `arrivals` in one after another, the first `after` ms from now and each `step` ms
    /// after the one before. One already on its way starts again.
    pub fn bring(&mut self, grove: &mut Grove, arrivals: &[Arrival], after: u64, step: u64) {
        for (n, &arrival) in arrivals.iter().enumerate() {
            self.forget(grove, arrival.leaf());
            let timer = grove.timer(Timing::ms(after + n as u64 * step));
            self.waiting.push((arrival, timer));
        }
    }

    /// Brings `arrival` in starting this frame -- or, if it is already showing, takes it back into
    /// the dark and brings it into focus again.
    pub fn now(&mut self, grove: &mut Grove, arrival: Arrival) {
        self.forget(grove, arrival.leaf());
        self.start(grove, arrival);
    }

    /// Whether `leaf` is waiting to arrive or arriving now.
    pub fn busy(&self, leaf: Leaf) -> bool {
        self.waiting.iter().any(|(waiting, _)| waiting.leaf() == leaf)
            || self.running.iter().any(|running| running.target == leaf)
    }

    /// Starts what is due and clears away the ghosts of what has arrived.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let mut n = 0;
        while n < self.waiting.len() {
            if pollen.finished(self.waiting[n].1) {
                let (arrival, _) = self.waiting.swap_remove(n);
                self.start(grove, arrival);
            } else {
                n += 1;
            }
        }
        self.running.retain(|running| {
            let done = pollen.finished(running.done);
            if done {
                for &ghost in &running.ghosts {
                    grove.prune(ghost);
                }
            }
            !done
        });
    }

    /// Stops whatever was bringing `leaf` in, leaving it where it is.
    fn forget(&mut self, grove: &mut Grove, leaf: Leaf) {
        self.waiting.retain(|(waiting, timer)| {
            let same = waiting.leaf() == leaf;
            if same {
                grove.stop(*timer);
            }
            !same
        });
        self.running.retain(|running| {
            let same = running.target == leaf;
            if same {
                for &ghost in &running.ghosts {
                    grove.prune(ghost);
                }
            }
            !same
        });
    }

    fn start(&mut self, grove: &mut Grove, arrival: Arrival) {
        let ghosts = match arrival {
            Arrival::Words {
                leaf,
                font,
                size,
                reach,
            } => self.pull(grove, leaf, font, size, reach),
            Arrival::Plain(_) => None,
        };
        let target = arrival.leaf();
        match ghosts {
            Some(ghosts) => {
                // The words themselves come up under their ghosts through the back half of the
                // pull, and are what is left when the ghosts go.
                grove.opacity(target, 0.0);
                grove.animate(
                    target,
                    Motion::Opacity(1.0),
                    Timing::ms((FOCUS_MS * 0.5) as u64)
                        .after((FOCUS_MS * 0.45) as u64)
                        .ease(Ease::Emphasis),
                );
                let done = grove.timer(Timing::ms(FOCUS_MS as u64 + 60));
                self.running.push(Running {
                    target,
                    ghosts,
                    done,
                });
            }
            None => {
                grove.opacity(target, 0.0);
                grove.shift(target, Position::new(0.0, 10.0));
                let timing = Timing::ms(RISE_MS).ease(Ease::Decelerate);
                grove.animate(target, Motion::Opacity(1.0), timing);
                grove.animate(target, Motion::Shift(Position::default()), timing);
            }
        }
    }

    /// Grows the ghosts that pull `leaf` into focus, or nothing if it is not words or has nowhere
    /// to grow them.
    fn pull(&mut self, grove: &mut Grove, leaf: Leaf, font: Option<Font>, size: FontSize, reach: f32) -> Option<Vec<Leaf>> {
        let Some(Sap::Text(words)) = grove.tap(leaf, Vein::Text) else {
            return None;
        };
        let Some(Sap::Leaf(Some(trunk))) = grove.tap(leaf, Vein::Trunk) else {
            return None;
        };
        let fill: Fill = match grove.tap(leaf, Vein::Color) {
            Some(Sap::Color(fill)) => fill,
            _ => Palette::Ink.into(),
        };
        self.pulled = self.pulled.wrapping_add(1);
        let turn = self.pulled as f32 * 0.61;
        let ghosts = (0..GHOSTS)
            .map(|n| {
                let mut text = Text::new(words.clone())
                    .font_size(size)
                    .color(fill)
                    .intangible()
                    .opacity(0.0)
                    .anchored(leaf)
                    .at(Location::new().xs(
                        left(anchor().left()).right(anchor().right()),
                        top(anchor().top()).bottom(anchor().bottom()),
                    ));
                if let Some(font) = font {
                    text = text.font(font);
                }
                let ghost = grove.branch(trunk, text);
                // Round a circle, every other ghost further out, so the blur reads as soft rather
                // than as six copies.
                let angle = turn + n as f32 * std::f32::consts::TAU / GHOSTS as f32;
                let out = reach * if n % 2 == 0 { 1.0 } else { 0.55 };
                grove.shift(ghost, Position::new(out * angle.cos(), out * angle.sin()));
                grove.animate(
                    ghost,
                    Motion::Shift(Position::default()),
                    Timing::ms(FOCUS_MS as u64).ease(Ease::Curve {
                        x1: 0.2,
                        y1: 0.0,
                        x2: 0.25,
                        y2: 1.0,
                    }),
                );
                grove.animate(ghost, Motion::Opacity(GHOST_REST), Timing::ms(FOCUS_MS as u64).ease(SWELL));
                ghost
            })
            .collect();
        Some(ghosts)
    }
}

impl Default for Emerge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cubic a swell follows, at `t` along its own parameter.
    fn swell(t: f32) -> (f32, f32) {
        let (x1, y1, x2, y2) = (0.3, SWELL_HEIGHT, 0.55, SWELL_HEIGHT);
        let u = 1.0 - t;
        let x = 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t;
        let y = 3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t;
        (x, y)
    }

    #[test]
    fn a_ghost_swells_to_about_its_peak_and_comes_back_down() {
        let brightest = (0..=100).map(|n| swell(n as f32 / 100.0).1 * GHOST_REST).fold(0.0, f32::max);
        assert!((brightest - GHOST_PEAK).abs() < 0.05, "{brightest}");
        assert!(swell(1.0).1 * GHOST_REST <= 0.017);
    }

    #[test]
    fn the_swell_runs_forward_in_time() {
        let mut last = -1.0;
        for n in 0..=100 {
            let (x, _) = swell(n as f32 / 100.0);
            assert!(x >= last);
            last = x;
        }
    }
}
