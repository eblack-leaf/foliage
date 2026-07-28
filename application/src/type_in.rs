use crate::icons::IconHandles;
use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, HrefLink, Icon, InteractionListener, InteractionPropagation,
    InteractionShape, Leaf, Line, Location, OnClick, Opacity, Panel, Polygon, Sprout, Text, Tree,
    Trigger, VerticalAlignment, anchor,
};

const TEXT: &str = "foliage.rs";
const FONT_SIZE: u32 = 34;
/// Raised off dead-center to leave room below the whole type-in block (line, subtitle,
/// heptagon chain) for the "Docs" button.
const FIELD_CENTER_Y_PCT: f32 = 38.0;
/// Index (0-based) where the ".rs" extension starts -- colored differently from the rest.
const EXTENSION_START: usize = 7;
const CELL_GAP_PX: i32 = 10; // spacing between letter cells (must exceed 2*CELL_PAD)
/// One cell per character, plus one trailing cell for the cursor's resting spot past the
/// last letter.
const TOTAL_COLS: i32 = TEXT.len() as i32 + 1;

const REVEAL_SNAP: u64 = 40; // a pop, not a fade -- appears/moves near-instantly
const BLINK_INTERVAL: u64 = 500;
const BLINK_SNAP: u64 = 20;
const TRAILING_BLINKS: u64 = 16; // keeps blinking a while after typing finishes

/// A short underline draws out from directly under `field` once "foliage.rs" is fully
/// typed, both endpoints starting at `field`'s own center-x (a zero-length point) and
/// animating outward -- a fixed px spread either side, not a fraction of `field`'s own
/// width or the screen's, so it stays a modest accent rather than a full-width rule at
/// any font size or viewport.
const LINE_WEIGHT: i32 = 2;
const LINE_UNDER_GAP: i32 = 14; // px between `field`'s own bottom and the line
const LINE_HALF_WIDTH: i32 = 90; // px each endpoint travels out from center
const LINE_DRAW: u64 = 450;

/// A second, plainer type-in below the line: same per-letter opacity-pop mechanism as
/// `type_in` itself, uniformly staggered (no hand-tuned `DELAYS`, no cursor) since this is
/// a quieter subtitle, not the headline.
const SUBTITLE_TEXT: &str = "native+web ui";
const SUBTITLE_FONT_SIZE: u32 = 22;
const SUBTITLE_CELL_GAP_PX: i32 = 4;
const SUBTITLE_CELL_PAD: i32 = 2;
/// `+ 1`, same reason as the headline's own `TOTAL_COLS`: without a spare trailing
/// column, the last letter's cell lands exactly on `SUBTITLE_TOTAL_COLS.letters()`'s own
/// right edge, and `subtitle_cell_location`'s `.adjust(SUBTITLE_CELL_PAD)` then pushes it
/// past the field's own boundary -- which clipped the "i" in "ui" before this was added.
const SUBTITLE_TOTAL_COLS: i32 = SUBTITLE_TEXT.len() as i32 + 1;
const SUBTITLE_GAP_FROM_LINE: i32 = 10; // px between the line and the subtitle's own top
const SUBTITLE_REVEAL_SNAP: u64 = 60;
const SUBTITLE_LETTER_STAGGER: u64 = 140;

/// A monochromatic `orange` gradient across every letter, first to last -- interpolated
/// directly on the RGB channels between the two named endpoints, not by feeding
/// `Color::orange` a linearly-stepped `i32` (that just snaps to whichever named Tailwind
/// step is nearest, which over 13 letters collapses most of them into the same 3-4
/// buckets and reads as chunky, not a gradient).
const SUBTITLE_GRADIENT_START: i32 = 300; // first letter -- lighter
const SUBTITLE_GRADIENT_END: i32 = 700; // last letter -- darker

