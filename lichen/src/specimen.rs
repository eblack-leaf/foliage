//! The specimen: a mosaic in the shape an app was sketched in, with tips to press, each opening a
//! section on the ground its cut clears.
//!
//! What an app is oriented by, and not where it lives. A tip is a region on the shape with a chip
//! on it, and pressing the chip cross-sections the shape at that tip: a straight line is drawn
//! across it, the mosaic on the far side goes, drawn in toward the tip, and what is left is cut to
//! the line and carries the whole ramp again, with a blip behind the chip. The outline stays whole
//! throughout, so what is missing reads as missing. Pressing the chip again brings the shape back
//! out from the tip -- part way, the going held still, so the shape carries which tip was last
//! open; and then, slowly, the rest of the way. One at a time and never two: while a tip is chosen
//! the others are not there to press.
//!
//! # Where a section opens
//!
//! From the cut. The ground a cut clears is the section's foothold: its controls stand there, in
//! the box the sketch drew for the tip, clear of the mosaic and right where the shape just went, so
//! what was pressed and what it opened read as one place. What the controls open stands past them
//! the way the cut faces, out to the room's edge, where there is room for it -- down, up, or to
//! either side; a column across the room where the cut faces up or down, and the room's height
//! where it faces to a side. Longer than that, it scrolls there.
//!
//! Which way each cut faces is the sketch's to choose, and the sketch chooses it toward the most
//! open side of the room: nothing here second-guesses it.
//!
//! A specimen is grown into a room the app hands it -- the window, or any box the app knows the
//! size of. The room's size is handed in rather than read, because a width in foliage's grammar may
//! not read a height, and a shape that has to fit both ways is placed from both. Hand it again with
//! [`fit`](Specimen::fit) whenever it changes.
//!
//! Everything about that is here and is the same for every specimen. What differs -- the outline,
//! where the tips are, what they cut, and where each one's controls stand -- is a [`Sketch`].

use foliage::{
    Area, Axes, Boxed, Ease, Elevation, Field, Grove, Grow, Key, Leaf, Location, Motion, Palette,
    Place, Pollen, Scheme, ScrollTo, Source, Stem, Timing, Tween, center_x, center_y, left, top,
};

use crate::{
    Chip, Cut, Mosaic, Press, Ramp, Region, Scatter, Section, Silhouette, Turn, measure, rgb,
};

/// The spectrum of the scheme in force the blip behind a chosen tip is drawn from: palest at its
/// middle, which the chip covers, so what shows round the chip is the deep end.
pub const BLIP: usize = Scheme::SPECTRA - 1;

/// How far what a section's controls open stands off them, in pixels.
pub const STANDOFF: f32 = 24.0;

/// How wide what a section opens is at most where its cut faces up or down: a phone's width and
/// some, so on a wide room a form is not a strip.
pub const COLUMN: f32 = 640.0;

/// How far through its crop the shape is put back to when a tip is let go: `0.0` the whole shape,
/// `1.0` none of it.
const HELD: f32 = 0.45;

/// How long the shape then takes to come the rest of the way back to whole, in milliseconds. A
/// settling pace, well off the scale anything else on the page moves at.
const RESTORE: u64 = 30_000;

/// How wide the blip behind a chosen tip's chip is, as a fraction of the shape's width.
const BLIP_WIDTH: f32 = 0.11;

/// How wide a tip's chip is, in chip heights: all alike, rather than sized by their names.
const CHIP: f32 = 4.0;

/// How long a tip takes to go, and to come back.
const TIP_MS: u64 = 260;

/// Where the shape stands across the room.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Placement {
    /// In the middle.
    Centred,
    /// With its middle at this fraction of the room's width, but never past the margin -- off
    /// centre, to leave the most room on the side its cuts face.
    Across(f32),
}

/// One tip, as the sketch drew it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tip {
    /// Where on the shape it stands.
    pub at: Region,
    /// What choosing it cuts the shape to: the line, and the tip's own side of it.
    pub cut: Cut,
    /// Where its section's controls stand: a box on the ground the cut clears -- its top-left
    /// corner and its width and height, in the shape's unit space. It may reach past the outline;
    /// what matters is that the cut leaves it clear.
    pub controls: ((f32, f32), (f32, f32)),
}

