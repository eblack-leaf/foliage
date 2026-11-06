//! conservatory -- *latent flora*, an exhibition of twelve works grown by a model that does not
//! exist, developed in front of the visitor.
//!
//! One long page on a dark, shaded wall: the foyer, the wall text, a hall of four rooms running
//! sideways, the studio, the visitors' wall and the colophon -- and over it, when a work is pressed,
//! the viewing room. Everything on it comes in out of the dark a patch at a time, the way the works
//! themselves do.

#[path = "../../../assets/gen/conservatory/icons.rs"]
#[allow(dead_code)]
mod icons;

mod api;
mod colophon;
mod develop;
mod foyer;
mod frame;
mod hall;
mod statement;
mod studio;
mod theme;
mod viewing;
mod wall;
mod works;

use foliage::{
    Area, Axes, Boxed, Ease, Elevation, Fit, Grove, Grow, Image, Leaf, Location, Motion, Origin,
    Palette, Panel, Place, Pollen, Root, Sap, ScrollTo, Stem, Timing, Vein,
};

use arboretum::sprout::{Emerge, Hover};

use crate::colophon::Colophon;
use crate::foyer::{Asked, Foyer};
use crate::hall::Hall;
use crate::icons::Icons;
use crate::statement::Statement;
use crate::studio::Studio;
use crate::viewing::{Left, Viewing};
use crate::wall::Wall;

struct Conservatory {
    page: Leaf,
    hover: Hover,
    emerge: Emerge,
    foyer: Foyer,
    statement: Statement,
    hall: Hall,
    studio: Studio,
    wall: Wall,
    colophon: Colophon,
    viewing: Viewing,
}

impl Root for Conservatory {
    fn take_root(grove: &mut Grove) -> Self {
        grove.repaint(theme::scheme());
        let icons = grove.marks::<Icons>();
        let face = grove.font(include_bytes!(
            "../../../assets/fonts/major-mono-display/MajorMonoDisplay-Regular.ttf"
        ));
        let window = grove.plant(Panel::new().color(Palette::Surface));
        // The light on the wall, which stays where it is while the page runs past under it.
        let (light, width, height) = theme::shading();
        let light = grove.pixels(light, Area::new(width as f32, height as f32));
        grove.branch(
            window,
            Image::new(light).fit(Fit::Stretch).intangible().at(Location::new()),
        );
        let page = grove.branch(
            window,
            Stem::new()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .scrolls(Axes::Vertical),
        );
        let mut hover = Hover::new(grove);
        let foyer = Foyer::grow(grove, page, face, &mut hover, &icons);
        let statement = Statement::grow(grove, page, foyer.tail(), face);
        let hall = Hall::grow(grove, page, statement.tail(), face, &mut hover, &icons);
        let studio = Studio::grow(grove, page, hall.leaf(), face, &icons);
        let wall = Wall::grow(grove, page, studio.tail(), face);
        let colophon = Colophon::grow(grove, page, wall.tail(), face, &icons);
        let viewing = Viewing::grow(grove, window, page, light, face, &icons);
        Conservatory {
            page,
            hover,
            emerge: Emerge::new(),
            foyer,
            statement,
            hall,
            studio,
            wall,
            colophon,
            viewing,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        let open = !self.viewing.is_open();
        let emerge = &mut self.emerge;
        emerge.frame(grove, &pollen);
        self.hover.frame(grove, open);
        match self.foyer.frame(grove, &pollen, &self.hover, open, emerge) {
            Some(Asked::Hall) => Self::go(grove, self.page, self.hall.leaf()),
            Some(Asked::Studio) => Self::go(grove, self.page, self.studio.leaf()),
            Some(Asked::View(work)) => {
                let picture = self.hall.picture(work).map(<[u8]>::to_vec);
                self.viewing.open(grove, work, picture.as_deref(), emerge);
            }
            None => {}
        }
        self.statement.frame(grove, emerge);
        if let Some(work) = self.hall.frame(grove, &pollen, &self.hover, open, emerge) {
            let picture = self.hall.picture(work).map(<[u8]>::to_vec);
            self.viewing.open(grove, work, picture.as_deref(), emerge);
        }
        if let Some(grown) = self.studio.frame(grove, &pollen, emerge) {
            self.wall.hang(grove, &grown.prompt, grown.picture, emerge);
        }
        self.wall.frame(grove, emerge);
        self.colophon.frame(grove, &pollen, emerge);
        if let Some(Left(work)) = self.viewing.frame(grove, &pollen, emerge) {
            self.hall.show(grove, work);
        }
    }
}

impl Conservatory {
    /// Scrolls `page` so `section` stands near the top of the window.
    fn go(grove: &mut Grove, page: Leaf, section: Leaf) {
        let Some(Sap::Section(placed)) = grove.tap(section, Vein::Placed) else {
            return;
        };
        grove.animate(
            page,
            Motion::Scroll(ScrollTo::px((placed.top() - 28.0).max(0.0)).on(Axes::Vertical)),
            Timing::ms(1100).ease(Ease::Emphasis),
        );
    }
}

/// Where a real picture of a work is read from: the site's own copy on the web, the crate's
/// directory natively.
pub fn works_origin(name: &str) -> Origin {
    #[cfg(target_family = "wasm")]
    {
        Origin::url(format!("{}conservatory/works/{name}", arboretum::PORTFOLIO))
    }
    #[cfg(not(target_family = "wasm"))]
    {
        Origin::path(format!("{}/assets/works/{name}", env!("CARGO_MANIFEST_DIR")))
    }
}

fn main() {
    arboretum::boot::<Conservatory>("conservatory", Area::new(1280.0, 800.0), |_| {});
}
