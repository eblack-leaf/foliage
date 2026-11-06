//! Escape, read the one way foliage leaves it to be read.
//!
//! foliage answers `Escape` itself: it takes focus away, wherever focus is, and sends the key to
//! nothing. So whatever closes on `Escape` reads it by what it leaves -- focus gone from something
//! to nothing. Two other things leave the same, and are told apart here:
//!
//! - a press on nothing that takes input, which takes focus away as a pointer comes up, in the
//!   frame the press is reported in. A key comes with no pointer.
//! - what held focus going out of reach -- disabled, hidden or pruned, as often as not by the app
//!   itself: a press armed inert while its answer is awaited, a button swapped for a confirm.
//!   foliage sweeps focus off it in the frame that happened, and it is then out of reach; after
//!   `Escape` it is still there to take focus again.

use foliage::{Grove, Leaf, Phase, Pollen, Presence, Sap, Vein};

/// Watches focus for `Escape`, for whatever closes on it: an [`Overlay`](crate::Overlay), a
/// [`Drawer`](crate::Drawer), a sheet of the app's own.
///
/// Held, and asked once a frame, because what held focus a frame ago is what says whether focus
/// leaving was `Escape`. Where it is one press that is asked about -- a tip's chip, a confirm --
/// [`took`](Self::took) answers in the frame, with nothing held.
#[derive(Debug, Default)]
pub struct Escaped {
    held: Option<Leaf>,
}

impl Escaped {
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets what held focus: for whatever closes on `Escape` as it opens, so focus it had
    /// before is not read against it.
    pub fn forget(&mut self) {
        self.held = None;
    }

    /// Whether this frame's change of focus was `Escape`. Call once a frame while what closes on
    /// it is up.
    pub fn frame(&mut self, grove: &Grove) -> bool {
        let now = grove.focused();
        let was = std::mem::replace(&mut self.held, now);
        read(was.map(|leaf| reachable(grove, leaf)), now.is_some(), lifted(grove))
    }

    /// Whether `Escape` took focus from `leaf` this frame: it lost focus, and to nothing, by no
    /// pointer and not by going out of reach. For a part that knows which of its presses holds
    /// focus, and lets something go only when that one is escaped from -- so a field it opened, which
    /// takes `Escape` for its own, is left by the key rather than closed by it.
    pub fn took(grove: &Grove, pollen: &Pollen, leaf: Leaf) -> bool {
        pollen.unfocused(leaf)
            && read(Some(reachable(grove, leaf)), grove.focused().is_some(), lifted(grove))
    }
}

/// Whether a pointer came up this frame: what a press on nothing takes focus away with.
fn lifted(grove: &Grove) -> bool {
    grove.pointer().iter().any(|pointer| pointer.phase == Phase::Up)
}

/// Whether focus leaving what held it was `Escape`: `was` is whether anything held it and, if so,
/// whether that is still in reach; `now` whether anything holds it; `lifted` whether a pointer came
/// up this frame.
fn read(was: Option<bool>, now: bool, lifted: bool) -> bool {
    was == Some(true) && !now && !lifted
}

/// Whether `leaf` could hold focus again: live, and neither it nor anything above it hidden or
/// disabled.
fn reachable(grove: &Grove, leaf: Leaf) -> bool {
    let mut at = Some(leaf);
    while let Some(leaf) = at {
        if grove.presence(leaf) != Presence::Live
            || matches!(grove.tap(leaf, Vein::Visible), Some(Sap::Visible(false)))
            || matches!(grove.tap(leaf, Vein::Disabled), Some(Sap::Disabled(true)))
        {
            return false;
        }
        at = match grove.tap(leaf, Vein::Trunk) {
            Some(Sap::Leaf(trunk)) => trunk,
            _ => None,
        };
    }
    true
}

#[cfg(test)]
mod tests {
    use super::read;

    #[test]
    fn escape_is_focus_leaving_what_is_still_there_with_no_pointer_up() {
        assert!(read(Some(true), false, false));
    }

    #[test]
    fn a_press_on_nothing_is_not_escape() {
        assert!(!read(Some(true), false, true));
    }

    #[test]
    fn what_went_out_of_reach_losing_focus_is_not_escape() {
        assert!(!read(Some(false), false, false));
    }

    #[test]
    fn focus_moving_on_or_never_held_is_not_escape() {
        assert!(!read(Some(true), true, false));
        assert!(!read(None, false, false));
    }
}
