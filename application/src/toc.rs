use crate::entry::AppRouter;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::lifecycle::Insert;
use foliage::bevy_ecs::query::With;
use foliage::{
    Animation, Color, EcsExtension, Ease, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, InteractionListener, InteractionPropagation, InteractionShape, Leaf,
    LeafSprout, Location, OnClick, OnEnd, Opacity, PageIndex, Polygon, Query, Sprout, Text,
    TextValue, Tree, Trigger, VerticalAlignment, component,
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
const HEPTA_HEIGHT_PCT: f32 = 92.0;
const HEPTA_TOP_PCT: f32 = 4.0;

const TITLE_FONT_SIZE: u32 = 20;
const TITLE_TOP_PCT: f32 = 32.0; // middle-top-ish of the heptagon backdrop
const TITLE_HEIGHT_PCT: f32 = 14.0;

const DESC_FONT_SIZE: u32 = 13;
const DESC_TOP_PCT: f32 = 50.0; // spans what used to be two separate hand-split lines
const DESC_HEIGHT_PCT: f32 = 25.0; // -- enough room for fontdue to wrap into as many as it needs

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
                            .color(Color::green(400))
                            .at(Location::new().xs(
                                50.pct().as_center_x().with(HEPTA_WIDTH_PCT.pct().as_width()),
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
                            .color(Color::green(950))
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
                            .color(Color::green(800))
                            .at(Location::new().xs(
                                50.pct().as_center_x().with(94.0.pct().as_width()),
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

const CARD_WIDTH_PX: i32 = 200;
const CARD_HEIGHT_PX: i32 = 220;
const GRID_GAP_PX: i32 = 16;

/// One entry per route, other than home/contents themselves -- `(target_page, title,
/// description)`.
const ROUTES: &[(usize, &str, &str)] = &[
    (2, "Next", "A static hexagon, just to prove routing works"),
    (3, "Third", "One more hop -- a pentagon, for variety"),
];

const COLS_XS: i32 = 1;
const COLS_MD: i32 = 2;

fn container_width_px(cols: i32) -> i32 {
    cols * CARD_WIDTH_PX + (cols - 1) * GRID_GAP_PX
}
fn container_height_px(rows: i32) -> i32 {
    rows * CARD_HEIGHT_PX + (rows - 1) * GRID_GAP_PX
}

/// Reachable via the global chrome's explicit ToC button (and, since `Router` treats
/// every route the same, via the main forward/back navigator too). One `ContentsItem`
/// per route in `ROUTES`, arranged in a `Grid` that's a single column below `Md` and two
/// columns at `Md`+ (`Layout::MD` = 600px) -- both the container `Grid`'s own
/// column/row counts and each card's own `.col()`/`.row()` index get explicit
/// `.xs(..)`/`.md(..)` placements. There's no auto-flow in this engine: spanning items
/// (used elsewhere for alignment, not every layout is 1 item per cell) make "place the
/// next item in the next free cell" ambiguous, so placement stays explicit and
/// hand-computed instead.
pub fn toc(tree: &mut Tree, slot: Entity) {
    let rows_xs = ROUTES.len() as i32;
    let rows_md = ROUTES.len() as i32 / COLS_MD + ROUTES.len() as i32 % COLS_MD;

    let container = tree.branch(
        slot,
        Leaf::sprout()
            .at(Location::new()
                .xs(
                    50.pct()
                        .as_center_x()
                        .with(container_width_px(COLS_XS).px().as_width()),
                    50.pct()
                        .as_center_y()
                        .with(container_height_px(rows_xs).px().as_height()),
                )
                .md(
                    50.pct()
                        .as_center_x()
                        .with(container_width_px(COLS_MD).px().as_width()),
                    50.pct()
                        .as_center_y()
                        .with(container_height_px(rows_md).px().as_height()),
                ))
            .elevate(Elevation::up(1))
            .with(
                Grid::new(COLS_XS.col().gap(GRID_GAP_PX), rows_xs.row().gap(GRID_GAP_PX))
                    .md(COLS_MD.col().gap(GRID_GAP_PX), rows_md.row().gap(GRID_GAP_PX)),
            ),
    );

    for (i, &(target_page, title, desc)) in ROUTES.iter().enumerate() {
        let i = i as i32;
        let col_xs = 1;
        let row_xs = i + 1;
        let col_md = i % COLS_MD + 1;
        let row_md = i / COLS_MD + 1;

        tree.branch(
            container,
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
                    ))
                .elevate(Elevation::up(1)),
        );
    }
}
