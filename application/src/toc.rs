use crate::entry::AppRouter;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::lifecycle::Insert;
use foliage::bevy_ecs::query::With;
use foliage::{
    Animation, Color, EcsExtension, Ease, Elevation, Entity, FontSize, Grid, GridExt,
    HorizontalAlignment, InteractionListener, InteractionPropagation, InteractionShape,
    LeafSprout, Location, OnClick, OnEnd, Opacity, PageIndex, Polygon, Query, Sprout, Text,
    TextValue, Tree, Trigger, VerticalAlignment, component,
};

/// A custom card-like element, not the generic `Card` composite -- purpose-built for one
/// job: represent one selectable route in the contents list. A morphed-in heptagon
/// (same triangle -> `sides: 7.0` technique used everywhere else in this app) reads as
/// the item's own icon/focal point, a title names the route, and a two-line description
/// sits underneath. Config (`.title(..)`/`.description(line1, line2)`) lands on the root
/// via `ContentsItemConfig`; `build`'s `react` spawns the three children once on first
/// insert and just updates their `TextValue`s on any later config write, same pattern
/// `Button`'s own `TextValue`/`IconValue` reactions use.
#[component]
#[derive(Copy, Clone)]
pub struct ContentsItem {}

#[component]
#[derive(Clone, Default)]
struct ContentsItemConfig {
    title: String,
    desc_1: String,
    desc_2: String,
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
const DESC_LINE_1_TOP_PCT: f32 = 50.0;
const DESC_LINE_2_TOP_PCT: f32 = 63.0;
const DESC_HEIGHT_PCT: f32 = 12.0;

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
        let mut children: Option<(Entity, Entity, Entity, Entity)> = None;
        tree.react::<ContentsItemConfig, _>(
            this,
            move |trigger: Trigger<Insert, ContentsItemConfig>,
                  configs: Query<&ContentsItemConfig>,
                  mut tree: Tree| {
                let e = trigger.entity;
                let config = configs.get(e).unwrap().clone();
                if let Some((_hepta, title, desc_1, desc_2)) = children {
                    tree.write_to(title, TextValue(config.title.clone()));
                    tree.write_to(desc_1, TextValue(config.desc_1.clone()));
                    tree.write_to(desc_2, TextValue(config.desc_2.clone()));
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
                            .color(Color::gray(50))
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
                    let desc_1 = tree.branch(
                        e,
                        Text::new(config.desc_1.clone())
                            .size(FontSize::new(DESC_FONT_SIZE))
                            .color(Color::stone(500))
                            .at(Location::new().xs(
                                50.pct().as_center_x().with(94.0.pct().as_width()),
                                DESC_LINE_1_TOP_PCT
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
                    let desc_2 = tree.branch(
                        e,
                        Text::new(config.desc_2.clone())
                            .size(FontSize::new(DESC_FONT_SIZE))
                            .color(Color::stone(500))
                            .at(Location::new().xs(
                                50.pct().as_center_x().with(94.0.pct().as_width()),
                                DESC_LINE_2_TOP_PCT
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
                            for target in [hepta, title, desc_1, desc_2] {
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

                    children = Some((hepta, title, desc_1, desc_2));
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
    pub fn description(mut self, line_1: impl Into<String>, line_2: impl Into<String>) -> Self {
        self.config.desc_1 = line_1.into();
        self.config.desc_2 = line_2.into();
        self
    }
    pub fn target_page(mut self, page: usize) -> Self {
        self.config.target_page = page;
        self
    }
}

const CARD_WIDTH_PX: i32 = 200;
const CARD_HEIGHT_PX: i32 = 220;

/// Placeholder second route -- reachable via the global chrome's explicit ToC button (and,
/// since `Router` treats every route the same, via the main forward/back navigator too).
/// One `ContentsItem` for now, centered, just to judge the style before building out the
/// real grid of them (one per other route) and wiring up click-to-navigate.
pub fn toc(tree: &mut Tree, slot: Entity) {
    tree.branch(
        slot,
        ContentsItem::new()
            .title("Next")
            .description("Another stop along", "the tour of foliage.rs")
            .target_page(2)
            .at(Location::new().xs(
                50.pct()
                    .as_center_x()
                    .with(CARD_WIDTH_PX.px().as_width()),
                50.pct()
                    .as_center_y()
                    .with(CARD_HEIGHT_PX.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
}
