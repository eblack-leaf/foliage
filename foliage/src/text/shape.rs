//! Shaping, and the one thing in the engine that is remembered between frames.
//!
//! # Rowan recomputes. This does not.
//!
//! Every other value is recomputed totally every frame, because the cost is arithmetic over a
//! handful of numbers and the saving is a whole class of stale-value bug (`rowan.md`). Turning a
//! string into glyphs is the measured exception: it is the one step that is a function of the
//! *string* rather than of the geometry, so it neither changes when the layout moves nor gets any
//! cheaper for being redone.
//!
//! So a run is shaped once per `(value, font, size)` and kept. Wrapping is **not** kept: it depends
//! on the width the layout produced, which is a different answer every time the layout moves, and
//! walking an already-shaped run to find its lines is cheap. That is the whole of the split, and it
//! is why there is one cache here and none anywhere else.

use std::collections::HashMap;
use std::sync::Arc;

use crate::coordinate::{Area, Position};
use crate::text::font::{Font, Fonts};

/// How much of a cell a line may be short of a whole one and still hold it.
///
/// A sixty-fourth: far below what any glyph occupies, so nothing that would genuinely overflow
/// fits, and far above the error a resolved box accumulates. It exists because a width is solved
/// rather than stated -- see [`columns`](Shaped::columns).
const SLACK: f32 = 1.0 / 64.0;

/// One run, shaped: what the string turned out to be, before anything knows how wide it may be.
///
/// The characters are in the run's own index space -- the space a per-character tint and a caret
/// both address -- and each occupies one cell whatever it is, because a monospaced run advances by
/// its pitch rather than by what is in it.
#[derive(Debug)]
pub(crate) struct Shaped {
    characters: Vec<char>,
    /// The pitch this run was shaped at, which is what makes the entry the run *at this size*.
    cell: Area,
    /// The widest hard line, in cells: the run's max-content width, before any wrapping.
    widest: usize,
}

impl Shaped {
    /// The widest the run would like to be, unwrapped: its longest hard line.
    ///
    /// Free, and exact. In a monospaced font this is a character count times a cell, so it needs no
    /// measure pass and is available before any layout has happened at all -- which is the whole
    /// reason width can flow down while height flows up.
    pub(crate) fn max_content(&self) -> f32 {
        self.widest as f32 * self.cell.width
    }

    /// How tall the run is at `width`.
    pub(crate) fn measure(&self, width: f32) -> f32 {
        self.lines(self.columns(width)) as f32 * self.cell.height
    }

    /// One character cell of this run: the advance every glyph shares, and the distance between two
    /// baselines.
    pub(crate) fn cell(&self) -> Area {
        self.cell
    }

    /// Where each of the run's characters lands at `width`, offset from the run's own top-left
    /// corner in logical pixels.
    ///
    /// The same walk that measures, so what is drawn is what was measured. A character that leaves
    /// no ink is not handed over: a space advances the walk and is nothing to draw.
    ///
    /// The index is the character's place in the **value**, spaces and newlines included, which is
    /// the space a [`tint`](crate::Grow::tint) and a caret are both addressed in. Counting drawn
    /// glyphs instead would make every index after a space mean something different from what was
    /// written.
    pub(crate) fn place(&self, width: f32, mut at: impl FnMut(char, usize, Position)) {
        let cell = self.cell;
        self.walk(self.columns(width), |index, column, line, ink| {
            if ink {
                at(
                    self.characters[index],
                    index,
                    Position::new(column as f32 * cell.width, line as f32 * cell.height),
                );
            }
        });
    }

    /// How many whole cells fit across `width`.
    ///
    /// Whole cells, because a monospaced line is an integral number of them: half a cell of room at
    /// the end of a line is not somewhere a character goes.
    ///
    /// Counted with [`SLACK`] of tolerance, because the width is a resolved box and a box is solved
    /// in floating point. An element sized to [`max_content`](Shaped::max_content) asks for exactly
    /// its own character count, and a placement that reaches that width by subtracting two
    /// coordinates -- which is every centred and every stretched one -- can land a fraction of a
    /// pixel under it. Floored exactly, that fraction is a whole column, and the one width at which
    /// a run must not wrap is the width it asked for.
    pub(crate) fn columns(&self, width: f32) -> usize {
        if self.cell.width <= 0.0 {
            return 0;
        }
        (width / self.cell.width + SLACK).floor().max(0.0) as usize
    }

    /// How many lines the run takes in `columns` cells.
    pub(crate) fn lines(&self, columns: usize) -> usize {
        self.walk(columns, |_, _, _, _| {})
    }

