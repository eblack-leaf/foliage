//! The leaf: a mosaic in the shape the site was sketched as, with three tips to press, and a place
//! for what each tip opens.
//!
//! The shape the suite's apps are built on. A tip is a region on the leaf with a chip on it, and
//! pressing the chip cross-sections the leaf at that tip: a line is drawn across it, the mosaic on
//! the far side goes, drawn in toward the tip, and what is left is cut to the line and carries the
//! whole ramp again, with a blip of green behind the chip. The outline stays whole throughout, so
//! what is missing reads as missing -- and the room the rest of the mosaic was taking is the
//! section's. Its controls stand in the box the sketch drew for that tip, on the leaf, and what
//! they open stands beside it.
//!
//! The leaf itself -- its tips, the cutting, the coming back -- is lichen's specimen. What is here
//! is the site's: where it stands the leaf, and where what each tip opens stands.
//!
//! The leaf does not move to make room, and the page is the one thing that scrolls. The leaf is
//! pinned to it, and what a tip opens is its content. On a window wide enough the leaf stands left of
//! centre and what a tip opens runs down the window to its right, scrolling past the leaf. Anywhere
//! narrower the leaf stands at the top, as wide as the window lets it be, and what a tip opens
//! starts under it on a ground of its own, and scrolls up over it like a sheet. The two are only
//! two placements of the same things, so a window resized across the line moves between them with
//! nothing grown again and nothing that scrolls inside anything else that does.
//!
//! Pressing the chip again brings the leaf back out from the tip -- part way, the going held still,
//! so the leaf carries which tip was last open, and then slowly the rest of the way.

use foliage::{
    Area, Boxed, Corners, Ease, Elevation, Escape, Field, FontSize, Grove, Grow, Leaf, Location,
    Motion, Palette, Panel, Place, Pollen, Rounding, Scheme, ScrollTo, Side, Source, Stem, Text,
    Timing, Tween, anchor, center_x, center_y, content, left, right, top,
};
use lichen::{Chip, Ramp, Region, measure};

use crate::leaf;
use crate::theme::{self, MARGIN, STANDOFF};

/// How long the leaf then takes to come the rest of the way back to whole, in milliseconds. A
/// settling pace, well off the scale anything else on the page moves at -- and shorter than an
/// app's, since a page is looked at for less time than a tool is kept open.
const RESTORE: u64 = 12_000;

/// How long what a tip opens takes to come in, and to go.
const TIP_MS: u64 = 260;

/// How wide a window has to be for what a tip opens to stand beside the leaf rather than under it.
const WIDE: f32 = 900.0;

/// Where the leaf's middle stands across a wide window, as a fraction of it: left of centre, so
/// the room it leaves on the right is the sections'.
const ACROSS: f32 = 0.28;

/// How much of a wide window's height the leaf takes, and how much of its width at most.
const SHARE: (f32, f32) = (0.94, 0.45);

/// How wide the leaf is at most on a narrow window: a phone's width, and a little over.
const NARROW: f32 = 560.0;

/// How far down the ground under what a tip opens reaches. Far past anything a section grows: it
/// floats, so it is not content and invents no room to scroll to however far it reaches, and it has
/// to cover the leaf for as long as there is anything scrolled up over it.
const GROUND: f32 = 40_000.0;

/// How far in front of the leaf what a tip opens stands, so it covers the leaf, and everything the
/// leaf carries, when it is scrolled up over it.
const OVER: i32 = 12;

/// Where the wordmark sits against the leaf, as fractions of the leaf's own box: how far in from
/// its left edge the wordmark ends, and how far down the box it is centred.
///
/// Inside the box rather than beside it. A leaf is not a rectangle, and the corner above its widest
/// point is empty at every size it is drawn -- so the wordmark tucks into the notch beside the tip,
/// where it costs the shape nothing.
const NOTCH: (f32, f32) = (0.28, 0.14);

/// The leaf, grown: lichen's specimen, and where the site stands it and what its tips open.
pub(crate) struct Specimen {
    /// The leaf, its tips, and the cutting: lichen's.
    shape: lichen::Specimen,
    aspect: f32,
    /// What everything is grown on: the window, as the one page that scrolls.
    page: Leaf,
    wordmark: Leaf,
    /// Whether the wordmark has been let in, once the leaf arrived.
    marked: bool,
    tips: Vec<Grown>,
    /// Whether what a tip opens stands under the leaf rather than beside it.
    stacked: bool,
}

