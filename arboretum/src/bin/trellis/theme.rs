//! What the sheet is drawn in: graphite on drafting vellum.
//!
//! The plan is a drawing, so it is dressed as one -- not for the look of it, but because a drawing
//! sheet already has a convention for everything a planner has to say. Relationships are line
//! types; a problem is boxed in red pencil with a revision delta; where things stand is a title
//! block and a revision block; where you are is a key plan; time is a scale bar; a closer look is
//! a detail callout on a leader. The ground is a cool, pale drafting film and the linework is
//! graphite, as a drawing is before anything is printed from it, with exactly two other colours
//! on it, each with one job: red pencil for what clashes or cannot be undone, and a highlighter
//! for what is happening now or not yet filed.
//!
//! The instruments -- the rail, the layers, the key plan, the scale, the inspector -- lie over the
//! drawing as a drafting machine's heads do: graphite plates, ruled in hairline, with registration
//! ticks at their corners, lettered in the vellum's own pale, and the paper sliding under them.

use foliage::{Color, Font, FontSize, Layout, Palette, Scheme};

/// The drawing's ground: drafting vellum.
pub const PAPER: (f32, f32, f32) = (0.952, 0.004, 165.0);
/// The grid, minor and major, ruled a step and two into the vellum.
pub const GRID: (f32, f32, f32) = (0.912, 0.006, 210.0);
pub const MAJOR: (f32, f32, f32) = (0.862, 0.008, 225.0);
/// The linework and lettering on the drawing, in graphite, and the lettering a step quieter.
pub const LINE: (f32, f32, f32) = (0.33, 0.012, 250.0);
pub const FAINT: (f32, f32, f32) = (0.54, 0.010, 250.0);
/// The instruments' plates, what is pressed on them, and the rule between their parts.
pub const PLATE: (f32, f32, f32) = (0.255, 0.006, 250.0);
pub const RAISED: (f32, f32, f32) = (0.335, 0.007, 250.0);
pub const RULE: (f32, f32, f32) = (0.48, 0.007, 250.0);
/// The instruments' lettering, in the vellum's pale, and a step quieter.
pub const INK: (f32, f32, f32) = (0.945, 0.004, 165.0);
pub const SOFT: (f32, f32, f32) = (0.76, 0.006, 250.0);
/// Red pencil: what clashes, and what cannot be taken back.
pub const PENCIL: (f32, f32, f32) = (0.61, 0.185, 32.0);
/// The highlighter: what is happening now, and what has been changed and not yet filed -- laid
/// under lettering on the drawing, and lettered with on the instruments.
pub const MARKER: (f32, f32, f32) = (0.89, 0.16, 98.0);
/// The highlighter where it is drawn with on the vellum rather than laid under: a ring, a word.
pub const MARKED: (f32, f32, f32) = (0.62, 0.14, 75.0);

pub fn oklch((l, c, h): (f32, f32, f32)) -> Color {
    Color::oklch(l, c, h)
}

/// The paper, as a fill.
pub fn paper() -> Color {
    oklch(PAPER)
}

/// Graphite: the drawing's linework and lettering.
pub fn line() -> Color {
    oklch(LINE)
}

/// Graphite, a step quieter: what is said under a name, a label in a block.
pub fn faint() -> Color {
    oklch(FAINT)
}

/// The instruments' lettering.
pub fn ink() -> Color {
    oklch(INK)
}

/// The instruments' lettering, a step quieter.
pub fn soft() -> Color {
    oklch(SOFT)
}

pub fn pencil() -> Color {
    oklch(PENCIL)
}

pub fn marker() -> Color {
    oklch(MARKER)
}

pub fn marked() -> Color {
    oklch(MARKED)
}

pub fn rule() -> Color {
    oklch(RULE)
}

pub fn plate() -> Color {
    oklch(PLATE)
}

/// Nothing at all: a panel filled with it is its outline alone.
pub fn clear() -> Color {
    Color::rgba(0.0, 0.0, 0.0, 0.0)
}

/// The sheet's reading. The scheme's own ground is the instruments' plate, since the instruments
/// are what lichen's parts stand on; the paper is a colour of its own.
pub fn scheme() -> Scheme {
    let on = (0.24, 0.006, 250.0);
    Scheme::new()
        .set(Palette::Surface, oklch(PLATE))
        .set(Palette::Raised, oklch(RAISED))
        .set(Palette::Muted, oklch(RULE))
        .set(Palette::Ink, oklch(INK))
        // The press to make is printed in reverse: the vellum's pale, lettered in graphite.
        .set(Palette::Accent, oklch(INK))
        .set(Palette::Accent.on(), oklch(on))
        // A caret, a lit bar, and a value changed and not yet filed are all highlighted.
        .set(Palette::Accent.mark(), oklch(MARKER))
        .set(Palette::Signal, oklch((0.70, 0.012, 250.0)))
        .set(Palette::Signal.mark(), oklch(INK))
        .set(Palette::Positive, oklch((0.82, 0.01, 250.0)))
        .set(Palette::Positive.on(), oklch(on))
        .set(Palette::Danger, oklch(PENCIL))
        .set(Palette::Danger.mark(), oklch(PENCIL))
        .set(Palette::Caution, oklch(MARKER))
        .set(Palette::Caution.mark(), oklch(MARKER))
        .set(Palette::hue(0), oklch(PAPER))
}

