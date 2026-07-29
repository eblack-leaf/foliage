//! The card, and the grid that arranges a set of them.
//!
//! [`card`] is the unit: a filled surface carrying the same three parts in the same places --
//! icon, title, body -- plus a cutout badge that resolves into its own polygon. That sameness
//! is what makes several of them read as a set rather than as paragraphs that happen to have
//! backgrounds.
//!
//! [`grid`] is one arrangement of that unit, not the only one. Placement is passed *in* as a
//! `Location` rather than computed inside, so a section can drop a single card beside a
//! figure, or run three across, without this module needing to know about the case. That
//! split is also what makes the card a candidate to move into `lichen` later: a widget that
//! owns its own placement cannot be reused, and one that takes a `Location` can.

use foliage::{
    EcsExtension, Elevation, Entity, FontSize, Grid, GridExt, HorizontalAlignment, Icon, IconId,
    InteractionPropagation, Leaf, Location, Panel, Rounding, Sprout, Text, TextContentHeight, Tree,
    VerticalAlignment,
};

use crate::icons::IconHandles;
use crate::site::{BADGE_OVERHANG, Column, cutout_badge, motion, role, space, type_scale};

/// Every card is built to this height. Tall enough for a two-line title over a four-line body
/// at the narrowest column the two-up grid runs at -- fixed so rows line up, with the text
/// given room rather than clipped to fit.
///
/// Fixed height is what makes the two-up breakpoint load-bearing: the body cannot push the
/// card taller when it wraps, so a column too narrow for the text does not stretch, it
/// overflows. See [`grid`].
pub(crate) const CARD_H: i32 = 208;
/// What a caller actually lays out: the card plus the room its badge hangs into. The overhang
/// is on the top and right only, so this is one [`BADGE_OVERHANG`], not two.
pub(crate) const CELL_H: i32 = CARD_H + BADGE_OVERHANG;
const GAP: i32 = space::MD;
const BADGE: i32 = 26;
const ICON: i32 = 26;
/// Two lines at `TITLE`. One-line titles simply leave the second empty, which keeps every
/// card's body starting on the same line as its neighbour's.
const TITLE_H: i32 = 44;
const TITLE_TOP: i32 = space::MD + ICON + space::MD;
const BODY_TOP: i32 = TITLE_TOP + TITLE_H + space::SM;
/// Half the column each, less half a gap, so the gutter between columns matches the gap
/// between rows.
const HALF: f32 = 50.0;

pub(crate) struct CardSpec {
    pub(crate) title: &'static str,
    pub(crate) body: &'static str,
    pub(crate) icon: IconHandles,
    /// The badge's final shape. Varying it across a set is what keeps a grid from reading as
    /// one card repeated.
    pub(crate) sides: f32,
}

/// One card, placed at `at` inside `parent`.
///
/// `at` positions the *cell*, not the card. The card is inset from it by [`BADGE_OVERHANG`] on
/// the top and right, which is the room the badge needs to hang past the card's corner without
/// leaving the box it is clipped to. Callers lay out cells and get the overhang handled for
/// them; see [`CELL_H`] for the height to give one.
pub(crate) fn card(
    tree: &mut Tree,
    parent: Entity,
    spec: &CardSpec,
    at: Location,
    seq: Entity,
    start: u64,
) -> Entity {
    let cell = tree.branch(
        parent,
        Leaf::sprout().at(at).elevate(Elevation::up(1)).with((
            Grid::new(1.col().gap(0), 1.row().gap(0)),
            InteractionPropagation::pass_through(),
        )),
    );
    let card = tree.branch(
        cell,
        Panel::new()
            .color(role::surface_container())
            .rounding(Rounding::Xs)
            .at(Location::new().xs(
                0.pct()
                    .as_left()
                    .with(100.pct().as_right().adjust(-BADGE_OVERHANG)),
                BADGE_OVERHANG.px().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
    crate::site::fade_in(tree, card, seq, start);

    tree.branch(
        card,
        Icon::new(IconId::from(spec.icon))
            .color(role::accent())
            .at(Location::new().xs(
                space::MD.px().as_left().with(ICON.px().as_width()),
                space::MD.px().as_top().with(ICON.px().as_height()),
            ))
            .elevate(Elevation::up(2)),
    );
    tree.branch(
        card,
        Text::new(spec.title)
            .size(FontSize::new(type_scale::TITLE))
            .color(role::on_surface())
            // full width, not inset for the badge: the badge sits in the top-right corner
            // opposite the icon, and the title starts below both -- reserving room beside it
            // was costing two lines of wrap for a collision that cannot happen
            .at(Location::new().xs(
                space::MD
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::MD)),
                TITLE_TOP.px().as_top().with(TITLE_H.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Top,
                TextContentHeight(true),
            )),
    );
    tree.branch(
        card,
        Text::new(spec.body)
            .size(FontSize::new(type_scale::BODY))
            .color(role::on_surface_variant())
            .at(Location::new().xs(
                space::MD
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-space::MD)),
                BODY_TOP
                    .px()
                    .as_top()
                    .with((CARD_H - BODY_TOP - space::MD).px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Top,
                TextContentHeight(true),
                // a card's body is prose, so it takes the same slant the column's prose does
                crate::site::italic(),
            )),
    );

    cutout_badge(tree, cell, spec.sides, BADGE, seq, start);
    card
}

/// A set of cards down the column: one-up until `lg`, two-up above it.
///
/// Two-up starts at `lg` rather than `md` because a card is a *fixed* height. At the bottom of
/// `md` the column is around 370px, so half of it leaves roughly 17 characters per line, and a
/// four-line body needs six -- text ran straight out of the bottom of the card. `lg` is the
/// first step where half a column is wide enough to hold the body it was measured for.
pub(crate) fn grid(tree: &mut Tree, column: &mut Column, specs: &[CardSpec]) {
    let seq = column.sequence();
    let n = specs.len() as i32;
    let height = |rows: i32| rows * CELL_H + (rows - 1) * GAP;
    let stacked = height(n);
    let region = column.region(tree, (stacked, stacked, height((n + 1) / 2)), space::MD);

    for (i, spec) in specs.iter().enumerate() {
        let i = i as i32;
        let leading = i % 2 == 0;
        let half_gap = GAP / 2;
        let left_lg = if leading { 0.0 } else { HALF };
        let at = Location::new()
            .xs(
                0.pct().as_left().with(100.pct().as_right()),
                (i * (CELL_H + GAP))
                    .px()
                    .as_top()
                    .with(CELL_H.px().as_height()),
            )
            .lg(
                left_lg
                    .pct()
                    .as_left()
                    .adjust(if leading { 0 } else { half_gap })
                    .with((left_lg + HALF).pct().as_right().adjust(if leading {
                        -half_gap
                    } else {
                        0
                    })),
                ((i / 2) * (CELL_H + GAP))
                    .px()
                    .as_top()
                    .with(CELL_H.px().as_height()),
            );
        card(tree, region, spec, at, seq, i as u64 * motion::STAGGER);
    }
}
