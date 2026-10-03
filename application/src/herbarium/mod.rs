//! The herbarium: one of the suite's apps, whole, on a page of its own over the site.
//!
//! The showcase has the parts, a card each. This is what they come to when an app is made of
//! them: the credentials app's window -- a skeleton key, traced from a pen drawing and filled with
//! lichen's mosaic, a lock along its shaft, and browse, intake and readout on its tips -- with a
//! toy vault behind it in place of a sealed one on a server. It is opened from the leaf's third
//! tip, whose cut is the way in rather than a section's room, since a whole app wants the whole
//! window: the page comes up over the site with a back press at its corner, and the site's scheme
//! is put away for the app's while it is up.
//!
//! Everything in the app is lichen's [`Specimen`] and lichen's parts, with nothing of the suite
//! under it. Leaving locks the vault, the way closing the app would.

mod browse;
mod intake;
mod key;
mod layout;
mod lock;
mod readout;
mod theme;
mod vault;

use std::rc::Rc;
use std::time::Duration;

use foliage::{
    Area, Boxed, Ease, Elevation, Font, Grove, Grow, Leaf, Location, Motion, Palette, Panel, Place,
    Pollen, Source, Stem, Text, Timing, Tween, anchor, bottom, center_y, left, top,
};
use lichen::{Chip, Ramp, Silhouette, Specimen, measure};

use crate::icons::Icons;
use browse::Browse;
use intake::Intake;
use layout::Sections;
use lock::Lock;
use readout::{Asked, Readout};
use vault::Vault;

/// Which tip each section stands on, in the order the chips are given to the key.
const BROWSE: usize = 0;
const INTAKE: usize = 1;
const READOUT: usize = 2;

/// How far the back press stands in from the window's corner, and how far under it the app's room
/// starts.
const CORNER: f32 = 16.0;
const GAP: f32 = 8.0;

/// How long the page takes to come up, and to go.
const FADE_MS: u64 = 220;

/// How long a copied secret is left on the clipboard.
pub(crate) const CLEARS: Duration = Duration::from_secs(20);

/// What the page says beside the back press.
const ABOUT: &str = "herbarium · a toy vault · nothing leaves this page";

/// The herbarium's page, and the app on it once it has been opened.
pub(crate) struct Herbarium {
    icons: Rc<Icons>,
    italic: Font,
    page: Leaf,
    back: Chip,
    room: Leaf,
    app: Option<Credentials>,
    open: bool,
    /// The fade the page is going out on, after which it is hidden and the site's scheme put back.
    going: Option<Tween>,
}

impl Herbarium {
    /// Grows the page over `window`, hidden. The app is grown the first time it is opened, so its
    /// key forms in front of whoever opened it.
    pub(crate) fn grow(grove: &mut Grove, window: Leaf, icons: Rc<Icons>, italic: Font) -> Self {
        // Solid, and in front of everything the site grows, so it takes every press meant for
        // what it covers.
        let page = grove.branch(
            window,
            Panel::new()
                .color(Palette::Surface)
                .at(Location::new())
                .elevate(Elevation::up(40))
                .visible(false)
                .opacity(0.0),
        );
        let height = measure().height;
        let back = Chip::grow(
            grove,
            page,
            Location::new().xs(
                left(CORNER.px()).width(Chip::width("back")),
                top(CORNER.px()).height(height.px()),
            ),
            Elevation::up(2),
            icons.arrow_left,
            "back",
        );
        grove.branch(
            page,
            Text::new(ABOUT)
                .at(Location::new().xs(
                    left(anchor().right() + (2.0 * GAP).px()).right(100.pct() - CORNER.px()),
                    center_y(anchor().top() + (height / 2.0).px()).height(2.letters()),
                ))
                .anchored(back.leaf())
                .elevate(Elevation::up(1))
                .intangible()
                .color(lichen::INERT.ink)
                .font(italic)
                .font_size(lichen::caption()),
        );
        let room = grove.branch(
            page,
            Stem::new()
                .at(Location::new().xs(
                    left(0.px()).width(100.pct()),
                    top(Self::bar().px()).bottom(100.pct()),
                ))
                .elevate(Elevation::up(1))
                .intangible(),
        );
        Self {
            icons,
            italic,
            page,
            back,
            room,
            app: None,
            open: false,
            going: None,
        }
    }

    /// How far down the window the app's room starts: under the back press.
    fn bar() -> f32 {
        CORNER + measure().height + GAP
    }

