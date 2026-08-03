//! A field with a fixed control, a stage, and a live readout.
//!
//! The control sits in its own strip at the top and never moves, whatever it does to the stage
//! below it. That is the entire reason it exists: the thing being demonstrated has to be free to
//! resize and disappear, and a control that *is* that thing walks out from under the reader
//! between one tap and the next.
//!
//! The readout is two rows, read back off the tree each frame.

use foliage::{
    Bare, Canopy, Elevation, FontSize, Grid, GridExt, Grows, HorizontalAlignment, Leaf, Location,
    Panel, Rounding, Sprout, Text, VerticalAlignment,
};

use crate::site::{Grow, fade_in, role, space, type_scale};

const ROW_H: i32 = 18;
const READOUT_ROWS: usize = 2;
const READOUT_H: i32 = ROW_H * READOUT_ROWS as i32;
/// At the narrowest the column runs, a field is about 256px inside its padding, which is some
/// thirty-five characters of the label size. This is the widest a row label may be.
const LABEL_W: i32 = 104;
/// Between the label column and the value. Without it a label that filled its column ran
/// straight into the number beside it.
const LABEL_GAP: i32 = space::SM;
const CONTROL_H: i32 = 26;

/// Bottom up: the control bar, the readout above it, then the stage. Every one of these has to
/// be counted here or they land on top of each other.
const CONTROL_FROM_BOTTOM: i32 = space::MD;
const READOUT_FROM_BOTTOM: i32 = CONTROL_FROM_BOTTOM + CONTROL_H + space::SM;
const ABOVE_STAGE: i32 = space::MD;
const BELOW_STAGE: i32 = READOUT_FROM_BOTTOM + READOUT_H + space::SM;

/// What a region has to be to hold a stage of `stage_h` plus the control and the readout.
pub(crate) fn height(stage_h: (i32, i32, i32)) -> (i32, i32, i32) {
    let extra = ABOVE_STAGE + BELOW_STAGE;
    (stage_h.0 + extra, stage_h.1 + extra, stage_h.2 + extra)
}

pub(crate) struct Blueprint {
    /// Where the demo's frame goes. Carries a grid, so its contents place in percentages of it.
    pub(crate) stage: Leaf,
    /// The one thing on the board that takes a tap.
    pub(crate) control: Leaf,
    control_label: Leaf,
    values: [Leaf; READOUT_ROWS],
    /// What each row currently says. A readout is driven from `drive`, which runs every frame,
    /// and re-sending the same string sixty times a second is a write storm for no change --
    /// so a row is only written when its text actually differs.
    shown: [String; READOUT_ROWS],
}

impl Blueprint {
    pub(crate) fn grow(
        g: &mut Grow,
        region: Leaf,
        labels: [&'static str; READOUT_ROWS],
        control: &'static str,
        seq: Leaf,
        start: u64,
    ) -> Self {
        let field = g.canopy.branch(
            region,
            Panel::new()
                .color(role::surface_container())
                .rounding(Rounding::Xs)
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.px().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
                .opacity(0.0),
        );
        fade_in(g.canopy, field, seq, start);

        // Across the bottom, in the chrome tone rather than one of the palette hues the demo
        // draws with. Top-right and accent-coloured, it read as another piece of the drawing.
        let control_leaf = g.canopy.branch(
            field,
            Panel::new()
                .color(role::surface())
                .rounding(Rounding::Sm)
                .at(Location::new().xs(
                    space::MD
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::MD)),
                    CONTROL_H
                        .px()
                        .as_height()
                        .with(100.pct().as_bottom().adjust(-CONTROL_FROM_BOTTOM)),
                ))
                .elevate(Elevation::up(3))
                .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
                .interactive(),
        );
        let control_label = g.canopy.branch(
            control_leaf,
            Text::new(control)
                .size(FontSize::new(type_scale::LABEL))
                .color(role::on_surface())
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    50.pct().as_center_y().with(16.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
                // the chip is the target; a label that won the hit-test would split it in two
                .pass_through(),
        );

        let stage = g.canopy.branch(
            field,
            Bare::new()
                .at(Location::new().xs(
                    space::MD
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::MD)),
                    ABOVE_STAGE
                        .px()
                        .as_top()
                        .with(100.pct().as_bottom().adjust(-BELOW_STAGE)),
                ))
                .elevate(Elevation::up(1))
                .grid(Grid::new(1.col().gap(0), 1.row().gap(0))),
        );

        let row = |i: usize, canopy: &mut Canopy| {
            let from_bottom = READOUT_FROM_BOTTOM + (READOUT_ROWS - 1 - i) as i32 * ROW_H;
            canopy.branch(
                field,
                Text::new(labels[i])
                    .size(FontSize::new(type_scale::LABEL))
                    .color(role::on_surface_variant())
                    .at(Location::new().xs(
                        space::MD.px().as_left().with(LABEL_W.px().as_width()),
                        ROW_H
                            .px()
                            .as_height()
                            .with(100.pct().as_bottom().adjust(-from_bottom)),
                    ))
                    .elevate(Elevation::up(2))
                    .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
            );
            canopy.branch(
                field,
                Text::new("--")
                    .size(FontSize::new(type_scale::LABEL))
                    .color(role::on_surface())
                    .at(Location::new().xs(
                        (space::MD + LABEL_W + LABEL_GAP)
                            .px()
                            .as_left()
                            .with(100.pct().as_right().adjust(-space::MD)),
                        ROW_H
                            .px()
                            .as_height()
                            .with(100.pct().as_bottom().adjust(-from_bottom)),
                    ))
                    .elevate(Elevation::up(2))
                    .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
            )
        };
        let values = [row(0, g.canopy), row(1, g.canopy)];

        Self {
            stage,
            control: control_leaf,
            control_label,
            values,
            shown: [String::from("--"), String::from("--")],
        }
    }

    pub(crate) fn set(&mut self, canopy: &mut Canopy, row: usize, text: impl Into<String>) {
        let text = text.into();
        if self.shown[row] == text {
            return;
        }
        canopy.text(self.values[row], text.clone());
        self.shown[row] = text;
    }

    /// Renames the control, for a board where the next press does something different.
    pub(crate) fn label(&self, canopy: &mut Canopy, text: impl Into<String>) {
        canopy.text(self.control_label, text);
    }
}

