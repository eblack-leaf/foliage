//! The specimen: a mosaic in the shape an app was sketched in, with tips on it to press, each of
//! which cuts the shape.
//!
//! A tip is a region on the shape with a chip on it, and pressing the chip cross-sections the
//! shape at that tip: a straight line is drawn across it, the mosaic on the far side goes, drawn
//! in toward the tip, and what is left is cut to the line and carries the whole ramp again, with a
//! blip behind the chip. The outline stays whole throughout, so what is missing reads as missing.
//! Pressing the chip again brings the shape back out from the tip -- part way, the going held
//! still, so the shape carries which tip was last open; and then, slowly, the rest of the way. One
//! at a time and never two: while a tip is chosen the others are not there to press.
//!
//! That is all of it. Where the shape stands, and what stands where a cut cleared, are the app's:
//! the app grows the specimen into a box it places, places it again whenever it likes through
//! [`root`](Specimen::root), and asks [`cut`](Specimen::cut) where a tip's line falls and which way
//! it faces, to stand whatever it opens there. [`GATHERED`](Specimen::GATHERED) is when the shape
//! has drawn in to a chosen tip, for what the app brings in on what it cleared.

use foliage::{
    Ease, Elevation, Field, Grove, Grow, Key, Leaf, Location, Motion, Palette, Pollen, Scheme,
    Source, Timing, center_x, center_y,
};

use crate::{
    Chip, Cut, Mosaic, Press, Ramp, Region, Rgb, Scatter, Section, Silhouette, measure, rgb,
};

/// The spectrum of the scheme in force the blip behind a chosen tip is drawn from: palest at its
/// middle, which the chip covers, so what shows round the chip is the deep end.
pub const BLIP: usize = Scheme::SPECTRA - 1;

/// How far through its crop the shape is put back to when a tip is let go: `0.0` the whole shape,
/// `1.0` none of it.
const HELD: f32 = 0.45;

/// How long the shape then takes to come the rest of the way back to whole, in milliseconds,
/// unless the app [says otherwise](Specimen::restore). A settling pace, well off the scale anything
/// else on the page moves at.
const RESTORE: u64 = 30_000;

/// How wide the blip behind a chosen tip's chip is, as a fraction of the shape's width.
const BLIP_WIDTH: f32 = 0.11;

/// How wide a tip's chip is, in chip heights: all alike, rather than sized by their names.
const CHIP: f32 = 4.0;

/// How long a tip takes to go, and to come back.
const TIP_MS: u64 = 260;

/// One tip, as the sketch drew it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tip {
    /// Where on the shape it stands.
    pub at: Region,
    /// What choosing it cuts the shape to: the line, and the tip's own side of it.
    pub cut: Cut,
}

/// A specimen, as the app sketched it: the shape and its tips. Nothing about where it stands.
#[derive(Clone, Debug)]
pub struct Sketch<'a> {
    /// The outline, in its own unit space, in order round it. Closed implicitly. Its tips and
    /// their cuts are in the same space.
    pub outline: &'a [(f32, f32)],
    /// Which way the ramp runs across it, and what its outline is dashed in: the ramp's middle
    /// unless stated.
    pub along: (f32, f32),
    pub edge: Option<Rgb>,
    /// How far apart tile centres sit, as a fraction of the shape's width.
    pub pitch: f32,
    /// How large a tip is, as a fraction of the shape's width.
    pub tip: f32,
    /// Its tips, in the order the app names them.
    pub tips: &'a [Tip],
}

/// Where a tip's cut falls on the shape, in the shape's unit space -- `x` across its width, `y`
/// down it in widths -- for an app to stand what the tip opens against.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Cleared {
    /// Where the line enters the outline and where it last leaves it.
    pub from: (f32, f32),
    pub to: (f32, f32),
    /// Which way the side the cut clears lies from the line: the line's perpendicular, of unit
    /// length, turned away from what it keeps.
    pub facing: (f32, f32),
}

/// A specimen, grown, and what is chosen on it.
pub struct Specimen {
    shape: Silhouette,
    mosaic: Mosaic,
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
    /// How long the shape takes to come the rest of the way back after a tip is let go.
    restore: u64,
}

/// One tip, grown.
struct Grown {
    /// The region itself, which is what goes and comes back with everything hung on it.
    square: Leaf,
    part: Silhouette,
    section: Section,
    chip: Chip,
    cleared: Cleared,
}

