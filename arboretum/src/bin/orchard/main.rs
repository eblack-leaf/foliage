//! orchard -- *bits & bytes*, a shop for digital goods, sold by the byte.
//!
//! One long page under a bar: a band running past, the window at the front, and four aisles of
//! labels. Over it, when a good is opened, its detail; from the side, the bag; and over everything,
//! the till -- checkout, then the receipt, sealed. Every good is weighed to the byte and shown as
//! the label it is sold under, and everything on the page is printed in as a label is: rules drawn
//! across, words typed at the head's pace, pictures fed through from the top.

#[path = "../../../assets/gen/orchard/icons.rs"]
#[allow(dead_code)]
mod icons;

mod api;
mod art;
mod bag;
mod card;
mod cart;
mod catalog;
mod colophon;
mod detail;
mod flight;
mod front;
mod grid;
mod label;
mod theme;
mod till;

use foliage::{
    Area, Axes, Boxed, Color, Ease, Elevation, Grove, Grow, Leaf, Location, Motion, Palette, Panel,
    Place, Pollen, Root, Sap, ScrollTo, Stem, Timing, Vein,
};

use lichen::Hover;
use arboretum::sprout::{Print, Shade};

use crate::api::Receipt;
use crate::bag::Bag;
use crate::cart::Cart;
use crate::colophon::Colophon;
use crate::detail::Detail;
use crate::flight::Flight;
use crate::front::{Bar, Front};
use crate::grid::Grid;
use crate::icons::Icons;
use crate::theme::{BAR_NARROW, BAR_WIDE, Faces};
use crate::till::Till;

/// How many steps of painting the shop takes a frame, across every picture.
const BUDGET: u32 = 3;

struct Orchard {
    page: Leaf,
    hover: Hover,
    print: Print,
    bar: Bar,
    front: Front,
    grid: Grid,
    colophon: Colophon,
    detail: Detail,
    bag: Bag,
    till: Till,
    flight: Flight,
    cart: Cart,
    /// Every order placed, the latest last: the library.
    receipts: Vec<Receipt>,
    /// What the drop in flight is carrying to the bag's tally.
    landing: Option<u32>,
}

impl Root for Orchard {
    fn take_root(grove: &mut Grove) -> Self {
        grove.repaint(theme::scheme());
        let icons = grove.marks::<Icons>();
        let faces = Faces {
            roman: grove.font(include_bytes!(
                "../../../assets/fonts/xanh-mono/XanhMono-Regular.ttf"
            )),
            italic: grove.font(include_bytes!(
                "../../../assets/fonts/xanh-mono/XanhMono-Italic.ttf"
            )),
            major: grove.font(include_bytes!(
                "../../../assets/fonts/major-mono-display/MajorMonoDisplay-Regular.ttf"
            )),
            plex: grove.font(include_bytes!(
                "../../../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf"
            )),
        };
        let window = grove.plant(Panel::new().color(Palette::Surface));
        let shop = grove.branch(window, Stem::new().intangible().at(Location::new()));
        let page = grove.branch(
            shop,
            Stem::new()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .scrolls(Axes::Vertical),
        );
        let shadow = Shade::plate(grove, Color::rgba(0.11, 0.08, 0.06, 0.20));
        let mut hover = Hover::new(grove);
        let bar = Bar::grow(grove, shop, &faces, &icons);
        let front = Front::grow(grove, page, &faces, &icons, shadow);
        hover.watch(front.featured_leaf());
        let grid = Grid::grow(
            grove,
            page,
            front.tail(),
            &faces,
            &icons,
            &mut hover,
            shadow,
        );
        let colophon = Colophon::grow(grove, page, grid.tail(), &faces, &icons);
        let detail = Detail::grow(grove, shop, page, &faces, &icons);
        let till = Till::grow(grove, shop, page, &faces, &icons);
        // The bag comes in over everything, the bar included.
        let layer = grove.branch(
            window,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(50))
                .at(Location::new()),
        );
        let bag = Bag::grow(grove, layer, shop, &faces, &icons);
        let flight = Flight::grow(grove, window);
        let mut print = Print::new();
        print.run(grove, bar.printed(), 120, 70);
        print.run(grove, front.printed(), 360, 55);
        Orchard {
            page,
            hover,
            print,
            bar,
            front,
            grid,
            colophon,
            detail,
            bag,
            till,
            flight,
            cart: Cart::new(),
            receipts: Vec::new(),
            landing: None,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        let open = !self.detail.is_open() && !self.bag.is_open() && !self.till.is_open();
        self.print.frame(grove, &pollen);
        self.hover.frame(grove, open);
        let mut budget = BUDGET;
        // The bar: the search, the bag, the library.
        let (edited, asked) = self.bar.frame(grove, &pollen);
        if edited {
            let query = self.bar.search().value(grove);
            if self.detail.is_open() {
                self.detail.close(grove);
            }
            self.till.close(grove);
            self.grid.search(grove, &query);
            self.go(grove, self.grid.leaf());
        }
        if let Some(asked) = asked {
            self.asked_front(grove, asked);
        }
        // The front and the aisles.
        let over = self.hover.is_over(self.front.featured_leaf());
        let seen = !self.detail.is_open() && !self.till.is_open();
        if let Some(asked) = self.front.frame(grove, &pollen, over, seen, &mut budget) {
            self.asked_front(grove, asked);
        }
        self.grid.arrive(grove, &mut self.print);
        self.colophon.arrive(grove, &mut self.print);
        self.colophon.frame(grove, &pollen);
        if let Some(asked) = self.grid.frame(grove, &pollen, &self.hover, &mut budget) {
            match asked {
                grid::Asked::Clear => {
                    self.bar.search().clear(grove);
                    self.grid.search(grove, "");
                }
                grid::Asked::Good(good) => self.detail.open(grove, good, &mut self.print),
            }
        }
        // A good opened.
        if let Some(detail::Asked::Add {
            good,
            licence,
            format,
            from,
        }) = self
            .detail
            .frame(grove, &pollen, &mut self.print, &mut budget)
        {
            self.add(grove, good, licence, format, from);
        }
        if self.flight.frame(grove, &pollen)
            && let Some(count) = self.landing.take()
        {
            self.bar.count(grove, count);
            self.detail.added(grove, count);
        }
        // The bag.
        match self.bag.frame(grove, &pollen, &mut self.cart, &mut budget) {
            Some(bag::Asked::Changed(count)) => {
                self.bar.count(grove, count);
                self.till.refresh(grove, &self.cart);
            }
            Some(bag::Asked::Browse) => {
                self.detail.close(grove);
                self.go(grove, self.grid.leaf());
            }
            Some(bag::Asked::Checkout) => self.checkout(grove),
            None => {}
        }
        // The till.
        match self.till.frame(grove, &pollen, &self.cart, &mut self.print) {
            Some(till::Asked::Bag) => self.bag.open(grove),
            Some(till::Asked::Paid(receipt)) => self.paid(grove, receipt),
            None => {}
        }
    }
}