/// A specimen, as the app sketched it.
#[derive(Clone, Debug)]
pub struct Sketch {
    /// The outline, as fractions of the page it was traced on, in order round it. Closed
    /// implicitly.
    pub outline: &'static [(f32, f32)],
    /// The page it was traced on, and how it is turned.
    pub page: (f32, f32),
    pub turn: Turn,
    /// Which way the ramp runs across it.
    pub along: (f32, f32),
    /// How far apart tile centres sit, as a fraction of the shape's width.
    pub pitch: f32,
    /// How large a tip is, as a fraction of the shape's width.
    pub tip: f32,
    /// How much of the room inside the margin the shape takes, of whichever side binds, and how
    /// far that margin is.
    pub share: f32,
    pub margin: f32,
    /// Where it stands across the room.
    pub placement: Placement,
    /// Its tips, in the order the app names them.
    pub tips: &'static [Tip],
}

/// A specimen, grown, and what is chosen on it.
pub struct Specimen {
    sketch: Sketch,
    shape: Silhouette,
    mosaic: Mosaic,
    /// What everything is grown on: the room.
    page: Leaf,
    /// The box the shape is drawn in.
    root: Leaf,
    /// What each choosing is roughened by, carried on, so no two are drawn alike.
    scatter: Scatter,
    tips: Vec<Grown>,
    chosen: Option<usize>,
    /// Whether the shape has arrived and the tips been let in. They wait on the mosaic: a tip to
    /// press on a shape still forming is a press on nothing.
    open: bool,
    /// Whether the app is holding the tips back, whatever the shape is doing.
    withheld: bool,
}

/// One tip, grown.
struct Grown {
    /// The region itself, which is what goes and comes back with everything hung on it.
    square: Leaf,
    part: Silhouette,
    section: Section,
    chip: Chip,
    /// Where the section's controls stand, and what they open, which scrolls: both hidden except
    /// while the tip is chosen -- hidden rather than only transparent, so nothing scrolls to room
    /// nothing is in, and nothing takes a press.
    controls: Leaf,
    details: Leaf,
    /// The fade a section that was let go is going out on, after which it is hidden.
    going: Option<Tween>,
}

/// A box in pixels, in the room: its left, top, width and height.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Rect(f32, f32, f32, f32);

impl Rect {
    fn location(self) -> Location {
        let Rect(x, y, width, height) = self;
        Location::new().xs(
            left(x.px()).width(width.max(0.0).px()),
            top(y.px()).height(height.max(0.0).px()),
        )
    }
}

/// Where everything stands, for one size of room.
struct Laid {
    shape: Rect,
    controls: Vec<Rect>,
    details: Vec<Rect>,
}

