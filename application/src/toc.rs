use crate::entry::AppRouter;
use crate::routes::ROUTE_NAMES;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::lifecycle::Insert;
use foliage::bevy_ecs::query::With;
use foliage::{
    Animation, Color, CurrentInteraction, Dragged, Ease, EcsExtension, Elevation, Entity, FontSize,
    Grid, GridExt, HorizontalAlignment, InteractionListener, InteractionPropagation,
    InteractionShape, Leaf, LeafSprout, Line, Location, Logical, OnClick, OnEnd, Opacity,
    PageIndex, Polygon, Query, Res, ScrollProgress, ScrollTo, Section, Sprout, Text, TextValue,
    Tree, Trigger, VerticalAlignment, component,
};

/// A custom card-like element, not the generic `Card` composite -- purpose-built for one
/// job: represent one selectable route in the contents list. A morphed-in heptagon
/// (same triangle -> `sides: 7.0` technique used everywhere else in this app) reads as
/// the item's own icon/focal point, a title names the route, and a description sits
/// underneath -- one `Text` entity, not two hand-split lines: given a box tall enough for
/// more than one line, `fontdue`'s own layout wraps it, so a fixed line split isn't
/// needed (and was clipping whenever the actual wrap point didn't match the hand-picked
/// one). Config (`.title(..)`/`.description(..)`) lands on the root via
/// `ContentsItemConfig`; `build`'s `react` spawns the three children once on first
/// insert and just updates their `TextValue`s on any later config write, same pattern
/// `Button`'s own `TextValue`/`IconValue` reactions use.
#[component]
#[derive(Copy, Clone)]
pub struct ContentsItem {}

#[component]
#[derive(Clone, Default)]
struct ContentsItemConfig {
    title: String,
    description: String,
    /// The router page index this item jumps to once clicks are wired up (not yet --
    /// this just gives the sprout somewhere to carry the value).
    target_page: usize,
}

const HEPTA_ROUNDING: f32 = 0.15; // same softening the rest of the app's heptagons use
const HEPTA_MORPH: u64 = 700;
const HEPTA_WIDTH_PCT: f32 = 92.0; // the backdrop -- fills almost the whole card
const HEPTA_HEIGHT_PCT: f32 = 97.0;
const HEPTA_TOP_PCT: f32 = 1.5;

// A heptagon isn't a rectangle -- it narrows fast above/below its own vertical center,
// so text placed near the top or bottom of its bounding box (as this used to be) spills
// past the actual green silhouette onto the dark page background behind it, unreadable,
// even when the text itself isn't clipped by its own box. Title and description together
// span ~14%-85.5% of the card -- still inside the heptagon's wider middle band.
// Same absolute pixel sizes/gap these boxes always had (45.6px title, 7.6px gap, 114px
// desc, on a 380px card) -- just re-percented against the smaller `CARD_HEIGHT_PX(235)`
// and shifted up to start near the card's own top instead of at the old 28%, which on
// this card would push the block (and its unchanged absolute height) past the bottom edge.
const TITLE_FONT_SIZE: u32 = 20;
const TITLE_TOP_PCT: f32 = 14.4;
const TITLE_HEIGHT_PCT: f32 = 19.4;

const DESC_FONT_SIZE: u32 = 13;
const DESC_TOP_PCT: f32 = 37.0;
const DESC_HEIGHT_PCT: f32 = 48.5;
const DESC_WIDTH_PCT: f32 = 76.0; // narrower than before (was 94%) -- same reason, safe margin either side

/// On click: the heptagon spins out (a few full turns, accelerating, same
/// "spin while changing" idea `navigator.rs`'s own morphs use) while the whole card fades
/// with it; only once that finishes does the actual route switch happen -- an instant
/// jump the moment you click reads as the click doing nothing for a beat, not a
/// deliberate transition.
const SPIN_OUT_DURATION: u64 = 500;
const SPIN_OUT_TURNS: f32 = 3.0 * 2.0 * std::f32::consts::PI;

