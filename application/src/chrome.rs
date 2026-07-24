use crate::icons::IconHandles;
use foliage::{
    Anchor, Animation, Color, Ease, EcsExtension, Elevation, Entity, GridExt, Icon,
    InteractionListener, InteractionPropagation, InteractionShape, Line, Location, OnClick, OnEnd,
    Opacity, PageIndex, Polygon, Sprout, Tree, Trigger, anchor,
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
        (ROW_CENTER_Y_PX - size_px / 2).px().as_top().with(size_px.px().as_height()),
    )
}

fn icon_bundle(target: Entity, handle: IconHandles, color: Color) -> impl Sprout {
    Icon::new(handle)
        .color(color)
        .at(Location::new().xs(
            anchor().center_x().as_center_x().with((anchor().width() * ICON_SCALE).as_width()),
            anchor().center_y().as_center_y().with((anchor().height() * ICON_SCALE).as_height()),
        ))
        .elevate(Elevation::up(21))
        .with((
            Anchor::new(target),
            Opacity::new(0.0),
            InteractionPropagation::pass_through(),
        ))
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
    build_control(tree, router, TOC_CENTER_X_PX, IconHandles::Menu, 1, HEXA_STAGGER);
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

    tree.on_click(shape, move |_: Trigger<OnClick>, mut tree: Tree| {
        // direct jump, no ceremony -- distinct on purpose from the inner-site navigator's
        // whole spin/hop/redraw transition.
        tree.write_to(router, PageIndex(target_page));
    });
}
