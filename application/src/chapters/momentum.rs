use foliage::{
    Anchor, Animation, Color, ConfigurationDescriptor, Ease, EcsExtension, Elevation, Entity,
    FontSize, Grid, GridExt, HorizontalAlignment, Interaction, InteractionMethod, InteractionPhase,
    Leaf, Location, Logical, OnEnd, Opacity, Panel, Polygon, Position, Query, Rounding, ScrollTo,
    Section, Sprout, Text, Tree, Trigger, VerticalAlignment, View, anchor,
};

// The substrate is the same shape `scroll.rs` uses -- a window shorter than its content --
// because momentum needs something with somewhere to go. Everything worth watching here
// happens after the gesture ends, so the stack is deliberately tall: a coast that runs out
// of room stops for the wrong reason.
const TILES: usize = 12;
const TILE_HEIGHT_PX: i32 = 34;
const TILE_GAP_PX: i32 = 8;
const CONTENT_HEIGHT_PX: i32 = TILES as i32 * TILE_HEIGHT_PX + (TILES as i32 - 1) * TILE_GAP_PX;

const WINDOW_LEFT_PCT: f32 = 20.0;
const WINDOW_WIDTH_PCT: f32 = 40.0;
const WINDOW_TOP_PCT: f32 = 24.0;
const WINDOW_HEIGHT_PX: i32 = 3 * TILE_HEIGHT_PX + 2 * TILE_GAP_PX;
const WINDOW_COLOR: i32 = 500; // slate, same muted outline every chapter's parent uses
const WINDOW_PAD_PX: i32 = 8;

const TILE_COLOR: i32 = 400; // sky
const TILE_ROUNDING: Rounding = Rounding::Sm;

// The flick, in real queued `Interaction` events. Velocity is measured off wall-clock time
// between moves, so these deltas and delays are the actual physics input -- `40px` every
// `24ms` is ~1.7 px/ms, comfortably past `ScrollMomentum`'s own 0.15 threshold.
const FLICK_STEP_PX: f32 = 40.0;
const FLICK_STEP_MS: u64 = 24;
const FLICK_MOVES: usize = 4;

const BEFORE_FLICK_MS: u64 = 900; // rest at the top so the start of the gesture is visible
const AFTER_STOP_MS: u64 = 1100; // let the coast's own ending register before resetting
const INTERRUPT_AFTER_MS: u64 = 380; // into the second coast, while it's clearly still moving
const AFTER_INTERRUPT_MS: u64 = 1300;

const REVEAL: u64 = 300;
const TILE_STAGGER: u64 = 45;
const CUE_FADE: u64 = 220;
const CUE_HOLD: u64 = 700;

// Same click indicator `interact.rs` uses -- stone, so it reads as "a pointer touched
// here" rather than as another piece of the content. Without it the second beat is
// indistinguishable from the first: the motion just stops, and nothing says a press is why.
const BLIP_SIZE_PX: i32 = 26;
const BLIP_COLOR: i32 = 400; // stone
const BLIP_FADE_IN: u64 = 140; // faster than `interact.rs`'s own -- this one marks an instant
const BLIP_HOLD: u64 = 260;
const BLIP_FADE_OUT: u64 = 320;

const LABEL_COLOR: i32 = 500; // slate
const LABEL_FONT_SIZE: u32 = 12;
const LABEL_WIDTH_PX: i32 = 150;
const LABEL_HEIGHT_PX: i32 = 18;
const LABEL_GAP_PX: i32 = 10;

const TAGLINE_TEXT: &str = "a fast release keeps going; a touch stops it";
const TAGLINE_FONT_SIZE: u32 = 13;
const TAGLINE_COLOR: i32 = 400;
const TAGLINE_XS_TOP_PCT: f32 = 76.0;
const TAGLINE_XS_HEIGHT_PCT: f32 = 20.0;
const TAGLINE_XS_LEFT_PCT: f32 = 8.0;
const TAGLINE_XS_WIDTH_PCT: f32 = 84.0;
const TAGLINE_MD_LEFT_PCT: f32 = 66.0;
const TAGLINE_MD_WIDTH_PCT: f32 = 26.0; // ends at 92%, flush with `window_frame`'s margin
const TAGLINE_MD_TOP_PCT: f32 = 26.0;
const TAGLINE_MD_HEIGHT_PCT: f32 = 52.0;