/// What one tip opens, grown.
struct Grown {
    /// What the controls stand on, on the leaf, and what what they open stands on, on the page:
    /// both hidden except while the tip is chosen -- hidden rather than only transparent, so the
    /// page does not scroll to room nothing is in.
    sheet: Leaf,
    controls: Leaf,
    details: Leaf,
    /// The fade a sheet that was let go is going out on, after which it is hidden.
    going: Option<Tween>,
}

impl Specimen {
    /// Grows the leaf on `page`, coloured from `scheme`'s [`SPECIMEN`](theme::SPECIMEN) spectrum,
    /// with a chip on each tip saying what it chooses, and brings it in.
    ///
    /// The scheme is handed in rather than read, because the one repainted in `take_root` is not in
    /// force until that frame's writes land.
    ///
    /// # Panics
    ///
    /// If there is not a chip for every tip.
    pub(crate) fn grow(
        grove: &mut Grove,
        page: Leaf,
        scheme: &Scheme,
        chips: &[(Field, &'static str)],
        columns: &[Vec<&'static str>],
    ) -> Self {
        assert_eq!(chips.len(), leaf::TIPS.len(), "a chip for every tip");
        assert_eq!(
            columns.len(),
            leaf::TIPS.len(),
            "a column of controls for every tip"
        );
        let traced = leaf::shape();
        let aspect = traced.aspect();
        let arranged = Arranged::of(aspect, grove.viewport());
        // Pinned, so the leaf stays where it is while the page scrolls past it, and counts for
        // nothing in how far it scrolls. Intangible, so its box takes no press meant for anything
        // on the page beside it.
        let holder = grove.branch(page, Stem::new().at(Location::new()).pinned().intangible());
        let tips: Vec<lichen::Tip> = leaf::TIPS
            .iter()
            .map(|tip| lichen::Tip {
                at: Region::Point(traced.at(tip.at)),
                cut: tip.cut,
            })
            .collect();
        let mut shape = lichen::Specimen::grow(
            grove,
            holder,
            arranged.leaf.clone(),
            &leaf::sketch(&tips),
            Ramp::of(&scheme.stops(theme::SPECIMEN)),
            chips,
        );
        shape.restore(RESTORE);
        let root = shape.root();
        let grown = leaf::TIPS
            .iter()
            .zip(columns)
            .map(|(tip, column)| {
                // The box the sketch drew for the controls, on the leaf, and intangible, so a box
                // over the leaf that is merely there does not eat every press meant for it.
                let place = grove.branch(
                    root,
                    Stem::new()
                        .at(held(tip.controls, aspect, column))
                        .elevate(Elevation::up(3))
                        .font_size(lichen::caption())
                        .intangible(),
                );
                let sheet = grove.branch(
                    place,
                    Stem::new()
                        .at(Location::new())
                        .elevate(Elevation::up(1))
                        .intangible()
                        .visible(false)
                        .opacity(0.0),
                );
                let controls = grove.branch(
                    sheet,
                    Stem::new()
                        .at(Location::new())
                        .elevate(Elevation::up(1))
                        .font_size(lichen::caption())
                        .focus_scope(),
                );
                // Content of the page, and not a region of its own: the page is what scrolls it.
                let details = grove.branch(
                    page,
                    Stem::new()
                        .at(arranged.details.clone())
                        .elevate(Elevation::up(OVER))
                        .font_size(lichen::caption())
                        .visible(false)
                        .opacity(0.0)
                        .focus_scope(),
                );
                // What it stands on: the page's own colour, reaching from the window's one edge to
                // the other and far down, and in front of the leaf -- so scrolled up over the leaf
                // it covers it, and takes the presses meant for what it covers.
                grove.branch(
                    details,
                    Panel::new()
                        .color(Palette::Surface)
                        .rounding(Corners::none().side(Side::Top, Rounding::Lg))
                        .floats(Escape::Region)
                        .at(Location::new().xs(
                            left(0.px() - MARGIN.px()).right(100.pct() + MARGIN.px()),
                            top((-STANDOFF / 2.0).px()).height(GROUND.px()),
                        )),
                );
                Grown {
                    sheet,
                    controls,
                    details,
                    going: None,
                }
            })
            .collect();
        // Over the leaf's own box rather than beside it, and its own element rather than a hole in
        // it: stated in fractions of the box it is set against, so the two read as one composition
        // at every size, and set in the corner the blade leaves empty.
        let wordmark = grove.branch(
            holder,
            Text::new("foliage")
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(22).sm(28).md(36).lg(46).xl(56).short(28))
                .intangible()
                .opacity(0.0)
                .anchored(root)
                .at(Location::new().xs(
                    right(anchor().left() + anchor().width() * NOTCH.0).width(content()),
                    center_y(anchor().top() + anchor().height() * NOTCH.1).height(content()),
                )),
        );
        Self {
            shape,
            aspect,
            page,
            wordmark,
            marked: false,
            tips: grown,
            stacked: arranged.stacked,
        }
    }

    /// Where tip `n`'s section grows its controls: the box on the leaf the cut empties, never
    /// smaller than the column it was said to hold.
    pub(crate) fn controls(&self, n: usize) -> Leaf {
        self.tips[n].controls
    }

    /// Where tip `n`'s section grows what its controls open, which scrolls.
    pub(crate) fn details(&self, n: usize) -> Leaf {
        self.tips[n].details
    }

    /// Which tip is chosen, if any.
    pub(crate) fn chosen(&self) -> Option<usize> {
        self.shape.chosen()
    }

    /// Which tip the frame reports a press on, if any.
    pub(crate) fn pressed(&self, pollen: &Pollen) -> Option<usize> {
        self.shape.pressed(pollen)
    }

    /// Whether the frame asks for whatever is chosen to be let go: `Escape`, with nothing holding
    /// focus that would have taken it.
    pub(crate) fn dismissed(&self, pollen: &Pollen) -> bool {
        self.shape.dismissed(pollen)
    }

    /// Scrolls the page to bring `to` into view, where what a tip opens stands beside the leaf --
    /// the controls stay where they are, and what they changed is brought to them. Where it stands
    /// under the leaf the page is left where it is: what was just pressed is on the leaf, and a page
    /// that scrolled away from it would lose the reader their place.
    pub(crate) fn follow(&self, grove: &mut Grove, to: ScrollTo) {
        if !self.stacked {
            grove.scroll(self.page, to);
        }
    }

    /// Carries the leaf: its arrival, the wordmark being let in once it has arrived, its fit to the
    /// window, and what a tip that was let go opened being hidden once it has gone. Call once a
    /// frame.
    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if let Some(viewport) = pollen.resized() {
            let arranged = Arranged::of(self.aspect, viewport);
            grove.at(self.shape.root(), arranged.leaf);
            for tip in &self.tips {
                grove.at(tip.details, arranged.details.clone());
            }
            self.stacked = arranged.stacked;
        }
        self.shape.frame(grove, pollen);
        if !self.marked && self.shape.arrived() {
            self.marked = true;
            grove.animate(
                self.wordmark,
                Motion::Opacity(1.0),
                Timing::ms(420).ease(Ease::Decelerate),
            );
        }
        for tip in &mut self.tips {
            if let Some(going) = tip.going
                && pollen.finished(going)
            {
                tip.going = None;
                for leaf in [tip.sheet, tip.details] {
                    grove.visible(leaf, false);
                }
            }
        }
    }

    /// Chooses tip `chosen`, or nothing.
    ///
    /// The leaf is cut down to the tip's part, or comes back out from the tip that was chosen, and
    /// the other tips go or come back, as lichen's specimen does it; the wordmark goes with them.
    /// And the chosen tip's sheet comes in, once the leaf has gathered, so its controls stand on
    /// ground and not on tiles still going; and any other's goes at once.
    pub(crate) fn choose(&mut self, grove: &mut Grove, chosen: Option<usize>) {
        let was = self.shape.chosen();
        if chosen == was || !self.shape.arrived() {
            return;
        }
        self.shape.choose(grove, chosen);
        grove.animate(
            self.wordmark,
            Motion::Opacity(chosen.is_none() as u8 as f32),
            Timing::ms(TIP_MS).ease(Ease::Decelerate),
        );
        if chosen.is_none() {
            grove.scroll(self.page, ScrollTo::start());
        }
        for (n, tip) in self.tips.iter_mut().enumerate() {
            match (chosen == Some(n), was == Some(n)) {
                (true, _) => {
                    tip.going = None;
                    for leaf in [tip.sheet, tip.details] {
                        grove.visible(leaf, true);
                        grove.animate(
                            leaf,
                            Motion::Opacity(1.0),
                            Timing::ms(TIP_MS)
                                .after(lichen::Specimen::GATHERED)
                                .ease(Ease::Decelerate),
                        );
                    }
                }
                (false, true) => {
                    for leaf in [tip.sheet, tip.details] {
                        grove.animate(
                            leaf,
                            Motion::Opacity(0.0),
                            Timing::ms(TIP_MS).ease(Ease::Accelerate),
                        );
                    }
                    tip.going = Some(grove.timer(Timing::ms(TIP_MS)));
                }
                (false, false) => {}
            }
        }
    }
}

/// The box the sketch drew for a tip's controls -- `(corner, size)` in the leaf's unit space --
/// held open for `column`, a press to a row, however small the leaf is drawn.
///
/// The box is a share of the leaf and a press is a fixed size, so on a leaf drawn small enough the
/// box would be shorter than the column, or narrower than its widest press, and the column would
/// run past it. Stated as floors rather than as a box sized to fit, so on a leaf large enough the
/// box is the sketch's, and on a small one it reaches a little past what the sketch drew rather
/// than cutting anything off.
fn held(
    ((x, y), (width, height)): ((f32, f32), (f32, f32)),
    aspect: f32,
    column: &[&str],
) -> Location {
    let m = measure();
    let rows = column.len() as f32;
    let tall = rows * m.height + (rows - 1.0).max(0.0) * m.gap;
    let widest = column
        .iter()
        .copied()
        .max_by_key(|name| name.chars().count())
        .unwrap_or_default();
    Location::new().xs(
        left((x * 100.0).pct())
            .width((width * 100.0).pct())
            .at_least(Chip::width(widest)),
        top((y / aspect * 100.0).pct())
            .height((height / aspect * 100.0).pct())
            .at_least(tall.px()),
    )
}

/// Where the leaf and what its tips open stand, for one size of window.
///
/// From the window's size rather than from a placement per breakpoint, because the leaf has to fit
/// both ways -- as wide as the room allows, and no taller than the window -- and a width in this
/// grammar may not read a height. Stated again on every resize.
struct Arranged {
    leaf: Location,
    details: Location,
    stacked: bool,
}

impl Arranged {
    fn of(aspect: f32, viewport: Area) -> Self {
        let room = (
            (viewport.width - 2.0 * MARGIN).max(1.0),
            (viewport.height - 2.0 * MARGIN).max(1.0),
        );
        match viewport.width >= WIDE {
            // Beside: the leaf as tall as the window lets it be, left of centre, and what a tip
            // opens from past the leaf to the margin, from the top margin on down the page.
            true => {
                let width = (room.1 * SHARE.0 / aspect).min(room.0 * SHARE.1);
                let from = (viewport.width * ACROSS - width / 2.0).max(MARGIN);
                Self {
                    leaf: Location::new().xs(
                        left(from.px()).width(width.px()),
                        center_y(50.pct()).height((width * aspect).px()),
                    ),
                    details: Location::new().xs(
                        left((from + width + STANDOFF).px()).right(100.pct() - MARGIN.px()),
                        top(MARGIN.px()).height(room.1.px()),
                    ),
                    stacked: false,
                }
            }
            // Under: the leaf as wide as the window lets it be, at the top, and what a tip opens
            // from under it on down the page, which scrolls it up over the leaf.
            false => {
                let width = room.0.min(room.1 / aspect).min(NARROW);
                let under = MARGIN + width * aspect + STANDOFF;
                Self {
                    leaf: Location::new().xs(
                        center_x(50.pct()).width(width.px()),
                        top(MARGIN.px()).height((width * aspect).px()),
                    ),
                    details: Location::new().xs(
                        left(MARGIN.px()).right(100.pct() - MARGIN.px()),
                        top(under.px()).height(room.1.px()),
                    ),
                    stacked: true,
                }
            }
        }
    }
}
