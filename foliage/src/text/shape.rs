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

use crate::coordinate::{Area, Position};
use crate::text::font::{Font, Fonts};

/// One run, shaped: what the string turned out to be, before anything knows how wide it may be.
///
/// The characters are in the run's own index space -- the space a per-character tint and a caret
/// both address -- and each occupies one cell whatever it is, because a monospaced run advances by
/// its pitch rather than by what is in it.
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
        self.walk(self.columns(width), |character, index, column, line| {
            at(
                character,
                index,
                Position::new(column as f32 * cell.width, line as f32 * cell.height),
            );
        });
    }

    /// How many whole cells fit across `width`.
    ///
    /// Whole cells, because a monospaced line is an integral number of them: half a cell of room at
    /// the end of a line is not somewhere a character goes.
    fn columns(&self, width: f32) -> usize {
        if self.cell.width <= 0.0 {
            return 0;
        }
        (width / self.cell.width).floor().max(0.0) as usize
    }

    /// How many lines the run takes in `columns` cells.
    pub(crate) fn lines(&self, columns: usize) -> usize {
        self.walk(columns, |_, _, _, _| {})
    }

    /// Wraps the run into `columns` cells, handing every character the cell it lands in, and reports
    /// how many lines that took.
    ///
    /// The one walk. How tall a run is and where its glyphs go are the same question asked for two
    /// reasons, and asking it twice is how a run comes to be measured at one height and drawn at
    /// another.
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
    /// Only characters that leave ink are handed over. A space is an advance and a newline is a
    /// break; neither is a glyph, and a walk that reported them would have the renderer deciding
    /// what is worth drawing.
    fn walk(&self, columns: usize, mut place: impl FnMut(char, usize, usize, usize)) -> usize {
        if self.characters.is_empty() {
            return 0;
        }
        let columns = columns.max(1);
        let mut lines = 1;
        // Cells committed to the current line, and the spaces since the last word that are not
        // committed to anything yet.
        let mut used = 0;
        let mut pending = 0;
        let mut index = 0;
        while index < self.characters.len() {
            match self.characters[index] {
                '\n' => {
                    lines += 1;
                    used = 0;
                    pending = 0;
                    index += 1;
                    continue;
                }
                ' ' => {
                    pending += 1;
                    index += 1;
                    continue;
                }
                _ => {}
            }
            let start = index;
            while index < self.characters.len() && !matches!(self.characters[index], ' ' | '\n') {
                index += 1;
            }
            let word = index - start;
            let laid = |place: &mut dyn FnMut(char, usize, usize, usize),
                        from: usize,
                        count: usize,
                        column: usize,
                        line: usize| {
                for offset in 0..count {
                    place(
                        self.characters[from + offset],
                        from + offset,
                        column + offset,
                        line,
                    );
                }
            };
            if word > columns {
                let mut left = word;
                let mut at = start;
                let mut room = columns.saturating_sub(used + pending);
                if room == 0 {
                    lines += 1;
                    used = 0;
                    room = columns;
                }
                let taken = room.min(left);
                laid(&mut place, at, taken, used + pending, lines - 1);
                at += taken;
                used += pending + taken;
                left -= taken;
                while left > 0 {
                    lines += 1;
                    used = columns.min(left);
                    laid(&mut place, at, used, 0, lines - 1);
                    at += used;
                    left -= used;
                }
            } else if used + pending + word > columns {
                lines += 1;
                used = word;
                laid(&mut place, start, word, 0, lines - 1);
            } else {
                laid(&mut place, start, word, used + pending, lines - 1);
                used += pending + word;
            }
            pending = 0;
        }
        lines
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
struct Held {
    shaped: Shaped,
    seen: u64,
}

impl Shaping {
    /// The shaped form of `value`, shaping it if this is the first frame that has asked.
    pub(crate) fn shape(&mut self, fonts: &Fonts, font: Font, size: u32, value: &str) -> &Shaped {
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
                    shaped: shape(value, cell),
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