impl ContentsItem {
    pub fn new() -> ContentsItemSprout {
        ContentsItemSprout::default()
    }
}

/// Same family each `chapters/*.rs` page already uses for its own placeholder shape --
/// a card reads as that chapter's own color before you ever click into it, not just one
/// of identical green cards.
fn chapter_color(target_page: usize, shade: i32) -> Color {
    match target_page {
        2 => Color::blue(shade),
        3 => Color::indigo(shade),
        4 => Color::cyan(shade),
        5 => Color::teal(shade),
        6 => Color::orange(shade),
        7 => Color::amber(shade),
        8 => Color::rose(shade),
        9 => Color::lime(shade),
        10 => Color::purple(shade),
        _ => Color::green(shade),
    }
}

#[derive(Default)]
pub struct ContentsItemSprout {
    leaf: LeafSprout,
    config: ContentsItemConfig,
}
impl Sprout for ContentsItemSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            ContentsItem {},
            self.config,
            Grid::new(1.col().gap(0), 1.row().gap(0)),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        let mut children: Option<(Entity, Entity, Entity)> = None;
        tree.react::<ContentsItemConfig, _>(
            this,
            move |trigger: Trigger<Insert, ContentsItemConfig>,
                  configs: Query<&ContentsItemConfig>,
                  mut tree: Tree| {
                let e = trigger.entity;
                let config = configs.get(e).unwrap().clone();
                if let Some((_hepta, title, desc)) = children {
                    tree.write_to(title, TextValue(config.title.clone()));
                    tree.write_to(desc, TextValue(config.description.clone()));
                } else {
                    let hepta = tree.branch(
                        e,
                        Polygon::new()
                            .sides(3.0)
                            .rounding(0.0)
                            .rotation(0.0)
                            .color(chapter_color(config.target_page, 400))
                            .at(Location::new().xs(
                                50.pct()
                                    .as_center_x()
                                    .with(HEPTA_WIDTH_PCT.pct().as_width()),
                                HEPTA_TOP_PCT
                                    .pct()
                                    .as_top()
                                    .with(HEPTA_HEIGHT_PCT.pct().as_height()),
                            ))
                            .elevate(Elevation::up(1))
                            .with((
                                Opacity::new(0.0),
                                InteractionListener::new(),
                                InteractionShape::Circle,
                            )),
                    );
                    let morph_seq = tree.sequence();
                    tree.animate(
                        Animation::new(Opacity::new(1.0))
                            .targeting(hepta)
                            .during(morph_seq)
                            .start(0)
                            .finish(HEPTA_MORPH)
                            .eased(Ease::Linear),
                    );
                    tree.animate(
                        Animation::new(Polygon {
                            sides: 7.0,
                            rounding: HEPTA_ROUNDING,
                            rotation: 0.0,
                        })
                        .targeting(hepta)
                        .during(morph_seq)
                        .start(0)
                        .finish(HEPTA_MORPH)
                        .eased(Ease::DECELERATE),
                    );

                    let title = tree.branch(
                        e,
                        Text::new(config.title.clone())
                            .size(FontSize::new(TITLE_FONT_SIZE))
                            .color(chapter_color(config.target_page, 950))
                            .at(Location::new().xs(
                                50.pct().as_center_x().with(90.0.pct().as_width()),
                                TITLE_TOP_PCT
                                    .pct()
                                    .as_top()
                                    .with(TITLE_HEIGHT_PCT.pct().as_height()),
                            ))
                            .elevate(Elevation::up(2))
                            .with((
                                HorizontalAlignment::Center,
                                VerticalAlignment::Middle,
                                InteractionPropagation::pass_through(),
                            )),
                    );
                    let desc = tree.branch(
                        e,
                        Text::new(config.description.clone())
                            .size(FontSize::new(DESC_FONT_SIZE))
                            .color(chapter_color(config.target_page, 800))
                            .at(Location::new().xs(
                                50.pct().as_center_x().with(DESC_WIDTH_PCT.pct().as_width()),
                                DESC_TOP_PCT
                                    .pct()
                                    .as_top()
                                    .with(DESC_HEIGHT_PCT.pct().as_height()),
                            ))
                            .elevate(Elevation::up(2))
                            .with((
                                HorizontalAlignment::Center,
                                VerticalAlignment::Middle,
                                InteractionPropagation::pass_through(),
                            )),
                    );
                    // registered here, not up front -- `hepta`/`title`/`desc_1`/`desc_2`
                    // exist as plain `Entity` values by this point, so the closure just
                    // captures them directly instead of needing shared/interior-mutable
                    // state to reach across from an earlier registration. This whole
                    // branch only runs once (the first config insert), so `on_click`
                    // only ever gets registered once too. Registered on `hepta`, not
                    // `this` -- `hepta` is the real visual/interactive surface (it has
                    // its own `InteractionListener`); `this` never had one, so title/desc
                    // being `pass_through()` wasn't enough on its own -- without also
                    // making `hepta` the actual listener, a click that fell through the
                    // text just landed on `hepta`'s own default (grab, no listener)
                    // propagation instead and got silently swallowed there.
                    tree.on_click(
                        hepta,
                        move |_: Trigger<OnClick>,
                              configs: Query<&ContentsItemConfig>,
                              routers: Query<Entity, With<AppRouter>>,
                              mut tree: Tree| {
                            let Ok(router) = routers.single() else {
                                return;
                            };
                            let target_page = configs.get(this).unwrap().target_page;

                            let spin_seq = tree.sequence();
                            for target in [hepta, title, desc] {
                                tree.animate(
                                    Animation::new(Opacity::new(0.0))
                                        .targeting(target)
                                        .during(spin_seq)
                                        .start(0)
                                        .finish(SPIN_OUT_DURATION)
                                        .eased(Ease::Linear),
                                );
                            }
                            tree.animate(
                                Animation::new(Polygon {
                                    sides: 7.0,
                                    rounding: HEPTA_ROUNDING,
                                    rotation: SPIN_OUT_TURNS,
                                })
                                .targeting(hepta)
                                .during(spin_seq)
                                .start(0)
                                .finish(SPIN_OUT_DURATION)
                                .eased(Ease::ACCELERATE),
                            );
                            tree.sequence_end(
                                spin_seq,
                                move |_: Trigger<OnEnd>, mut tree: Tree| {
                                    tree.write_to(router, PageIndex(target_page));
                                },
                            );
                        },
                    );

                    children = Some((hepta, title, desc));
                }
            },
        );
    }
}
impl ContentsItemSprout {
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.config.title = t.into();
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.config.description = d.into();
        self
    }
    pub fn target_page(mut self, page: usize) -> Self {
        self.config.target_page = page;
        self
    }
}