impl Specimen {
    /// Grows `sketch` on `page`, a box `room` in size, its tiles coloured from `ramp`, with a chip on
    /// each tip saying what it chooses, and brings it in.
    ///
    /// # Panics
    ///
    /// If there is not a chip for every tip, or a tip's cut keeps none of the shape.
    pub fn grow(
        grove: &mut Grove,
        page: Leaf,
        sketch: Sketch,
        ramp: Ramp,
        room: Area,
        chips: &[(Field, &str)],
    ) -> Self {
        assert_eq!(chips.len(), sketch.tips.len(), "a chip for every tip");
        let shape = Silhouette::traced(sketch.outline, sketch.page, sketch.turn);
        let mut mosaic = Mosaic::new(shape.clone(), ramp)
            .along(sketch.along)
            .pitch(sketch.pitch)
            .square(sketch.tip);
        let laid = lay(&sketch, &shape, room);
        let page = grove.branch(
            page,
            Stem::new().at(Location::new()).scrolls(Axes::Vertical),
        );
        let root = mosaic.grow(grove, page, laid.shape.location(), &mut Scatter::new());
        let regions: Vec<Region> = sketch.tips.iter().map(|tip| tip.at).collect();
        let squares = mosaic.regions(grove, &regions);
        let height = measure().height;
        let tips = sketch
            .tips
            .iter()
            .zip(squares)
            .zip(chips)
            .enumerate()
            .map(|(n, ((tip, square), &(mark, name)))| {
                let part = shape.cut(tip.cut);
                // Said here, as the specimen is grown, rather than when the tip is pressed and the
                // crop finds nothing to keep.
                assert!(
                    !part.is_empty(),
                    "the {name} tip's cut keeps none of the outline: its keep point is outside the \
                     shape or on the far side of its line -- a cut is in page fractions, as the \
                     outline is, not in unit space"
                );
                let section = Section::grow(grove, root, &shape, tip.cut);
                let chip = Chip::grow(
                    grove,
                    square,
                    Location::new().xs(
                        center_x(50.pct()).width((CHIP * height).px()),
                        center_y(50.pct()).height(height.px()),
                    ),
                    Elevation::up(3),
                    mark,
                    name,
                );
                // Out of sight and out of reach until the shape has arrived.
                grove.opacity(square, 0.0);
                grove.disable(square);
                // In front of the shape, whose cleared ground the controls stand on. Intangible, so
                // where they are merely there they take no press meant for anything else.
                let controls = grove.branch(
                    page,
                    Stem::new()
                        .at(laid.controls[n].location())
                        .elevate(Elevation::up(4))
                        .intangible()
                        .visible(false)
                        .opacity(0.0)
                        .focus_scope(),
                );
                // Solid, not intangible: a wheel notch goes to whatever is on top under the
                // pointer, and over the empty room between what a section grew that has to be this,
                // or the notch lands on what is under it and scrolls nothing. It stands only on
                // what the cut cleared, so there is nothing under it to take a press from.
                let details = grove.branch(
                    page,
                    Stem::new()
                        .at(laid.details[n].location())
                        .elevate(Elevation::up(4))
                        .scrolls(Axes::Vertical)
                        .visible(false)
                        .opacity(0.0)
                        .focus_scope(),
                );
                Grown {
                    square,
                    part,
                    section,
                    chip,
                    controls,
                    details,
                    going: None,
                }
            })
            .collect();
        mosaic.form(grove);
        Self {
            sketch,
            shape,
            mosaic,
            page,
            root,
            scatter: Scatter::new(),
            tips,
            chosen: None,
            open: false,
            withheld: false,
        }
    }

    /// The room is now `room` in size: everything is placed again for it.
    pub fn fit(&mut self, grove: &mut Grove, room: Area) {
        let laid = lay(&self.sketch, &self.shape, room);
        grove.at(self.root, laid.shape.location());
        for (n, tip) in self.tips.iter().enumerate() {
            grove.at(tip.controls, laid.controls[n].location());
            grove.at(tip.details, laid.details[n].location());
        }
    }

    /// The region tip `n` stands on, to hang more on beside its chip. What is hung should be
    /// `intangible`, so a press lands on the chip and nothing else.
    pub fn tip(&self, n: usize) -> Leaf {
        self.tips[n].square
    }

    /// Tip `n`'s chip: its name to write over, a badge to hang on its corner.
    pub fn chip(&self, n: usize) -> &Chip {
        &self.tips[n].chip
    }

    /// Where tip `n`'s section grows its controls: the box the sketch drew on the ground its cut
    /// clears.
    pub fn controls(&self, n: usize) -> Leaf {
        self.tips[n].controls
    }

    /// Where tip `n`'s section grows what its controls open: past them the way the cut faces, out
    /// to the room's edge. What reaches past that is scrolled to.
    pub fn details(&self, n: usize) -> Leaf {
        self.tips[n].details
    }

    /// What scrolls: the room, as the specimen grew into it.
    pub fn page(&self) -> Leaf {
        self.page
    }

    /// The box the shape is drawn in, to stand something on the shape itself with
    /// [`Silhouette::boxed`] -- something that is neither a tip nor a section's, such as what
    /// stands over the shape while the tips are [withheld](Self::withhold).
    pub fn root(&self) -> Leaf {
        self.root
    }

    /// The shape, in its unit space: to place more on it, or to cut it.
    pub fn shape(&self) -> &Silhouette {
        &self.shape
    }

    /// How many tips it has.
    pub fn len(&self) -> usize {
        self.tips.len()
    }

