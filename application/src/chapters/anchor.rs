use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, GridExt,
    HorizontalAlignment, Line, Location, OnEnd, Opacity, Polygon, Sprout, Text, Tree, Trigger,
    VerticalAlignment, anchor,
};

const LEADER_LEFT_PCT: f32 = 18.0; // same proportions `location.rs`'s own hepta uses
const LEADER_TOP_PCT: f32 = 35.0;
const LEADER_WIDTH_PCT: f32 = 26.0;
const LEADER_HEIGHT_PCT: f32 = 30.0;
const LEADER_ROUNDING: f32 = 0.15;
const MORPH_DURATION: u64 = 500;

// after the intro settles, `leader` slides left, pauses, slides right past rest, pauses,
// then returns -- `follower` (and the line between them) never gets its own position
// animation at all: both are `Anchor`ed to `leader`, so they're recomputed from its
// *live* box every frame for free, "moving in sync" without ever being told to move.
const LEFT_DELTA_PCT: f32 = 10.0;
const MOVE_LEFT_PCT: f32 = LEADER_LEFT_PCT - LEFT_DELTA_PCT;
const MOVE_RIGHT_PCT: f32 = LEADER_LEFT_PCT + LEFT_DELTA_PCT;
const MOVE_DURATION: u64 = 500;
const MOVE_PAUSE: u64 = 400;

// `follower` sits `GAP_PX` off `leader`'s own right edge, same size as `leader`
// (`anchor().width()`/`.height()`, not a fixed px/pct of its own) -- so it stays the same
// size as `leader` even if that ever changed, the same "derive from the live target
// instead of a separate guess" idea the gap/position already use.
const GAP_PX: i32 = 40;
const FOLLOWER_FADE: u64 = 250;

const LABEL_COLOR: i32 = 500; // slate, same blueprint tone the other chapters use
const LABEL_FONT_SIZE: u32 = 13;
const LABEL_GAP_PX: i32 = 10; // below `follower`'s own bottom edge
// one row, not two -- "anchor().right()" fits a single line at this width, and this
// label's own fixed-px offset below `follower`'s live bottom edge was running past the
// frame's own bottom in short landscape viewports (`follower` already sits fairly low at
// rest); halving the vertical footprint by dropping the second row was the actual fix.
const LABEL_WIDTH_PX: i32 = 140;
const LABEL_HEIGHT_PX: i32 = 20;
const LABEL_FADE: u64 = 300;
const LABEL_DELAY: u64 = 150; // after `follower` finishes fading in

const TAGLINE_TEXT: &str = "position can follow another entity's live box";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400; // slate, muted subtitle -- orients without competing with the demo
/// `Xs` (portrait) stacks this above the demo, in the frame's own empty band before
/// `LEADER_TOP_PCT` -- same band `location.rs`'s own tagline uses, since `leader` sits at
/// the same rest position its hepta does. `Md`+ goes beside the pair instead, using a
/// static percent estimate of where `follower` rests (not a live `Anchor` to it) --
/// `leader` slides left/right during the demo, and anchoring here would drag the tagline
/// along with it, which isn't the point of a caption meant to just sit still and orient.
const TAGLINE_XS_TOP_PCT: f32 = 6.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 40.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_LEFT_PCT: f32 = 74.0;
const TAGLINE_MD_WIDTH_PCT: f32 = 18.0; // ends at 92%, flush with `window_frame`'s own margin -- wider than before, fewer wrapped lines
// dedicated, not `LEADER_HEIGHT_PCT` reused -- that's `leader`'s own visual size, not
// how much room this narrow column actually needs to wrap into.
const TAGLINE_MD_TOP_PCT: f32 = 18.0;
const TAGLINE_MD_HEIGHT_PCT: f32 = 78.0;

