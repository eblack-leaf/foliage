//! Application-defined widgets -- the external-crate stress test of the authoring
//! contract: same `Sprout` trait, same `react`/`forward` door, same `#[targeted_event]`
//! events as the library's own Button/TextInput. Nothing here is foliage-internal.

// bevy's own derives (Component) resolve `bevy_ecs::` relative to this module --
// the foliage re-export satisfies them with no direct dependency.
use foliage::bevy_ecs;
use foliage::{
    targeted_event, AssetKey, Children, Color, Component, Dragged, EcsExtension, Elevation, Entity,
    FocusBehavior, FontSize, Grid, GridExt, Image, ImageView, Insert, InteractionListener,
    InteractionPropagation, Leaf, LeafSprout, Line, Location, Logical, MemoryId, OnClick, Panel,
    Query, Res, Rounding, Section, Sprout, Text, TextValue, Tree, Trigger,
};

// ===========================================================================
// ProjectCard -- a portfolio card: image + title + description + launch button.
// Components in (ProjectInfo), Launch out. The screen never sees its insides.
// ===========================================================================

/// ProjectCard's public data component. Poke it, the card redraws.
#[derive(Component, Clone, Default)]
pub(crate) struct ProjectInfo {
    pub(crate) title: String,
    pub(crate) desc: String,
    pub(crate) image: Option<(MemoryId, AssetKey)>,
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
    pub(crate) fn image(mut self, memory: MemoryId, key: AssetKey) -> Self {
        self.info.image = Some((memory, key));
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
    fn build<T: EcsExtension>(this: Entity, kids: &mut Children<T>) {
        // static skeleton
        let _backdrop = kids.spawn(
            Panel::new()
                .color(Color::gray(800))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1)),
        );
        let info = kids.spawn(
            Leaf::sprout()
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    70.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(2))
                .with(Grid::new(1.col().gap(8), 3.row().gap(8))),
        );
        let mut info_kids = Children::new(info, kids.tree());
        let title = info_kids.spawn(
            Text::new("")
                .size(FontSize::new(16))
                .color(Color::gray(200))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    1.row().as_top().with(1.row().as_bottom()),
                ))
                .elevate(Elevation::up(1)),
        );
        let desc = info_kids.spawn(
            Text::new("")
                .size(FontSize::new(14))
                .color(Color::gray(500))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    2.row().as_top().with(3.row().as_bottom()),
                ))
                .elevate(Elevation::up(1)),
        );
        let launch = info_kids.spawn(
            foliage::Button::new()
                .icon(crate::icons::IconHandles::Box.value())
                .rounding(Rounding::Full)
                .colors(Color::gray(900), Color::orange(800))
                .at(Location::new().xs(
                    100.pct().as_right().adjust(-8).with(44.px().as_width()),
                    100.pct().as_bottom().adjust(-8).with(44.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        kids.tree()
            .on_click(launch, move |_: Trigger<OnClick>, mut tree: Tree| {
                tree.trigger_targets(Launch::new(), this);
            });

        // data-dependent: texts patch in place; the image respawns (Image's spawn config
        // carries the asset key -- it has no public value channel yet, so respawn is this
        // author's policy; captured FnMut state tracks the previous entity).
        let mut current_image: Option<Entity> = None;
        kids.react::<ProjectInfo, _>(
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
                if let Some((memory, key)) = info.image {
                    let display = Children::new(card, &mut tree).spawn(
                        Image::new(memory, key)
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
    fn build<T: EcsExtension>(this: Entity, kids: &mut Children<T>) {
        // static skeleton
        let _track = kids.spawn(
            Line::new(4)
                .color(Color::gray(700))
                .at(Location::new().xs(
                    0.pct().as_x().with(50.pct().as_y()),
                    100.pct().as_x().with(50.pct().as_y()),
                ))
                .elevate(Elevation::up(1)),
        );
        // no Location: progress-dependent, set by the reaction's first fire
        let elapsed = kids.spawn(
            Line::new(4)
                .color(Color::green(300))
                .elevate(Elevation::up(2)),
        );
        let knob = kids.spawn(
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
        kids.tree().subscribe(
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
        kids.react::<Progress, _>(
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