fn subtitle_letter_color(i: usize, ch: char) -> Color {
    if ch == '+' {
        return Color::stone(700); // a divider, not part of either word -- kept out of the gradient
    }
    let last = SUBTITLE_TEXT.chars().count() - 1;
    let t = i as f32 / last as f32;
    let start = Color::orange(SUBTITLE_GRADIENT_START);
    let end = Color::orange(SUBTITLE_GRADIENT_END);
    Color::new(
        start.r() + t * (end.r() - start.r()),
        start.g() + t * (end.g() - start.g()),
        start.b() + t * (end.b() - start.b()),
        1.0,
    )
}

/// Delay before each letter of `TEXT` lands, one per character -- asymmetric, like
/// someone hunting for the right key: mostly quick, with a couple of real hesitations
/// (before "g", and again before the period).
const DELAYS: [u64; 10] = [350, 130, 150, 420, 140, 600, 160, 480, 150, 200];

/// `N.col().as_left().with(N.col().as_right())`, the SAME index on both edges, is cell N
/// (1-indexed): `Left` is exclusive (`(N-1)*pitch`), `Right` is inclusive (`N*pitch`), so
/// matching indices give exactly one cell's span. Mismatched indices (`i`/`i+1`) double
/// the width instead -- the bug the hand-rolled version sidestepped by not using `.col()`
/// at all.
///
/// The pitch itself is `character_block('a', FONT_SIZE).a()` -- ONE reference glyph's
/// advance width, measured from `'a'` specifically. `HorizontalAlignment::Center` then
/// centers each *actual* glyph inside that exact-pitch cell; any character whose real
/// rendered width is even a pixel wider than `'a'`'s overflows symmetrically past both of
/// its own cell edges, into whichever neighbor is closest -- which reads as one letter
/// "clipping" another, with the overflow (and so the apparent gap) differing per
/// character rather than being uniform. `CELL_PAD` pads the cell wider than the raw pitch
/// on both sides (without moving its center, so column spacing stays even) so a glyph a
/// few px off from `'a'` still fits.
const CELL_PAD: i32 = 3;
fn cell_location(i: i32) -> Location {
    let n = i + 1;
    Location::new().xs(
        n.col()
            .as_left()
            .adjust(-CELL_PAD)
            .with(n.col().as_right().adjust(CELL_PAD)),
        1.row().as_top().with(1.row().as_bottom()),
    )
}

fn cursor_location(i: i32) -> Location {
    let n = i + 1;
    Location::new().xs(
        n.col().as_left().with(n.col().as_right()),
        100.pct().as_bottom().adjust(-4).with(3.px().as_height()),
    )
}

fn subtitle_cell_location(i: i32) -> Location {
    let n = i + 1;
    Location::new().xs(
        n.col()
            .as_left()
            .adjust(-SUBTITLE_CELL_PAD)
            .with(n.col().as_right().adjust(SUBTITLE_CELL_PAD)),
        1.row().as_top().with(1.row().as_bottom()),
    )
}

