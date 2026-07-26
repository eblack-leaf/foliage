use foliage::{
    Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, GridExt,
    HorizontalAlignment, Location, OnEnd, Opacity, Panel, Polygon, Rounding, Sprout, Text, Tree,
    Trigger, VerticalAlignment,
};

// `passthrough` (outline only, drawn on top) and `interactive` (solid, drawn behind it,
// offset down+right) overlap in their shared corner -- the whole point: a click landing
// in that shared space should be understood as reaching whichever one actually listens
// for it, not just whichever one is visually on top.
const PANEL_WIDTH_PCT: f32 = 32.0;
const PANEL_HEIGHT_PCT: f32 = 28.0;
const PASSTHROUGH_LEFT_PCT: f32 = 20.0;
const PASSTHROUGH_TOP_PCT: f32 = 22.0;
const OFFSET_PCT: f32 = 14.0; // how far down+right `interactive` sits from `passthrough`
const INTERACTIVE_LEFT_PCT: f32 = PASSTHROUGH_LEFT_PCT + OFFSET_PCT;
const INTERACTIVE_TOP_PCT: f32 = PASSTHROUGH_TOP_PCT + OFFSET_PCT;
const PANEL_FADE: u64 = 400;

const PASSTHROUGH_COLOR: i32 = 500; // slate outline -- reads as a plain frame, not a surface
const INTERACTIVE_COLOR: i32 = 500; // blue -- the real, solid, listening surface
// emerald, not amber -- amber sat too close to `BLIP_COLOR`'s own rose to read as two
// distinct colors at a glance; emerald is far enough from both rose and blue to be
// unambiguous.
const TOGGLED_COLOR: i32 = 500;
const COLOR_CHANGE_DURATION: u64 = 400;

const LABEL_COLOR: i32 = 500; // slate, same blueprint tone the other chapters use
const LABEL_FONT_SIZE: u32 = 13;
const LABEL_FADE: u64 = 300;
const LABEL_DELAY: u64 = 150; // after the panels finish fading in
// fixed above `passthrough` / below `interactive` at every width, spanning that panel's
// exact horizontal footprint (wide enough that "passthrough"/"interactive" -- 11
// characters each -- don't wrap mid-word the way a narrower box did).
const LABEL_XS_HEIGHT_PCT: f32 = 12.0;

// the blip sits in the exact rectangle the two panels actually share
// (`INTERACTIVE_LEFT_PCT..PASSTHROUGH_LEFT_PCT + PANEL_WIDTH_PCT` and the same for `top`)
// -- roughly centered in it, not anywhere else, since the whole demo hinges on the click
// visibly landing where *both* panels could plausibly claim it.
const BLIP_WIDTH_PCT: f32 = 9.0;
const BLIP_HEIGHT_PCT: f32 = 9.0;
const BLIP_LEFT_PCT: f32 =
    INTERACTIVE_LEFT_PCT + (PASSTHROUGH_LEFT_PCT + PANEL_WIDTH_PCT - INTERACTIVE_LEFT_PCT) / 2.0
        - BLIP_WIDTH_PCT / 2.0;
const BLIP_TOP_PCT: f32 =
    INTERACTIVE_TOP_PCT + (PASSTHROUGH_TOP_PCT + PANEL_HEIGHT_PCT - INTERACTIVE_TOP_PCT) / 2.0
        - BLIP_HEIGHT_PCT / 2.0;
const BLIP_COLOR: i32 = 400; // rose -- a click indicator, distinct from either panel's own color
const BLIP_FADE_IN: u64 = 180;
const BLIP_FADE_OUT: u64 = 180;

const CLICK_WAIT: u64 = 900; // between the first click's effect settling and the second click

// above `passthrough` / below `interactive` -- fixed, not responsive: unlike the other
// chapters' tagline (which genuinely needs the axis swap since it's competing for the
// same space as the demo), these two just need to sit next to their own panel, and one
// plain position that works at every width is simpler than maintaining an `Xs`/`Md` split
// that was only ever there to dodge a clobbering issue with the tagline's own column.
const PASSTHROUGH_LABEL_TOP_PCT: f32 = 8.0;
const INTERACTIVE_LABEL_TOP_PCT: f32 = 67.0;

const TAGLINE_TEXT: &str = "interactive elements can either pass or grab input";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400; // slate, muted subtitle -- orients without competing with the demo
// just past `interactive_label`'s own bottom edge (67 + 12 = 79), down almost to
// `window_frame`'s own bottom margin -- this tagline's text is long enough (51
// characters) to wrap several lines even at `Xs`'s own width, so it gets more room than
// the other chapters' shorter taglines needed.
const TAGLINE_XS_TOP_PCT: f32 = 80.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 19.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_LEFT_PCT: f32 = 78.0;
const TAGLINE_MD_WIDTH_PCT: f32 = 14.0; // ends at 92%, flush with `window_frame`'s own margin
// the labels no longer share this column at any width, so the tagline gets almost the
// whole frame height to wrap into (this column is only 14% wide, so the long text still
// needs many lines), not just the two panels' own combined span.
const TAGLINE_MD_TOP_PCT: f32 = 4.0;
const TAGLINE_MD_HEIGHT_PCT: f32 = 92.0;

