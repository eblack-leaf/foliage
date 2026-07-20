//! The small single-value interactive family: `Button`, `Toggle`, `Checkbox`,
//! `RadioGroup`, `SegmentedControl`, `Slider`. Configs reused from
//! `application/src/portfolio/composites.rs`. Run with
//! `cargo run --example controls -p foliage`.
//!
//! `Checkbox` here has no `.check_icon(..)` -- it's genuinely optional (unlike it used to
//! be): the box's own fill-vs-outline switch already carries checked/unchecked on its own,
//! so this renders as a plain filled/outlined box with no glyph. A real check mark would
//! still need a registered `Icon::memory_sized` backed by actual `foliage_icons`-processed
//! `.icon` byte data -- out of scope for this general-composites example.

use foliage::{
    Button, Checkbox, Color, EcsExtension, Elevation, Foliage, GridExt, Location, RadioGroup,
    Rounding, SegmentedControl, Slider, Sprout, Toggle,
};

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((260, 500));

    let mut top = 20;
    let row = |height: i32, top: &mut i32| {
        let t = *top;
        *top += height + 20;
        (t, t + height)
    };

    let (t, b) = row(44, &mut top);
    foliage.world.leaf(
        Button::new()
            .text("Button")
            .rounding(Rounding::Sm)
            .colors(Color::gray(900), Color::green(500))
            .at(Location::new().xs(
                8.px().as_left().with(160.px().as_width()),
                t.px().as_top().with(b.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    let (t, b) = row(22, &mut top);
    foliage.world.leaf(
        Toggle::new()
            .on(true)
            .colors(Color::green(500), Color::gray(700), Color::gray(200))
            .at(Location::new().xs(
                8.px().as_left().with(40.px().as_width()),
                t.px().as_top().with(b.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    let (t, b) = row(24, &mut top);
    foliage.world.leaf(
        Checkbox::new()
            .on(true)
            .colors(Color::gray(600), Color::green(500), Color::gray(900))
            .at(Location::new().xs(
                8.px().as_left().with(24.px().as_width()),
                t.px().as_top().with(b.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    let (t, b) = row(96, &mut top);
    foliage.world.leaf(
        RadioGroup::new()
            .options(["One", "Two", "Three"])
            .selected(0)
            .colors(Color::green(500), Color::gray(600))
            .at(Location::new().xs(
                8.px().as_left().with(200.px().as_width()),
                t.px().as_top().with(b.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    let (t, b) = row(36, &mut top);
    foliage.world.leaf(
        SegmentedControl::new()
            .options(["A", "B", "C"])
            .selected(0)
            .colors(Color::green(500), Color::gray(600), Color::gray(900))
            .rounding(Rounding::Sm)
            .at(Location::new().xs(
                8.px().as_left().with(240.px().as_width()),
                t.px().as_top().with(b.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    let (t, b) = row(24, &mut top);
    foliage.world.leaf(
        Slider::new()
            .progress(0.4)
            .colors(Color::gray(700), Color::green(300))
            .at(Location::new().xs(
                8.px().as_left().with(240.px().as_width()),
                t.px().as_top().with(b.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.photosynthesize();
}
