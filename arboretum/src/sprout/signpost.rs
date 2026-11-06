//! The way between the portfolio's pieces.

use foliage::{Elevation, Field, Grove, Grow, Leaf, Length, Location, Pollen, Source, left, top};
use lichen::{Chip, measure};

/// One of the portfolio's pages.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum App {
    /// The hub, which links the three.
    Hub,
    Conservatory,
    Orchard,
    Trellis,
}

impl App {
    /// The three pieces, in the order the hub shows them.
    pub const PIECES: [App; 3] = [App::Conservatory, App::Orchard, App::Trellis];

    /// What it is called in a URL, and on a signpost.
    pub fn name(self) -> &'static str {
        match self {
            App::Hub => "portfolio",
            App::Conservatory => "conservatory",
            App::Orchard => "orchard",
            App::Trellis => "trellis",
        }
    }

    /// Where it is. On the web, the absolute path on the site, which holds whether or not the
    /// address it is read from ends in a slash; natively, the published page.
    pub fn href(self) -> String {
        let base = match cfg!(target_arch = "wasm32") {
            true => crate::PORTFOLIO,
            false => crate::PUBLISHED,
        };
        match self {
            App::Hub => base.to_string(),
            app => format!("{base}{}/", app.name()),
        }
    }
}

/// A row of presses leading to the other pieces and back to the hub.
pub struct Signpost {
    ways: Vec<(App, Chip)>,
}

impl Signpost {
    /// A row under `under`, its left edge at `left_px` and its top `top_px` down, leading from
    /// `here` to the hub (marked `home`) and to each other piece (marked `away`).
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        left_px: f32,
        top_px: f32,
        here: App,
        home: Field,
        away: Field,
    ) -> Self {
        let m = measure();
        let mut from: Length = left_px.px();
        let mut ways = Vec::new();
        let destinations = std::iter::once(App::Hub).chain(App::PIECES.into_iter().filter(|app| *app != here));
        for (n, app) in destinations.enumerate() {
            let mark = if app == App::Hub { home } else { away };
            let width = Chip::width(app.name());
            // One above another on a narrow window, where three side by side would not fit.
            let below = top_px + n as f32 * (m.height + m.gap);
            let chip = Chip::grow(
                grove,
                under,
                Location::new()
                    .xs(
                        left(left_px.px()).width(width.clone()),
                        top(below.px()).height(m.height.px()),
                    )
                    .sm(
                        left(from.clone()).width(width.clone()),
                        top(top_px.px()).height(m.height.px()),
                    ),
                Elevation::up(1),
                mark,
                app.name(),
            );
            from = from + width + m.gap.px();
            ways.push((app, chip));
        }
        Self { ways }
    }

    /// How tall a signpost stands: stacked on a narrow window, and in one row on a wider one.
    pub fn heights() -> (f32, f32) {
        let m = measure();
        (3.0 * m.height + 2.0 * m.gap, m.height)
    }

    /// How wide a signpost from `here` is on a window wide enough for one row, in the chips' own
    /// letters.
    pub fn width(here: App) -> Length {
        let m = measure();
        let mut total: Length = 0.px();
        let mut first = true;
        for app in std::iter::once(App::Hub).chain(App::PIECES.into_iter().filter(|app| *app != here)) {
            if !first {
                total = total + m.gap.px();
            }
            first = false;
            total = total + Chip::width(app.name());
        }
        total
    }

    /// The chips themselves, one to a destination.
    pub fn leaves(&self) -> Vec<Leaf> {
        self.ways.iter().map(|(_, chip)| chip.leaf()).collect()
    }

    /// Goes where a pressed signpost leads.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        for (app, chip) in &self.ways {
            if chip.pressed(pollen) {
                grove.navigate(app.href());
            }
        }
    }
}
