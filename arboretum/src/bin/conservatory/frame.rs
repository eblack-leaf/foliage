//! One work on the wall: its plate, hung bare on the dark, and the placard under it.

use foliage::{
    Boxed, Fit, Grove, Grow, Image, Leaf, Location, Palette, Place, Plate, Pollen, Sap, Section,
    Source, Stem, Text, Vein, aspect, content, left, top,
};

use arboretum::genart::{Painter, Params};
use arboretum::sprout::Lean;

use crate::develop::{Develop, Event, area, blank};
use crate::works::{Art, WORKS};

/// How wide a frame is, narrow and wide; its plate is square.
pub const SIDE: (f32, f32) = (220.0, 280.0);
/// How much of a frame is placard, under the plate: room for two lines.
pub const PLACARD: f32 = 52.0;
/// The picture's size in texels.
pub const PICTURE: u32 = 256;
/// How long a frame takes to develop once painted.
const DEVELOP_MS: u64 = 1500;

/// Where a frame stands, wide and narrow.
#[derive(Copy, Clone, Debug)]
pub struct Spot {
    pub narrow: f32,
    pub wide: f32,
}

/// One work, hung.
pub struct Frame {
    pub work: usize,
    root: Leaf,
    plate: Plate,
    image: Leaf,
    placard: Leaf,
    number: usize,
    lean: Lean,
    develop: Option<Develop>,
    /// A real picture asked for, and whether it has been answered.
    file: Option<Plate>,
    lit: bool,
}

impl Frame {
    /// Hangs work `work` in `corridor`, its left edge at `at`. It waits in the dark until it is
    /// brought in.
    pub fn grow(grove: &mut Grove, corridor: Leaf, work: usize, at: Spot) -> Self {
        let w = &WORKS[work];
        let root = grove.branch(
            corridor,
            Stem::new()
                .interactive()
                .opacity(0.0)
                .at(Location::new()
                    .xs(
                        left(at.narrow.px()).width(SIDE.0.px()),
                        top(0.px()).height((SIDE.0 + PLACARD).px()),
                    )
                    .md(
                        left(at.wide.px()).width(SIDE.1.px()),
                        top(0.px()).height((SIDE.1 + PLACARD).px()),
                    )),
        );
        // The plate stands dark until the work develops out of it.
        let plate = grove.plate();
        grove.load(plate, blank((PICTURE, PICTURE)), area((PICTURE, PICTURE)));
        let image = grove.branch(
            root,
            Image::new(plate)
                .fit(Fit::Crop)
                .intangible()
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(aspect(1.0)))),
        );
        let number = format!("no. {:02}", w.no);
        let placard = grove.branch(
            root,
            Text::new(format!("{number} · {}", w.title))
                .font_size(lichen::caption())
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(100.pct() - (PLACARD - 12.0).px()).height(content()),
                )),
        );
        grove.tint(placard, [(0..number.len(), Palette::Muted)]);
        let file = match w.art {
            Art::File(name) => Some(grove.image(crate::works_origin(name))),
            Art::Grown => None,
        };
        Self {
            work,
            root,
            plate,
            image,
            placard,
            number: number.len(),
            lean: Lean::new(image, 4.0),
            develop: None,
            file,
            lit: false,
        }
    }

    /// What takes the press, what the hover watches, and what comes in out of the dark.
    pub fn leaf(&self) -> Leaf {
        self.root
    }

    /// The picture, which the outline is drawn round.
    pub fn picture_leaf(&self) -> Leaf {
        self.image
    }

    /// Whether it has started developing.
    pub fn started(&self) -> bool {
        self.develop.is_some()
    }

    /// Whether it is developing now.
    pub fn busy(&self) -> bool {
        self.develop.as_ref().is_some_and(|develop| !develop.done())
    }

    /// The finished picture, once it is.
    pub fn picture(&self) -> Option<&[u8]> {
        self.develop.as_ref().filter(|d| d.done()).and_then(|d| d.picture())
    }

    /// Where the frame is drawn now.
    pub fn drawn(&self, grove: &Grove) -> Option<Section> {
        match grove.tap(self.root, Vein::Drawn) {
            Some(Sap::Section(drawn)) => Some(drawn),
            _ => None,
        }
    }

    /// Starts painting and developing the work.
    pub fn develop(&mut self, grove: &mut Grove) {
        if self.develop.is_some() {
            return;
        }
        let w = &WORKS[self.work];
        let painter = Painter::new(
            w.style,
            Params::of(w.style, w.seed()),
            &w.ramp(),
            PICTURE,
            PICTURE,
            0.0,
        );
        self.develop = Some(Develop::new(grove, self.plate, painter, w.steps, DEVELOP_MS, w.seed()));
    }

    /// Carries the frame: its develop, its real picture if it has one, and how it answers the
    /// pointer. Whether it finished developing this frame.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, over: bool) -> bool {
        if let Some(file) = self.file {
            if pollen.loaded(file) {
                grove.depict(self.image, file);
                self.file = None;
            } else if pollen.missing(file) {
                // Not there after all: the work is grown, as it would be with no file named.
                self.file = None;
            }
        }
        let mut developed = false;
        if let Some(develop) = &mut self.develop
            && develop.frame(grove) == Some(Event::Developed)
        {
            developed = true;
        }
        self.lean.frame(grove, over);
        if over != self.lit {
            self.lit = over;
            let number = if over { Palette::Accent } else { Palette::Muted };
            grove.tint(self.placard, [(0..self.number, number)]);
        }
        developed
    }

    /// Whether a real picture is still on its way, which holds the develop back.
    pub fn waiting_on_file(&self) -> bool {
        self.file.is_some()
    }
}
