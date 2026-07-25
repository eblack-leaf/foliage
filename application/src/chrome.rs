use crate::icons::IconHandles;
use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, FontSize, GridExt,
    HorizontalAlignment, Icon, InteractionListener, InteractionPropagation, InteractionShape,
    Line, Location, OnClick, OnEnd, Opacity, PageIndex, Polygon, Query, Sprout, Text, Tree,
    Trigger, anchor,
};

/// Global, site-wide chrome -- distinct from the inner-site forward/back navigator
/// (`crate::navigator`), which only ever moves one page at a time and is meant to be
/// browsed linearly. This is the "jump anywhere / who made this" layer: a small brand
/// mark, a github link, and direct Home/ToC controls. Lives outside the router's subtree
/// (survives every route switch, same reasoning as the navigator) and is elevated well
/// above it so the two read as separate layers, never competing for the same visual space.
/// Shared row for every element -- real pixels, not a percent (a percent `top` still
/// scales with screen height, pushing everything down on a taller viewport), and not
/// derived from any shape's own center, so nothing here depends on another element's
/// position.
const ROOT_TOP_PX: i32 = 8;

/// Real pixels, not a percentage -- a percent-based size on a small chrome shape scales
/// with the *whole screen*, so on a wide desktop window it blows up into something huge.
/// Fixed pixels keep these shapes the same physical size regardless of viewport.
const HEPTA_SIZE_PX: i32 = 38;
const HEPTA_CENTER_X_PX: i32 = 32;
/// Every element's own *center* aligns to this row (derived from the heptagon's own
/// placement, since that's what the connecting line's `anchor().center_y()` follows) --
/// not a shared `top`, which misaligns centers whenever sizes differ (as they do here:
/// the heptagon and the smaller Home/ToC controls).
const ROW_CENTER_Y_PX: i32 = ROOT_TOP_PX + HEPTA_SIZE_PX / 2;

const FADE_IN: u64 = 500;
/// Staggered behind the fade start, not concurrent with it -- the mark visibly begins
/// appearing first, then starts spinning a beat later, rather than both reading as one
/// indistinguishable motion.
const SPIN_DELAY: u64 = 180;
const SPIN_IN: u64 = 700;
/// A full turn -- combined with the shape actually changing (see `build`'s spawn shape),
/// not a fixed shape rotating in place, which barely reads as motion regardless of amount.
const SPIN_ROTATION: f32 = std::f32::consts::PI * 2.0;

const GITHUB_FADE: u64 = 400;
const LINE_WEIGHT: i32 = 2;
const LINE_DRAW: u64 = 500;
/// Real pixels -- gap between the heptagon's own edge and where the connecting line
/// actually starts, so it doesn't visually touch/overlap the heptagon.
const LINE_MARGIN: i32 = 20;
/// Real pixels -- estimate of Home's real left edge. `Line` only gets one `Anchor`
/// target (already spent on the heptagon here), so the far end can't also be anchored to
/// Home; same "estimate" class of approximation `navigator.rs`'s `back_r_estimate` uses
/// for the same reason.
const LINE_END_PX: i32 = 173;

const CONTROL_SIZE_PX: i32 = 30;
/// Everything here is real pixels, not a percent -- a percent center paired with a
/// fixed-pixel size means the gap between two elements, in actual pixels, shrinks on a
/// narrower screen (confirmed directly: 6 percentage points apart was only ~22px on the
/// 360px-wide desktop window, less than the shapes' own 30px width, so they overlapped).
/// Fixed pixels for position too means the gap stays whatever it visibly looks like here,
/// regardless of screen width.
const HOME_CENTER_X_PX: i32 = 198;
/// Home/ToC sit close together (small gap between them) so they read as one group of
/// controls; the much larger gap between the heptagon cluster and this one is what the
/// connecting line actually spans.
const TOC_CENTER_X_PX: i32 = 234;
/// Icon fills this fraction of whatever its target polygon's own resolved size actually
/// is -- scales with the polygon instead of a fixed pixel count, the same
/// `anchor().width() * scale` mechanism `navigator.rs`'s `shadow_box` uses.
const ICON_SCALE: f32 = 0.55;
const CONTROL_FADE: u64 = 400;
const CONTROL_MORPH_DELAY: u64 = 150;
const CONTROL_MORPH: u64 = 550;
const ICON_FADE_IN: u64 = 350;
/// Home starts first; ToC follows this long after -- so the two visibly don't move in
/// lockstep, the same "catching up with a lag" read `navigator.rs`'s `SHADOW_LAG` uses for
/// its own staggered layers.
const HEXA_STAGGER: u64 = 220;

