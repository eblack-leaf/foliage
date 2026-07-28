use foliage::{
    Anchor, Animation, Color, ConfigurationDescriptor, Ease, EcsExtension, Elevation, Entity,
    FontSize, Grid, GridExt, HorizontalAlignment, Leaf, Location, OnEnd, Opacity, Panel, Query,
    Rounding, ScrollTo, Sprout, Text, Tree, Trigger, VerticalAlignment, anchor,
};

// Six tiles in a window tall enough for two and a half of them -- the overflow is the
// whole subject, so the ratio matters more than the numbers. A tile is deliberately taller
// than the leftover strip at the bottom, guaranteeing one is always caught mid-slice at
// the boundary rather than the content happening to land on a clean edge.
const TILES: usize = 6;
const TILE_HEIGHT_PX: i32 = 46;
const TILE_GAP_PX: i32 = 10;
const CONTENT_HEIGHT_PX: i32 = TILES as i32 * TILE_HEIGHT_PX + (TILES as i32 - 1) * TILE_GAP_PX;

const WINDOW_LEFT_PCT: f32 = 18.0;
const WINDOW_WIDTH_PCT: f32 = 44.0;
const WINDOW_TOP_PCT: f32 = 22.0;
const WINDOW_HEIGHT_PX: i32 = 2 * TILE_HEIGHT_PX + TILE_GAP_PX + TILE_HEIGHT_PX / 2;
const WINDOW_COLOR: i32 = 500; // slate, same muted outline every chapter's parent uses
const WINDOW_PAD_PX: i32 = 8;

const TILE_COLOR: i32 = 400; // sky
const TILE_ROUNDING: Rounding = Rounding::Sm;

// Scroll is driven by writing `ScrollTo` -- the public door, a 0..1 fraction of whatever
// the live scrollable range currently is, never a pixel offset. It applies instantly, so
// smoothness here is a matter of step size: small enough steps, close enough together,
// and the discrete writes read as a glide.
const STEP: f32 = 0.02;
const STEP_MS: u64 = 24;
const END_HOLD_MS: u64 = 900; // rest at each end before reversing

const REVEAL: u64 = 300;
const TILE_STAGGER: u64 = 70;
const START_DELAY_MS: u64 = 700; // let the tiles finish landing before anything moves

const LABEL_COLOR: i32 = 500; // slate
const LABEL_FONT_SIZE: u32 = 12;
const LABEL_WIDTH_PX: i32 = 120;
const LABEL_HEIGHT_PX: i32 = 18;
const LABEL_GAP_PX: i32 = 10;

const TAGLINE_TEXT: &str = "a view shows a window onto larger content";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400;
const TAGLINE_XS_TOP_PCT: f32 = 74.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 22.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_LEFT_PCT: f32 = 70.0;
const TAGLINE_MD_WIDTH_PCT: f32 = 22.0; // ends at 92%, flush with `window_frame`'s margin
const TAGLINE_MD_TOP_PCT: f32 = 24.0;
const TAGLINE_MD_HEIGHT_PCT: f32 = 56.0;