/// Two rectangles, not one shape moving -- `passthrough` (outline only) sits on top of
/// `interactive` (solid, offset down+right), overlapping in one shared corner. A blip
/// (a real click stand-in, since nothing here is actually clickable yet -- that's a later
/// chapter's own composite) fades in and out right in that shared space, then
/// `interactive`'s own color changes -- never `passthrough`'s -- because propagation
/// decides who actually *receives* a hit, not who's visually on top. A pause, a second
/// blip, and `interactive` changes back, proving the first change wasn't a one-off.
pub fn build(tree: &mut Tree, slot: Entity) {
    let frame = crate::chapters::window_frame(tree, slot);

    let passthrough = tree.branch(
        frame,
        Panel::new()
            .color(Color::slate(PASSTHROUGH_COLOR))
            .outline(2)
            .rounding(Rounding::None)
            .at(Location::new().xs(
                PASSTHROUGH_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(PANEL_WIDTH_PCT.pct().as_width()),
                PASSTHROUGH_TOP_PCT
                    .pct()
                    .as_top()
                    .with(PANEL_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with(Opacity::new(0.0)),
    );

    let interactive = tree.branch(
        frame,
        Panel::new()
            .color(Color::blue(INTERACTIVE_COLOR))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                INTERACTIVE_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(PANEL_WIDTH_PCT.pct().as_width()),
                INTERACTIVE_TOP_PCT
                    .pct()
                    .as_top()
                    .with(PANEL_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with(Opacity::new(0.0)),
    );

    let seq = tree.sequence();
    for panel in [passthrough, interactive] {
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(panel)
                .during(seq)
                .start(0)
                .finish(PANEL_FADE)
                .eased(Ease::Linear),
        );
    }

    let passthrough_label = tree.branch(
        frame,
        Text::new("passthrough")
            .size(FontSize::new(LABEL_FONT_SIZE))
            .color(Color::slate(LABEL_COLOR))
            .at(Location::new().xs(
                PASSTHROUGH_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(PANEL_WIDTH_PCT.pct().as_width()),
                PASSTHROUGH_LABEL_TOP_PCT
                    .pct()
                    .as_top()
                    .with(LABEL_XS_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
            )),
    );
    let interactive_label = tree.branch(
        frame,
        Text::new("interactive")
            .size(FontSize::new(LABEL_FONT_SIZE))
            .color(Color::slate(LABEL_COLOR))
            .at(Location::new().xs(
                INTERACTIVE_LEFT_PCT
                    .pct()
                    .as_left()
                    .with(PANEL_WIDTH_PCT.pct().as_width()),
                INTERACTIVE_LABEL_TOP_PCT
                    .pct()
                    .as_top()
                    .with(LABEL_XS_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
            )),
    );
    for label in [passthrough_label, interactive_label] {
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(label)
                .during(seq)
                .start(PANEL_FADE + LABEL_DELAY)
                .finish(PANEL_FADE + LABEL_DELAY + LABEL_FADE)
                .eased(Ease::Linear),
        );
    }

    let blip = tree.branch(
        frame,
        Polygon::new()
            .sides(4.0)
            .rounding(1.0) // full rounding -- a true circle regardless of side count
            .rotation(0.0)
            .color(Color::rose(BLIP_COLOR))
            .at(Location::new().xs(
                BLIP_LEFT_PCT.pct().as_left().with(BLIP_WIDTH_PCT.pct().as_width()),
                BLIP_TOP_PCT.pct().as_top().with(BLIP_HEIGHT_PCT.pct().as_height()),
            ))
            .elevate(Elevation::up(5))
            .with(Opacity::new(0.0)),
    );

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
            .finish(PANEL_FADE)
            .eased(Ease::Linear),
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        blip_in(&mut tree, blip, interactive);
    });
}

fn blip_in(tree: &mut Tree, blip: Entity, interactive: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(blip)
            .during(seq)
            .start(0)
            .finish(BLIP_FADE_IN)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        blip_out(&mut tree, blip, interactive);
    });
}

fn blip_out(tree: &mut Tree, blip: Entity, interactive: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(0.0))
            .targeting(blip)
            .during(seq)
            .start(0)
            .finish(BLIP_FADE_OUT)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        toggle_color(&mut tree, blip, interactive);
    });
}

fn toggle_color(tree: &mut Tree, blip: Entity, interactive: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Color::emerald(TOGGLED_COLOR))
            .targeting(interactive)
            .during(seq)
            .start(0)
            .finish(COLOR_CHANGE_DURATION)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        wait_then_click_again(&mut tree, blip, interactive);
    });
}

// a real waiting step, not a delay folded into the next click's own animation -- same
// reasoning `animate.rs`'s/`sequence.rs`'s own `pause_before_color` uses: a no-op
// `Opacity` tween (`blip`'s already at `0.0`) just gives this a timed `sequence_end`.
fn wait_then_click_again(tree: &mut Tree, blip: Entity, interactive: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(0.0))
            .targeting(blip)
            .during(seq)
            .start(0)
            .finish(CLICK_WAIT)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        blip_in_again(&mut tree, blip, interactive);
    });
}

fn blip_in_again(tree: &mut Tree, blip: Entity, interactive: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(blip)
            .during(seq)
            .start(0)
            .finish(BLIP_FADE_IN)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        blip_out_again(&mut tree, blip, interactive);
    });
}

fn blip_out_again(tree: &mut Tree, blip: Entity, interactive: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(0.0))
            .targeting(blip)
            .during(seq)
            .start(0)
            .finish(BLIP_FADE_OUT)
            .eased(Ease::Linear),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        toggle_color_back(&mut tree, interactive);
    });
}

// final phase -- nothing needs to react once this finishes, so no `sequence_end` at all.
fn toggle_color_back(tree: &mut Tree, interactive: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Color::blue(INTERACTIVE_COLOR))
            .targeting(interactive)
            .during(seq)
            .start(0)
            .finish(COLOR_CHANGE_DURATION)
            .eased(Ease::Linear),
    );
}
