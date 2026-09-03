//! The app: what it holds between frames, and what it does in one.

use core::time::Duration;

use foliage::{Color, Grove, Grow, Leaf, Palette, Pollen, Root, Sap, Scheme, Vein};

use crate::shell::{self, Shell};

/// How long each section holds the marker before the tour moves on.
const DWELL: Duration = Duration::from_millis(1200);

/// How long the notice stays up.
const NOTICE: Duration = Duration::from_millis(3000);

/// The site.
///
/// Everything it keeps is a `Leaf` and a little state of its own. Nothing here is handed to the
/// engine, and the engine has no way to reach it.
pub(crate) struct Site {
    shell: Shell,
    notice: Notice,
    selected: usize,
    /// Whether the tour is still walking the rail on its own. The first tap ends it: the reader has
    /// said what they want to look at.
    touring: bool,
    /// How far the knob has been dragged along its track.
    travelled: f32,
    open: bool,
}

/// The notice, across the frames it takes to come down.
///
/// Three states rather than an `Option`, because pruning and being gone are a frame apart: the
/// drain takes the element down, and the tree reports it on the app's next turn.
#[derive(Copy, Clone)]
enum Notice {
    Up(Leaf),
    Going(Leaf),
    Gone,
}

impl Root for Site {
    fn take_root(grove: &mut Grove) -> Self {
        let shell = shell::grow(grove);
        // The scheme is the app's, and stating it is one op rather than a value threaded through
        // every element that reads a role.
        grove.repaint(Scheme::new().set(Palette::Accent, Color::rgb(0.42, 0.68, 0.96)));
        grove.color(shell.entries[0], Palette::Accent);
        let notice = Notice::Up(shell.notice);
        Self {
            shell,
            notice,
            selected: 0,
            touring: true,
            travelled: 0.0,
            open: false,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        self.retire_notice(grove, &pollen);
        self.take_taps(grove, &pollen);
        self.slide(grove, &pollen);
        self.mark_focus(grove, &pollen);
        self.tour(grove);
    }
}

impl Site {
    /// What the reader pressed, asked about element by element.
    ///
    /// There is no list of what happened to walk, and so no order to depend on: this app asks about
    /// the elements it owns, in the order its own logic wants the answers.
    fn take_taps(&mut self, grove: &mut Grove, pollen: &Pollen) {
        for entry in 0..self.shell.entries.len() {
            if pollen.clicked(self.shell.entries[entry]) {
                self.touring = false;
                self.select(grove, entry);
            }
        }
        if pollen.clicked(self.shell.opener) {
            self.open(grove);
        }
        if pollen.clicked(self.shell.drawer.close) {
            self.close(grove);
        }
        // Stepping focus is a verb, and this is the button that will be a Tab key when there is a
        // keyboard. Focus cycles inside the drawer because the drawer says it is a scope.
        if pollen.clicked(self.shell.drawer.advance) {
            grove.focus_next();
        }
        for field in 0..self.shell.drawer.fields.len() {
            let field = self.shell.drawer.fields[field];
            // A press moves focus nowhere on its own. An app that wants a tap to focus something
            // says so, in the one line that says it.
            if pollen.clicked(field) {
                grove.focus(field);
            }
        }
        // A card in the scrolling column. Tapping one lights it; dragging from one scrolls the
        // column and leaves it alone, and neither the card nor the column was told about the other.
        for card in 0..self.shell.cards.len() {
            let card = self.shell.cards[card];
            if pollen.clicked(card) {
                grove.color(card, Palette::Accent);
            }
            if pollen.disengaged(card) && !pollen.clicked(card) {
                grove.color(card, Palette::Raised);
            }
        }
    }