/// The shared vertical clearance every content area (this page's own scrollable
/// `viewport`, and `chapters::window_frame`, used by every chapter page) needs to stay
/// clear of the persistent chrome bar and forward/back navigator -- canonical here since
/// `toc` is the page that actually scrolls its content and first needed this measured
/// precisely; `chapters::window_frame` imports these rather than keeping its own
/// independently-derived (and previously drifted) copies.
///
/// Real pixels, not a percent -- `chrome`'s whole row is fixed-pixel by design (doesn't
/// shrink on a short viewport). Chrome's *true* bottom edge isn't the heptagon icon itself
/// (`ROOT_TOP_PX(8) + HEPTA_SIZE_PX(38)` = 46) -- `chrome.rs`'s `build_label` puts the
/// "github" caption *below* that icon too: `ROW_CENTER_Y_PX(27) + HEPTA_SIZE_PX/2(19) +
/// LABEL_GAP_PX(6) + LABEL_HEIGHT_PX(16)` = 68. `+ 10` margin past that.
pub(crate) const CONTENT_AREA_TOP_PX: i32 = 78;
/// Tied to `navigator.rs`'s own resting geometry, not an arbitrary round percent: its
/// polygon's top edge is `REST_CENTER_Y(91.0) - HEIGHT(11.0) / 2` = 85.5% of the screen.
/// Using any other percent here left a gap that grows with screen height (a fixed percent
/// difference is a growing *pixel* gap on a taller screen) -- matching nav's own percent
/// exactly cancels that out, so the only remaining gap is the fixed-px one below.
pub(crate) const CONTENT_AREA_BOTTOM_PCT: f32 = 85.5;
/// `navigator.rs`'s route-name label sits a fixed `LABEL_GAP_PX(10) + LABEL_HEIGHT_PX(24)`
/// = 34px above the polygon's top edge (`navigator.rs`'s `label_bottom`) regardless of
/// viewport height, since both are real pixels, not percent. `+ 8` margin past that.
pub(crate) const CONTENT_AREA_BOTTOM_CLEARANCE_PX: i32 = 42;

