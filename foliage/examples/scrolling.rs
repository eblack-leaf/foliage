//! A scrolling region, read back and driven from the frame. Run with
//! `cargo run --example scrolling -p foliage`.
//!
//! Any element with a grid is already a view -- there is nothing to opt into. Wheel or drag
//! over the column and the readout follows, sampled rather than tracked: the app never
//! accumulates a scroll position of its own, it just asks. Click either half of the strip at
//! the bottom to jump, which is the same value going the other way.

use foliage::{
    Bloom, Canopy, Color, Elevation, Foliage, FontSize, Grid, GridExt, Grows, HorizontalAlignment,
    Leaf, Location, Panel, Rounding, ScrollTo, Sprout, Text, VerticalAlignment,
};

const ROWS: usize = 24;

struct Scene {
    column: Leaf,
    readout: Leaf,
    to_top: Leaf,
    to_bottom: Leaf,
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((360, 420));

    let mut scene: Option<Scene> = None;
    foliage.define_frame(move |canopy: &mut Canopy, blooms| {
        let scene = scene.get_or_insert_with(|| grow(canopy));

        for bloom in blooms {
            if let Bloom::Clicked(leaf) = bloom {
                if leaf == scene.to_top {
                    canopy.scroll(scene.column, ScrollTo::y(0.0));
                } else if leaf == scene.to_bottom {
                    canopy.scroll(scene.column, ScrollTo::y(1.0));
                }
            }
        }

        // Sampled every frame rather than remembered. `None` while the column is still being
        // grown, which is exactly the first frame.
        if let (Some(offset), Some(progress)) = (
            canopy.scroll_offset(scene.column),
            canopy.sample(scene.column, foliage::Sap::ScrollProgress),
        ) {
            if let foliage::Sample::Pair(_, y) = progress {
                canopy.text(
                    scene.readout,
                    format!("offset {:>6.1}px   {:>3.0}%", offset.top(), y * 100.0),
                );
            }
        }
    });
    foliage.photosynthesize();
}

fn grow(canopy: &mut Canopy) -> Scene {
    let readout = canopy.leaf(
        Text::new("scroll the column")
            .size(FontSize::new(13))
            .color(Color::gray(400))
            .at(Location::new().xs(
                20.px().as_left().with(340.px().as_right()),
                12.px().as_top().with(20.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
    );

    // The column. Its grid is what makes it a view, and `overscroll(false)` traps scrolling
    // inside it rather than letting the remainder move anything behind.
    let column = canopy.leaf(
        Panel::new()
            .color(Color::gray(850))
            .rounding(Rounding::Sm)
            .at(Location::new().xs(
                20.px().as_left().with(340.px().as_right()),
                44.px().as_top().with(300.px().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::default())
            .overscroll(false),
    );
    for row in 0..ROWS {
        let top = row as i32 * 34;
        canopy.branch(
            column,
            Text::new(format!("row {:02}", row))
                .size(FontSize::new(15))
                .color(if row % 2 == 0 {
                    Color::gray(200)
                } else {
                    Color::gray(400)
                })
                .at(Location::new().xs(
                    16.px().as_left().with(300.px().as_width()),
                    top.px().as_top().with(28.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
        );
    }

    let mut jump = |left: i32, label: &str, color: Color| {
        let button = canopy.leaf(
            Panel::new()
                .color(color)
                .rounding(Rounding::Sm)
                .at(Location::new().xs(
                    left.px().as_left().with(160.px().as_width()),
                    356.px().as_top().with(40.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .grid(Grid::default())
                .interactive(),
        );
        canopy.branch(
            button,
            Text::new(label)
                .size(FontSize::new(14))
                .color(Color::gray(950))
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
                .pass_through(),
        );
        button
    };
    let to_top = jump(20, "top", Color::cyan(400));
    let to_bottom = jump(190, "bottom", Color::orange(400));

    Scene {
        column,
        readout,
        to_top,
        to_bottom,
    }
}