    /// Moves the knob by what the drag it is holding did this frame.
    ///
    /// The knob takes drags along one axis and no other, so this hears about a drag across and
    /// hears nothing at all about a drag down -- that one went to the column, which scrolled.
    fn slide(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let Some(drag) = pollen.dragged(self.shell.knob) else {
            return;
        };
        let Some(Sap::Section(track)) = grove.tap(self.shell.track, Vein::Drawn) else {
            return;
        };
        self.travelled =
            (self.travelled + drag.delta.x).clamp(0.0, shell::knob_room(track.width()));
        grove.at(self.shell.knob, shell::knob_at(self.travelled));
    }

    /// Draws the focus, because the engine does not.
    ///
    /// foliage reports focus and paints no mark of its own: a focused element may have no visible
    /// part at all, so there is no mark it could draw that would be right.
    fn mark_focus(&mut self, grove: &mut Grove, pollen: &Pollen) {
        for field in self.shell.drawer.fields.iter().copied() {
            if pollen.focused(field) {
                grove.color(field, Palette::Accent);
            }
            if pollen.unfocused(field) {
                grove.color(field, Palette::Muted);
            }
        }
    }

    /// Opens the drawer over the page.
    ///
    /// One write makes the page inert. A disabled element still draws and still blocks, so nothing
    /// behind the drawer can be pressed and there is no scrim to arrange -- and the drawer is not
    /// under the page, so it stays live.
    fn open(&mut self, grove: &mut Grove) {
        if self.open {
            return;
        }
        self.open = true;
        grove.visible(self.shell.drawer.sheet, true);
        grove.disable(self.shell.page);
        // Held back rather than hidden, so it reads as out of play. Opacity is a product down the
        // tree, so this is one write for the whole page -- and it is well above zero, which is what
        // keeps the page in the stack to be swallowed by rather than absent from.
        grove.opacity(self.shell.page, 0.35);
        grove.focus(self.shell.drawer.fields[0]);
    }

    fn close(&mut self, grove: &mut Grove) {
        self.open = false;
        grove.visible(self.shell.drawer.sheet, false);
        grove.enable(self.shell.page);
        grove.opacity(self.shell.page, 1.0);
        grove.unfocus();
    }

    /// Walks the marker down the rail, a section at a time, until the reader takes over.
    ///
    /// Driven from the clock rather than from a gesture, so there is nothing the engine can detect
    /// and the app has to ask for the next frame itself. Once the tour is done -- or once a tap has
    /// ended it -- it stops asking and the loop is free to idle.
    fn tour(&mut self, grove: &mut Grove) {
        if !self.touring {
            return;
        }
        let step = (grove.elapsed().as_millis() / DWELL.as_millis()) as usize;
        if step >= self.shell.entries.len() {
            self.touring = false;
            return;
        }
        if step != self.selected {
            self.select(grove, step);
        }
        grove.again();
    }

    /// Lights one entry and points the marker at it.
    fn select(&mut self, grove: &mut Grove, entry: usize) {
        if entry == self.selected {
            return;
        }
        // Two writes, and extraction sends two instances: the entry that lost the fill and the
        // one that took it. Everything else on the page is compared and found unchanged.
        grove.color(self.shell.entries[self.selected], Palette::Muted);
        grove.color(self.shell.entries[entry], Palette::Accent);
        self.selected = entry;
        // An element has one anchor, and pointing it somewhere else replaces it. So the marker's
        // own placement is written once and never again -- what moves it is which element it is
        // reading, not what it says about itself.
        grove.anchor(self.shell.marker, self.shell.entries[entry]);
    }

    /// Takes the notice down once it has been up long enough, and lets go of it once the tree
    /// says it went.
    fn retire_notice(&mut self, grove: &mut Grove, pollen: &Pollen) {
        self.notice = match self.notice {
            Notice::Up(notice) if grove.elapsed() >= NOTICE => {
                grove.prune(notice);
                Notice::Going(notice)
            }
            // Asking `Pollen` about an element this app owns is the whole of how an emission is
            // read: there is no list to walk, and so no order to accidentally depend on.
            Notice::Going(notice) if pollen.withered(notice) => Notice::Gone,
            held => held,
        };
    }
}