const CARD_WIDTH_PX: i32 = 240; // was 200 -- more room lowers the wrapped-line count too
/// Every `Polygon` (including `hepta` below) carries a built-in `AspectRatio::new().xs(1.0)`
/// (see `foliage_proper`'s `PolygonSprout::root`), which clamps its resolved box to
/// `min(width, height)` and *centers* the shrunk square inside whatever box it was given
/// (`AspectRatio::constrain`) -- it never stretches into a non-square box. `hepta`'s own box
/// is `HEPTA_WIDTH_PCT(92%)` of `CARD_WIDTH_PX` wide vs `HEPTA_HEIGHT_PCT(97%)` of this tall;
/// as long as this stays above roughly `(92/97) * CARD_WIDTH_PX` (~228), the heptagon is
/// permanently width-bound at ~221x221px -- *any* value above that just adds equal dead
/// space above and below the already-fixed-size shape, not a bigger heptagon. The old `380`
/// (raised from `220`, then `300`, chasing what looked like a clipping fix) was actually
/// widening that dead zone, not helping text -- title/desc are plain percent-of-card boxes,
/// unaffected by the polygon's own aspect lock, so the real fix for their clipping was
/// always available at any card height. Set close to the true square size instead, so the
/// heptagon fills its card with no leftover gap before the next one.
const CARD_HEIGHT_PX: i32 = 235;
const GRID_GAP_PX: i32 = 16;
// `Md`'s 2 columns sit side by side in the same viewport width `Xs`'s single column
// has to itself, so the same 16px that reads fine as vertical breathing room between
// stacked cards reads as cramped between two cards side by side -- a wider gap just for
// that horizontal split, row spacing (and `Xs`) unchanged.
const MD_COL_GAP_PX: i32 = 32;

/// One entry per route, other than home/contents themselves -- `(target_page,
/// description)`. Title isn't duplicated here -- it comes from `ROUTE_NAMES[target_page]`
/// (see `toc`'s own body), the same single source of truth `navigator.rs` uses for its
/// forward/back labels, so this list and the nav labels can't drift out of sync the way
/// `navigator.rs`'s own now-removed local copy just did. The chapter walkthrough of how
/// `foliage_proper` builds up a composite, in learning order -- see `crate::chapters`'
/// own doc comment.
const ROUTES: &[(usize, &str)] = &[
    (
        2,
        "Where it sits: percent/px/point, as a live value that can change",
    ),
    (3, "Stacking order between shapes, and changing it live"),
    (4, "The same percentages, against a real, visible parent"),
    (
        5,
        "How a parent divides into columns/rows so children can address cells",
    ),
    (
        6,
        "Position relative to another entity's live box, not just your parent",
    ),
    (
        7,
        "A component as a tweenable value, interpolating over time",
    ),
    (
        8,
        "Chaining animations together and reacting once they all finish",
    ),
    (
        9,
        "Clicks and hit-testing: listeners, propagation, pass-through",
    ),
    (
        10,
        "Font size, per-character color, and the monospace grid's own pitch",
    ),
    (
        11,
        "One declaration, resolved differently per breakpoint -- nothing respawns",
    ),
    (
        12,
        "A window onto content taller than itself, clipped at its own edge",
    ),
    (
        13,
        "Release fast and it keeps going; press once and it stops",
    ),
];

