//! Things printed onto the page, the way a label printer feeds them.

use foliage::{
    Boxed, Color, Ease, Elevation, Fill, Grove, Grow, Leaf, Location, Motion, Panel, Place, Pollen,
    Sap, Source, Stem, Timing, Tween, Vein, left, top,
};

use super::typewriter::Typewriter;

/// How long the head takes over a character, how long a rule takes to be drawn across, and how
/// long a picture takes to feed through.
const TYPE_MS: u64 = 9;
const RULE_MS: u64 = 460;
const FEED_MS: u64 = 680;
/// How long something that is neither words, a rule nor a picture takes to come up.
const PLAIN_MS: u64 = 320;
/// How thick the head is drawn, riding the edge of what has fed through.
const HEAD: f32 = 2.0;

/// The way a rule is drawn across and a picture fed: quick off the mark and settling at the end,
/// as a carriage does.
const CARRIAGE: Ease = Ease::Curve {
    x1: 0.3,
    y1: 0.0,
    x2: 0.15,
    y2: 1.0,
};

/// One thing to be printed, grown or taken over by [`Print`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Printed {
    /// A rule, drawn across from its left end.
    Rule {
        /// What holds it, where the caller placed it.
        holder: Leaf,
        /// The bar itself, which grows across the holder.
        bar: Leaf,
    },
    /// A run of words, typed in at the head's pace.
    Words(Leaf),
    /// A picture, fed through from its top under a cover that slides away.
    Feed {
        /// What is fed: the picture's box.
        picture: Leaf,
        /// What covers it until it is fed.
        cover: Leaf,
    },
    /// Anything else -- a press, a panel -- which simply comes up.
    Plain(Leaf),
}

impl Printed {
    /// What is printed: the rule's holder, the words, the picture, the thing.
    pub fn leaf(&self) -> Leaf {
        match *self {
            Printed::Rule { holder, .. } => holder,
            Printed::Words(leaf) | Printed::Plain(leaf) => leaf,
            Printed::Feed { picture, .. } => picture,
        }
    }
}

/// A label printer for the page: things are laid out where they belong and left unprinted, then
/// printed in one after another -- rules drawn across from their left ends, words typed in at the
/// pace of a print head, pictures fed through from the top past a thin head in the ink's colour,
/// and anything else simply brought up.
///
/// Every part of it is laid out from the start and nothing moves a box but a rule's own bar, so a
/// page printing in does not reflow: the words are there all along, clear until the head has
/// passed them ([`Typewriter`]), and a picture is there under a cover of whatever it stands on,
/// which slides down off it. There is no blur and no noise in any of it -- what arrives arrives
/// crisp, a line at a time.
pub struct Print {
    waiting: Vec<(Printed, Tween, Option<String>)>,
    typing: Vec<Typewriter>,
    feeding: Vec<(Leaf, Tween)>,
}

impl Print {
    pub fn new() -> Self {
        Self {
            waiting: Vec::new(),
            typing: Vec::new(),
            feeding: Vec::new(),
        }
    }

    /// A rule under `under`, standing where `at` says, filled with `fill`. Undrawn.
    pub fn rule(grove: &mut Grove, under: Leaf, at: Location, fill: impl Into<Fill>) -> Printed {
        let holder = grove.branch(under, Stem::new().intangible().at(at));
        Self::rule_in(grove, holder, fill)
    }

    /// A rule filling `holder` -- placed, and anchored, however the caller likes -- filled with
    /// `fill`. Undrawn.
    pub fn rule_in(grove: &mut Grove, holder: Leaf, fill: impl Into<Fill>) -> Printed {
        let bar = grove.branch(holder, Panel::new().color(fill).intangible().at(undrawn()));
        Printed::Rule { holder, bar }
    }

    /// Takes over `leaf`, a run of words, and leaves it unprinted: laid out, and clear.
    pub fn words(grove: &mut Grove, leaf: Leaf) -> Printed {
        clear(grove, leaf);
        Printed::Words(leaf)
    }