/// Two heptagons and a line between them -- `follower` and the line are both `Anchor`ed
/// to `leader`, not positioned in percent of the window frame at all, so when `leader`
/// slides left/pause/right/pause/rest, they visibly move in lockstep without ever being
/// animated themselves: a `Location` can resolve against *another entity's own live box*,
/// not just a structural parent's. The label sits under `follower` (not `leader`) since
/// `follower` is the one actually anchored -- its text is the real call driving its own
/// position, `anchor().right()`. A muted tagline orients the reader on what they're
/// looking at, same Xs-above/Md-beside placement `location.rs`'s own tagline uses.
pub fn build(tree: &mut Tree, slot: Entity) {
    let frame = crate::chapters::window_frame(tree, slot);

    let leader = tree.branch(
        frame,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::green(400))
            .at(leader_location(LEADER_LEFT_PCT))
            .elevate(Elevation::up(2))
            .with(Opacity::new(0.0)),
    );

    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(leader)
            .during(seq)
            .start(0)
            .finish(MORPH_DURATION)
            .eased(Ease::Linear),
    );
    tree.animate(
        Animation::new(Polygon {
            sides: 7.0,
            rounding: LEADER_ROUNDING,
            rotation: 0.0,
        })
        .targeting(leader)
        .during(seq)
        .start(0)
        .finish(MORPH_DURATION)
        .eased(Ease::DECELERATE),
    );

    // line: leader's own right edge -> `GAP_PX` further right, at its own vertical
    // center -- `Anchor::new(leader)` is what makes both ends resolve off leader's
    // *live* box.
    let line = tree.branch(
        frame,
        Line::new(2)
            .color(Color::slate(LABEL_COLOR))
            .at(Location::new().xs(
                anchor().right().as_x().with(anchor().center_y().as_y()),
                anchor()
                    .right()
                    .as_x()
                    .adjust(GAP_PX)
                    .with(anchor().center_y().as_y()),
            ))
            .elevate(Elevation::up(2))
            .with((Anchor::new(leader), Opacity::new(0.0))),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(line)
            .during(seq)
            .start(MORPH_DURATION)
            .finish(MORPH_DURATION + FOLLOWER_FADE)
            .eased(Ease::Linear),
    );

    // already a heptagon at spawn (not morphed in like `leader`) -- `follower` only ever
    // fades in, since the point here is the live anchor tracking, not a second morph.
    let follower = tree.branch(
        frame,
        Polygon::new()
            .sides(7.0)
            .rounding(LEADER_ROUNDING)
            .rotation(0.0)
            .color(Color::cyan(400))
            .at(Location::new().xs(
                anchor()
                    .right()
                    .as_left()
                    .adjust(GAP_PX)
                    .with((anchor().width() * 1.0).as_width()),
                anchor()
                    .top()
                    .as_top()
                    .with((anchor().height() * 1.0).as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((Anchor::new(leader), Opacity::new(0.0))),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(follower)
            .during(seq)
            .start(MORPH_DURATION)
            .finish(MORPH_DURATION + FOLLOWER_FADE)
            .eased(Ease::Linear),
    );

    // plausible `Location` syntax naming the real call driving `follower`'s own left
    // edge, in one row -- wide enough at `LABEL_WIDTH_PX` that it doesn't need the
    // two-row split every other chapter's label uses.
    let label = tree.branch(
        frame,
        Text::new("anchor().right()")
            .size(FontSize::new(LABEL_FONT_SIZE))
            .color(Color::slate(LABEL_COLOR))
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .with(LABEL_WIDTH_PX.px().as_width()),
                anchor()
                    .bottom()
                    .as_top()
                    .adjust(LABEL_GAP_PX)
                    .with(LABEL_HEIGHT_PX.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Anchor::new(follower),
                Opacity::new(0.0),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(label)
            .during(seq)
            .start(MORPH_DURATION + FOLLOWER_FADE + LABEL_DELAY)
            .finish(MORPH_DURATION + FOLLOWER_FADE + LABEL_DELAY + LABEL_FADE)
            .eased(Ease::Linear),
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
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                Opacity::new(0.0),
            )),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(tagline)
            .during(seq)
            .start(0)
            .finish(MORPH_DURATION)
            .eased(Ease::Linear),
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        move_left(&mut tree, leader);
    });
}

fn leader_location(left_pct: f32) -> Location {
    Location::new().xs(
        left_pct
            .pct()
            .as_left()
            .with(LEADER_WIDTH_PCT.pct().as_width()),
        LEADER_TOP_PCT
            .pct()
            .as_top()
            .with(LEADER_HEIGHT_PCT.pct().as_height()),
    )
}

fn move_left(tree: &mut Tree, leader: Entity) {
    move_leader(tree, leader, MOVE_LEFT_PCT, 0, Some(move_right));
}

fn move_right(tree: &mut Tree, leader: Entity) {
    move_leader(tree, leader, MOVE_RIGHT_PCT, MOVE_PAUSE, Some(move_rest));
}

fn move_rest(tree: &mut Tree, leader: Entity) {
    move_leader(tree, leader, LEADER_LEFT_PCT, MOVE_PAUSE, None);
}

fn move_leader(
    tree: &mut Tree,
    leader: Entity,
    target_left_pct: f32,
    pause_before: u64,
    next: Option<fn(&mut Tree, Entity)>,
) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(leader_location(target_left_pct))
            .targeting(leader)
            .during(seq)
            .start(pause_before)
            .finish(pause_before + MOVE_DURATION)
            .eased(Ease::EMPHASIS),
    );
    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        if let Some(next_fn) = next {
            next_fn(&mut tree, leader);
        }
    });
}