/// One shared row -- every element's own *center* lands on `ROW_CENTER_Y_PX` regardless
/// of its own size (see that constant's doc for why: a shared `top` instead would
/// misalign centers whenever sizes differ). Everything real pixels, both position and
/// size -- see `HOME_CENTER_X_PX`'s doc for why position is pixels too, not just size.
fn row_box(center_x_px: i32, size_px: i32) -> Location {
    Location::new().xs(
        center_x_px.px().as_center_x().with(size_px.px().as_width()),
        (ROW_CENTER_Y_PX - size_px / 2)
            .px()
            .as_top()
            .with(size_px.px().as_height()),
    )
}

fn icon_bundle(target: Entity, handle: IconHandles, color: Color) -> impl Sprout {
    Icon::new(handle)
        .color(color)
        .at(Location::new().xs(
            anchor()
                .center_x()
                .as_center_x()
                .with((anchor().width() * ICON_SCALE).as_width()),
            anchor()
                .center_y()
                .as_center_y()
                .with((anchor().height() * ICON_SCALE).as_height()),
        ))
        .elevate(Elevation::up(21))
        .with((
            Anchor::new(target),
            Opacity::new(0.0),
            InteractionPropagation::pass_through(),
        ))
}

/// Px each shadow shifts left of its own front shape -- same offset idea `navigator.rs`'s
/// own shadow layers use, just a lot smaller, since these shapes (30-38px) are a lot
/// smaller than the navigator's own.
const SHADOW_OFFSET_PX: i32 = 6;
const SHADOW_Y_OFFSET_PX: i32 = 4; // px the shadow also sits below the front shape's own row
const SHADOW_COLOR: i32 = 600; // muted stone -- neutral, not a shade of the front shape's own hue

/// Same row as `row_box`, shifted `SHADOW_OFFSET_PX` left and `SHADOW_Y_OFFSET_PX` down.
fn shadow_row_box(center_x_px: i32, size_px: i32) -> Location {
    Location::new().xs(
        (center_x_px - SHADOW_OFFSET_PX)
            .px()
            .as_center_x()
            .with(size_px.px().as_width()),
        (ROW_CENTER_Y_PX - size_px / 2 + SHADOW_Y_OFFSET_PX)
            .px()
            .as_top()
            .with(size_px.px().as_height()),
    )
}

/// One muted shadow copy of a clickable shape's own final form, offset to the left (see
/// `SHADOW_OFFSET_PX`) rather than rotated out of sync with it -- same `rotation` as the
/// front shape's own final rotation, so the two read as one shape with a shifted
/// backdrop, not two shapes spinning independently. The positional offset alone is
/// enough to show the shadow past the front shape's edge; it doesn't need to bring its
/// own corners out from directly behind anymore. Morphs in alongside the front shape's
/// own morph (same `start`/`morph_duration`/`rotation`), one elevation step behind it.
fn build_shadow(
    tree: &mut Tree,
    seq: Entity,
    center_x_px: i32,
    size_px: i32,
    sides: f32,
    rounding: f32,
    rotation: f32,
    start: u64,
    morph_duration: u64,
) {
    let shadow = tree.leaf(
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::stone(SHADOW_COLOR))
            .at(shadow_row_box(center_x_px, size_px))
            .elevate(Elevation::up(19))
            .with(Opacity::new(0.0)),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(shadow)
            .during(seq)
            .start(start)
            .finish(start + morph_duration)
            .eased(Ease::DECELERATE),
    );
    tree.animate(
        Animation::new(Polygon {
            sides,
            rounding,
            rotation,
        })
        .targeting(shadow)
        .during(seq)
        .start(start)
        .finish(start + morph_duration)
        .eased(Ease::EMPHASIS),
    );
}

