//! The visual walkthrough of how `foliage_proper` builds up a composite -- one page per
//! concept, in the order you actually need to learn them: `location` first (an entity is
//! nothing without a position -- that's `location`'s own opening beat, not a separate
//! page), then `elevate` (which of two overlapping entities actually renders in front,
//! and that it's just a component you can change), then `relative` (the same `Location`
//! percentages, but against a real, visible parent instead of the invisible window
//! frame), then `grid`/`anchor` (the two ways a child's `Location` resolves), then
//! `animate`/`sequence` (motion), then `interact` (clicks), and finally `text` (font
//! size, per-character color, and the monospace grid's own pitch/kerning story). Most
//! are currently just the shared window frame -- the real infographic for each concept
//! lands per-page later.

pub mod anchor;
pub mod animate;
pub mod breakpoints;
pub mod elevate;
pub mod grid;
pub mod interact;
pub mod location;
pub mod momentum;
pub mod relative;
pub mod scroll;
pub mod sequence;
pub mod text;

use crate::toc::{CONTENT_AREA_BOTTOM_CLEARANCE_PX, CONTENT_AREA_BOTTOM_PCT, CONTENT_AREA_TOP_PX};
use foliage::{
    Color, EcsExtension, Elevation, Entity, Grid, GridExt, Location, Panel, Rounding, Sprout, Tree,
};

const WINDOW_LEFT_PCT: f32 = 8.0;
const WINDOW_RIGHT_PCT: f32 = 92.0;
const WINDOW_COLOR: i32 = 700; // slate

const DOT_SIZE_PX: i32 = 7;
const DOT_GAP_PX: i32 = 6; // between dot centers, beyond DOT_SIZE_PX
const DOT_INSET_PX: i32 = 10; // from the frame's own top-left corner

/// The one visual every chapter page shares -- a plain window frame, standing in for "the
/// screen is the only place any of this ever actually shows up." Outline only (`Panel`'s
/// `.outline(w)` draws a border, transparent interior -- confirmed straight from
/// `panel.wgsl`'s fragment shader: `weight >= 0` restricts coverage to a strip near each
/// edge instead of filling solid), plus three small corner dots (the actual "this is a
/// window" signal -- a bare rectangle just reads as a box). Deliberately muted so each
/// page's own colorful content is what actually draws the eye.
pub(crate) fn window_frame(tree: &mut Tree, slot: Entity) -> Entity {
    let frame = tree.branch(
        slot,
        Panel::new()
            .color(Color::slate(WINDOW_COLOR))
            .outline(2)
            .rounding(Rounding::None)
            .at(Location::new().xs(
                WINDOW_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(WINDOW_RIGHT_PCT.pct().as_right()),
                CONTENT_AREA_TOP_PX.px().as_top().with(
                    CONTENT_AREA_BOTTOM_PCT
                        .pct()
                        .as_bottom()
                        .adjust(-CONTENT_AREA_BOTTOM_CLEARANCE_PX),
                ),
            ))
            .elevate(Elevation::up(0))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
    for i in 0..3 {
        let left_px = DOT_INSET_PX + i * (DOT_SIZE_PX + DOT_GAP_PX);
        tree.branch(
            frame,
            Panel::new()
                .color(Color::slate(WINDOW_COLOR))
                .rounding(Rounding::Full)
                .at(Location::new().xs(
                    left_px.px().as_left().with(DOT_SIZE_PX.px().as_width()),
                    DOT_INSET_PX
                        .px()
                        .as_top()
                        .with(DOT_SIZE_PX.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
    }
    frame
}