    /// The room the app has, in a window `viewport` in size.
    fn room(viewport: Area) -> Area {
        Area::new(viewport.width, (viewport.height - Self::bar()).max(1.0))
    }

    /// Whether the page is up.
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Brings the page up over the site, in the app's scheme, growing the app if it never was.
    pub(crate) fn open(&mut self, grove: &mut Grove) {
        if self.open {
            return;
        }
        self.open = true;
        self.going = None;
        grove.repaint(theme::scheme());
        grove.visible(self.page, true);
        grove.animate(
            self.page,
            Motion::Opacity(1.0),
            Timing::ms(FADE_MS).ease(Ease::Decelerate),
        );
        let room = Self::room(grove.viewport());
        match &mut self.app {
            // What was pressed on the site since may hold focus: the lock takes it back.
            Some(app) => {
                app.fit(grove, room);
                app.lock.shown(grove);
            }
            None => {
                self.app = Some(Credentials::grow(
                    grove,
                    self.room,
                    room,
                    &self.icons,
                    self.italic,
                ));
            }
        }
    }

    /// Takes the page away: the vault is locked, and the site's scheme comes back once the page
    /// has gone.
    fn close(&mut self, grove: &mut Grove) {
        self.open = false;
        if let Some(app) = &mut self.app {
            app.shut(grove);
        }
        grove.animate(
            self.page,
            Motion::Opacity(0.0),
            Timing::ms(FADE_MS).ease(Ease::Accelerate),
        );
        self.going = Some(grove.timer(Timing::ms(FADE_MS)));
    }

    /// Carries the page for a frame: back, the window's size, and the app. Call every frame,
    /// open or not, so a page going out is hidden once it has gone. Whether back was pressed.
    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> bool {
        if let Some(going) = self.going
            && pollen.finished(going)
        {
            self.going = None;
            grove.visible(self.page, false);
            grove.repaint(crate::theme::scheme());
        }
        if !self.open {
            return false;
        }
        if self.back.pressed(pollen) {
            self.close(grove);
            return true;
        }
        if let Some(app) = &mut self.app {
            if let Some(viewport) = pollen.resized() {
                app.fit(grove, Self::room(viewport));
            }
            app.frame(grove, pollen);
        }
        false
    }
}

/// The toy credentials app: the key, the lock over it, its three sections, and the vault.
struct Credentials {
    key: Specimen,
    /// Where the key stands, and what each of its tips opens.
    sections: Sections,
    lock: Lock,
    browse: Browse,
    intake: Intake,
    readout: Readout,
    vault: Vault,
    unlocked: bool,
    clip: Clip,
}

impl Credentials {
    fn grow(grove: &mut Grove, room: Leaf, size: Area, icons: &Icons, italic: Font) -> Self {
        let scheme = theme::scheme();
        let ramp = Ramp::of(&scheme.stops(theme::SPECIMEN));
        let page = Sections::page(grove, room);
        let shape = Silhouette::new(key::KEY.outline);
        let mut key = Specimen::grow(
            grove,
            page,
            Sections::shape(size, &shape),
            &key::KEY,
            ramp,
            &[
                (icons.list, "browse"),
                (icons.plus, "intake"),
                (icons.activity, "readout"),
            ],
        );
        // Shut until the passphrase opens it: the tips wait on the lock, not only on the shape.
        key.withhold(grove, true);
        let sections = Sections::grow(grove, page, &key, size);
        let lock = Lock::grow(grove, &key, icons, italic);
        let browse = Browse::grow(
            grove,
            sections.controls(BROWSE),
            sections.details(BROWSE),
            icons,
            italic,
        );
        let intake = Intake::grow(
            grove,
            sections.controls(INTAKE),
            sections.details(INTAKE),
            icons,
            italic,
        );
        let readout = Readout::grow(
            grove,
            sections.controls(READOUT),
            sections.details(READOUT),
            icons,
            italic,
        );
        let clip = Clip::grow(grove, room);
        let mut vault = Vault::seeded();
        vault.stir(grove.elapsed().as_nanos() as u64);
        let mut app = Self {
            key,
            sections,
            lock,
            browse,
            intake,
            readout,
            vault,
            unlocked: false,
            clip,
        };
        app.redraw(grove);
        app
    }

    /// Everything drawn from the vault, after it has moved.
    fn redraw(&mut self, grove: &mut Grove) {
        self.browse.redraw(grove, &self.vault);
        self.readout.visited(grove, &self.vault.log());
    }