const LABEL_FONT_SIZE: u32 = 12;
const LABEL_GAP_PX: i32 = 6; // px between a shape's own bottom edge and its label's top
const LABEL_WIDTH_PX: i32 = 74; // fits "github" alone
const COMBINED_LABEL_WIDTH_PX: i32 = 110; // fits "home|contents"
const LABEL_HEIGHT_PX: i32 = 16;

/// A small `stone(500)` label centered under a chrome shape (or between a group of
/// them, given their shared midpoint `center_x_px` and `size_px` as the row height),
/// `LABEL_GAP_PX` below its own bottom edge. `width_px` varies per label -- "github" and
/// the combined "home|contents" need different amounts of room.
fn build_label(
    tree: &mut Tree,
    seq: Entity,
    center_x_px: i32,
    size_px: i32,
    width_px: i32,
    text: &str,
    start: u64,
    fade: u64,
) {
    let label = tree.leaf(
        Text::new(text)
            .size(FontSize::new(LABEL_FONT_SIZE))
            .color(Color::stone(500))
            .at(Location::new().xs(
                center_x_px.px().as_center_x().with(width_px.px().as_width()),
                (ROW_CENTER_Y_PX + size_px / 2 + LABEL_GAP_PX)
                    .px()
                    .as_top()
                    .with(LABEL_HEIGHT_PX.px().as_height()),
            ))
            .elevate(Elevation::up(21))
            .with((HorizontalAlignment::Center, Opacity::new(0.0))),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(label)
            .during(seq)
            .start(start)
            .finish(start + fade)
            .eased(Ease::DECELERATE),
    );
}

pub fn build(tree: &mut Tree, router: Entity) {
    let seq = tree.sequence();
    // spawns sharp-triangle -- a fixed shape rotating in place (especially a rounded,
    // near-circular one) barely reads as motion; the main navigator's own morph never
    // does pure rotation either, it always rotates *while changing shape*, which is what
    // actually makes a spin legible regardless of the final shape's own symmetry.
    let hepta = tree.leaf(
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::cyan(500))
            .at(row_box(HEPTA_CENTER_X_PX, HEPTA_SIZE_PX))
            .elevate(Elevation::up(20))
            .with(Opacity::new(0.0)),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(hepta)
            .during(seq)
            .start(0)
            .finish(FADE_IN)
            .eased(Ease::DECELERATE),
    );
    tree.animate(
        Animation::new(Polygon {
            sides: 7.0,
            rounding: 0.3,
            rotation: SPIN_ROTATION,
        })
        .targeting(hepta)
        .during(seq)
        .start(SPIN_DELAY)
        .finish(SPIN_DELAY + SPIN_IN)
        .eased(Ease::EMPHASIS),
    );
    build_shadow(
        tree,
        seq,
        HEPTA_CENTER_X_PX,
        HEPTA_SIZE_PX,
        7.0,
        0.3,
        SPIN_ROTATION,
        SPIN_DELAY,
        SPIN_IN,
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        build_github_and_line(&mut tree, router, hepta);
    });
}

fn build_github_and_line(tree: &mut Tree, router: Entity, hepta: Entity) {
    let seq = tree.sequence();

    let github = tree.leaf(icon_bundle(hepta, IconHandles::Github, Color::blue(900)));
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(github)
            .during(seq)
            .start(0)
            .finish(GITHUB_FADE)
            .eased(Ease::DECELERATE),
    );
    build_label(
        tree,
        seq,
        HEPTA_CENTER_X_PX,
        HEPTA_SIZE_PX,
        LABEL_WIDTH_PX,
        "github",
        0,
        GITHUB_FADE,
    );

    let line_start = anchor()
        .right()
        .as_x()
        .adjust(LINE_MARGIN)
        .with(anchor().center_y().as_y());
    let line = tree.leaf(
        Line::new(LINE_WEIGHT)
            .color(Color::stone(400))
            .at(Location::new().xs(line_start, line_start))
            .elevate(Elevation::up(20))
            .with(Anchor::new(hepta)),
    );
    tree.animate(
        Animation::new(Location::new().xs(
            line_start,
            LINE_END_PX.px().as_x().with(anchor().center_y().as_y()),
        ))
        .targeting(line)
        .during(seq)
        .start(0)
        .finish(LINE_DRAW)
        .eased(Ease::DECELERATE),
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        build_controls(&mut tree, router);
    });
}

