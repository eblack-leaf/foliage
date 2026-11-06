//! Odometer figures: a run of characters, each in its own window, that rolls to what it says next.

use foliage::{
    Axes, Boxed, Ease, Elevation, Fill, Font, FontSize, Grove, Grow, Leaf, Location, Motion, Place,
    Position, Sap, Source, Stem, Text, Timing, Vein, content, left, top,
};

/// What every window can show, in the order it rolls through them. A blank first, so a figure
/// that grows a digit rolls it down out of nothing.
const ALPHABET: &str = " 0123456789";

/// How long one window takes to roll, and how much later each window to its left starts.
const ROLL_MS: u64 = 520;
const STEP_MS: u64 = 55;

/// A little past and back: the click of a wheel settling.
const SETTLE: Ease = Ease::Curve {
    x1: 0.3,
    y1: 1.35,
    x2: 0.55,
    y2: 1.0,
};

/// One window: the clip, and the column of glyphs behind it.
/// How far, in letters, each window reaches past its glyph's advance on either side: an italic
/// glyph leans past its own, and a window as narrow as the advance would cut it off.
const LEAN: f32 = 0.35;

struct Window {
    column: Leaf,
}

/// A figure on an odometer: `width` windows, right-aligned, each a column of every glyph the
/// figure can show that slides to the one it shows now.
///
/// One run of glyphs per window -- the whole column, a line each -- clipped by a region a little
/// wider than the glyph's advance, so an italic's lean is kept, that
/// scrolls on no reader's say and holds its column [`pinned`](Place::pinned), and moved by
/// [`Motion::Shift`], which lays nothing out. So a total changing costs a few shifts, staggered from
/// the right as a carry would run.
///
/// The column's pitch is not known until it is laid out, so the first [`frame`](Roll::frame)
/// after growing measures it and puts every window where it belongs; a figure set before then is
/// held until it can be shown.
pub struct Roll {
    root: Leaf,
    windows: Vec<Window>,
    alphabet: Vec<char>,
    shown: Vec<char>,
    pitch: Option<f32>,
}

impl Roll {
    /// A figure `width` windows wide under `under`, standing where `at` says, set at `size` in
    /// `font` (foliage's own where `None`), in `fill`. Every glyph of [`ALPHABET`] can be shown, and
    /// so can each of `extra` -- a currency sign, a point, a percent.
    ///
    /// `at` reads letters in the figure's own face and size, so `width(n.letters())` is `n`
    /// windows exactly.
    #[allow(clippy::too_many_arguments)]
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        width: usize,
        extra: &str,
        size: FontSize,
        font: Option<Font>,
        fill: impl Into<Fill>,
    ) -> Self {
        let fill = fill.into();
        let alphabet = alphabet(extra);
        let column_text = alphabet.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("\n");
        let dressed = |stem: Stem| match font {
            Some(font) => stem.font(font).font_size(size),
            None => stem.font_size(size),
        };
        let root = grove.branch(under, dressed(Stem::new()).intangible().at(at));
        let windows = (0..width)
            .map(|n| {
                let window = grove.branch(
                    root,
                    dressed(Stem::new())
                        .intangible()
                        .scrolls(Axes::Vertical)
                        .elevate(Elevation::up(1))
                        .at(Location::new().xs(
                            left((n as f32 - LEAN).letters()).width((1.0 + 2.0 * LEAN).letters()),
                            top(0.px()).height(1.letters()),
                        )),
                );
                let text = Text::new(column_text.clone()).color(fill).intangible().pinned();
                let text = match font {
                    Some(font) => text.font(font).font_size(size),
                    None => text.font_size(size),
                };
                let column = grove.branch(
                    window,
                    text.at(Location::new().xs(
                        left(LEAN.letters()).width(content()),
                        top(0.px()).height(content()),
                    )),
                );
                Window { column }
            })
            .collect();
        Self {
            root,
            windows,
            alphabet,
            shown: vec![' '; width],
            pitch: None,
        }
    }

    /// The figure's own box, to place things against.
    pub fn leaf(&self) -> Leaf {
        self.root
    }

    /// What the figure says now: what it will roll to, where it has not rolled yet.
    pub fn says(&self) -> String {
        self.shown.iter().collect::<String>().trim_start().to_string()
    }

    /// Rolls the figure to `text`, right-aligned; a glyph it cannot show reads as a blank. Every
    /// window that changes rolls, the rightmost first. Nothing, if it says that already.
    pub fn set(&mut self, grove: &mut Grove, text: &str) {
        let next = align(text, self.windows.len());
        let changed = changed(&self.shown, &next);
        self.shown = next;
        let Some(pitch) = self.pitch else {
            return;
        };
        for (order, &n) in changed.iter().enumerate() {
            let to = self.offset(self.shown[n], pitch);
            grove.animate(
                self.windows[n].column,
                Motion::Shift(to),
                Timing::ms(ROLL_MS).after(order as u64 * STEP_MS).ease(SETTLE),
            );
        }
    }

    /// Puts the figure at `text` without rolling.
    pub fn place(&mut self, grove: &mut Grove, text: &str) {
        self.shown = align(text, self.windows.len());
        if let Some(pitch) = self.pitch {
            for n in 0..self.windows.len() {
                let to = self.offset(self.shown[n], pitch);
                grove.shift(self.windows[n].column, to);
            }
        }
    }

    /// Measures the columns once they are laid out, and puts each window at what it says -- and
    /// again whenever their pitch changes, as it does when a figure set in a size that differs by
    /// breakpoint crosses into another.
    pub fn frame(&mut self, grove: &mut Grove) {
        let Some(window) = self.windows.first() else {
            return;
        };
        let Some(Sap::Section(drawn)) = grove.tap(window.column, Vein::Drawn) else {
            return;
        };
        let pitch = drawn.height() / self.alphabet.len() as f32;
        if pitch <= 0.0 || self.pitch.is_some_and(|was| (was - pitch).abs() < 0.5) {
            return;
        }
        self.pitch = Some(pitch);
        for n in 0..self.windows.len() {
            let to = self.offset(self.shown[n], pitch);
            grove.shift(self.windows[n].column, to);
        }
    }

    fn offset(&self, glyph: char, pitch: f32) -> Position {
        let row = self.alphabet.iter().position(|&c| c == glyph).unwrap_or(0);
        Position::new(0.0, -(row as f32) * pitch)
    }
}