impl Orchard {
    fn asked_front(&mut self, grove: &mut Grove, asked: front::Asked) {
        match asked {
            front::Asked::Aisles => self.go(grove, self.grid.leaf()),
            front::Asked::Bag => self.bag.open(grove),
            front::Asked::Good(good) => self.detail.open(grove, good, &mut self.print),
            front::Asked::Library => {
                if let Some(receipt) = self.receipts.last() {
                    self.bag.close(grove);
                    self.detail.close(grove);
                    self.till.receipt(grove, receipt, &mut self.print);
                }
            }
        }
    }

    /// To the till with what is in the bag.
    fn checkout(&mut self, grove: &mut Grove) {
        if self.cart.is_empty() {
            return;
        }
        self.bag.close(grove);
        self.detail.close(grove);
        self.till.checkout(grove, &self.cart);
    }

    /// An order went through: into the library with it, and the bag emptied.
    fn paid(&mut self, grove: &mut Grove, receipt: Receipt) {
        self.receipts.push(receipt);
        self.cart.clear();
        self.bag.empty(grove, &self.cart);
        self.bar.count(grove, 0);
        self.bar.library(grove);
    }

    /// Puts a seat of `good` in the bag, and sends a drop of wax from the press to the bag.
    fn add(
        &mut self,
        grove: &mut Grove,
        good: usize,
        licence: cart::Licence,
        format: usize,
        from: foliage::Section,
    ) {
        let n = self.cart.add(good, licence, format);
        self.bag.sync(grove, &self.cart, n);
        let to = match grove.tap(self.bar.bag(), Vein::Drawn) {
            Some(Sap::Section(bag)) => bag.center(),
            _ => from.center(),
        };
        self.flight.launch(grove, from.center(), to);
        self.landing = Some(self.cart.count());
    }

    /// Scrolls the page so `section` stands just under the bar.
    fn go(&mut self, grove: &mut Grove, section: Leaf) {
        let Some(Sap::Section(placed)) = grove.tap(section, Vein::Placed) else {
            return;
        };
        let bar = if grove.viewport().width >= foliage::Layout::LG {
            BAR_WIDE
        } else {
            BAR_NARROW
        };
        grove.animate(
            self.page,
            Motion::Scroll(ScrollTo::px((placed.top() - bar - 20.0).max(0.0)).on(Axes::Vertical)),
            Timing::ms(900).ease(Ease::Emphasis),
        );
    }
}

fn main() {
    arboretum::boot::<Orchard>("orchard", Area::new(1280.0, 820.0), |_| {});
}