    /// Whether it has no tips.
    pub fn is_empty(&self) -> bool {
        self.tips.is_empty()
    }

    /// Which tip is chosen, if any.
    pub fn chosen(&self) -> Option<usize> {
        self.chosen
    }

    /// Which tip the frame reports a press on, if any.
    pub fn pressed(&self, pollen: &Pollen) -> Option<usize> {
        self.tips.iter().position(|tip| tip.chip.pressed(pollen))
    }

    /// Whether the frame asks for whatever is chosen to be let go: `Escape`, with nothing holding
    /// focus that would have taken it.
    pub fn dismissed(&self, pollen: &Pollen) -> bool {
        self.chosen.is_some()
            && pollen
                .root_keys()
                .iter()
                .any(|stroke| stroke.key == Key::Escape)
    }

    /// Carries the specimen: its arrival, the tips being let in once it has arrived, and the section
    /// of a tip that was let go being hidden once it has gone. Call once a frame.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        self.mosaic.frame(grove, pollen);
        if !self.open && self.mosaic.formed(pollen) {
            self.open = true;
            if !self.withheld {
                for tip in &self.tips {
                    let_in(grove, tip.square);
                }
            }
        }
        for tip in &mut self.tips {
            if let Some(going) = tip.going
                && pollen.finished(going)
            {
                tip.going = None;
                for leaf in [tip.controls, tip.details] {
                    grove.visible(leaf, false);
                }
            }
        }
    }

    /// Holds every tip out of reach and out of sight while `withheld`, and lets them in again
    /// after: for a specimen whose sections wait on something the shape does not -- a passphrase,
    /// a connection. Withholding lets go of whatever is chosen first, so the shape comes back as
    /// it would from a press. The shape itself is untouched either way; only the tips wait.
    ///
    /// Stated before the shape has arrived, the tips are not let in when it does, and come in
    /// when this is let go of.
    pub fn withhold(&mut self, grove: &mut Grove, withheld: bool) {
        if withheld == self.withheld {
            return;
        }
        if withheld {
            self.choose(grove, None);
        }
        self.withheld = withheld;
        for tip in &self.tips {
            match (withheld, self.open) {
                (true, _) => take_out(grove, tip.square),
                (false, true) => let_in(grove, tip.square),
                // The shape is still arriving, and lets the tips in itself when it has.
                (false, false) => {}
            }
        }
    }

    /// Whether the tips are being held back.
    pub fn withheld(&self) -> bool {
        self.withheld
    }

    /// Chooses tip `chosen`, or nothing.
    ///
    /// Three things move, and they are one decision. The shape is cut down to the tip's part,
    /// drawn in toward the tip, and the section line drawn; or the whole comes back out from the
    /// tip that was chosen and its line goes. The other tips go, and are not there to press until
    /// nothing is chosen again. And the chosen tip's section comes in, once the shape has
    /// gathered, so its controls stand on ground and not on tiles still going; and any other's
    /// goes at once.
    pub fn choose(&mut self, grove: &mut Grove, chosen: Option<usize>) {
        if chosen == self.chosen || (self.withheld && chosen.is_some()) {
            return;
        }
        let scheme = grove.scheme();
        let ground = rgb(scheme.color(Palette::Surface));
        let blip = Ramp::of(&scheme.stops(BLIP));
        let middle = |tip: usize| self.mosaic.middle_of(tip).unwrap_or((0.5, 0.5));
        // A crop over a crop is the second crop, so switching from one tip to another is answered
        // by the same line as choosing one.
        match chosen {
            Some(tip) => self.mosaic.crop(
                grove,
                &self.tips[tip].part,
                middle(tip),
                ground,
                Some((middle(tip), BLIP_WIDTH, &blip)),
                &mut self.scatter,
            ),
            None => self.mosaic.uncrop(
                grove,
                middle(self.chosen.unwrap_or(0)),
                ground,
                HELD,
                Some(RESTORE),
            ),
        }
        if chosen.is_none() {
            grove.scroll(self.page, ScrollTo::start());
        }
        for (n, tip) in self.tips.iter_mut().enumerate() {
            let this = chosen == Some(n);
            tip.section.draw(grove, this);
            match !self.withheld && chosen.is_none_or(|chosen| chosen == n) {
                true => let_in(grove, tip.square),
                false => take_out(grove, tip.square),
            }
            // The chosen chip wears the deep end of the blip behind it, so the two read as one.
            let worn = match this {
                true => Press::Chosen,
                false => Press::Rest,
            };
            tip.chip.arm(grove, worn);
            match (this, self.chosen == Some(n)) {
                (true, _) => {
                    tip.going = None;
                    grove.scroll(tip.details, ScrollTo::start());
                    for leaf in [tip.controls, tip.details] {
                        grove.visible(leaf, true);
                        grove.animate(
                            leaf,
                            Motion::Opacity(1.0),
                            Timing::ms(TIP_MS)
                                .after(Mosaic::CHANGE_MS)
                                .ease(Ease::Decelerate),
                        );
                    }
                }
                (false, true) => {
                    for leaf in [tip.controls, tip.details] {
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
        self.chosen = chosen;
    }
}

/// Puts a tip in reach and brings it in.
fn let_in(grove: &mut Grove, tip: Leaf) {
    grove.enable(tip);
    grove.animate(
        tip,
        Motion::Opacity(1.0),
        Timing::ms(TIP_MS).ease(Ease::Decelerate),
    );
}

/// Takes a tip out of reach and away.
fn take_out(grove: &mut Grove, tip: Leaf) {
    grove.disable(tip);
    grove.animate(
        tip,
        Motion::Opacity(0.0),
        Timing::ms(TIP_MS).ease(Ease::Accelerate),
    );
}

/// Where the shape, each tip's controls and what each opens stand in a room `room` in size.
///
/// The shape's unit space is its width, so a point `(x, y)` in it is `x` and `y` widths from the
/// shape's top-left corner whichever way the shape is drawn.
fn lay(sketch: &Sketch, shape: &Silhouette, room: Area) -> Laid {
    let margin = sketch.margin;
    let inner = (
        (room.width - 2.0 * margin).max(1.0),
        (room.height - 2.0 * margin).max(1.0),
    );
    let aspect = shape.aspect();
    let width = (inner.0 * sketch.share)
        .min(inner.1 * sketch.share / aspect)
        .max(1.0);
    let height = width * aspect;
    let x = match sketch.placement {
        Placement::Centred => (room.width - width) / 2.0,
        Placement::Across(at) => (room.width * at - width / 2.0).max(margin),
    };
    // At the top, under the margin: the cuts' room is what is left, and a cut facing down -- the
    // way a room scrolls -- has all of it.
    let shaped = Rect(x, margin, width, height);
    let controls: Vec<Rect> = sketch
        .tips
        .iter()
        .map(|tip| {
            let ((cx, cy), (cw, ch)) = tip.controls;
            Rect(
                shaped.0 + cx * width,
                shaped.1 + cy * width,
                cw * width,
                ch * width,
            )
        })
        .collect();
    let details = sketch
        .tips
        .iter()
        .zip(&controls)
        .map(|(tip, &controls)| beyond(room, margin, controls, shape.facing(tip.cut)))
        .collect();
    Laid {
        shape: shaped,
        controls,
        details,
    }
}

/// Where what a section's controls open stands: past them the way the cut faces, out to the
/// room's margin. Where the cut faces up or down, a column no wider than [`COLUMN`], centred on
/// the controls and kept inside the margins; where it faces to a side, the room's height, margin
/// to margin.
fn beyond(room: Area, margin: f32, Rect(x, y, width, height): Rect, (nx, ny): (f32, f32)) -> Rect {
    if nx.abs() > ny.abs() {
        let tall = room.height - 2.0 * margin;
        return match nx > 0.0 {
            true => {
                let from = x + width + STANDOFF;
                Rect(from, margin, room.width - margin - from, tall)
            }
            false => {
                let to = x - STANDOFF;
                Rect(margin, margin, to - margin, tall)
            }
        };
    }
    let column = (room.width - 2.0 * margin).min(COLUMN);
    let from = (x + width / 2.0 - column / 2.0).clamp(margin, room.width - margin - column);
    match ny > 0.0 {
        true => {
            let below = y + height + STANDOFF;
            Rect(from, below, column, room.height - margin - below)
        }
        false => {
            let above = y - STANDOFF;
            Rect(from, margin, column, above - margin)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit square, and lines across it a quarter in from either end.
    const SQUARE: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    const DOWN: Cut = Cut::new((0.0, 0.25), (1.0, 0.25), (0.5, 0.1));
    const UP: Cut = Cut::new((0.0, 0.75), (1.0, 0.75), (0.5, 0.9));
    const RIGHT: Cut = Cut::new((0.25, 0.0), (0.25, 1.0), (0.1, 0.5));

    /// Controls just past each cut, on what it clears.
    const TIPS: [Tip; 3] = [
        Tip {
            at: Region::Point((0.5, 0.1)),
            cut: DOWN,
            controls: ((0.1, 0.3), (0.8, 0.2)),
        },
        Tip {
            at: Region::Point((0.5, 0.9)),
            cut: UP,
            controls: ((0.1, 0.5), (0.8, 0.2)),
        },
        Tip {
            at: Region::Point((0.1, 0.5)),
            cut: RIGHT,
            controls: ((0.3, 0.1), (0.3, 0.8)),
        },
    ];

    fn laid(room: Area) -> Laid {
        let sketch = Sketch {
            outline: &SQUARE,
            page: (1.0, 1.0),
            turn: Turn::None,
            along: (1.0, 0.0),
            pitch: 0.05,
            tip: 0.1,
            share: 0.5,
            margin: 20.0,
            placement: Placement::Centred,
            tips: &TIPS,
        };
        let shape = Silhouette::traced(sketch.outline, sketch.page, sketch.turn);
        lay(&sketch, &shape, room)
    }

    /// In a 1000×600 room the square is 280 across, at 360 from the left and at the top margin.
    fn room() -> Area {
        Area::new(1000.0, 600.0)
    }

    /// The controls stand where the sketch drew them on the shape.
    #[test]
    fn the_controls_stand_on_the_shape() {
        let laid = laid(room());
        assert_eq!(laid.shape, Rect(360.0, 20.0, 280.0, 280.0));
        let Rect(x, y, width, height) = laid.controls[0];
        assert!((x - (360.0 + 0.1 * 280.0)).abs() < 0.01);
        assert!((y - (20.0 + 0.3 * 280.0)).abs() < 0.01);
        assert!((width - 0.8 * 280.0).abs() < 0.01 && (height - 0.2 * 280.0).abs() < 0.01);
    }

    /// Facing down, what they open runs from under the controls to the room's foot, a column
    /// centred on them.
    #[test]
    fn facing_down_it_opens_below() {
        let laid = laid(room());
        let controls = laid.controls[0];
        let Rect(x, y, width, height) = laid.details[0];
        assert!((y - (controls.1 + controls.3 + STANDOFF)).abs() < 0.01);
        assert!((y + height - 580.0).abs() < 0.01);
        assert_eq!(width, COLUMN);
        assert!((x + width / 2.0 - (controls.0 + controls.2 / 2.0)).abs() < 0.01);
    }

    /// Facing up, from the room's top to above the controls.
    #[test]
    fn facing_up_it_opens_above() {
        let laid = laid(room());
        let controls = laid.controls[1];
        let Rect(_, y, _, height) = laid.details[1];
        assert_eq!(y, 20.0);
        assert!((y + height - (controls.1 - STANDOFF)).abs() < 0.01);
    }

    /// Facing to a side, from past the controls to the margin that way, the room's height.
    #[test]
    fn facing_sideways_it_opens_to_that_side() {
        let laid = laid(room());
        let controls = laid.controls[2];
        let Rect(x, y, width, height) = laid.details[2];
        assert!((x - (controls.0 + controls.2 + STANDOFF)).abs() < 0.01);
        assert!((x + width - 980.0).abs() < 0.01);
        assert_eq!((y, height), (20.0, 560.0));
    }

    /// On a phone, a column is the room's width inside the margins, and kept inside them.
    #[test]
    fn a_column_keeps_inside_the_margins() {
        let laid = laid(Area::new(390.0, 844.0));
        let Rect(x, _, width, _) = laid.details[0];
        assert_eq!((x, width), (20.0, 350.0));
    }
}