fn build_controls(tree: &mut Tree, router: Entity) {
    build_control(tree, router, HOME_CENTER_X_PX, IconHandles::Terminal, 0, 0);
    build_control(
        tree,
        router,
        TOC_CENTER_X_PX,
        IconHandles::Menu,
        1,
        HEXA_STAGGER,
    );
    // One combined label, not one per control -- Home and ToC sit close enough together
    // (see `TOC_CENTER_X_PX`'s doc) that two separate `LABEL_WIDTH_PX`-wide labels, one
    // per shape, overlapped each other. Timed off ToC's own landing (the later of the
    // two, `HEXA_STAGGER` behind Home's) so it appears once both are actually settled.
    let controls_land = HEXA_STAGGER + CONTROL_MORPH_DELAY + CONTROL_MORPH + ICON_FADE_IN;
    let seq = tree.sequence();
    build_label(
        tree,
        seq,
        (HOME_CENTER_X_PX + TOC_CENTER_X_PX) / 2,
        CONTROL_SIZE_PX,
        COMBINED_LABEL_WIDTH_PX,
        "home|contents",
        controls_land,
        ICON_FADE_IN,
    );
}

/// One Home/ToC control, fully self-contained: fade + shape-morph (sharp triangle growing
/// into the real hexagon while spinning, same technique as the heptagon) first, *then* --
/// only once that's actually finished -- the icon fades in. Doing the icon at the same
/// time as the morph looked wrong: sized against the shape's final resolved box, it reads
/// as oversized next to the shape's own current (still mostly-triangle) silhouette
/// partway through. `stagger` offsets this whole control's start relative to the other
/// one, so the two visibly don't move in lockstep.
fn build_control(
    tree: &mut Tree,
    router: Entity,
    center_x_px: i32,
    icon: IconHandles,
    target_page: usize,
    stagger: u64,
) {
    let seq = tree.sequence();
    let shape = tree.leaf(
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(Color::blue(300))
            .at(row_box(center_x_px, CONTROL_SIZE_PX))
            .elevate(Elevation::up(20))
            .with((
                InteractionListener::new(),
                InteractionShape::Circle,
                Opacity::new(0.0),
            )),
    );
    tree.disable(shape);

    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(shape)
            .during(seq)
            .start(stagger)
            .finish(stagger + CONTROL_FADE)
            .eased(Ease::DECELERATE),
    );
    tree.animate(
        Animation::new(Polygon {
            sides: 6.0,
            rounding: 0.25,
            rotation: SPIN_ROTATION,
        })
        .targeting(shape)
        .during(seq)
        .start(stagger + CONTROL_MORPH_DELAY)
        .finish(stagger + CONTROL_MORPH_DELAY + CONTROL_MORPH)
        .eased(Ease::EMPHASIS),
    );
    build_shadow(
        tree,
        seq,
        center_x_px,
        CONTROL_SIZE_PX,
        6.0,
        0.25,
        SPIN_ROTATION,
        stagger + CONTROL_MORPH_DELAY,
        CONTROL_MORPH,
    );

    tree.sequence_end(seq, move |_: Trigger<OnEnd>, mut tree: Tree| {
        let icon_entity = tree.leaf(icon_bundle(shape, icon, Color::gray(900)));
        let icon_seq = tree.sequence();
        tree.animate(
            Animation::new(Opacity::new(1.0))
                .targeting(icon_entity)
                .during(icon_seq)
                .start(0)
                .finish(ICON_FADE_IN)
                .eased(Ease::DECELERATE),
        );
        tree.enable(shape);
    });

    tree.on_click(
        shape,
        move |_: Trigger<OnClick>, page_index: Query<&PageIndex>, mut tree: Tree| {
            // Router's own no-op guard skips rebuilding the scene when the index doesn't
            // actually change, but it still fires `PageChanged` unconditionally -- which
            // (for index 0) re-triggers `NavigatorLanded`, restarting `home.rs`'s type-in
            // animation from scratch even though nothing really changed. Guard here too,
            // so clicking Home while already on Home is a genuine no-op.
            if page_index.get(router).map(|p| p.0) != Ok(target_page) {
                // direct jump, no ceremony -- distinct on purpose from the inner-site
                // navigator's whole spin/hop/redraw transition.
                tree.write_to(router, PageIndex(target_page));
            }
        },
    );
}
