//! Application-defined widgets -- the external-crate stress test of the authoring
//! contract: same `Sprout` trait, same `react`/`forward` door, same `#[targeted_event]`
//! events as the library's own Button/TextInput. Nothing here is foliage-internal.

// bevy's own derives (Component) resolve `bevy_ecs::` relative to this module --
// the foliage re-export satisfies them with no direct dependency.
use foliage::bevy_ecs;
use foliage::{
    anchor, targeted_event, Anchor, Animation, AssetKey, Button, ButtonSprout, Children, Color,
    Component, Dragged, Ease, EcsExtension, Elevation, Entity, EntityEvent, FocusBehavior,
    FontSize, Grid, GridExt, IconId, Image, ImageView, Insert, InteractionListener,
    InteractionPropagation, Leaf, LeafSprout, Line, Location, Logical, OnClick, OnEnd, Opacity,
    Panel, Query, Res, Rounding, Section, Sequence, Sprout, Text, TextValue, Tree, Trigger,
};
use std::sync::Arc;

/// A round icon-only button -- the shape every icon button in this app shares, differing only
/// in icon/colors/outline. Just a canned `ButtonSprout` config, not a new widget: callers keep
/// chaining `.at()`/`.elevate()`/`.outline()` exactly as they would on `Button::new()` directly.
pub(crate) fn icon_button<ID: Into<IconId>>(
    icon: ID,
    primary: Color,
    secondary: Color,
) -> ButtonSprout {
    Button::new()
        .rounding(Rounding::Full)
        .icon(icon.into())
        .colors(primary, secondary)
}

// ===========================================================================
// ProjectCard -- a portfolio card: image + title + description + launch button.
// Components in (ProjectInfo), Launch out. The screen never sees its insides.
// ===========================================================================

/// ProjectCard's public data component. Poke it, the card redraws.
#[derive(Component, Clone, Default)]
pub(crate) struct ProjectInfo {
    pub(crate) title: String,
    pub(crate) desc: String,
    pub(crate) image: Option<AssetKey>,
}

/// Emitted at the card root when the user activates it (image or launch button).
#[targeted_event]
#[derive(Copy)]
pub(crate) struct Launch {}

#[derive(Component)]
pub(crate) struct ProjectCard {}
impl ProjectCard {
    pub(crate) fn new() -> ProjectCardSprout {
        ProjectCardSprout::default()
    }
}
#[derive(Default)]
pub(crate) struct ProjectCardSprout {
    leaf: LeafSprout,
    info: ProjectInfo,
}
impl ProjectCardSprout {
    pub(crate) fn title(mut self, t: impl Into<String>) -> Self {
        self.info.title = t.into();
        self
    }
    pub(crate) fn desc(mut self, d: impl Into<String>) -> Self {
        self.info.desc = d.into();
        self
    }
    pub(crate) fn image(mut self, key: AssetKey) -> Self {
        self.info.image = Some(key);
        self
    }
}
impl Sprout for ProjectCardSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl foliage::Bundle {
        (ProjectCard {}, self.info, Grid::default())
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton
        let _backdrop = tree.branch(
            this,
            Panel::new()
                .color(Color::gray(800))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1)),
        );
        let info = tree.branch(
            this,
            Leaf::sprout()
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    70.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(2))
                .with(Grid::new(1.col().gap(8), 3.row().gap(8))),
        );
        let title = tree.branch(
            info,
            Text::new("")
                .size(FontSize::new(16))
                .color(Color::gray(200))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    1.row().as_top().with(1.row().as_bottom()),
                ))
                .elevate(Elevation::up(1)),
        );
        let desc = tree.branch(
            info,
            Text::new("")
                .size(FontSize::new(14))
                .color(Color::gray(500))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    2.row().as_top().with(3.row().as_bottom()),
                ))
                .elevate(Elevation::up(1)),
        );
        let launch = tree.branch(
            info,
            icon_button(
                crate::icons::IconHandles::Box,
                Color::gray(900),
                Color::orange(800),
            )
            .at(Location::new().xs(
                100.pct().as_right().adjust(-8).with(44.px().as_width()),
                100.pct().as_bottom().adjust(-8).with(44.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
        );
        tree.on_click(launch, move |_: Trigger<OnClick>, mut tree: Tree| {
            tree.trigger_targets(Launch::new(), this);
        });

        // data-dependent: texts patch in place; the image respawns (Image's spawn config
        // carries the asset key -- it has no public value channel yet, so respawn is this
        // author's policy; captured FnMut state tracks the previous entity).
        let mut current_image: Option<Entity> = None;
        tree.react::<ProjectInfo, _>(
            this,
            move |trigger: Trigger<Insert, ProjectInfo>,
                  infos: Query<&ProjectInfo>,
                  mut tree: Tree| {
                let card = trigger.entity;
                let info = infos.get(card).unwrap();
                tree.write_to(title, TextValue(info.title.clone()));
                tree.write_to(desc, TextValue(info.desc.clone()));
                if let Some(prev) = current_image.take() {
                    tree.remove(prev);
                }
                if let Some(key) = info.image {
                    let display = tree.branch(
                        card,
                        Image::new(key)
                            .view(ImageView::Crop)
                            .at(Location::new().xs(
                                1.col().as_left().with(1.col().as_right()),
                                0.pct().as_top().with(70.pct().as_bottom()),
                            ))
                            .elevate(Elevation::up(2))
                            .with(InteractionListener::new()),
                    );
                    tree.on_click(display, move |_: Trigger<OnClick>, mut tree: Tree| {
                        tree.trigger_targets(Launch::new(), card);
                    });
                    current_image = Some(display);
                }
            },
        );
    }
}

// ===========================================================================
// Scrubber -- a draggable progress bar. Components in (Progress), Scrubbed out.
// Drag writes and programmatic writes go through the same Progress door.
// ===========================================================================

/// Scrubber's public value component (0.0..=1.0).
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct Progress(pub(crate) f32);

/// Emitted at the scrubber root whenever progress changes (drag or programmatic).
#[targeted_event]
#[derive(Copy)]
pub(crate) struct Scrubbed {
    pub(crate) progress: f32,
}

#[derive(Component)]
pub(crate) struct Scrubber {}
impl Scrubber {
    pub(crate) fn new() -> ScrubberSprout {
        ScrubberSprout::default()
    }
}
#[derive(Default)]
pub(crate) struct ScrubberSprout {
    leaf: LeafSprout,
    progress: f32,
}
impl ScrubberSprout {
    pub(crate) fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }
}
impl Sprout for ScrubberSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl foliage::Bundle {
        (Scrubber {}, Progress(self.progress), Grid::default())
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton
        let _track = tree.branch(
            this,
            Line::new(4)
                .color(Color::gray(700))
                .at(Location::new().xs(
                    0.pct().as_x().with(50.pct().as_y()),
                    100.pct().as_x().with(50.pct().as_y()),
                ))
                .elevate(Elevation::up(1)),
        );
        // no Location: progress-dependent, set by the reaction's first fire
        let elapsed = tree.branch(
            this,
            Line::new(4)
                .color(Color::green(300))
                .elevate(Elevation::up(2)),
        );
        let knob = tree.branch(
            this,
            Panel::new()
                .rounding(Rounding::Full)
                .color(Color::green(300))
                .at(Location::new().xs(
                    foliage::anchor()
                        .right()
                        .as_center_x()
                        .with(16.px().as_width()),
                    50.pct().as_center_y().with(16.px().as_height()),
                ))
                .elevate(Elevation::up(3))
                .with((
                    foliage::Anchor::new(elapsed),
                    InteractionListener::new(),
                    InteractionPropagation::grab().disable_drag(),
                    FocusBehavior::ignore(),
                )),
        );