/// A short window over a tall stack of tiles, panning down and back without pause.
///
/// The single thing worth watching is the boundary: a tile crossing it is *sliced*, not
/// hidden. Clipping is per-pixel against the ancestor's own box (see `ash/clip.rs`), so
/// half a tile draws and the other half doesn't -- which is what makes a scrolling region
/// read as a window rather than as a list that shortens.
///
/// Nothing moves except one number. The tiles' own `Location`s never change; the window's
/// [`View`](foliage::View) offset does, and every child resolves through it. That is the
/// same offset a drag or a wheel tick would write -- this page just writes it directly
/// through [`ScrollTo`], the public request form, so the page drives itself.
pub fn build(tree: &mut Tree, slot: Entity) {
    let frame = crate::chapters::window_frame(tree, slot);

    let window = tree.branch(
        frame,
        Panel::new()
            .color(Color::slate(WINDOW_COLOR))
            .outline(2)
            .rounding(Rounding::None)
            .at(Location::new().xs(
                WINDOW_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(WINDOW_WIDTH_PCT.pct().as_width()),
                WINDOW_TOP_PCT
                    .pct()
                    .as_top()
                    .with(WINDOW_HEIGHT_PX.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            // A `Panel` carries no `Grid` of its own, and `content` below positions itself
            // against this one -- so it needs one even though nothing addresses a cell.
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );

    // `content` is taller than `window`, which is the entire mechanism: `extent_check`
    // grows the window's own `View::extent` to cover its children, and the difference
    // between that and the window's own box is exactly how far it can scroll.
    let content = tree.branch(
        window,
        Leaf::sprout()
            .at(Location::new().xs(
                WINDOW_PAD_PX
                    .px()
                    .as_left()
                    .with(100.pct().as_right().adjust(-WINDOW_PAD_PX)),
                WINDOW_PAD_PX
                    .px()
                    .as_top()
                    .with(CONTENT_HEIGHT_PX.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );

    let seq = tree.sequence();
    for i in 0..TILES {
        let top = i as i32 * (TILE_HEIGHT_PX + TILE_GAP_PX);
        let tile = tree.branch(
            content,
            Panel::new()
                .color(Color::sky(TILE_COLOR))
                .rounding(TILE_ROUNDING)
                .at(Location::new().xs(
                    0.px().as_left().with(100.pct().as_width()),
                    top.px().as_top().with(TILE_HEIGHT_PX.px().as_height()),
                ))
                .elevate(Elevation::up(2))
                .with(Opacity::new(0.0)),
        );
        let start = i as u64 * TILE_STAGGER;
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(tile)
                .during(seq)
                .start(start)
                .finish(start + REVEAL)
                .eased(Ease::Linear),
        );
    }

    // Anchored to the window's own bottom edge -- the boundary is what it names, and the
    // boundary is the one thing on this page that never moves.
    let clip_label = annotation(
        tree,
        frame,
        window,
        "clip",
        anchor()
            .bottom()
            .as_top()
            .adjust(LABEL_GAP_PX)
            .with(LABEL_HEIGHT_PX.px().as_height()),
    );
    // Anchored above instead, so the two labels bracket the window rather than stacking.
    let offset_label = annotation(
        tree,
        frame,
        window,
        "offset",
        anchor()
            .top()
            .as_bottom()
            .adjust(-LABEL_GAP_PX - LABEL_HEIGHT_PX)
            .with(LABEL_HEIGHT_PX.px().as_height()),
    );
    for label in [clip_label, offset_label] {
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(label)
                .during(seq)
                .start(TILES as u64 * TILE_STAGGER)
                .finish(TILES as u64 * TILE_STAGGER + REVEAL)
                .eased(Ease::Linear),
        );
    }

    let tagline = tree.branch(
        frame,
        Text::new(TAGLINE_TEXT)
            .size(FontSize::new(TAGLINE_FONT_SIZE))
            .color(Color::slate(TAGLINE_COLOR))
            .at(Location::new()
                .xs(
                    TAGLINE_XS_LEFT_PCT
                        .pct()
                        .as_left()
                        .with(TAGLINE_XS_WIDTH_PCT.pct().as_width()),
                    TAGLINE_XS_TOP_PCT
                        .pct()
                        .as_top()
                        .with(TAGLINE_XS_HEIGHT_PCT.pct().as_height()),
                )
                .md(
                    TAGLINE_MD_LEFT_PCT
                        .pct()
                        .as_left()
                        .with(TAGLINE_MD_WIDTH_PCT.pct().as_width()),
                    TAGLINE_MD_TOP_PCT
                        .pct()
                        .as_top()
                        .with(TAGLINE_MD_HEIGHT_PCT.pct().as_height()),
                ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(tagline)
            .during(seq)
            .start(0)
            .finish(REVEAL)
            .eased(Ease::Linear),
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        tree.timer(START_DELAY_MS, move |_: Trigger<OnEnd>, mut tree: Tree| {
            step(&mut tree, window, 0.0, true);
        });
    });
}

fn annotation(
    tree: &mut Tree,
    frame: Entity,
    target: Entity,
    text: &str,
    vertical: ConfigurationDescriptor,
) -> Entity {
    tree.branch(
        frame,
        Text::new(text)
            .size(FontSize::new(LABEL_FONT_SIZE))
            .color(Color::slate(LABEL_COLOR))
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .with(LABEL_WIDTH_PX.px().as_width()),
                vertical,
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Anchor::new(target),
                Opacity::new(0.0),
            )),
    )
}

// One `ScrollTo` write per tick, reversing at each end. A chain of timers rather than an
// `Animation`: `ScrollTo` is a one-shot request, not an interpolable value, so there is
// nothing for a tween to hold between frames -- the position lives in the view's own
// offset, and each write asks for the next one.
//
// Same guard `text.rs`/`breakpoints.rs` use: `write_to` inserts by entity id with no
// existence check, and this chain can still be pending after the page is gone.
fn step(tree: &mut Tree, window: Entity, at: f32, descending: bool) {
    let next = if descending { at + STEP } else { at - STEP };
    let (next, descending, hold) = if next >= 1.0 {
        (1.0, false, true)
    } else if next <= 0.0 {
        (0.0, true, true)
    } else {
        (next, descending, false)
    };
    let delay = if hold { END_HOLD_MS } else { STEP_MS };
    tree.timer(
        delay,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            if !existing.contains(window) {
                return;
            }
            tree.write_to(window, ScrollTo::y(next));
            step(&mut tree, window, next, descending);
        },
    );
}