    /// Covers `picture` in `ground` -- whatever the picture stands on -- with a head of `head`
    /// riding the cover's top edge, so the picture can be fed through. `lift` is how far in front
    /// of the picture's own parts the cover stands. Unfed.
    pub fn feed(
        grove: &mut Grove,
        picture: Leaf,
        ground: impl Into<Fill>,
        head: impl Into<Fill>,
        lift: Elevation,
    ) -> Printed {
        let cover = grove.branch(
            picture,
            Panel::new()
                .color(ground)
                .intangible()
                .elevate(lift)
                .at(covering()),
        );
        grove.branch(
            cover,
            Panel::new()
                .color(head)
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new()
                    .xs(left(0.px()).right(100.pct()), top(0.px()).height(HEAD.px()))),
        );
        Printed::Feed { picture, cover }
    }

    /// Takes over `leaf`, anything at all, and leaves it unprinted: there, and unseen.
    pub fn plain(grove: &mut Grove, leaf: Leaf) -> Printed {
        grove.opacity(leaf, 0.0);
        Printed::Plain(leaf)
    }

    /// Puts each of `items` back as it was before it was printed -- to be printed again, with
    /// whatever it says now.
    pub fn unprint(&mut self, grove: &mut Grove, items: &[Printed]) {
        for item in items {
            self.forget(grove, item.leaf());
            match *item {
                Printed::Rule { bar, .. } => grove.at(bar, undrawn()),
                Printed::Words(leaf) => clear(grove, leaf),
                Printed::Feed { cover, .. } => {
                    grove.visible(cover, true);
                    grove.at(cover, covering());
                }
                Printed::Plain(leaf) => grove.opacity(leaf, 0.0),
            }
        }
    }

    /// Prints `items` one after another: the first `after` ms from now and each `step` ms after
    /// the one before. One already printing starts again.
    pub fn run(&mut self, grove: &mut Grove, items: &[Printed], after: u64, step: u64) {
        for (n, item) in items.iter().enumerate() {
            self.forget(grove, item.leaf());
            let timer = grove.timer(Timing::ms(after + n as u64 * step));
            self.waiting.push((*item, timer, None));
        }
    }

    /// Prints `item` now.
    pub fn now(&mut self, grove: &mut Grove, item: Printed) {
        self.forget(grove, item.leaf());
        self.start(grove, item, None);
    }

    /// Writes `text` over `leaf`, a run of words, and types it in `after` ms from now: for words
    /// that say something new each time they are printed.
    pub fn retype(&mut self, grove: &mut Grove, leaf: Leaf, text: &str, after: u64) {
        self.forget(grove, leaf);
        grove.text(leaf, text);
        clear(grove, leaf);
        let timer = grove.timer(Timing::ms(after.max(1)));
        self.waiting
            .push((Printed::Words(leaf), timer, Some(text.to_string())));
    }

    /// Whether anything is waiting to be printed or printing.
    pub fn busy(&self) -> bool {
        !self.waiting.is_empty() || !self.typing.is_empty() || !self.feeding.is_empty()
    }

    /// Starts what is due, moves the head along what is being typed, and clears away the covers of
    /// what has fed through.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let mut n = 0;
        while n < self.waiting.len() {
            if pollen.finished(self.waiting[n].1) {
                let (item, _, text) = self.waiting.swap_remove(n);
                self.start(grove, item, text);
            } else {
                n += 1;
            }
        }
        self.typing.retain_mut(|typewriter| {
            typewriter.frame(grove, pollen);
            typewriter.busy()
        });
        self.feeding.retain(|&(cover, done)| {
            let fed = pollen.finished(done);
            if fed {
                grove.visible(cover, false);
            }
            !fed
        });
    }

    fn start(&mut self, grove: &mut Grove, item: Printed, text: Option<String>) {
        match item {
            Printed::Rule { bar, .. } => {
                grove.at(bar, undrawn());
                grove.animate(
                    bar,
                    Motion::Location(drawn()),
                    Timing::ms(RULE_MS).ease(CARRIAGE),
                );
            }
            Printed::Words(leaf) => {
                let text = match (text, grove.tap(leaf, Vein::Text)) {
                    (Some(text), _) => text,
                    (None, Some(Sap::Text(text))) => text,
                    _ => String::new(),
                };
                if text.is_empty() {
                    grove.untint(leaf);
                    return;
                }
                let mut typewriter = Typewriter::new(grove, leaf, &text);
                typewriter.type_out(grove, TYPE_MS, 0);
                self.typing.push(typewriter);
            }
            Printed::Feed { cover, .. } => {
                grove.visible(cover, true);
                grove.at(cover, covering());
                let done = grove.animate(
                    cover,
                    Motion::Location(fed()),
                    Timing::ms(FEED_MS).ease(CARRIAGE),
                );
                self.feeding.push((cover, done));
            }
            Printed::Plain(leaf) => {
                grove.animate(
                    leaf,
                    Motion::Opacity(1.0),
                    Timing::ms(PLAIN_MS).ease(Ease::Decelerate),
                );
            }
        }
    }

    /// Stops whatever was printing `leaf`, leaving it where it got to.
    fn forget(&mut self, grove: &mut Grove, leaf: Leaf) {
        self.waiting.retain(|(item, timer, _)| {
            let same = item.leaf() == leaf;
            if same {
                grove.stop(*timer);
            }
            !same
        });
        self.typing.retain(|typewriter| typewriter.leaf() != leaf);
    }
}

impl Default for Print {
    fn default() -> Self {
        Self::new()
    }
}

/// Clears every character of a run of words, leaving it laid out. The range runs past any run's
/// end, so it holds whatever the run says -- including words written this frame, which a tap
/// would not yet read.
fn clear(grove: &mut Grove, leaf: Leaf) {
    grove.tint(leaf, [(0..CLEAR, Color::rgba(0.0, 0.0, 0.0, 0.0))]);
}

/// Further than any run of words reaches.
const CLEAR: usize = 1 << 20;

/// A rule not yet drawn: no length, at its left end.
fn undrawn() -> Location {
    Location::new().xs(left(0.px()).width(0.px()), top(0.px()).bottom(100.pct()))
}

/// A rule drawn all the way across.
fn drawn() -> Location {
    Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).bottom(100.pct()))
}

/// A cover over the whole of its picture.
fn covering() -> Location {
    Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).bottom(100.pct()))
}

/// A cover slid down off its picture: its top at the picture's foot.
fn fed() -> Location {
    Location::new().xs(
        left(0.px()).right(100.pct()),
        top(100.pct()).bottom(100.pct()),
    )
}