        // input: drag -> value. Touches ONE component; drawing happens in the reaction.
        tree.subscribe(
            knob,
            move |_: Trigger<Dragged>,
                  interaction: Res<foliage::CurrentInteraction>,
                  sections: Query<&Section<Logical>>,
                  mut tree: Tree| {
                let bounds = sections.get(this).unwrap();
                let pct = ((interaction.click().current.left() - bounds.left()) / bounds.width())
                    .clamp(0.0, 1.0);
                tree.write_to(this, Progress(pct));
            },
        );

        // render: value -> geometry, identical for drag and programmatic writes.
        tree.react::<Progress, _>(
            this,
            move |trigger: Trigger<Insert, Progress>,
                  progress: Query<&Progress>,
                  mut tree: Tree| {
                let value = progress.get(trigger.entity).unwrap().0;
                tree.write_to(
                    elapsed,
                    Location::new().xs(
                        0.pct().as_x().with(50.pct().as_y()),
                        (value * 100.0).pct().as_x().with(50.pct().as_y()),
                    ),
                );
                tree.trigger_targets(Scrubbed::new(value), trigger.entity);
            },
        );
    }
}

// ===========================================================================
// Modal -- a backdrop that grows from an anchor entity to fill the screen, holding
// caller-injected content, with a close button that shrinks it back and removes it.
// One entity: `tree.leaf(Modal::new(anchor_to).content(|tree, backdrop| ..))`, listen for
// `Closed`. Backdrop/terminate/content are all private -- nothing outside needs their ids,
// which is exactly why this can be a real widget where `OptionRow`-shaped page decoration
// can't: nothing here needs per-child animation timing driven from outside `build`.
// ===========================================================================

#[derive(Component)]
pub(crate) struct Modal {}

/// Emitted once Modal has finished animating closed and removed itself -- the caller's cue
/// to re-enable whatever it disabled when the modal opened.
#[targeted_event]
#[derive(Copy)]
pub(crate) struct Closed {}

#[derive(Component, Clone)]
struct ModalConfig {
    anchor_to: Entity,
    /// Builds the modal's content as a child of the backdrop, once the backdrop actually
    /// exists (spawning is deferred behind `react`, so this can't happen at `Modal::new`
    /// call time) -- returns the content's own root so it can be removed immediately on
    /// close instead of waiting for the whole backdrop to finish animating away.
    content: Arc<dyn Fn(&mut Tree, Entity) -> Entity + Send + Sync>,
}