/// Behind the rest of THIS route's own content (`Elevation::down(1)`, under every other
/// route entity's own `up(1)`) -- but not behind the navigator, which is global chrome
/// elevated well above any route (`up(9..11)`) and stays there regardless of how far
/// down a route's own content goes. So instead of trying to out-elevate it, this whole
/// chain is sized and placed to simply never reach the navigator's own screen region:
/// every link is pinned to `field` via `Anchor` (`anchor().left()`/`.bottom()`, same
/// technique the underline and subtitle already use) and sized off `field`'s own live
/// width/height, not the screen's.
///
/// `HEPTAGON_CHAIN_LEN` muted heptagons morph in from a triangle, one after another
/// (`HEPTAGON_CHAIN_LAG` apart -- same staggered-reveal idea as `navigator.rs`'s own
/// shadow layers, `SHADOW_LAG`), each landing further along `heptagon_arc_offset`'s arc
/// (up first, then curving down-right), each smaller than the last (`HEPTAGON_SHRINK`,
/// same geometric falloff idea as `SHADOW_SCALE`) so they overlap like a card stack, and
/// each a different step along a muted green gradient (`heptagon_chain_color`, same
/// direct-RGB-interpolation technique the subtitle's own gradient uses -- picking
/// `Color::green` values by a linear step snaps to only a few named Tailwind shades and
/// looks chunky over more than 2-3 links). Every link still does its own settling
/// rotation once its morph finishes, same `HEPTAGON_SPIN_ANGLE`/`HEPTAGON_SPIN_DURATION`
/// as before.
const HEPTAGON_ROUNDING: f32 = 0.15;
const HEPTAGON_WIDTH_SCALE: f32 = 1.1; // x field's own width, before per-link shrink
const HEPTAGON_HEIGHT_SCALE: f32 = 3.2; // x field's own height, before per-link shrink -- flatter than it is wide
const HEPTAGON_X_OFFSET: i32 = 40; // px right of field's own left edge -- first link centers roughly on "fol"
const HEPTAGON_Y_OFFSET: i32 = -10; // px down from field's own bottom (negative -- above it) -- first link starts higher, over more of "foliage.rs" itself
const HEPTAGON_OPACITY: f32 = 0.08; // more muted than before
const HEPTAGON_MORPH_DURATION: u64 = 900;
const HEPTAGON_SPIN_DURATION: u64 = 45_000; // a lot longer than before (was 16s)
const HEPTAGON_SPIN_ANGLE: f32 = std::f32::consts::PI; // a half turn -- more rotation than before (was a quarter turn)

const HEPTAGON_CHAIN_LEN: usize = 7;
const HEPTAGON_SHRINK: f32 = 0.72; // each link this fraction of the previous link's size
const HEPTAGON_CHAIN_LAG: u64 = 180; // ms each link's own morph-in starts after the previous one's
const HEPTAGON_ARC_RIGHT_SPAN: i32 = 220; // px the last link drifts right of the first -- more than before
const HEPTAGON_ARC_UP_AMPLITUDE: i32 = 40; // px the arc's early hump rises above the first link
const HEPTAGON_ARC_DOWN_DRIFT: i32 = 45; // px the arc ends up below the first link, by the last one -- less than before
const HEPTAGON_GREEN_START: i32 = 200; // first link -- lighter
const HEPTAGON_GREEN_END: i32 = 700; // last link -- darker

/// `t = i / (HEPTAGON_CHAIN_LEN - 1)`, 0.0 at the first link, 1.0 at the last. A sine
/// hump (peaks mid-chain, back to 0 by the end) pulls early links up, added to a `t^2`
/// term that grows slowly at first then dominates -- pulling the tail end down past
/// zero -- so the combined curve reads as "up, then down and to the right", not a
/// straight diagonal.
fn heptagon_arc_offset(i: usize) -> (i32, i32) {
    let t = i as f32 / (HEPTAGON_CHAIN_LEN - 1) as f32;
    let dx = HEPTAGON_ARC_RIGHT_SPAN as f32 * t;
    let dy = -(HEPTAGON_ARC_UP_AMPLITUDE as f32) * (std::f32::consts::PI * t).sin()
        + HEPTAGON_ARC_DOWN_DRIFT as f32 * t * t;
    (dx as i32, dy as i32)
}

fn heptagon_chain_color(i: usize) -> Color {
    let t = i as f32 / (HEPTAGON_CHAIN_LEN - 1) as f32;
    let start = Color::green(HEPTAGON_GREEN_START);
    let end = Color::green(HEPTAGON_GREEN_END);
    Color::new(
        start.r() + t * (end.r() - start.r()),
        start.g() + t * (end.g() - start.g()),
        start.b() + t * (end.b() - start.b()),
        1.0,
    )
}