/// The two weights of the drawing's lettering.
#[derive(Copy, Clone)]
pub struct Faces {
    pub regular: Font,
    pub bold: Font,
}

pub fn size(px: u32) -> FontSize {
    FontSize::new().xs(px)
}

/// A node's name, a block's value.
pub const TITLE: u32 = 12;
/// What is said under it.
pub const SUB: u32 = 11;
/// A block's label: the small capitals over a value.
pub const LABEL: u32 = 9;
/// The drawing number, the one large thing on the sheet.
pub const NUMBER: u32 = 22;
/// The zone letters round the border.
pub const ZONE: u32 = 11;
/// An instrument's heading.
pub const HEAD: u32 = 10;

/// The sheet, at full scale: an A-size drawing in sheet pixels, with its border and the frame of
/// zones inside it.
pub const SHEET: (f32, f32) = (1872.0, 1392.0);
pub const OUTER: f32 = 12.0;
pub const INNER: f32 = 36.0;
/// The grid, minor and major.
pub const CELL: f32 = 24.0;
pub const MAJOR_EVERY: usize = 5;
/// The zones round the border: numbers across, letters down.
pub const ZONES: (usize, usize) = (8, 6);
/// The title block and the revision block, in the corners of the drawing they belong in.
pub const TITLE_BLOCK: (f32, f32) = (416.0, 172.0);
pub const REVISIONS: (f32, f32) = (360.0, 128.0);
/// The steps the sheet can be read at.
pub const SCALES: [f32; 4] = [0.5, 0.75, 1.0, 1.25];
/// Read smaller than this, the sheet is an overview: cards in brief, no minor grid, and the
/// clashes boxed without their callouts.
pub const OVERVIEW: f32 = 0.8;

/// The smallest the sheet is read at, and the smallest it is read at in full: the scales at
/// which a card -- whose lettering keeps its size -- takes the most of the sheet, in brief and in
/// full.
pub fn least() -> (f32, f32) {
    let full = SCALES.iter().copied().find(|&s| s >= OVERVIEW).unwrap_or(1.0);
    (SCALES[0], full)
}

/// The title block's box on the sheet read at `scale`, `(left, top, right, bottom)` in the
/// sheet's full-scale units: its corner is pinned to the frame, but its lettering -- and so the
/// block -- does not shrink with the sheet.
pub fn title_block(scale: f32) -> (f32, f32, f32, f32) {
    let (w, h) = SHEET;
    let (bw, bh) = (TITLE_BLOCK.0 / scale, TITLE_BLOCK.1 / scale);
    (w - INNER - bw, h - INNER - bh, w - INNER, h - INNER)
}

/// The revision block's box on the sheet read at `scale`.
pub fn revision_block(scale: f32) -> (f32, f32, f32, f32) {
    let (w, _) = SHEET;
    let (bw, bh) = (REVISIONS.0 / scale, REVISIONS.1 / scale);
    (w - INNER - bw, INNER, w - INNER, INNER + bh)
}

/// The room between the window's edge and an instrument, and between two instruments.
pub const GUTTER: f32 = 12.0;
/// How tall the rail's plate is, and where the instruments under it start.
pub const RAIL: f32 = 50.0;
pub const TOP: f32 = GUTTER + RAIL + GUTTER;
/// How tall the scale bar's plate is.
pub const SCALE_BAR: f32 = 86.0;
/// Where the instruments over it stop, measured up from the window's foot.
pub const BOTTOM: f32 = GUTTER + SCALE_BAR + GUTTER;

/// How wide the column of layers and key plan is, where there is one.
pub fn dock(layout: Layout) -> f32 {
    match layout {
        Layout::Xs | Layout::Sm => 0.0,
        Layout::Md | Layout::Lg => 220.0,
        Layout::Xl => 232.0,
    }
}

/// How wide the inspector is where it is docked on the right; none where it rises from the foot.
pub fn docked(layout: Layout) -> Option<f32> {
    match layout {
        Layout::Xs | Layout::Sm | Layout::Md => None,
        Layout::Lg => Some(320.0),
        Layout::Xl => Some(340.0),
    }
}

/// How the scale reads in the title block: as a drafting scale.
pub fn scale_words(scale: f32) -> &'static str {
    match (scale * 100.0).round() as u32 {
        50 => "1:2",
        75 => "3:4",
        100 => "1:1",
        125 => "5:4",
        _ => "—",
    }
}

/// `text` cut to `most` characters, with an ellipsis where anything was cut.
pub fn clip(text: &str, most: usize) -> String {
    if text.chars().count() <= most {
        return text.to_string();
    }
    let cut: String = text.chars().take(most.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

/// How wide a chip with `name` on it is, in pixels -- what a row of them is laid out with.
pub fn chip_px(name: &str) -> f32 {
    let m = lichen::measure();
    m.height + 1.0 + (name.chars().count() as f32 + 2.0) * m.caption as f32 * 0.6
}