    /// The room is now `room` in size.
    fn fit(&mut self, grove: &mut Grove, room: Area) {
        self.sections.fit(grove, &self.key, room);
    }

    /// Shuts the vault: the tips held back, the lock put back, and every section lets go of
    /// whatever it had opened.
    fn shut(&mut self, grove: &mut Grove) {
        if !self.unlocked {
            return;
        }
        self.unlocked = false;
        self.key.withhold(grove, true);
        self.sections.show(grove, None);
        self.lock.locked(grove);
        self.browse.lock(grove, &self.vault);
        self.intake.lock(grove);
        self.readout.lock(grove);
        self.clip.now(grove);
    }

    fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if let Some(tip) = self.key.pressed(pollen) {
            let chosen = (self.key.chosen() != Some(tip)).then_some(tip);
            self.key.choose(grove, chosen);
        }
        if self.key.dismissed(pollen) {
            self.key.choose(grove, None);
        }
        self.sections.show(grove, self.key.chosen());
        self.key.frame(grove, pollen);
        self.sections.frame(grove, pollen);
        if self.lock.frame(grove, pollen) {
            self.unlocked = true;
            self.key.withhold(grove, false);
        }
        if self.unlocked {
            let did = self.browse.frame(grove, pollen, &mut self.vault);
            if let Some(secret) = did.copied {
                self.clip.hold(grove, secret);
            }
            let added = self.intake.frame(grove, pollen, &mut self.vault);
            if did.changed || added {
                self.redraw(grove);
            }
            match self.readout.frame(pollen) {
                Some(Asked::Check) => {
                    let found = self.vault.health();
                    self.readout.checked(grove, &found);
                }
                Some(Asked::Lock) => self.shut(grove),
                None => {}
            }
        }
        self.clip.frame(grove, pollen);
    }
}

/// What was put on the clipboard, and the bar along the foot of the room that says how long it
/// stays: when it has drained, the clipboard is read back, and emptied if it still holds what was
/// put there -- so something copied since is not taken away. What was copied is said by the pane
/// that copied it.
struct Clip {
    bar: Leaf,
    /// The secret and when it went on, as the engine's clock had it, while it is there.
    held: Option<(Duration, String)>,
    /// Whether the clipboard has been asked what it holds.
    asked: bool,
}

impl Clip {
    fn grow(grove: &mut Grove, room: Leaf) -> Self {
        let bar = grove.branch(
            room,
            Panel::new()
                .at(full())
                .elevate(Elevation::up(8))
                .intangible()
                .opacity(0.0)
                .color(Palette::Accent),
        );
        Self {
            bar,
            held: None,
            asked: false,
        }
    }

    /// Puts `secret` on the clipboard, and starts the bar draining.
    fn hold(&mut self, grove: &mut Grove, secret: String) {
        grove.copy(secret.clone());
        self.held = Some((grove.elapsed(), secret));
        self.asked = false;
        grove.at(self.bar, full());
        grove.opacity(self.bar, 1.0);
        grove.animate(
            self.bar,
            Motion::Location(empty()),
            Timing::ms(CLEARS.as_millis() as u64),
        );
    }

    /// Makes the time up: the clipboard is asked about on the next frame.
    fn now(&mut self, grove: &mut Grove) {
        if let Some((at, _)) = &mut self.held {
            *at = grove.elapsed().saturating_sub(CLEARS);
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let Some((at, secret)) = &self.held else {
            return;
        };
        if !self.asked && grove.elapsed().saturating_sub(*at) >= CLEARS {
            grove.paste();
            self.asked = true;
        }
        if self.asked
            && let Some(holds) = pollen.pasted()
        {
            let ours = holds == secret;
            if ours {
                grove.copy(String::new());
            }
            self.held = None;
            self.asked = false;
            grove.opacity(self.bar, 0.0);
        }
        // The answer comes in a later frame, and the time runs out between frames: there is a
        // reason for another until both have.
        grove.again();
    }
}

/// The bar, whole: along the room's foot, inside the margin.
fn full() -> Location {
    Location::new().xs(
        left(key::MARGIN.px()).right(100.pct() - key::MARGIN.px()),
        bottom(100.pct() - (key::MARGIN / 2.0).px()).height(2.px()),
    )
}

/// The bar, drained: no width, at the left.
fn empty() -> Location {
    Location::new().xs(
        left(key::MARGIN.px()).width(0.px()),
        bottom(100.pct() - (key::MARGIN / 2.0).px()).height(2.px()),
    )
}
