//! The app: what it holds between frames, and what it does in one.

use core::time::Duration;

use foliage::{
    Ease, Grove, Grow, Leaf, Motion, Palette, Pollen, Root, Sap, ScrollTo, Sequence, Timing, Tween,
    Vein,
};

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

/// How long the column takes to travel to a section the reader picked.
const JUMP: u64 = 320;

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
    /// Which cards the reader has lit.
    ///
    /// Selection is the app's and the press visual is the engine's report, and they are two things.
    /// Keeping the choice here is what lets a press put its own visual back without touching it.
    lit: Vec<bool>,
    /// Whether the last card's menu is open.
    menu: bool,
    /// Where the scrollbar's thumb was last put. Kept so the placement is written when it changes
    /// and not on every frame -- an op queued every frame is a loop that never idles, and the
    /// engine has no way to know this one would have been a no-op.
    thumb: Option<(f32, f32)>,
    open: bool,
    /// Everything the drawer's last opening or closing set running, as one name. What the sheet is
    /// waiting on is the whole group rather than its own arrival: the ground moves on a channel the
    /// engine only reports, so no single element's landing is the end of it.
    moving: Option<Sequence>,
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
        let cards = shell.cards.len();
        Self {
            shell,
            notice,
            waiting,
            selected: 0,
            touring: true,
            travelled: 0.0,
            lit: vec![false; cards],
            menu: false,
            thumb: None,
            open: false,
            moving: None,
            dimming: None,
            dimmed: 0.0,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        self.dim(grove, &pollen);
        self.retire_notice(grove, &pollen);
        self.take_taps(grove, &pollen);
        self.slide(grove, &pollen);
        self.mark_scroll(grove);
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
                self.jump(grove, entry);
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
        for field in self.shell.drawer.fields.clone() {
            // Nothing here focuses a field. A tap puts the caret where it landed and takes focus,
            // because a field declares that it does -- and an app that wanted focus somewhere else
            // would write it here and win.
            // Enter commits the form. What it means is the app's: foliage says the key was pressed
            // in that field and holds no opinion about it.
            if pollen.submitted(field) {
                self.close(grove);
            }
            // Typing is reported, and what was typed is read back rather than kept: a value you can
            // set and cannot read back is a value an app has to hold a copy of.
            if pollen.edited(field) {
                self.mark_form(grove);
            }
        }
        // A card in the scrolling column. Tapping one lights it; dragging from one scrolls the
        // column and leaves it as it found it, and neither the card nor the column was told about
        // the other.
        // A tap is the only thing that moves a card's fill. A drag that scrolled the column is not
        // a choice about the card it began on, and neither is a press that caught the column still
        // coasting -- so both leave it exactly as they found it, and neither is written here.
        for card in 0..self.shell.cards.len() {
            let leaf = self.shell.cards[card];
            if !pollen.clicked(leaf) {
                continue;
            }
            self.lit[card] = !self.lit[card];
            grove.animate(leaf, Motion::Palette(self.fill(card)), Timing::ms(160));
            // The last one opens a menu under itself. It hangs past the bottom of the column and
            // is drawn over what is below rather than being cut off there, and the column gains no
            // scroll range leading down to it -- both from the one mark on the menu.
            if card == self.shell.cards.len() - 1 {
                self.menu = self.lit[card];
                grove.visible(self.shell.menu, self.menu);
            }
        }
    }

    /// What a card is filled with when nothing is pressing it.
    fn fill(&self, card: usize) -> Palette {
        match self.lit[card] {
            true => Palette::Accent,
            false => Palette::Raised,
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

    /// Draws the scrollbar, because the engine does not.
    ///
    /// Three readings and no state: how far through its range the column is, how far its content
    /// reaches, and how much of it is on screen. A drag, a wheel notch, a coast still running and a
    /// `Motion::Scroll` all move the same one offset, so all four of them move the thumb without
    /// any of them being known about here.
    fn mark_scroll(&mut self, grove: &mut Grove) {
        let column = self.shell.column;
        let (Some(Sap::Section(seen)), Some(Sap::Area(extent)), Some(Sap::Progress(progress))) = (
            grove.tap(column, Vein::Drawn),
            grove.tap(column, Vein::Extent),
            grove.tap(column, Vein::Progress),
        ) else {
            return;
        };
        let thumb = shell::thumb(seen.height(), extent.height, progress.y);
        if self.thumb == Some(thumb) {
            return;
        }
        self.thumb = Some(thumb);
        grove.at(self.shell.thumb, shell::thumb_at(thumb.0, thumb.1));
    }

    /// Draws the focus, because the engine does not.
    ///
    /// foliage reports focus and paints no mark of its own: a focused element may have no visible
    /// part at all, so there is no mark it could draw that would be right.
    fn mark_focus(&mut self, grove: &mut Grove, pollen: &Pollen) {
        for (ground, field) in self
            .shell
            .drawer
            .grounds
            .iter()
            .copied()
            .zip(self.shell.drawer.fields.clone())
        {
            if pollen.focused(field) {
                grove.color(ground, Palette::Accent);
            }
            if pollen.unfocused(field) {
                grove.color(ground, Palette::Muted);
            }
        }
    }

    /// What the drawer's second button says, from what the form currently holds.
    ///
    /// Read back rather than kept: the value is the field's, and an app holding a second copy of it
    /// is an app with two answers to one question.
    fn mark_form(&mut self, grove: &mut Grove) {
        let filled = self.shell.drawer.fields.iter().any(|field| {
            matches!(grove.tap(*field, Vein::Text), Some(Sap::Text(value)) if !value.is_empty())
        });
        grove.text(
            self.shell.drawer.verb,
            match filled {
                true => shell::SAVE,
                false => shell::CLOSE,
            },
        );
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
        // One name for three things that have nothing in common but when they are over: a placement
        // on the sheet, an opacity on the page, and a channel driving a value foliage has no concept
        // of. None of them has to be written beside the others to be counted in.
        let moving = grove.sequence();
        self.moving = Some(moving);
        grove.visible(self.shell.drawer.sheet, true);
        grove.animate(
            self.shell.drawer.sheet,
            Motion::Location(shell::sheet_at(true)),
            Timing::ms(OPENING).ease(Ease::Decelerate).within(moving),
        );
        grove.disable(self.shell.page);
        // Held back rather than hidden, so it reads as out of play. Opacity is a product down the
        // tree, so this is one motion for the whole page -- and it stops well above zero, which is
        // what keeps the page in the stack to be swallowed by rather than absent from.
        grove.animate(
            self.shell.page,
            Motion::Opacity(0.35),
            Timing::ms(OPENING).ease(Ease::Decelerate).within(moving),
        );
        self.dimming = Some(self.dim_to(grove, 1.0, OPENING, moving));
        grove.focus(self.shell.drawer.fields[0]);
    }

    fn close(&mut self, grove: &mut Grove) {
        self.open = false;
        let moving = grove.sequence();
        self.moving = Some(moving);
        grove.animate(
            self.shell.drawer.sheet,
            Motion::Location(shell::sheet_at(false)),
            Timing::ms(CLOSING).ease(Ease::Accelerate).within(moving),
        );
        grove.enable(self.shell.page);
        grove.animate(
            self.shell.page,
            Motion::Opacity(1.0),
            Timing::ms(CLOSING).ease(Ease::Accelerate).within(moving),
        );
        self.dimming = Some(self.dim_to(grove, 0.0, CLOSING, moving));
        grove.unfocus();
    }

    /// Takes the sheet out of the picture once it has finished leaving.
    ///
    /// There is nothing to settle: the sheet declared where it was going from the moment it was told
    /// to go there, so a landing is the hook for what happens *next* rather than a correction.
    fn settle_drawer(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let Some(moving) = self.moving else {
            return;
        };
        if !pollen.sequence_finished(moving) {
            return;
        }
        self.moving = None;
        if !self.open {
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
    fn dim_to(&mut self, grove: &mut Grove, to: f32, millis: u64, within: Sequence) -> Tween {
        if let Some(running) = self.dimming.take() {
            grove.stop(running);
        }
        grove.tween(self.dimmed, to, Timing::ms(millis).within(within))
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
        // The article says something else about each section, and each says it in a different number
        // of words. Rewriting the run re-measures it in the same frame, so the card resizes and the
        // slider and the cards anchored below it all follow -- none of which is written here.
        grove.text(self.shell.prose, shell::prose(entry));
        // An element has one anchor, and pointing it somewhere else replaces it. So the marker's
        // own placement is written once and never again -- what moves it is which element it is
        // reading, not what it says about itself.
        grove.anchor(self.shell.marker, self.shell.entries[entry]);
        // The legend's dot is a shape, and a shape is three numbers -- so this is an ordinary
        // motion with no machinery behind it, passing through every side count between the two.
        grove.animate(
            self.shell.figure.legend,
            Motion::Polygon(shell::legend(entry)),
            shift,
        );
        // Which word of the caption is picked out, over the run's own index space. A range rather
        // than a second element over the top of the first: the run stays one run, one entry in the
        // stack, and one thing to lay out.
        grove.tint(self.shell.figure.caption, [shell::emphasis(entry)]);
    }

    /// Runs the column to the card that goes with a section of the rail.
    ///
    /// A destination rather than a distance: the least movement that brings the card into view, and
    /// no more -- so a card already on screen moves the column nowhere and one below it is brought
    /// just to the bottom edge. Nothing here knows how long the column is or where the card sits,
    /// and it stays right through a resize or a rewritten article because both ends are answered
    /// again every frame.
    ///
    /// Taking hold of the column part way through cancels it, because a drag is a write. The reader
    /// wins, and this app says nothing to make that happen.
    fn jump(&mut self, grove: &mut Grove, entry: usize) {
        let Some(&card) = self.shell.cards.get(entry) else {
            return;
        };
        grove.animate(
            self.shell.column,
            Motion::Scroll(ScrollTo::show(card)),
            Timing::ms(JUMP).ease(Ease::Emphasis),
        );
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
