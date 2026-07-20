//! Application-defined widgets -- the external-crate stress test of the authoring
//! contract: same `Sprout` trait, same `react`/`forward` door, same `#[targeted_event]`
//! events as the library's own Button/TextInput. Nothing here is foliage-internal.

use foliage::{
    component, targeted_event, AssetKey, Button, ButtonSprout, Color, EcsExtension, Elevation,
    Entity, EntityEvent, FontSize, Grid, GridExt, IconId, Image, ImageView, Insert,
    InteractionListener, Leaf, LeafSprout, Location, OnClick, Panel, Query, Rounding, Sprout,
    Text, TextValue, Tree, Trigger,
};

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
#[component]
#[derive(Clone, Default)]
pub(crate) struct ProjectInfo {
    pub(crate) title: String,
    pub(crate) desc: String,
    pub(crate) image: Option<AssetKey>,
}

/// Emitted at the card root when the user activates it (image or launch button).
#[targeted_event]
#[derive(Copy)]
pub(crate) struct Launch {}

#[component]
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

