//! The page: one leaf, and the three tips on it.
//!
//! The leaf arrives the way lichen brings a mosaic in -- the outline as dashes, then the tiles
//! sweeping along the ramp they are coloured from -- and then its three tips are let in. Two open a
//! section beside the leaf: where else foliage is written down, and the parts it builds. The third
//! is a way out of the leaf: its cut plays as it would, and once the leaf has gathered to the tip
//! the herbarium comes up over the site -- one of the suite's apps, whole, on a page of its own.
//! Leaving it lets go of the tip, and the leaf comes back out.

use std::rc::Rc;

use foliage::{
    Axes, Boxed, Elevation, Grove, Grow, Location, Palette, Panel, Place, Pollen, Root, ScrollTo,
    Stem, Timing, Tween,
};
use lichen::Mosaic;
use tracing::info;

use crate::herbarium::Herbarium;
use crate::icons::Icons;
use crate::links::{self, Links};
use crate::showcase::{self, Showcase};
use crate::specimen::Specimen;
use crate::theme;

/// Which tip each section stands on, in the order the chips are given to the leaf.
const LINKS: usize = 0;
const SHOWCASE: usize = 1;
const HERBARIUM: usize = 2;

/// The app.
///
/// Whatever it keeps is its own. Nothing here is handed to the engine, and the engine has no way to
/// reach it: the value stays on this side and touches the tree only through the [`Grove`] it is
/// lent for the length of a frame.
pub(crate) struct Site {
    leaf: Specimen,
    links: Links,
    showcase: Showcase,
    herbarium: Herbarium,
    /// The leaf gathering to the herbarium's tip, after which its page comes up.
    entering: Option<Tween>,
}

impl Root for Site {
    fn take_root(grove: &mut Grove) -> Self {
        let scheme = theme::scheme();
        grove.repaint(scheme);
        let icons = Rc::new(grove.marks::<Icons>());
        let italic = theme::italic(grove);
        let window = grove.plant(Panel::new().color(Palette::Surface));
        // The page scrolls, for a window too narrow to have what a tip opens beside the leaf: it
        // stands under it instead, and is scrolled to.
        let page = grove.branch(
            window,
            Stem::new()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .scrolls(Axes::Vertical),
        );
        let leaf = Specimen::grow(
            grove,
            page,
            &scheme,
            &[
                (icons.link, "links"),
                (icons.layers, "showcase"),
                (icons.key, "herbarium"),
            ],
            // The herbarium's tip opens nothing beside the leaf, so it holds no controls.
            &[links::column(), showcase::column(), Vec::new()],
        );
        let links = Links::grow(
            grove,
            leaf.controls(LINKS),
            leaf.details(LINKS),
            &icons,
            italic,
        );
        // The showcase and the herbarium keep the marks: each grows what it shows the first time
        // it is shown rather than up front.
        let showcase = Showcase::grow(
            grove,
            leaf.controls(SHOWCASE),
            leaf.details(SHOWCASE),
            icons.clone(),
            italic,
        );
        let herbarium = Herbarium::grow(grove, window, icons, italic);
        info!("leaf planted");
        Site {
            leaf,
            links,
            showcase,
            herbarium,
            entering: None,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        // While the herbarium is up it is the page: the site under it is covered, and carries on
        // only with what it was already doing. Leaving it lets go of its tip.
        if self.herbarium.frame(grove, &pollen) {
            self.leaf.choose(grove, None);
        }
        if self.herbarium.is_open() {
            self.leaf.frame(grove, &pollen);
            return;
        }
        if let Some(entering) = self.entering
            && pollen.finished(entering)
        {
            self.entering = None;
            self.herbarium.open(grove);
        }
        // A press on the chosen tip lets go of it, and a press on another chooses it -- which only
        // happens with nothing chosen, since the leaf takes the other two away while one is.
        let mut chose = None;
        if let Some(tip) = self.leaf.pressed(&pollen) {
            chose = Some((self.leaf.chosen() != Some(tip)).then_some(tip));
        }
        if self.leaf.dismissed(&pollen) {
            chose = Some(None);
        }
        if let Some(chosen) = chose {
            self.leaf.choose(grove, chosen);
            // What the leaf chose, which is nothing where it has not arrived yet to choose from.
            self.entering = None;
            match self.leaf.chosen() {
                Some(SHOWCASE) => self.showcase.opened(grove),
                // The cut is the way in: the page comes up once the leaf has gathered to the tip.
                Some(HERBARIUM) => {
                    self.entering = Some(grove.timer(Timing::ms(Mosaic::CHANGE_MS)));
                }
                _ => {}
            }
        }
        self.leaf.frame(grove, &pollen);
        self.links.frame(grove, &pollen);
        // What a press on the controls changed, brought to the controls where it stands beside
        // them: a theme from its top.
        if self.showcase.frame(grove, &pollen) {
            self.leaf.follow(grove, ScrollTo::start());
        }
    }
}
