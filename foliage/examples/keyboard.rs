//! Keyboard emissions, which arrive whether or not anything is focused. Run with
//! `cargo run --example keyboard -p foliage`.
//!
//! Two streams for two different jobs: [`Bloom::Key`] is the key as the layout produces it,
//! which is what typed text is made of, and [`Bloom::PhysicalKey`] is the key by position
//! regardless of layout, which is what a chord bound to *where* a key sits should use. Press
//! things and both lines update.

use foliage::{
    Bloom, Canopy, Color, Elevation, Foliage, FontSize, GridExt, Grows, HorizontalAlignment, Leaf,
    Location, Modifiers, Root, Sprout, Text, VerticalAlignment,
};

struct Readout {
    logical: Leaf,
    physical: Leaf,
    typed: Leaf,
    text: String,
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((460, 200));

    foliage.root::<Readout>();
    foliage.photosynthesize();
}

impl Root for Readout {
    fn take_root(canopy: &mut Canopy) -> Self {
        grow(canopy)
    }
    fn frame(&mut self, canopy: &mut Canopy, blooms: Vec<Bloom>) {
        for bloom in blooms {
            match bloom {
                Bloom::Key { key, mods } => {
                    canopy.text(self.logical, format!("key: {key:?}{}", modifiers(mods)));
                    // The layout-produced key is the one that becomes text.
                    if let foliage::Key::Character(c) = &key {
                        self.text.push_str(c);
                    } else if matches!(key, foliage::Key::Backspace) {
                        self.text.pop();
                    }
                    canopy.text(self.typed, self.text.clone());
                }
                Bloom::PhysicalKey { key, mods } => {
                    canopy.text(
                        self.physical,
                        format!("physical: {key:?}{}", modifiers(mods)),
                    );
                }
                _ => {}
            }
        }
    }
}

fn modifiers(mods: Modifiers) -> String {
    let mut held = Vec::new();
    if mods.contains(Modifiers::SHIFT) {
        held.push("shift");
    }
    if mods.contains(Modifiers::CONTROL) {
        held.push("ctrl");
    }
    if mods.contains(Modifiers::ALT) {
        held.push("alt");
    }
    if mods.contains(Modifiers::SUPER) {
        held.push("super");
    }
    if held.is_empty() {
        String::new()
    } else {
        format!("  [{}]", held.join("+"))
    }
}

fn grow(canopy: &mut Canopy) -> Readout {
    let mut line = |top: i32, color: Color, initial: &str| {
        canopy.leaf(
            Text::new(initial)
                .size(FontSize::new(14))
                .color(color)
                .at(Location::new().xs(
                    20.px().as_left().with(440.px().as_right()),
                    top.px().as_top().with(24.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
        )
    };
    Readout {
        logical: line(30, Color::gray(300), "key: press something"),
        physical: line(64, Color::gray(500), "physical: press something"),
        typed: line(120, Color::cyan(400), ""),
        text: String::new(),
    }
}