/// A flick, then a coast, then the same flick cut short by a press.
///
/// Unlike the other chapters, nothing here is staged: this page queues real
/// [`Interaction`] events at the window's own live position and lets the engine do the
/// rest. The velocity is measured from wall-clock time between those moves, the
/// [`ScrollMomentum`](foliage::ScrollMomentum) threshold decides whether it counts as a
/// flick, and the decay that follows is the engine's own. Cut the flick's speed and the
/// coast genuinely stops happening.
///
/// Two beats. First: release fast and the view keeps travelling on its own, slowing to a
/// stop -- momentum, with no input left driving it. Second: the same flick, interrupted
/// mid-glide by a single press, which halts it dead. One pointer means one gesture, so
/// putting a finger down is how a user cancels momentum; there is no relation between the
/// press and the coasting view that has to hold for that to work.
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
            // A `Panel` carries no `Grid` of its own, and `content` positions itself
            // against this one -- so it needs one even though nothing addresses a cell.
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
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

    // Four cues, each hidden until the moment it names actually happens -- so the labels
    // read as narration of the physics rather than a static legend.
    let above = |gap: i32| {
        anchor()
            .top()
            .as_bottom()
            .adjust(-LABEL_GAP_PX - LABEL_HEIGHT_PX - gap)
            .with(LABEL_HEIGHT_PX.px().as_height())
    };
    let below = |gap: i32| {
        anchor()
            .bottom()
            .as_top()
            .adjust(LABEL_GAP_PX + gap)
            .with(LABEL_HEIGHT_PX.px().as_height())
    };
    // Anchored to the window's own center, which is also exactly where `interrupt_coast`
    // aims its press -- so the mark and the event it stands for can't drift apart.
    let blip = tree.branch(
        frame,
        Polygon::new()
            .sides(4.0)
            .rounding(1.0) // full rounding -- a true circle regardless of side count
            .rotation(0.0)
            .color(Color::stone(BLIP_COLOR))
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .with(BLIP_SIZE_PX.px().as_width()),
                anchor()
                    .center_y()
                    .as_center_y()
                    .with(BLIP_SIZE_PX.px().as_height()),
            ))
            .elevate(Elevation::up(5))
            .with((Anchor::new(window), Opacity::new(0.0))),
    );

    let cues = Cues {
        velocity: cue(tree, frame, window, "velocity", above(0)),
        decay: cue(tree, frame, window, "decay", below(0)),
        stop: cue(tree, frame, window, "stop", below(LABEL_HEIGHT_PX)),
        press: cue(tree, frame, window, "a press cancels it", below(0)),
        blip,
    };

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
        wait(&mut tree, BEFORE_FLICK_MS, move |tree| {
            flick(tree, window, cues, false)
        });
    });
}

// The four narration labels, carried through the phase chain as one value so each step
// doesn't need four parameters.
#[derive(Copy, Clone)]
struct Cues {
    velocity: Entity,
    decay: Entity,
    stop: Entity,
    press: Entity,
    /// The click indicator, not a label -- what makes the cancelling press visible at all.
    blip: Entity,
}

fn cue(
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

// A timed step with nothing to animate -- the same no-op-tween-for-timing pattern
// `text.rs` and `breakpoints.rs` use, wrapped up since this page needs it repeatedly.
fn wait(tree: &mut Tree, ms: u64, then: impl Fn(&mut Tree) + Send + Sync + 'static) {
    tree.timer(ms, move |_: Trigger<OnEnd>, mut tree: Tree| then(&mut tree));
}

// Each cue fade gets a sequence of its own. Every animation belongs to one -- there is no
// unsequenced form -- and these fire on their own schedule rather than alongside anything
// else, so a fresh single-animation sequence is the whole timeline they need.
fn fade(tree: &mut Tree, cue: Entity, to: f32) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(to))
            .targeting(cue)
            .during(seq)
            .start(0)
            .finish(CUE_FADE)
            .eased(Ease::Linear),
    );
}
fn show(tree: &mut Tree, cue: Entity) {
    fade(tree, cue, 1.0);
}
fn hide(tree: &mut Tree, cue: Entity) {
    fade(tree, cue, 0.0);
}

// Queues a real gesture: press at the window's own center, drag upward past
// `DRAG_THRESHOLD` in fast successive moves, release. Positions are read from the window's
// live `Section`, so this follows the layout instead of assuming coordinates.
//
// `interrupt` decides which beat this is -- the plain coast, or the one cut short.
fn flick(tree: &mut Tree, window: Entity, cues: Cues, interrupt: bool) {
    tree.timer(
        0,
        move |_: Trigger<OnEnd>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>,
              existing: Query<Entity>| {
            if !existing.contains(window) {
                return;
            }
            let Ok(section) = sections.get(window) else {
                return;
            };
            let x = section.center().left();
            let start_y = section.center().top() + section.height() * 0.25;
            queue_at(&mut tree, InteractionPhase::Start, x, start_y);
            drag(&mut tree, window, cues, interrupt, x, start_y, 0);
        },
    );
}