/// Every glyph a window can show: the blank and the digits, then `extra` in the order given, each
/// once.
fn alphabet(extra: &str) -> Vec<char> {
    let mut glyphs: Vec<char> = ALPHABET.chars().collect();
    for c in extra.chars() {
        if !glyphs.contains(&c) {
            glyphs.push(c);
        }
    }
    glyphs
}

/// `text` right-aligned in `width` windows: blanks to its left, and its leftmost glyphs dropped if
/// it is wider.
pub fn align(text: &str, width: usize) -> Vec<char> {
    let glyphs: Vec<char> = text.chars().collect();
    let skip = glyphs.len().saturating_sub(width);
    let mut out = vec![' '; width.saturating_sub(glyphs.len())];
    out.extend_from_slice(&glyphs[skip..]);
    out
}

/// Which windows differ between `from` and `to`, rightmost first: the order a carry runs in.
pub fn changed(from: &[char], to: &[char]) -> Vec<usize> {
    (0..from.len().min(to.len()))
        .rev()
        .filter(|&n| from[n] != to[n])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{align, alphabet, changed};

    #[test]
    fn a_figure_is_right_aligned_and_cropped_from_the_left() {
        assert_eq!(align("9.50", 6), vec![' ', ' ', '9', '.', '5', '0']);
        assert_eq!(align("123456", 4), vec!['3', '4', '5', '6']);
        assert_eq!(align("", 2), vec![' ', ' ']);
    }

    #[test]
    fn a_carry_runs_from_the_right() {
        let from = align("$9.99", 6);
        let to = align("$10.00", 6);
        // The point stands in the same window in both, so it stays put.
        assert_eq!(changed(&from, &to), vec![5, 4, 2, 1, 0]);
        let same = align("$4.20", 6);
        assert!(changed(&same, &same).is_empty());
        assert_eq!(changed(&align("41", 2), &align("42", 2)), vec![1]);
    }

    #[test]
    fn extras_are_added_once_after_the_digits() {
        let glyphs = alphabet("$.$%");
        assert_eq!(glyphs.len(), 11 + 3);
        assert_eq!(glyphs[0], ' ');
        assert_eq!(&glyphs[11..], &['$', '.', '%']);
    }
}