impl Specimen {
    /// How long after a tip is chosen the shape has drawn in to it, in milliseconds: when what the
    /// app stands on the cleared side can come in on ground, and not on tiles still going.
    pub const GATHERED: u64 = Mosaic::CHANGE_MS;

    /// Grows `sketch` under `under`, in a box placed `at`, its tiles coloured from `ramp`, with a
    /// chip on each tip saying what it chooses, and brings it in.
    ///
    /// # Panics
    ///
    /// If there is not a chip for every tip, or a tip's cut keeps none of the shape or misses it.
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        sketch: &Sketch<'_>,
        ramp: Ramp,
        chips: &[(Field, &str)],
    ) -> Self {
        assert_eq!(chips.len(), sketch.tips.len(), "a chip for every tip");
        let shape = Silhouette::new(sketch.outline);
        let mut mosaic = Mosaic::new(shape.clone(), ramp)
            .along(sketch.along)
            .pitch(sketch.pitch)
            .square(sketch.tip);
        if let Some(edge) = sketch.edge {
            mosaic = mosaic.edge(edge);
        }
        let root = mosaic.grow(grove, under, at, &mut Scatter::new());
        let regions: Vec<Region> = sketch.tips.iter().map(|tip| tip.at).collect();
        let squares = mosaic.regions(grove, &regions);
        let height = measure().height;
        let tips = sketch
            .tips
            .iter()
            .zip(squares)
            .zip(chips)
            .map(|((tip, square), &(mark, name))| {
                let part = shape.cut(tip.cut);
                // Said here, as the specimen is grown, rather than when the tip is pressed and the
                // crop finds nothing to keep.
                assert!(
                    !part.is_empty(),
                    "the {name} tip's cut keeps none of the outline: its keep point is outside the \
                     shape or on the far side of its line"
                );
                let (from, to) = shape
                    .crossings(tip.cut)
                    .unwrap_or_else(|| panic!("the {name} tip's cut misses the outline"));
                let cleared = Cleared {
                    from,
                    to,
                    facing: shape.facing(tip.cut),
                };
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
                Grown {
                    square,
                    part,
                    section,
                    chip,
                    cleared,
                }
            })
            .collect();
        mosaic.form(grove);
        Self {
            shape,
            mosaic,
            root,
            scatter: Scatter::new(),
            tips,
            chosen: None,
            open: false,
            withheld: false,
            restore: RESTORE,
        }
    }

    /// How long, in milliseconds, the shape takes to come the rest of the way back to whole after
    /// a tip is let go: a page looked at for less time than a tool is kept open may want it
    /// sooner.
    pub fn restore(&mut self, ms: u64) {
        self.restore = ms;
    }

    /// Whether the shape has arrived, and its tips -- unless they are held back -- been let in.
    pub fn arrived(&self) -> bool {
        self.open
    }

    /// The box the shape is drawn in: to place again with [`at`](Grow::at), to anchor to, or to
    /// stand something on the shape itself with [`Silhouette::boxed`].
    pub fn root(&self) -> Leaf {
        self.root
    }

    /// The shape, in its unit space: to place more on it, or to cut it.
    pub fn shape(&self) -> &Silhouette {
        &self.shape
    }

    /// Where tip `n`'s cut falls, and which way the side it clears lies.
    pub fn cut(&self, n: usize) -> Cleared {
        self.tips[n].cleared
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

    /// Carries the specimen: its arrival, and the tips being let in once it has arrived. Call once
    /// a frame.
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
    }

    /// Holds every tip out of reach and out of sight while `withheld`, and lets them in again
    /// after: for a specimen whose tips wait on something the shape does not -- a passphrase, a
    /// connection. Withholding lets go of whatever is chosen first, so the shape comes back as it
    /// would from a press. The shape itself is untouched either way; only the tips wait.
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
    /// The shape is cut down to the tip's part, drawn in toward the tip, and the section line
    /// drawn; or the whole comes back out from the tip that was chosen and its line goes. The
    /// other tips go, and are not there to press until nothing is chosen again. What stands on
    /// the cleared side is the app's to bring in, [`GATHERED`](Self::GATHERED) after this.
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
                Some(self.restore),
            ),
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