pub(crate) struct ModalSprout {
    leaf: LeafSprout,
    anchor_to: Entity,
    content: Option<Arc<dyn Fn(&mut Tree, Entity) -> Entity + Send + Sync>>,
}
impl Modal {
    /// `anchor_to` is the entity the backdrop grows from (a card's own root, say).
    pub(crate) fn new(anchor_to: Entity) -> ModalSprout {
        ModalSprout {
            leaf: LeafSprout::default(),
            anchor_to,
            content: None,
        }
    }
}
impl ModalSprout {
    pub(crate) fn content(
        mut self,
        f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static,
    ) -> Self {
        self.content = Some(Arc::new(f));
        self
    }
}
impl Sprout for ModalSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl foliage::Bundle {
        (
            Modal {},
            ModalConfig {
                anchor_to: self.anchor_to,
                content: self.content.expect("Modal::content(..) is required"),
            },
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        tree.react::<ModalConfig, _>(
            this,
            move |trigger: Trigger<Insert, ModalConfig>,
                  configs: Query<&ModalConfig>,
                  mut tree: Tree| {
                let cfg = configs.get(trigger.event_target()).unwrap().clone();
                // top-level tree.leaf(..), not tree.branch(this, ..): before Modal was a
                // Sprout, these were always spawned stem-less -- keeping them that way here
                // instead of nesting under Modal's own wrapper root.
                let backdrop = tree.leaf(
                    Panel::new()
                        .color(Color::gray(800))
                        .at(Location::new().xs(
                            anchor().left().as_left().with(anchor().right().as_right()),
                            anchor().top().as_top().with(anchor().bottom().as_bottom()),
                        ))
                        .elevate(Elevation::abs(50))
                        .with((
                            Anchor::new(cfg.anchor_to),
                            Opacity::new(0.0),
                            Grid::default(),
                        )),
                );
                let terminate = tree.leaf(
                    icon_button(
                        crate::icons::IconHandles::X,
                        Color::gray(200),
                        Color::orange(800),
                    )
                    .at(Location::new().xs(
                        16.px().as_left().with(40.px().as_width()),
                        16.px().as_top().with(40.px().as_height()),
                    ))
                    .elevate(Elevation::abs(95)),
                );
                let content = (cfg.content)(&mut tree, backdrop);

                Sequence::new(&mut tree)
                    .animate(
                        Animation::new(Opacity::new(1.0))
                            .targeting(backdrop)
                            .start(0)
                            .finish(200),
                    )
                    .animate(
                        Animation::new(
                            Location::new().xs(
                                0.pct()
                                    .as_left()
                                    .adjust(24)
                                    .with(100.pct().as_right().adjust(-24))
                                    .max(450.0),
                                0.pct()
                                    .as_top()
                                    .adjust(36)
                                    .with(100.pct().as_bottom().adjust(-36)),
                            ),
                        )
                        .targeting(backdrop)
                        .start(0)
                        .finish(750)
                        .eased(Ease::INWARD),
                    )
                    .animate(
                        Animation::new(Location::new().xs(
                            0.pct().as_left().with(100.pct().as_right()),
                            0.pct().as_top().with(100.pct().as_bottom()),
                        ))
                        .targeting(backdrop)
                        .start(1000)
                        .finish(1500),
                    );

                tree.on_click(terminate, move |_: Trigger<OnClick>, mut tree: Tree| {
                    tree.disable(terminate);
                    tree.remove(content);
                    // fired now, not in .end() below -- the caller's own fade-back-in needs
                    // to run in parallel with this close animation (matching how it used to
                    // share one Sequence), not wait for this one to fully finish first.
                    tree.trigger_targets(Closed::new(), this);
                    Sequence::new(&mut tree)
                        .animate(
                            Animation::new(Opacity::new(0.0))
                                .targeting(terminate)
                                .start(0)
                                .finish(500),
                        )
                        .animate(
                            Animation::new(
                                Location::new().xs(
                                    0.pct()
                                        .as_left()
                                        .adjust(24)
                                        .with(100.pct().as_right().adjust(-24))
                                        .max(450.0),
                                    0.pct()
                                        .as_top()
                                        .adjust(36)
                                        .with(100.pct().as_bottom().adjust(-36)),
                                ),
                            )
                            .targeting(backdrop)
                            .start(0)
                            .finish(500)
                            .eased(Ease::INWARD),
                        )
                        .animate(
                            Animation::new(Location::new().xs(
                                anchor().left().as_left().with(anchor().right().as_right()),
                                anchor().top().as_top().with(anchor().bottom().as_bottom()),
                            ))
                            .targeting(backdrop)
                            .start(750)
                            .finish(1250),
                        )
                        .end(move |_: Trigger<OnEnd>, mut tree: Tree| {
                            tree.remove([this, backdrop, terminate]);
                        });
                });
            },
        );
    }
}