// One `Moved` per tick so the engine measures real elapsed time between them -- a velocity
// EMA fed by moves that all arrive in the same frame would have nothing to divide by.
fn drag(
    tree: &mut Tree,
    window: Entity,
    cues: Cues,
    interrupt: bool,
    x: f32,
    y: f32,
    move_index: usize,
) {
    tree.timer(
        FLICK_STEP_MS,
        move |_: Trigger<OnEnd>, mut tree: Tree, existing: Query<Entity>| {
            if !existing.contains(window) {
                return;
            }
            let next_y = y - FLICK_STEP_PX;
            queue_at(&mut tree, InteractionPhase::Moved, x, next_y);
            if move_index + 1 < FLICK_MOVES {
                drag(
                    &mut tree,
                    window,
                    cues,
                    interrupt,
                    x,
                    next_y,
                    move_index + 1,
                );
                return;
            }
            // Release. Everything past this point is the engine's own doing.
            queue_at(&mut tree, InteractionPhase::End, x, next_y);
            show(&mut tree, cues.velocity);
            show(&mut tree, cues.decay);
            if interrupt {
                wait(&mut tree, INTERRUPT_AFTER_MS, move |tree| {
                    interrupt_coast(tree, window, cues)
                });
            } else {
                wait(&mut tree, CUE_HOLD, move |tree| {
                    hide(tree, cues.velocity);
                    settle(tree, window, cues);
                });
            }
        },
    );
}

// Waits out the coast rather than timing it: `stop_epsilon` decides when it ends, and how
// long that takes depends on the release velocity, so polling the view's own position is
// the only honest way to know it has finished.
fn settle(tree: &mut Tree, window: Entity, cues: Cues) {
    tree.timer(
        90,
        move |_: Trigger<OnEnd>, mut tree: Tree, views: Query<&View>, existing: Query<Entity>| {
            if !existing.contains(window) {
                return;
            }
            let Ok(view) = views.get(window) else { return };
            let at = view.offset().top();
            wait_for_rest(&mut tree, window, cues, at);
        },
    );
}
fn wait_for_rest(tree: &mut Tree, window: Entity, cues: Cues, last: f32) {
    tree.timer(
        90,
        move |_: Trigger<OnEnd>, mut tree: Tree, views: Query<&View>, existing: Query<Entity>| {
            if !existing.contains(window) {
                return;
            }
            let Ok(view) = views.get(window) else { return };
            let at = view.offset().top();
            if (at - last).abs() > 0.5 {
                wait_for_rest(&mut tree, window, cues, at);
                return;
            }
            hide(&mut tree, cues.decay);
            show(&mut tree, cues.stop);
            wait(&mut tree, AFTER_STOP_MS, move |tree| {
                hide(tree, cues.stop);
                reset(tree, window, cues, true);
            });
        },
    );
}

// The second beat's payoff: a single `Start` mid-glide, nothing else. No drag follows, so
// this is purely "a finger went down" -- and the coast ends on that alone.
//
// Aimed at the window's own center, read fresh rather than reusing where the flick's last
// move landed. That position is well above the window by then (four 40px moves upward from
// just below center), so a press there would fall outside the box it is supposed to be
// stopping -- and the blip marking it would float in empty space.
fn interrupt_coast(tree: &mut Tree, window: Entity, cues: Cues) {
    tree.timer(
        0,
        move |_: Trigger<OnEnd>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>,
              existing: Query<Entity>| {
            if !existing.contains(window) {
                return;
            }
            let Ok(section) = sections.get(window) else {
                return;
            };
            let at = section.center();
            queue_at(&mut tree, InteractionPhase::Start, at.left(), at.top());
            hide(&mut tree, cues.velocity);
            hide(&mut tree, cues.decay);
            show(&mut tree, cues.press);
            blip(&mut tree, cues.blip);
            // Release the synthetic press so the pointer isn't left down afterwards.
            wait(&mut tree, 120, move |tree| {
                queue_at(tree, InteractionPhase::End, at.left(), at.top());
                wait(tree, AFTER_INTERRUPT_MS, move |tree| {
                    hide(tree, cues.press);
                    reset(tree, window, cues, false);
                });
            });
        },
    );
}

// In, hold, out -- one sequence, so the fade-out is timed off the fade-in rather than
// racing it.
fn blip(tree: &mut Tree, marker: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(marker)
            .during(seq)
            .start(0)
            .finish(BLIP_FADE_IN)
            .eased(Ease::Linear),
    );
    tree.animate(
        Animation::new(Opacity::new(0.0))
            .targeting(marker)
            .during(seq)
            .start(BLIP_FADE_IN + BLIP_HOLD)
            .finish(BLIP_FADE_IN + BLIP_HOLD + BLIP_FADE_OUT)
            .eased(Ease::Linear),
    );
}

// Back to the top and around again, alternating which beat runs.
fn reset(tree: &mut Tree, window: Entity, cues: Cues, next_interrupts: bool) {
    tree.write_to(window, ScrollTo::y(0.0));
    wait(tree, BEFORE_FLICK_MS, move |tree| {
        flick(tree, window, cues, next_interrupts)
    });
}

fn queue_at(tree: &mut Tree, phase: InteractionPhase, x: f32, y: f32) {
    // `Commands::queue` (inherent, for `Command`s) shadows `EcsExtension::queue`, so the
    // message form has to be named through the trait.
    EcsExtension::queue(
        tree,
        Interaction::new(
            phase,
            Position::<Logical>::new((x, y)),
            InteractionMethod::TouchScreen,
        ),
    );
}
