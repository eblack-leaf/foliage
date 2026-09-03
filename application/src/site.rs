//! The app: what it holds between frames, and what it does in one.

use core::time::Duration;

use foliage::{Ease, Grove, Grow, Leaf, Motion, Palette, Pollen, Root, Sap, Timing, Tween, Vein};

use crate::shell::{self, Shell};

/// How long each section holds the marker before the tour moves on.
const DWELL: Duration = Duration::from_millis(1200);

/// How long the notice stays up before it starts going, in milliseconds.
const NOTICE: u64 = 3000;

/// How long the notice takes to fade out.
const FADE: u64 = 400;

/// How long the drawer takes to arrive, and to leave.
const OPENING: u64 = 260;
const CLOSING: u64 = 220;

/// The site.
///
/// Everything it keeps is a `Leaf`, a `Tween` and a little state of its own. Nothing here is handed
/// to the engine, and the engine has no way to reach it.
pub(crate) struct Site {
    shell: Shell,
    notice: Notice,
    /// The timer the notice comes down on. A timer is a tween whose value is not read: what is
    /// wanted is the report that the time is up.
    waiting: Tween,
    selected: usize,
    /// Whether the tour is still walking the rail on its own. The first tap ends it: the reader has
    /// said what they want to look at.
    touring: bool,
    /// How far the knob has been dragged along its track.
    travelled: f32,
    open: bool,
    /// The channel moving the page's ground while the drawer is over it, and where it has reached.
    dimming: Option<Tween>,
    dimmed: f32,
}

/// The notice, across the frames it takes to come down.
///
/// Four states rather than an `Option`, because each step is a frame apart from the next: the fade
/// runs, the tree reports where it landed, the drain takes the element down, and the tree reports
/// that it went.
#[derive(Copy, Clone)]
enum Notice {
    Up(Leaf),
    Fading(Leaf),
    Going(Leaf),
    Gone,
}

impl Root for Site {
    fn take_root(grove: &mut Grove) -> Self {
        let shell = shell::grow(grove);
        // The scheme is the app's, and stating it is one op rather than a value threaded through
        // every element that reads a role.
        grove.repaint(shell::scheme(0.0));
        grove.color(shell.entries[0], Palette::Accent);
        let notice = Notice::Up(shell.notice);
        let waiting = grove.timer(Timing::ms(NOTICE));
        Self {
            shell,
            notice,
            waiting,
            selected: 0,
            touring: true,
            travelled: 0.0,
            open: false,
            dimming: None,
            dimmed: 0.0,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        self.dim(grove, &pollen);
        self.retire_notice(grove, &pollen);
        self.take_taps(grove, &pollen);
        self.slide(grove, &pollen);
        self.mark_focus(grove, &pollen);
        self.settle_drawer(grove, &pollen);
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
                grove.animate(card, Motion::Palette(Palette::Accent), Timing::ms(160));
            }
            // A direct write, and it cancels whatever was still moving that fill. The card is at
            // what was written, with nothing left over to be reconciled -- so a fill being animated
            // in one place does not oblige every other place to animate it.
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
    /// One write makes the page inert, and it takes effect at once however long the fade beside it
    /// runs for -- being disabled is not a matter of degree. A disabled element still draws and
    /// still blocks, so nothing behind the drawer can be pressed and there is no scrim to arrange.
    fn open(&mut self, grove: &mut Grove) {
        if self.open {
            return;
        }
        self.open = true;
        grove.visible(self.shell.drawer.sheet, true);
        grove.animate(
            self.shell.drawer.sheet,
            Motion::Location(shell::sheet_at(true)),
            Timing::ms(OPENING).ease(Ease::Decelerate),
        );
        grove.disable(self.shell.page);
        // Held back rather than hidden, so it reads as out of play. Opacity is a product down the
        // tree, so this is one motion for the whole page -- and it stops well above zero, which is
        // what keeps the page in the stack to be swallowed by rather than absent from.
        grove.animate(
            self.shell.page,
            Motion::Opacity(0.35),
            Timing::ms(OPENING).ease(Ease::Decelerate),
        );
        self.dimming = Some(self.dim_to(grove, 1.0, OPENING));
        grove.focus(self.shell.drawer.fields[0]);
    }

    fn close(&mut self, grove: &mut Grove) {
        self.open = false;
        grove.animate(
            self.shell.drawer.sheet,
            Motion::Location(shell::sheet_at(false)),
            Timing::ms(CLOSING).ease(Ease::Accelerate),
        );
        grove.enable(self.shell.page);
        grove.animate(
            self.shell.page,
            Motion::Opacity(1.0),
            Timing::ms(CLOSING).ease(Ease::Accelerate),
        );
        self.dimming = Some(self.dim_to(grove, 0.0, CLOSING));
        grove.unfocus();
    }

    /// Takes the sheet out of the picture once it has finished leaving.
    ///
    /// There is nothing to settle: the sheet declared where it was going from the moment it was told
    /// to go there, so a landing is the hook for what happens *next* rather than a correction.
    fn settle_drawer(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if !self.open && pollen.landed(self.shell.drawer.sheet) {
            grove.visible(self.shell.drawer.sheet, false);
        }
    }

    /// Moves the ground under the page, from a channel the engine only reports.
    ///
    /// A `Scheme` is this app's value and foliage has no concept of one, so there is no `Motion`
    /// that could carry it. What the engine lends is its clock and its easing; the write stays here.
    fn dim(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let Some(dimming) = self.dimming else {
            return;
        };
        let Some(at) = pollen.tween(dimming) else {
            return;
        };
        self.dimmed = at;
        grove.repaint(shell::scheme(at));
        if pollen.finished(dimming) {
            self.dimming = None;
        }
    }

    /// Restarts the ground's channel, from wherever the last one had reached.
    ///
    /// A channel writes nothing, so no write of the app's cancels it the way one cancels a motion.
    /// Stopping it is a verb, and that is the whole difference.
    fn dim_to(&mut self, grove: &mut Grove, to: f32, millis: u64) -> Tween {
        if let Some(running) = self.dimming.take() {
            grove.stop(running);
        }
        grove.tween(self.dimmed, to, Timing::ms(millis))
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
        // Two motions, and the fills cross rather than swap. A fill is a role and a blend of two
        // roles is a colour, so this is resolved where a role becomes one -- which is also why a
        // repaint part way through moves both ends of it.
        let shift = Timing::ms(180).ease(Ease::Decelerate);
        grove.animate(
            self.shell.entries[self.selected],
            Motion::Palette(Palette::Muted),
            shift,
        );
        grove.animate(
            self.shell.entries[entry],
            Motion::Palette(Palette::Accent),
            shift,
        );
        self.selected = entry;
        // An element has one anchor, and pointing it somewhere else replaces it. So the marker's
        // own placement is written once and never again -- what moves it is which element it is
        // reading, not what it says about itself.
        grove.anchor(self.shell.marker, self.shell.entries[entry]);
    }

    /// Fades the notice out once it has been up long enough, takes it down once there is nothing
    /// left to see, and lets go of it once the tree says it went.
    fn retire_notice(&mut self, grove: &mut Grove, pollen: &Pollen) {
        self.notice = match self.notice {
            // The timer is up. A tap on a timer's report is the same shape as a tap on anything
            // else's: ask about the one you own.
            Notice::Up(notice) if pollen.finished(self.waiting) => {
                grove.animate(
                    notice,
                    Motion::Opacity(0.0),
                    Timing::ms(FADE).ease(Ease::Accelerate),
                );
                Notice::Fading(notice)
            }
            Notice::Fading(notice) if pollen.landed(notice) => {
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