/// One row of a reference table: the call, and what it does.
pub(crate) struct Entry {
    pub(crate) call: &'static str,
    pub(crate) gloss: &'static str,
}

const ENTRY_CALL_H: i32 = 18;
/// Three lines of the label size at the narrowest the card runs -- about thirty-five characters
/// each. A fixed box, not content-sized: the rows are laid out at fixed offsets, so a gloss that
/// grew its own box would run under the next entry's rule.
const ENTRY_GLOSS_H: i32 = 3 * 16;
const ENTRY_GAP: i32 = space::MD;
const ENTRY_H: i32 = ENTRY_GAP + ENTRY_CALL_H + ENTRY_GLOSS_H;
const TABLE_TITLE_H: i32 = 16;
const TABLE_TOP: i32 = space::MD + TABLE_TITLE_H;

pub(crate) fn reference_height(count: usize) -> i32 {
    TABLE_TOP + count as i32 * ENTRY_H + space::MD
}

/// The detail a field has no room to act out. A rule above every entry but the first, so the
/// rows read as separate things rather than one paragraph.
pub(crate) fn reference(g: &mut Grow, region: Leaf, entries: &[Entry], seq: Leaf, start: u64) {
    let card = g.canopy.branch(
        region,
        Panel::new()
            .color(role::surface_container())
            .rounding(Rounding::Xs)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.px().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .opacity(0.0),
    );
    fade_in(g.canopy, card, seq, start);

    g.canopy.branch(
        card,
        Text::new("REFERENCE")
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_surface_heading())
            .at(Location::new().xs(
                space::MD
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::MD)),
                space::MD.px().as_top().with(TABLE_TITLE_H.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
    );

    for (i, entry) in entries.iter().enumerate() {
        let top = TABLE_TOP + i as i32 * ENTRY_H;
        if i > 0 {
            g.canopy.branch(
                card,
                Panel::new()
                    .color(role::outline())
                    .rounding(Rounding::None)
                    .at(Location::new().xs(
                        space::MD
                            .px()
                            .as_left()
                            .with(100.pct().as_right().adjust(-space::MD)),
                        (top + ENTRY_GAP / 2).px().as_top().with(1.px().as_height()),
                    ))
                    .elevate(Elevation::up(2)),
            );
        }
        // The call is code and the gloss is prose about it. On one tone a step apart they read as
        // a single wrapped paragraph, so the call takes the accent the demos draw their children
        // in and the gloss stays chrome.
        g.canopy.branch(
            card,
            Text::new(entry.call)
                .size(FontSize::new(type_scale::LABEL))
                .color(role::accent())
                .at(Location::new().xs(
                    space::MD
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::MD)),
                    (top + ENTRY_GAP)
                        .px()
                        .as_top()
                        .with(ENTRY_CALL_H.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .align(HorizontalAlignment::Left, VerticalAlignment::Middle),
        );
        g.canopy.branch(
            card,
            Text::new(entry.gloss)
                .size(FontSize::new(type_scale::LABEL))
                .color(role::on_surface_variant())
                .at(Location::new().xs(
                    space::MD
                        .px()
                        .as_left()
                        .with(100.pct().as_right().adjust(-space::MD)),
                    (top + ENTRY_GAP + ENTRY_CALL_H)
                        .px()
                        .as_top()
                        .with(ENTRY_GLOSS_H.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .align(HorizontalAlignment::Left, VerticalAlignment::Top),
        );
    }
}
