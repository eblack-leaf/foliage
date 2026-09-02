//! The app: what it holds between frames, and what it does in one.

use core::time::Duration;

use foliage::{Color, Grove, Grow, Leaf, Palette, Pollen, Root, Scheme};

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
    entries: Vec<Leaf>,
    marker: Leaf,
    notice: Notice,
    selected: usize,
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
        let Shell {
            entries,
            marker,
            notice,
        } = shell::grow(grove);
        // The scheme is the app's, and stating it is one op rather than a value threaded through
        // every element that reads a role.
        grove.repaint(Scheme::new().set(Palette::Accent, Color::rgb(0.42, 0.68, 0.96)));
        grove.color(entries[0], Palette::Accent);
        Self {
            entries,
            marker,
            notice: Notice::Up(notice),
            selected: 0,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        self.retire_notice(grove, &pollen);
        self.tour(grove);
    }
}

impl Site {
    /// Walks the marker down the rail, a section at a time.
    ///
    /// Driven from the clock rather than from a gesture, so there is nothing the engine can detect
    /// and the app has to ask for the next frame itself. Once the tour has been round, it stops
    /// asking and the loop is free to idle.
    fn tour(&mut self, grove: &mut Grove) {
        let step = (grove.elapsed().as_millis() / DWELL.as_millis()) as usize;
        if step >= self.entries.len() {
            return;
        }
        if step != self.selected {
            // Two writes, and extraction sends two instances: the entry that lost the fill and the
            // one that took it. Everything else on the page is compared and found unchanged.
            grove.color(self.entries[self.selected], Palette::Muted);
            grove.color(self.entries[step], Palette::Accent);
            self.selected = step;
            // An element has one anchor, and pointing it somewhere else replaces it. So the
            // marker's own placement is written once and never again -- what moves it is which
            // element it is reading, not what it says about itself.
            grove.anchor(self.marker, self.entries[step]);
        }
        grove.again();
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