fn draw_heptagon(tree: &mut Tree, parent: Entity, field: Entity, seq: Entity, start: u64) {
    for i in 0..HEPTAGON_CHAIN_LEN {
        let (dx, dy) = heptagon_arc_offset(i);
        let shrink = HEPTAGON_SHRINK.powi(i as i32);
        let width_scale = HEPTAGON_WIDTH_SCALE * shrink;
        let height_scale = HEPTAGON_HEIGHT_SCALE * shrink;
        let morph_start = start + i as u64 * HEPTAGON_CHAIN_LAG;

        let heptagon = tree.branch(
            parent,
            Polygon::new()
                .sides(3.0)
                .rounding(0.0)
                .rotation(0.0)
                .color(heptagon_chain_color(i))
                .at(Location::new().xs(
                    anchor()
                        .left()
                        .as_center_x()
                        .adjust(HEPTAGON_X_OFFSET + dx)
                        .with((anchor().width() * width_scale).as_width()),
                    anchor()
                        .bottom()
                        .as_center_y()
                        .adjust(HEPTAGON_Y_OFFSET + dy)
                        .with((anchor().height() * height_scale).as_height()),
                ))
                .elevate(Elevation::down(1))
                .with((Opacity::new(0.0), Anchor::new(field))),
        );
        tree.animate(
            Animation::new(Opacity::new(HEPTAGON_OPACITY))
                .targeting(heptagon)
                .during(seq)
                .start(morph_start)
                .finish(morph_start + HEPTAGON_MORPH_DURATION)
                .eased(Ease::Linear),
        );
        tree.animate(
            Animation::new(Polygon {
                sides: 7.0,
                rounding: HEPTAGON_ROUNDING,
                rotation: 0.0,
            })
            .targeting(heptagon)
            .during(seq)
            .start(morph_start)
            .finish(morph_start + HEPTAGON_MORPH_DURATION)
            .eased(Ease::DECELERATE),
        );

        let spin_start = morph_start + HEPTAGON_MORPH_DURATION;
        tree.animate(
            Animation::new(Polygon {
                sides: 7.0,
                rounding: HEPTAGON_ROUNDING,
                rotation: HEPTAGON_SPIN_ANGLE,
            })
            .targeting(heptagon)
            .during(seq)
            .start(spin_start)
            .finish(spin_start + HEPTAGON_SPIN_DURATION)
            .eased(Ease::Linear),
        );
    }
}