const COLS_XS: i32 = 1;
const COLS_MD: i32 = 2;
const COLS_LG: i32 = 3;

fn container_width_px(cols: i32, col_gap: i32) -> i32 {
    cols * CARD_WIDTH_PX + (cols - 1) * col_gap
}
fn container_height_px(rows: i32) -> i32 {
    rows * CARD_HEIGHT_PX + (rows - 1) * GRID_GAP_PX
}

/// Reachable via the global chrome's explicit ToC button (and, since `Router` treats
/// every route the same, via the main forward/back navigator too). One `ContentsItem`
/// per route in `ROUTES`, arranged in a `Grid` that's a single column below `Md`, two
/// columns at `Md`+ (`Layout::MD` = 600px), and three at `Lg`+ (`Layout::LG` = 840px) --
/// both the `content` `Grid`'s own column/row counts and each card's own `.col()`/`.row()`
/// index get explicit `.xs(..)`/`.md(..)`/`.lg(..)` placements. There's no auto-flow in
/// this engine: spanning items (used elsewhere for alignment, not every layout is 1 item
/// per cell) make "place the next item in the next free cell" ambiguous, so placement
/// stays explicit and hand-computed instead.
///
/// Three levels deep, not two -- `viewport` (sized to the content area below chrome,
/// carrying the real scrollable `View`) holding `content` (sized to the full card-stack
/// height, structurally overflowing `viewport` the same way `viewport` itself used to
/// overflow `slot` before this split) holding the cards. `build_scrollbar` spawns as
/// `viewport`'s *sibling*, not its child: any child of a `View`-holder gets that view's
/// scroll offset subtracted from its own resolved position on every cascade (see
/// `grid/location.rs`'s `resolution.section.position -= view.offset`), so a scrollbar
/// nested *inside* the thing it scrolls would itself scroll away and out of view the
/// moment you used it -- it needs to sit outside the view it's driving.
pub fn toc(tree: &mut Tree, slot: Entity) {
    let rows_xs = ROUTES.len() as i32;
    let rows_md = ROUTES.len() as i32 / COLS_MD + ROUTES.len() as i32 % COLS_MD;
    let rows_lg = (ROUTES.len() as i32 + COLS_LG - 1) / COLS_LG; // proper ceil -- `rows_md`'s
    // own `len/cols + len%cols` only happens to work out for `COLS_MD`'s specific 8/2 split

    let viewport = tree.branch(
        slot,
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                CONTENT_AREA_TOP_PX.px().as_top().with(
                    CONTENT_AREA_BOTTOM_PCT
                        .pct()
                        .as_bottom()
                        .adjust(-CONTENT_AREA_BOTTOM_CLEARANCE_PX),
                ),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
    );
    let content = tree.branch(
        viewport,
        Leaf::sprout()
            .at(Location::new()
                .xs(
                    50.pct()
                        .as_center_x()
                        .with(container_width_px(COLS_XS, GRID_GAP_PX).px().as_width()),
                    0.px()
                        .as_top()
                        .with(container_height_px(rows_xs).px().as_height()),
                )
                .md(
                    50.pct()
                        .as_center_x()
                        .with(container_width_px(COLS_MD, MD_COL_GAP_PX).px().as_width()),
                    0.px()
                        .as_top()
                        .with(container_height_px(rows_md).px().as_height()),
                )
                .lg(
                    50.pct()
                        .as_center_x()
                        .with(container_width_px(COLS_LG, MD_COL_GAP_PX).px().as_width()),
                    0.px()
                        .as_top()
                        .with(container_height_px(rows_lg).px().as_height()),
                ))
            .elevate(Elevation::up(1))
            .with(
                Grid::new(
                    COLS_XS.col().gap(GRID_GAP_PX),
                    rows_xs.row().gap(GRID_GAP_PX),
                )
                .md(
                    COLS_MD.col().gap(MD_COL_GAP_PX),
                    rows_md.row().gap(GRID_GAP_PX),
                )
                .lg(
                    COLS_LG.col().gap(MD_COL_GAP_PX),
                    rows_lg.row().gap(GRID_GAP_PX),
                ),
            ),
    );

    for (i, &(target_page, desc)) in ROUTES.iter().enumerate() {
        let i = i as i32;
        let col_xs = 1;
        let row_xs = i + 1;
        let col_md = i % COLS_MD + 1;
        let row_md = i / COLS_MD + 1;
        let col_lg = i % COLS_LG + 1;
        let row_lg = i / COLS_LG + 1;

        // `ROUTE_NAMES` is lowercase (matches the nav labels' own convention); the card
        // title capitalizes just its own copy for display, not the shared data itself.
        let name = ROUTE_NAMES[target_page];
        let title = name
            .get(..1)
            .map(|c| c.to_uppercase() + &name[1..])
            .unwrap_or_default();

        tree.branch(
            content,
            ContentsItem::new()
                .title(title)
                .description(desc)
                .target_page(target_page)
                .at(Location::new()
                    .xs(
                        col_xs.col().as_left().with(col_xs.col().as_right()),
                        row_xs.row().as_top().with(row_xs.row().as_bottom()),
                    )
                    .md(
                        col_md.col().as_left().with(col_md.col().as_right()),
                        row_md.row().as_top().with(row_md.row().as_bottom()),
                    )
                    .lg(
                        col_lg.col().as_left().with(col_lg.col().as_right()),
                        row_lg.row().as_top().with(row_lg.row().as_bottom()),
                    ))
                .elevate(Elevation::up(1)),
        );
    }

    build_scrollbar(tree, slot, viewport);
}

