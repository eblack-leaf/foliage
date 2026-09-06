//! The app: what it holds between frames, and what it does in one.

use foliage::{
    Boxed, Grove, Grow, Leaf, Location, Palette, Panel, Place, Pollen, Root, Rounding, Source, left,
    top,
};

/// The app.
///
/// Whatever it keeps is its own. Nothing here is handed to the engine, and the engine has no way to
/// reach it: the value stays on this side and touches the tree only through the [`Grove`] it is
/// lent for the length of a frame.
pub(crate) struct Site {
    /// The one thing on the page that takes a gesture.
    button: Leaf,
    /// Whether it is currently showing as pressed.
    pressed: bool,
}

impl Root for Site {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new().color(Palette::Surface));
        let button = grove.branch(
            page,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Rounding::Md)
                // What puts it in the hit test at all.
                .interactive()
                .at(Location::new().xs(
                    left(40.px()).width(120.px()),
                    top(40.px()).height(48.px()),
                )),
        );
        Site {
            button,
            pressed: false,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.button) {
            self.pressed = !self.pressed;
            let fill = match self.pressed {
                true => Palette::Accent.recede(),
                false => Palette::Accent,
            };
            grove.color(self.button, fill);
        }
    }
}