/// Fires once "foliage.rs" is fully typed: a short underline draws out from `field`'s own
/// center (see `LINE_HALF_WIDTH`), both endpoints pinned to `field` via `Anchor` the same
/// way `navigator.rs`'s `drop_line` keeps its row -- so this reads as coming off the text
/// itself, not floating at some independent screen position. Once the line finishes, a
/// second, quieter type-in reveals `SUBTITLE_TEXT` directly under it, same `Anchor`.
/// Returns the time the subtitle's own last letter finishes revealing, so callers can
/// sequence anything that should wait for this whole effect to finish first.
fn draw_line_and_subtitle(
    tree: &mut Tree,
    parent: Entity,
    field: Entity,
    seq: Entity,
    start: u64,
) -> u64 {
    let center_x = anchor().center_x().as_x();
    let line_y = anchor().bottom().as_y().adjust(LINE_UNDER_GAP);
    let center_point = center_x.with(line_y);
    let left_point = center_x.adjust(-LINE_HALF_WIDTH).with(line_y);
    let right_point = center_x.adjust(LINE_HALF_WIDTH).with(line_y);

    let line = tree.branch(
        parent,
        Line::new(LINE_WEIGHT)
            .color(Color::stone(600))
            .at(Location::new().xs(center_point, center_point))
            .elevate(Elevation::up(1))
            .with(Anchor::new(field)),
    );
    tree.animate(
        Animation::new(Location::new().xs(left_point, right_point))
            .targeting(line)
            .during(seq)
            .start(start)
            .finish(start + LINE_DRAW)
            .eased(Ease::DECELERATE),
    );

    let subtitle_grid = Grid::new(1.letters().gap(SUBTITLE_CELL_GAP_PX), 1.letters());
    let subtitle_field = tree.branch(
        parent,
        Leaf::sprout()
            .at(Location::new().xs(
                anchor().center_x().as_center_x().with(
                    SUBTITLE_TOTAL_COLS
                        .letters()
                        .as_width()
                        .adjust((SUBTITLE_TOTAL_COLS - 1) * SUBTITLE_CELL_GAP_PX),
                ),
                anchor()
                    .bottom()
                    .as_top()
                    .adjust(LINE_UNDER_GAP + SUBTITLE_GAP_FROM_LINE)
                    .with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with((
                subtitle_grid,
                FontSize::new(SUBTITLE_FONT_SIZE),
                Anchor::new(field),
            )),
    );

    let mut letter_time = start + LINE_DRAW;
    for (i, ch) in SUBTITLE_TEXT.chars().enumerate() {
        let col = i as i32;
        let letter = tree.branch(
            subtitle_field,
            Text::new(ch.to_string())
                .size(FontSize::new(SUBTITLE_FONT_SIZE))
                .color(subtitle_letter_color(i, ch))
                .at(subtitle_cell_location(col))
                .elevate(Elevation::up(1))
                .with((HorizontalAlignment::Center, Opacity::new(0.0))),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(letter)
                .during(seq)
                .start(letter_time)
                .finish(letter_time + SUBTITLE_REVEAL_SNAP)
                .eased(Ease::Linear),
        );
        letter_time += SUBTITLE_LETTER_STAGGER;
    }
    // `letter_time` was bumped once more after the last letter's own start time above --
    // undo that to get the last letter's real start, then add its own reveal duration.
    letter_time - SUBTITLE_LETTER_STAGGER + SUBTITLE_REVEAL_SNAP
}

/// One of the two links ("docs" -> rustdoc, "book" -> the mdbook) below everything else
/// the type-in builds -- see [`LinkButton`] for what differs between them and
/// `draw_link_button`'s own body for how the pair is laid out. Restyled to match the
/// decorative heptagon chain's own visual language instead of looking like a plain
/// stock button: a morphed-in heptagon (same triangle -> `sides: 7.0` technique as
/// `draw_heptagon`) is the actual clickable target (`InteractionListener` +
/// `InteractionShape::Circle`, hand-added since this is a bare `Polygon`, not the
/// `Button` composite), a green `Icon` sits anchored to its center, and one muted stone
/// heptagon morphs in behind it at a lower elevation as a shadow -- offset left by
/// `DOCS_SHADOW_OFFSET_PX` and sharing the front heptagon's own rotation exactly (see
/// that constant's doc for why a positional shift, not a rotation mismatch, is what
/// actually shows it). The icon sits at a HIGHER elevation than the heptagon (so
/// it renders visibly on top of the fill), which would otherwise let it win the hit-test
/// and swallow clicks meant for the heptagon underneath -- `InteractionPropagation::
/// pass_through()` on the icon is what lets the click fall through to it instead. Plus a
/// separate text label, both groups `Anchor`-pinned under `field` the same way the
/// underline/subtitle/heptagons already are. `start` should be well after
/// everything else has landed (see `type_in`'s own call site) -- it showed up alongside
/// the very first letter before, which read as competing with content that hadn't
/// finished yet.
const DOCS_BTN_PX: i32 = 40;
const DOCS_HEPTA_ROUNDING: f32 = 0.15; // same softening the decorative heptagon chain uses
const DOCS_COLOR: i32 = 400; // green -- the clickable heptagon's own fill
const DOCS_SHADOW_COLOR: i32 = 500; // muted stone -- the shadow heptagon behind it
/// Px the shadow shifts left of the front heptagon -- same offset idea `navigator.rs`'s
/// own shadow layers use, just a lot smaller since this shape is a lot smaller. The
/// shadow's own rotation matches the front heptagon's final rotation exactly (no
/// rotation offset) -- the positional shift alone is enough to show it past the front
/// shape's edge, and matching rotation reads as one shape with a shifted backdrop
/// rather than two shapes spinning out of sync.
const DOCS_SHADOW_OFFSET_PX: i32 = 8;
const DOCS_SHADOW_Y_OFFSET_PX: i32 = 4; // px the shadow also sits below the front heptagon -- matches chrome.rs's own SHADOW_Y_OFFSET_PX
const DOCS_ICON_SCALE: f32 = 0.5; // fraction of the heptagon's own box the icon fills
const DOCS_LABEL_GAP_PX: i32 = 10;
const DOCS_LABEL_WIDTH_PX: i32 = 90;
const DOCS_FONT_SIZE: u32 = 22;
const DOCS_GAP_FROM_FIELD_BOTTOM: i32 = 90; // px -- clears the line + subtitle below field
const DOCS_MORPH_DURATION: u64 = 700;
const DOCS_START_GAP: u64 = 400; // ms after the subtitle finishes before the Docs link appears
const DOCS_PAIR_GAP_PX: i32 = 24; // md and up: between the two groups sitting side by side
const DOCS_ROW_GAP_PX: i32 = 20; // xs: between the two groups once they stack instead
// Real absolute URLs (scheme + host), not root-relative paths. Both are siblings of
// `docs/index.html`, which GitHub Pages serves at `https://eblack-leaf.github.io/foliage/`:
// `api.sh` writes rustdoc into `docs/api/` and `book.sh` writes the mdbook into
// `docs/book/` (`web-release.sh` runs both, since its own `rm -rf ../docs/*` clears them).
// The api link points at the crate page rather than `docs/api/` itself -- `cargo doc` emits
// no root index.html, so the directory alone would 404.
const DOCS_API_HREF: &str = "https://eblack-leaf.github.io/foliage/api/foliage/index.html";
const DOCS_BOOK_HREF: &str = "https://eblack-leaf.github.io/foliage/book/";

/// One of the two link buttons -- everything that differs between them. `slot` is the
/// position in the pair: `0` is left (md) / top (xs), `1` is right / bottom.
struct LinkButton {
    label: &'static str,
    icon: IconHandles,
    href: &'static str,
    slot: i32,
}

fn draw_link_button(
    tree: &mut Tree,
    parent: Entity,
    field: Entity,
    seq: Entity,
    start: u64,
    link: LinkButton,
) {
    let group_width = DOCS_BTN_PX + DOCS_LABEL_GAP_PX + DOCS_LABEL_WIDTH_PX;
    // md and up the two groups sit side by side, so the *pair* is what gets centered and
    // each group is offset within it. On xs that would be ~300px of button in a viewport
    // that may not have it, so they stack instead: each group centered on its own row,
    // the second dropped by one button height plus the gap.
    let pair_width = 2 * group_width + DOCS_PAIR_GAP_PX;
    let md_icon_offset = -(pair_width / 2) + link.slot * (group_width + DOCS_PAIR_GAP_PX);
    let md_label_offset = md_icon_offset + DOCS_BTN_PX + DOCS_LABEL_GAP_PX;
    let xs_icon_offset = -(group_width / 2);
    let xs_label_offset = xs_icon_offset + DOCS_BTN_PX + DOCS_LABEL_GAP_PX;
    let xs_row_drop = link.slot * (DOCS_BTN_PX + DOCS_ROW_GAP_PX);

    let left = anchor().center_x().as_left();
    let row_top = |drop: i32| {
        anchor()
            .bottom()
            .as_top()
            .adjust(DOCS_GAP_FROM_FIELD_BOTTOM + drop)
    };
    let btn_box = Location::new()
        .xs(
            left.adjust(xs_icon_offset)
                .with(DOCS_BTN_PX.px().as_width()),
            row_top(xs_row_drop).with(DOCS_BTN_PX.px().as_height()),
        )
        .md(
            left.adjust(md_icon_offset)
                .with(DOCS_BTN_PX.px().as_width()),
            row_top(0).with(DOCS_BTN_PX.px().as_height()),
        );
    let shadow_box = Location::new()
        .xs(
            left.adjust(xs_icon_offset - DOCS_SHADOW_OFFSET_PX)
                .with(DOCS_BTN_PX.px().as_width()),
            row_top(xs_row_drop + DOCS_SHADOW_Y_OFFSET_PX).with(DOCS_BTN_PX.px().as_height()),
        )
        .md(
            left.adjust(md_icon_offset - DOCS_SHADOW_OFFSET_PX)
                .with(DOCS_BTN_PX.px().as_width()),
            row_top(DOCS_SHADOW_Y_OFFSET_PX).with(DOCS_BTN_PX.px().as_height()),
        );

    let shadow = tree.branch(
        parent,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::stone(DOCS_SHADOW_COLOR))
            .at(shadow_box)
            .elevate(Elevation::up(1))
            .with((Opacity::new(0.0), Anchor::new(field))),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(shadow)
            .during(seq)
            .start(start)
            .finish(start + DOCS_MORPH_DURATION)
            .eased(Ease::Linear),
    );
    tree.animate(
        Animation::new(Polygon {
            sides: 7.0,
            rounding: DOCS_HEPTA_ROUNDING,
            rotation: 0.0,
        })
        .targeting(shadow)
        .during(seq)
        .start(start)
        .finish(start + DOCS_MORPH_DURATION)
        .eased(Ease::DECELERATE),
    );

    let link_btn = tree.branch(
        parent,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::green(DOCS_COLOR))
            .at(btn_box)
            .elevate(Elevation::up(2))
            .with((
                InteractionListener::new(),
                InteractionShape::Circle,
                Opacity::new(0.0),
                Anchor::new(field),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(link_btn)
            .during(seq)
            .start(start)
            .finish(start + DOCS_MORPH_DURATION)
            .eased(Ease::Linear),
    );
    tree.animate(
        Animation::new(Polygon {
            sides: 7.0,
            rounding: DOCS_HEPTA_ROUNDING,
            rotation: 0.0,
        })
        .targeting(link_btn)
        .during(seq)
        .start(start)
        .finish(start + DOCS_MORPH_DURATION)
        .eased(Ease::DECELERATE),
    );
    let href = link.href;
    tree.on_click(link_btn, move |_: Trigger<OnClick>, _: Tree| {
        HrefLink::new(href).navigate();
    });

    let icon = tree.branch(
        parent,
        Icon::new(link.icon)
            .color(Color::gray(950))
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .with((anchor().width() * DOCS_ICON_SCALE).as_width()),
                anchor()
                    .center_y()
                    .as_center_y()
                    .with((anchor().height() * DOCS_ICON_SCALE).as_height()),
            ))
            .elevate(Elevation::up(3))
            .with((
                Opacity::new(0.0),
                Anchor::new(link_btn),
                InteractionPropagation::pass_through(),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(icon)
            .during(seq)
            .start(start)
            .finish(start + DOCS_MORPH_DURATION)
            .eased(Ease::Linear),
    );

    let link_label = tree.branch(
        parent,
        Text::new(link.label)
            .size(FontSize::new(DOCS_FONT_SIZE))
            .color(Color::stone(500))
            .at(Location::new()
                .xs(
                    left.adjust(xs_label_offset)
                        .with(DOCS_LABEL_WIDTH_PX.px().as_width()),
                    row_top(xs_row_drop).with(DOCS_BTN_PX.px().as_height()),
                )
                .md(
                    left.adjust(md_label_offset)
                        .with(DOCS_LABEL_WIDTH_PX.px().as_width()),
                    row_top(0).with(DOCS_BTN_PX.px().as_height()),
                ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
                Anchor::new(field),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(link_label)
            .during(seq)
            .start(start)
            .finish(start + DOCS_MORPH_DURATION)
            .eased(Ease::Linear),
    );
}

/// A terminal-style type-in: a blinking `_` sits where the first letter will go, then
/// "foliage.rs" is typed out one character at a time with uneven, human timing, the
/// cursor hopping to just past each new letter as it lands. `start` is when this whole
/// effect should begin (already-settled, still-spinning content upstream of it).
pub fn type_in(tree: &mut Tree, parent: Entity, seq: Entity, start: u64) {
    // `Grid::new(1.letters().gap(N), 1.letters())` makes one column exactly one
    // character of *this entity's own* `FontSize`, with `N` logical px between cells --
    // the same mechanism `TextInput`'s field/cursor grid uses, now with a real
    // regression test behind it (see `text::monospaced::tests::col_against_a_letters_grid_...`).
    let grid = Grid::new(1.letters().gap(CELL_GAP_PX), 1.letters());

    // Sized off its own content, not a guessed screen percentage: `TOTAL_COLS.letters()`
    // is the *other* `.letters()` mechanism (a value, off this entity's own `FontSize`,
    // covered by `letters_resolves_...`) -- exactly as wide as `TOTAL_COLS` pitches, plus
    // an explicit `.adjust()` for the `TOTAL_COLS - 1` gaps the grid will actually place
    // between them. Bump `FONT_SIZE` (or make it responsive per breakpoint later) and
    // this box follows without anything here needing to be re-tuned by hand.
    let field = tree.branch(
        parent,
        Leaf::sprout()
            .at(Location::new().xs(
                50.pct().as_center_x().with(
                    TOTAL_COLS
                        .letters()
                        .as_width()
                        .adjust((TOTAL_COLS - 1) * CELL_GAP_PX),
                ),
                FIELD_CENTER_Y_PCT
                    .pct()
                    .as_center_y()
                    .with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with((grid, FontSize::new(FONT_SIZE))),
    );

    draw_heptagon(tree, parent, field, seq, start);

    let cursor = tree.branch(
        field,
        Panel::new()
            .color(Color::gray(50))
            .at(cursor_location(0))
            .elevate(Elevation::up(1))
            .with(Opacity::new(0.0)),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(cursor)
            .during(seq)
            .start(start)
            .finish(start + REVEAL_SNAP)
            .eased(Ease::Linear),
    );

    let mut letter_time = start;
    for (i, ch) in TEXT.chars().enumerate() {
        letter_time += DELAYS[i];
        let col = i as i32;

        let color = if i >= EXTENSION_START {
            Color::orange(600) // ".rs" -- darker than the polygon's own orange
        } else {
            Color::gray(50)
        };
        let letter = tree.branch(
            field,
            Text::new(ch.to_string())
                .size(FontSize::new(FONT_SIZE))
                .color(color)
                .at(cell_location(col))
                .elevate(Elevation::up(1))
                .with((HorizontalAlignment::Center, Opacity::new(0.0))),
        );
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(letter)
                .during(seq)
                .start(letter_time)
                .finish(letter_time + REVEAL_SNAP)
                .eased(Ease::Linear),
        );

        // the cursor hops to just past the letter that just landed.
        tree.animate(
            Animation::new(cursor_location(col + 1))
                .targeting(cursor)
                .during(seq)
                .start(letter_time)
                .finish(letter_time + REVEAL_SNAP)
                .eased(Ease::Linear),
        );
    }

    let subtitle_end = draw_line_and_subtitle(tree, parent, field, seq, letter_time + REVEAL_SNAP);
    let links_start = subtitle_end + DOCS_START_GAP;
    draw_link_button(
        tree,
        parent,
        field,
        seq,
        links_start,
        LinkButton {
            label: "docs",
            icon: IconHandles::Code,
            href: DOCS_API_HREF,
            slot: 0,
        },
    );
    draw_link_button(
        tree,
        parent,
        field,
        seq,
        links_start,
        LinkButton {
            label: "book",
            icon: IconHandles::BookOpen,
            href: DOCS_BOOK_HREF,
            slot: 1,
        },
    );

    // cursor blink: a snapped on/off toggle at a steady cadence, running through the
    // whole typing pass and a while after it settles.
    let blink_end = letter_time + TRAILING_BLINKS * BLINK_INTERVAL;
    let mut blink_time = start;
    let mut on = true;
    while blink_time < blink_end {
        blink_time += BLINK_INTERVAL;
        on = !on;
        tree.animate(
            Animation::new(Opacity::new(if on { 1.0 } else { 0.0 }))
                .targeting(cursor)
                .during(seq)
                .start(blink_time)
                .finish(blink_time + BLINK_SNAP)
                .eased(Ease::Linear),
        );
    }
}