const SCROLLBAR_RIGHT_INSET_PX: i32 = 14; // from `parent`'s own right edge
const SCROLLBAR_HIT_WIDTH_PX: i32 = 44; // wider than the visual track -- an easier drag/tap target
const SCROLLBAR_TRACK_TOP_PCT: f32 = 26.0;
const SCROLLBAR_TRACK_BOTTOM_PCT: f32 = 78.0;
const SCROLLBAR_TRACK_WEIGHT: i32 = 2;
const SCROLLBAR_KNOB_VISUAL_SIZE_PX: i32 = 28; // the drawn heptagon -- stays dainty
// invisible, larger than the drawn knob -- the real listener/drag target, Material's
// "small visible dot, bigger invisible touch target" split. The drawn knob itself is
// just along for the ride (`InteractionPropagation::pass_through()`), not a listener.
const SCROLLBAR_KNOB_HIT_SIZE_PX: i32 = 44;
const SCROLLBAR_KNOB_ROUNDING: f32 = 0.15; // same softening every other heptagon in this app uses
// left + down -- this app's one established shadow direction (`chrome.rs`'s
// `build_shadow`, `navigator.rs`'s `shadow_box`), not right.
const SCROLLBAR_SHADOW_OFFSET_PX: i32 = 3;

/// A vertical, hepta-knobbed scrollbar along the page's right edge -- `ToC` is the only
/// page in the app that scrolls, so this is purpose-built here rather than a reusable
/// composite (same "custom, not the generic thing" call `ContentsItem` already made over
/// `Card`). Structurally the same split the built-in `Slider` uses (a root that's the
/// click-to-seek surface, a thin visual track, a separately-listened knob for drag) just
/// rotated to vertical and with the knob swapped for a heptagon + shadow to match this
/// app's own visual language, and driven by `view_target`'s own [`ScrollProgress`]/
/// [`ScrollTo`] instead of a `Progress` component of its own -- there's already exactly
/// one source of truth for "how far scrolled" (`view_target`'s `View`), so the knob reads
/// it directly rather than keeping a second, shadow copy that could drift out of sync.
/// `parent` (where this actually spawns) is deliberately a *different* entity than
/// `view_target` (whose scroll it reads/drives) -- see `toc`'s own doc comment for why
/// this can't itself be a descendant of the view it scrolls.
fn build_scrollbar(tree: &mut Tree, parent: Entity, view_target: Entity) {
    let root = tree.branch(
        parent,
        Leaf::sprout()
            .at(Location::new().xs(
                100.pct()
                    .as_right()
                    .adjust(-SCROLLBAR_RIGHT_INSET_PX)
                    .with(SCROLLBAR_HIT_WIDTH_PX.px().as_width()),
                SCROLLBAR_TRACK_TOP_PCT.pct().as_top().with(
                    (SCROLLBAR_TRACK_BOTTOM_PCT - SCROLLBAR_TRACK_TOP_PCT)
                        .pct()
                        .as_height(),
                ),
            ))
            .elevate(Elevation::up(4))
            .with((
                Grid::new(1.col().gap(0), 1.row().gap(0)),
                InteractionListener::new(),
                InteractionShape::Rectangle,
            )),
    );
    tree.branch(
        root,
        Line::new(SCROLLBAR_TRACK_WEIGHT)
            .color(Color::stone(700))
            .at(Location::new().xs(
                50.pct().as_x().with(0.pct().as_y()),
                50.pct().as_x().with(100.pct().as_y()),
            ))
            .elevate(Elevation::up(1))
            .with(InteractionPropagation::pass_through()),
    );
    // no `Location` on either -- both are position-dependent (the knob's own scroll
    // progress), set by the `ScrollProgress` reaction's first fire, same "static skeleton,
    // data-dependent placement lands via the reaction" split `Slider`'s own `fill` uses.
    let shadow = tree.branch(
        root,
        Polygon::new()
            .sides(7.0)
            .rounding(SCROLLBAR_KNOB_ROUNDING)
            .rotation(0.0)
            .color(Color::stone(900))
            .elevate(Elevation::up(2))
            .with(InteractionPropagation::pass_through()),
    );
    let knob = tree.branch(
        root,
        Polygon::new()
            .sides(7.0)
            .rounding(SCROLLBAR_KNOB_ROUNDING)
            .rotation(0.0)
            .color(Color::orange(400))
            .elevate(Elevation::up(3))
            .with(InteractionPropagation::pass_through()),
    );
    // invisible -- see `SCROLLBAR_KNOB_HIT_SIZE_PX`'s own doc. Elevated above the drawn
    // knob so it always wins the hit-test regardless of draw order, though it being
    // invisible means nothing renders differently either way.
    let knob_hit = tree.branch(
        root,
        Leaf::sprout()
            .elevate(Elevation::up(4))
            .with((InteractionListener::new(), InteractionShape::Circle)),
    );

    // render: `view_target`'s scroll position -> knob/shadow placement. Fires once at
    // spawn (parking the knob at the top) and again every time the scroll changes,
    // regardless of whether that came from dragging this knob, wheeling over the content
    // directly, or a future unrelated `ScrollTo` write -- one door, same as `Slider`'s own
    // `Progress` reaction.
    tree.react::<ScrollProgress, _>(
        view_target,
        move |trigger: Trigger<Insert, ScrollProgress>,
              progress: Query<&ScrollProgress>,
              sections: Query<&Section<Logical>>,
              mut tree: Tree| {
            let y = progress.get(trigger.entity).unwrap().y();
            // half the *hit* circle's own size (the larger of the two), as a percent of
            // `root`'s live height -- without this, mapping `y` straight onto 0%..100%
            // centers it exactly on `root`'s own top/bottom edge at the extremes, and
            // `root` (its immediate `Stem` parent) clips its children to its own bounds.
            // Margining by the bigger hit circle rather than the smaller drawn knob keeps
            // both comfortably inside `root` at either end -- known, accepted tradeoff:
            // the *visible* knob shares this same (larger-than-it-needs) margin, so it
            // visibly stops a little short of the true top/bottom rather than reaching
            // all the way to `root`'s own edge, even though its own smaller size would
            // technically allow it to travel further. Reads fine in practice; the
            // alternative (a second, knob-sized margin just for the visible one) would
            // desync the drawn knob's position from the invisible hit circle it's
            // supposed to sit on top of, which would look worse than the small unused gap.
            let bounds = sections.get(root).unwrap();
            let margin_pct = (SCROLLBAR_KNOB_HIT_SIZE_PX as f32 / 2.0 / bounds.height() * 100.0)
                .clamp(0.0, 50.0);
            let center_y_pct = margin_pct + y * (100.0 - 2.0 * margin_pct);
            tree.write_to(
                knob,
                Location::new().xs(
                    50.pct()
                        .as_center_x()
                        .with(SCROLLBAR_KNOB_VISUAL_SIZE_PX.px().as_width()),
                    center_y_pct
                        .pct()
                        .as_center_y()
                        .with(SCROLLBAR_KNOB_VISUAL_SIZE_PX.px().as_height()),
                ),
            );
            tree.write_to(
                shadow,
                Location::new().xs(
                    50.pct()
                        .as_center_x()
                        .adjust(-SCROLLBAR_SHADOW_OFFSET_PX)
                        .with(SCROLLBAR_KNOB_VISUAL_SIZE_PX.px().as_width()),
                    center_y_pct
                        .pct()
                        .as_center_y()
                        .adjust(SCROLLBAR_SHADOW_OFFSET_PX)
                        .with(SCROLLBAR_KNOB_VISUAL_SIZE_PX.px().as_height()),
                ),
            );
            tree.write_to(
                knob_hit,
                Location::new().xs(
                    50.pct()
                        .as_center_x()
                        .with(SCROLLBAR_KNOB_HIT_SIZE_PX.px().as_width()),
                    center_y_pct
                        .pct()
                        .as_center_y()
                        .with(SCROLLBAR_KNOB_HIT_SIZE_PX.px().as_height()),
                ),
            );
        },
    );

    // input: drag the (invisible) hit circle, or tap anywhere on the track to seek there
    // -- both go through the same `ScrollTo` door `extent_check` resolves against
    // `view_target`'s own live `View`/`Section`, so neither one can push the knob further
    // than a real drag over the content itself would ever be allowed to scroll.
    tree.subscribe(
        knob_hit,
        move |_: Trigger<Dragged>,
              interaction: Res<CurrentInteraction>,
              sections: Query<&Section<Logical>>,
              mut tree: Tree| {
            let bounds = sections.get(root).unwrap();
            let pct = ((interaction.click().current.top() - bounds.top()) / bounds.height())
                .clamp(0.0, 1.0);
            tree.write_to(view_target, ScrollTo::y(pct));
        },
    );
    tree.on_click(
        root,
        move |_: Trigger<OnClick>,
              interaction: Res<CurrentInteraction>,
              sections: Query<&Section<Logical>>,
              mut tree: Tree| {
            let bounds = sections.get(root).unwrap();
            let pct = ((interaction.click().current.top() - bounds.top()) / bounds.height())
                .clamp(0.0, 1.0);
            tree.write_to(view_target, ScrollTo::y(pct));
        },
    );
}