    /// The run wrapped into `columns` cells, for the questions asked per index rather than per
    /// glyph: where a caret stands, and which character a point falls on.
    pub(crate) fn wrap(&self, columns: usize) -> Wrap<'_> {
        Wrap {
            shaped: self,
            columns,
        }
    }

    /// Wraps the run into `columns` cells, handing every index the cell it lands in, and reports how
    /// many lines that took.
    ///
    /// The one walk. How tall a run is, where its glyphs go and where a caret stands are the same
    /// question asked for three reasons, and asking it three times is how a run comes to be measured
    /// at one height and drawn at another.
    ///
    /// Greedy, on word boundaries, with three rules and no fourth:
    ///
    /// - a newline ends a line
    /// - a word that does not fit in what is left starts a new one, and the spaces before it go with
    ///   the break rather than trailing off the end of the line
    /// - a word longer than a whole line fills what is left and breaks inside itself, because there
    ///   is no width at which it would fit
    ///
    /// A run with nothing in it takes no lines at all, which is what makes an empty element measure
    /// to zero rather than to one line of nothing.
    ///
    /// **Every index is handed over, and one more.** A character that leaves ink is handed the cell
    /// it is drawn in. One that does not -- a space, a newline -- is handed the cell a caret standing
    /// before it would occupy: the next cell along on the line it is on, held inside the line's
    /// width, so the spaces that went with a break are all at the end of the line they ended. The
    /// index past the last character is handed over last, because that is where a caret stands at
    /// the end of a run. What leaves ink is said alongside, so a walker that only draws can tell.
    fn walk(&self, columns: usize, mut at: impl FnMut(usize, usize, usize, bool)) -> usize {
        let columns = columns.max(1);
        let count = self.characters.len();
        let mut lines = 1;
        // Cells committed to the current line, and the spaces since the last word that are not
        // committed to anything yet.
        let mut used = 0;
        let mut pending = 0;
        let mut index = 0;
        while index < count {
            match self.characters[index] {
                '\n' => {
                    at(index, (used + pending).min(columns), lines - 1, false);
                    lines += 1;
                    used = 0;
                    pending = 0;
                    index += 1;
                    continue;
                }
                ' ' => {
                    at(index, (used + pending).min(columns), lines - 1, false);
                    pending += 1;
                    index += 1;
                    continue;
                }
                _ => {}
            }
            let start = index;
            while index < count && !matches!(self.characters[index], ' ' | '\n') {
                index += 1;
            }
            let word = index - start;
            let laid = |at: &mut dyn FnMut(usize, usize, usize, bool),
                        from: usize,
                        count: usize,
                        column: usize,
                        line: usize| {
                for offset in 0..count {
                    at(from + offset, column + offset, line, true);
                }
            };
            if word > columns {
                let mut left = word;
                let mut from = start;
                let mut room = columns.saturating_sub(used + pending);
                // No room at all is a break, and the spaces go with it exactly as they do before a
                // word that fits on the next line.
                if room == 0 {
                    lines += 1;
                    used = 0;
                    pending = 0;
                    room = columns;
                }
                let taken = room.min(left);
                laid(&mut at, from, taken, used + pending, lines - 1);
                from += taken;
                used += pending + taken;
                left -= taken;
                while left > 0 {
                    lines += 1;
                    used = columns.min(left);
                    laid(&mut at, from, used, 0, lines - 1);
                    from += used;
                    left -= used;
                }
            } else if used + pending + word > columns {
                lines += 1;
                used = word;
                laid(&mut at, start, word, 0, lines - 1);
            } else {
                laid(&mut at, start, word, used + pending, lines - 1);
                used += pending + word;
            }
            pending = 0;
        }
        at(count, (used + pending).min(columns), lines - 1, false);
        match count {
            0 => 0,
            _ => lines,
        }
    }
}

/// A shaped run at one width: the questions that are asked of an index rather than of a glyph.
///
/// What a caret and a hit test read. Both are stated in the run's own index space and answered in
/// its cells, and both are answered by the walk that measures and draws it -- so a caret stands
/// exactly where the glyph it precedes is drawn, at whatever width the run wrapped to.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Wrap<'a> {
    shaped: &'a Shaped,
    columns: usize,
}

