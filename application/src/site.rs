//! The page: one leaf, and the three tips on it.
//!
//! The leaf arrives the way lichen brings a mosaic in -- the outline as dashes, then the tiles
//! sweeping along the ramp they are coloured from -- and then its three tips are let in. Each opens
//! a section: where else foliage is written down, what it builds, and how it builds it.

use foliage::{
    Axes, Boxed, Elevation, Grove, Grow, Location, Palette, Panel, Place, Pollen, Root, ScrollTo,
    Stem,
};
use tracing::info;

use crate::icons::Icons;
use crate::internals::{self, Internals};
use crate::links::{self, Links};
use crate::showcase::{self, Showcase};
use crate::specimen::Specimen;
use crate::theme;

/// Which tip each section stands on, in the order the chips are given to the leaf.
const LINKS: usize = 0;
const SHOWCASE: usize = 1;
const INTERNALS: usize = 2;

/// The app.
///
/// Whatever it keeps is its own. Nothing here is handed to the engine, and the engine has no way to
/// reach it: the value stays on this side and touches the tree only through the [`Grove`] it is
/// lent for the length of a frame.
pub(crate) struct Site {
    leaf: Specimen,
    links: Links,
    showcase: Showcase,
    internals: Internals,
}

impl Root for Site {
    fn take_root(grove: &mut Grove) -> Self {
        let scheme = theme::scheme();
        grove.repaint(scheme);
        let icons = grove.marks::<Icons>();
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
                (icons.cpu, "internals"),
            ],
            &[links::column(), showcase::column(), internals::column()],
        );
        let links = Links::grow(
            grove,
            leaf.controls(LINKS),
            leaf.details(LINKS),
            &icons,
            italic,
        );
        let internals = Internals::grow(
            grove,
            leaf.controls(INTERNALS),
            leaf.details(INTERNALS),
            &icons,
            italic,
        );
        // Last, because the showcase keeps the marks: it grows each theme the first time it is
        // chosen rather than all of them up front.
        let showcase = Showcase::grow(
            grove,
            leaf.controls(SHOWCASE),
            leaf.details(SHOWCASE),
            icons,
            italic,
        );
        info!("leaf planted");
        Site {
            leaf,
            links,
            showcase,
            internals,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
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
            match self.leaf.chosen() {
                Some(SHOWCASE) => self.showcase.opened(grove),
                Some(INTERNALS) => self.internals.opened(grove),
                _ => {}
            }
        }
        self.leaf.frame(grove, &pollen);
        self.links.frame(grove, &pollen);
        // What a press on the controls changed, brought to the controls where it stands beside
        // them: a theme from its top, a system lit where it stands.
        if self.showcase.frame(grove, &pollen) {
            self.leaf.follow(grove, ScrollTo::start());
        }
        if let Some(lit) = self.internals.frame(grove, &pollen) {
            self.leaf.follow(grove, ScrollTo::show(lit));
        }
    }
}
