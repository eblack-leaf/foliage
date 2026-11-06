//! Words that are typed out and taken back, a character at a time.

use foliage::{Color, Ease, Grove, Grow, Leaf, Pollen, Timing, Tween};

/// What a typewriter reports when it comes to rest.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Typed {
    /// Every character is showing.
    Out,
    /// None is.
    Erased,
}

/// Types a run of glyphs out and takes it back, by tinting what is not yet showing clear.
///
/// The run is shaped once, at its whole length, and never again: what changes is one tint range,
/// so it does not reflow as it types, and its box is the box of the whole line from the start --
/// which is what a caret walking across a line already laid out looks like.
pub struct Typewriter {
    leaf: Leaf,
    count: usize,
    shown: usize,
    run: Option<(Tween, usize, usize)>,
}

impl Typewriter {
    /// Takes over `leaf`, a [`Text`](foliage::Text) saying `text`, and hides all of it.
    pub fn new(grove: &mut Grove, leaf: Leaf, text: &str) -> Self {
        let mut typewriter = Self {
            leaf,
            count: text.chars().count(),
            shown: 0,
            run: None,
        };
        typewriter.reveal(grove, 0);
        typewriter
    }

    /// The run being typed.
    pub fn leaf(&self) -> Leaf {
        self.leaf
    }

    /// Whether it is typing or erasing.
    pub fn busy(&self) -> bool {
        self.run.is_some()
    }

    /// How many characters are showing.
    pub fn shown(&self) -> usize {
        self.shown
    }

    /// Says `text` instead, none of it showing yet.
    pub fn write(&mut self, grove: &mut Grove, text: &str) {
        self.stop(grove);
        grove.text(self.leaf, text);
        self.count = text.chars().count();
        self.shown = usize::MAX;
        self.reveal(grove, 0);
    }

    /// Types out what is not showing, `ms` per character, starting `after` ms from now.
    pub fn type_out(&mut self, grove: &mut Grove, ms: u64, after: u64) {
        self.run_to(grove, self.count, ms, after, Ease::Linear);
    }

    /// Takes back what is showing, `ms` per character, from the end.
    pub fn erase(&mut self, grove: &mut Grove, ms: u64, after: u64) {
        self.run_to(grove, 0, ms, after, Ease::Accelerate);
    }

    /// Shows all of it at once.
    pub fn show_all(&mut self, grove: &mut Grove) {
        self.stop(grove);
        self.reveal(grove, self.count);
    }

    /// Follows the run. Reports when it comes to rest.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Option<Typed> {
        let (tween, _, to) = self.run?;
        if let Some(at) = pollen.tween(tween) {
            self.reveal(grove, at.round().max(0.0) as usize);
        }
        if pollen.finished(tween) {
            self.run = None;
            self.reveal(grove, to);
            return Some(if to == 0 { Typed::Erased } else { Typed::Out });
        }
        None
    }

    fn run_to(&mut self, grove: &mut Grove, to: usize, ms: u64, after: u64, ease: Ease) {
        self.stop(grove);
        let from = self.shown.min(self.count);
        let span = from.abs_diff(to) as u64;
        let tween = grove.tween(
            from as f32,
            to as f32,
            Timing::ms((span * ms).max(1)).after(after).ease(ease),
        );
        self.run = Some((tween, from, to));
    }

    fn stop(&mut self, grove: &mut Grove) {
        if let Some((tween, _, _)) = self.run.take() {
            grove.stop(tween);
        }
    }

    fn reveal(&mut self, grove: &mut Grove, shown: usize) {
        let shown = shown.min(self.count);
        if shown == self.shown {
            return;
        }
        self.shown = shown;
        match shown >= self.count {
            true => grove.untint(self.leaf),
            false => grove.tint(self.leaf, [(shown..self.count, Color::rgba(0.0, 0.0, 0.0, 0.0))]),
        }
    }
}