impl Wrap<'_> {
    /// The cell a caret before character `index` stands in, as a column and a line.
    ///
    /// An index past the end is the place after the last character, which is where a caret stands
    /// at the end of a run.
    pub(crate) fn cell_of(&self, index: usize) -> (usize, usize) {
        let index = index.min(self.shaped.characters.len());
        let mut found = (0, 0);
        self.shaped.walk(self.columns, |at, column, line, _| {
            if at == index {
                found = (column, line);
            }
        });
        found
    }

    /// Which index a point at `column` on `line` falls on.
    ///
    /// The last index on the line that stands at or before the column, so a point further along a
    /// line is never an earlier place in the value; a point before the line's first index is that
    /// index, and a line past the last is the last. The column is already rounded to the nearer
    /// cell boundary by the caller, because which boundary a point is nearer is a question about
    /// pixels and this is a question about cells.
    pub(crate) fn index_at(&self, column: usize, line: usize) -> usize {
        let last = self.last_line();
        let line = line.min(last);
        let mut found = None;
        self.shaped.walk(self.columns, |index, at, on, _| {
            if on != line {
                return;
            }
            match found {
                None => found = Some(index),
                Some(_) if at <= column => found = Some(index),
                Some(_) => {}
            }
        });
        found.unwrap_or_default()
    }

    /// The first and last index on `line`, or on the last line where there is no such line.
    ///
    /// What `Home` and `End` go to in a run that wraps: the ends of the line the caret is on, which
    /// on a run of one line are the ends of the run.
    pub(crate) fn ends(&self, line: usize) -> (usize, usize) {
        let line = line.min(self.last_line());
        let mut ends = None;
        self.shaped.walk(self.columns, |index, _, on, _| {
            if on != line {
                return;
            }
            ends = Some(match ends {
                None => (index, index),
                Some((first, _)) => (first, index),
            });
        });
        ends.unwrap_or_default()
    }

    /// The line the index past the end stands on, which is the last line a caret can be on.
    fn last_line(&self) -> usize {
        self.cell_of(self.shaped.characters.len()).1
    }
}

/// Every run that has been shaped, keyed on `(value, font, size)`.
///
/// The font and the size are the outer key so that a lookup is by `&str` and allocates nothing: a
/// run is looked up twice a frame -- once to measure it, once to wrap it -- and a cache that built a
/// key each time would cost more than it saved.
#[derive(Default)]
pub(crate) struct Shaping {
    runs: HashMap<(Font, u32), HashMap<String, Held>>,
    /// Which sweep is running. An entry left at an older one belongs to a run nothing states any
    /// more.
    pass: u64,
}

/// One shaped run, and the sweep that last asked for it.
///
/// Shared rather than owned, because the run is also held by the element it was shaped for --
/// where a placement reading a character of that element resolves against it -- and a sweep that
/// drops the entry here must not take it out from under the element.
struct Held {
    shaped: Arc<Shaped>,
    seen: u64,
}

impl Shaping {
    /// The shaped form of `value`, shaping it if this is the first frame that has asked.
    pub(crate) fn shape(
        &mut self,
        fonts: &Fonts,
        font: Font,
        size: u32,
        value: &str,
    ) -> &Arc<Shaped> {
        let pass = self.pass;
        let cell = fonts.cell(font, size);
        let runs = self.runs.entry((font, size)).or_default();
        // Looked up by `&str`, so the ordinary frame -- in which every run has been seen before --
        // allocates nothing. Only a run that is genuinely new pays for a key, and it is already
        // paying to walk the whole string.
        if !runs.contains_key(value) {
            runs.insert(
                value.to_string(),
                Held {
                    shaped: Arc::new(shape(value, cell)),
                    seen: pass,
                },
            );
        }
        let held = runs.get_mut(value).expect("a run just shaped");
        held.seen = pass;
        &held.shaped
    }

    /// The shaped form of `value`, if a pass this frame has already shaped it.
    ///
    /// The read-only half, for the passes that come after the ones that measure. Extraction is one:
    /// every run it draws was shaped by R1 to be measured at all, so a run that is not here is a run
    /// nothing is laying out -- and a phase that shaped one of its own would be inserting entries
    /// after the sweep that decides what is still stated.
    pub(crate) fn shaped(&self, font: Font, size: u32, value: &str) -> Option<&Shaped> {
        Some(&self.runs.get(&(font, size))?.get(value)?.shaped)
    }

    /// Drops every run nothing asked for this frame, and opens the next sweep.
    ///
    /// Called once, at the end of the pass that measures, so what is kept is exactly what the tree
    /// currently states. A run that comes back is shaped again, which is the same cost it was the
    /// first time and is what keeps this the size of the tree rather than the size of the session.
    pub(crate) fn sweep(&mut self) {
        let pass = self.pass;
        self.runs.retain(|_, runs| {
            runs.retain(|_, held| held.seen == pass);
            !runs.is_empty()
        });
        self.pass += 1;
    }

    /// How many runs are held. The one measurable fact about the exception, so it is the one the
    /// suite asserts on.
    #[cfg(test)]
    pub(crate) fn held(&self) -> usize {
        self.runs.values().map(HashMap::len).sum()
    }
}

/// Turns a string into cells, and counts its widest hard line on the way through.
pub(crate) fn shape(value: &str, cell: Area) -> Shaped {
    let mut characters = Vec::with_capacity(value.len());
    let mut widest = 0;
    let mut line = 0;
    for character in value.chars() {
        match character {
            '\n' => {
                widest = widest.max(line);
                line = 0;
            }
            _ => line += 1,
        }
        characters.push(character);
    }
    Shaped {
        characters,
        cell,
        widest: widest.max(line),
    }
}
