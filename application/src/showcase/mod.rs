//! The showcase tip: what foliage and lichen build, shown as the parts of five kinds of app.
//!
//! Not five apps. Each theme is a column of small widgets, each on its own labelled card -- a
//! product card, a composer, a stat tile -- arranged the way that kind of app would arrange them,
//! and each doing the little it would do when pressed. Nothing goes anywhere: a widget answers a
//! press by changing itself, which is all a showcase owes.
//!
//! The controls are the five themes, one chosen. A theme is grown the first time it is chosen and
//! kept after, hidden while another is showing -- hidden rather than faded, because a hidden subtree
//! is not content, and the column scrolls only as far as the theme that is showing reaches.

mod blog;
mod chat;
mod dashboard;
mod settings;
mod storefront;

use std::rc::Rc;

use foliage::{Boxed, Elevation, Font, Grove, Grow, Leaf, Location, Place, Pollen, Stem};
use lichen::{Chip, Press};

use crate::icons::Icons;
use crate::parts;

/// The themes, in the order the controls list them.
const THEMES: [&str; 5] = ["storefront", "llm chat", "dashboard", "settings", "blog"];

/// What the controls hold, one to a row.
pub(crate) fn column() -> Vec<&'static str> {
    THEMES.to_vec()
}

/// One theme, grown.
enum Theme {
    Storefront(storefront::Storefront),
    Chat(chat::Chat),
    Dashboard(dashboard::Dashboard),
    Settings(settings::Settings),
    Blog(blog::Blog),
}

impl Theme {
    /// Grows theme `n` into `page`.
    fn grow(n: usize, grove: &mut Grove, page: Leaf, icons: &Icons, italic: Font) -> Self {
        match n {
            0 => Theme::Storefront(storefront::Storefront::grow(grove, page, icons, italic)),
            1 => Theme::Chat(chat::Chat::grow(grove, page, icons, italic)),
            2 => Theme::Dashboard(dashboard::Dashboard::grow(grove, page, icons, italic)),
            3 => Theme::Settings(settings::Settings::grow(grove, page, icons, italic)),
            _ => Theme::Blog(blog::Blog::grow(grove, page, icons, italic)),
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        match self {
            Theme::Storefront(theme) => theme.frame(grove, pollen),
            Theme::Chat(theme) => theme.frame(grove, pollen),
            Theme::Dashboard(theme) => theme.frame(grove, pollen),
            Theme::Settings(theme) => theme.frame(grove, pollen),
            Theme::Blog(theme) => theme.frame(grove, pollen),
        }
    }
}

/// The showcase tip, grown, and which theme is showing.
pub(crate) struct Showcase {
    icons: Rc<Icons>,
    italic: Font,
    details: Leaf,
    tabs: Vec<Chip>,
    /// Each theme's page, and the theme on it, once it has been chosen.
    themes: Vec<Option<(Leaf, Theme)>>,
    chosen: usize,
}

impl Showcase {
    pub(crate) fn grow(
        grove: &mut Grove,
        controls: Leaf,
        details: Leaf,
        icons: Rc<Icons>,
        italic: Font,
    ) -> Self {
        let marks = [
            icons.shopping_cart,
            icons.message_square,
            icons.activity,
            icons.user,
            icons.feather,
        ];
        let presses: Vec<_> = marks.into_iter().zip(THEMES).collect();
        let mut tabs = parts::chips(grove, controls, &presses);
        tabs[0].arm(grove, Press::Chosen);
        Self {
            icons,
            italic,
            details,
            tabs,
            themes: (0..THEMES.len()).map(|_| None).collect(),
            chosen: 0,
        }
    }

    /// The tip was chosen: the theme showing is grown, if it never was.
    pub(crate) fn opened(&mut self, grove: &mut Grove) {
        self.show(grove, self.chosen);
    }

    /// Carries the showcase for a frame. Whether a theme was chosen.
    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> bool {
        let mut chose = false;
        if let Some(n) = self.tabs.iter().position(|tab| tab.pressed(pollen))
            && n != self.chosen
        {
            chose = true;
            if let Some((page, _)) = &self.themes[self.chosen] {
                grove.visible(*page, false);
            }
            self.tabs[self.chosen].arm(grove, Press::Rest);
            self.tabs[n].arm(grove, Press::Chosen);
            self.chosen = n;
            self.show(grove, n);
        }
        for (_, theme) in self.themes.iter_mut().flatten() {
            theme.frame(grove, pollen);
        }
        chose
    }

    /// Shows theme `n`, growing it first if it has never been shown.
    fn show(&mut self, grove: &mut Grove, n: usize) {
        match &self.themes[n] {
            Some((page, _)) => grove.visible(*page, true),
            None => {
                let page = grove.branch(
                    self.details,
                    Stem::new()
                        .at(Location::new())
                        .elevate(Elevation::up(1))
                        .font_size(lichen::caption())
                        .intangible(),
                );
                let theme = Theme::grow(n, grove, page, &self.icons, self.italic);
                self.themes[n] = Some((page, theme));
            }
        }
    }
}
