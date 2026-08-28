//! Does spawning a `Text` and hiding it in the same frame leave its render group without an
//! atlas?
//!
//! A text's render group is created per entity, and its `TextureAtlas` is installed only when a
//! `ResolvedFontSize` packet arrives -- which is differential, so it is sent when the value
//! *changes*. A `Text` also subscribes `Visibility::push_remove_packet::<Text>`, so hiding one is
//! itself a renderer event. This walks the three orderings an app can produce and prints which
//! one it is about to run, so a panic names its own case.
//!
//! Run with `cargo run --example spawn_hide_text -p foliage`.

use foliage::{
    Moss, Forest, Color, Elevation, Foliage, FontSize, GridExt, Leaf, Location, Root, Text,
};
use foliage::{Grows, Sprout};

struct Cases {
    frame: u32,
    grown: Vec<Leaf>,
}

/// Frames to wait between cases, so each one gets its own render pass.
const STEP: u32 = 30;

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((400, 300));
    foliage.root::<Cases>();
    foliage.photosynthesize();
}

impl Root for Cases {
    fn take_root(_forest: &mut Forest) -> Self {
        Cases {
            frame: 0,
            grown: Vec::new(),
        }
    }
    fn frame(&mut self, forest: &mut Forest, _mosses: Vec<Moss>) {
        self.frame += 1;
        match self.frame {
            // The control: spawned after boot and left alone. If this panics, the trigger is
            // "spawn a text after the first frame" and visibility has nothing to do with it.
            f if f == STEP => {
                println!("case A: spawn only");
                let leaf = grow(forest, "case A", 1);
                self.grown.push(leaf);
            }
            // Spawn and hide in the same frame.
            f if f == STEP * 2 => {
                println!("case B: spawn + hide, one frame");
                let leaf = grow(forest, "case B", 2);
                forest.visible(leaf, false);
                self.grown.push(leaf);
            }
            // The sequence a pooled widget produces: spawned, parked, then immediately shown
            // because it turned out to be needed this very frame.
            f if f == STEP * 3 => {
                println!("case C: spawn + hide + show, one frame");
                let leaf = grow(forest, "case C", 3);
                forest.visible(leaf, false);
                forest.visible(leaf, true);
                self.grown.push(leaf);
                // Case C is the known panic. Skip it to reach D and E: run with
                // `SKIP_C=1 cargo run --example spawn_hide_text -p foliage`.
                if std::env::var("SKIP_C").is_ok() {
                    forest.prune(leaf);
                }
            }
            // Case D and E act on a text that already has a render group, to separate "spawned
            // this frame" from "hidden and shown at all".
            f if f == STEP * 4 => {
                println!("case D: grow, to be toggled later");
                let leaf = grow(forest, "case D", 4);
                self.grown.push(leaf);
            }
            // Hide and show a text that has been drawing for 30 frames, in one frame.
            f if f == STEP * 5 => {
                println!("case D: hide + show, one frame, existing group");
                let leaf = self.grown[3];
                forest.visible(leaf, false);
                forest.visible(leaf, true);
            }
            // The same, spread over two frames: hidden here...
            f if f == STEP * 6 => {
                println!("case E: hide");
                forest.visible(self.grown[3], false);
            }
            // ...shown again here.
            f if f == STEP * 7 => {
                println!("case E: show");
                forest.visible(self.grown[3], true);
            }
            f if f == STEP * 8 => println!("all cases survived"),
            _ => {}
        }
    }
}

fn grow(forest: &mut Forest, what: &str, row: i32) -> Leaf {
    forest.leaf(
        Text::new(what)
            .size(FontSize::new(24))
            .color(Color::gray(200))
            .at(Location::new().xs(
                20.px().as_left().with(90.pct().as_right()),
                (20 + row * 40).px().as_top().with(30.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    )
}
