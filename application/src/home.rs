use crate::icons::IconHandles;
use crate::portfolio;
use crate::widgets::icon_button;
use foliage::{
    Anchor, Animation, Color, EcsExtension, Elevation, Entity, EntityEvent, FontSize, GlyphColors,
    Grid, GridExt, HorizontalAlignment, HrefLink, IconId, Insert, Keyring, Leaf, LeafSprout, Line,
    Location, Logical, OnClick, OnEnd, Opacity, Query, Res, Rounding, Section, Sequence, Sprout,
    Text, TextValue, Tree, Trigger, VerticalAlignment, Write, anchor, component,
};
use std::ops::Range;

/// The header's opacity fades + the two measurement lines' grow-in animations. The only
/// entrance animation still driven from the top level -- header's pieces (title, two
/// measurement lines) aren't a repeatable widget shape, just a lot of one-off page content
/// that needs a name.
fn animate_header_entrance<'t, T: EcsExtension>(
    seq: Sequence<'t, T>,
    header: &HeaderEntities,
) -> Sequence<'t, T> {
    seq.animate(
        Animation::new(Opacity::new(1.0))
            .start(500)
            .finish(1500)
            .targeting(header.name),
    )
    .animate(
        Animation::new(Opacity::new(1.0))
            .start(1000)
            .finish(1250)
            .targeting(header.top_desc),
    )
    .animate(
        Animation::new(Opacity::new(1.0))
            .start(1100)
            .finish(1350)
            .targeting(header.side_desc),
    )
    .animate(
        Animation::new(Opacity::new(1.0))
            .start(1500)
            .finish(1750)
            .targeting(header.pad_desc),
    )
    .animate(
        Animation::new(Opacity::new(1.0))
            .start(1750)
            .finish(2750)
            .targeting(header.desc),
    )
    .animate(
        Animation::new(Location::new().xs(
            4.col().as_x().with(5.row().as_y()),
            9.col().as_x().with(5.row().as_y()),
        ))
        .start(1000)
        .finish(3000)
        .targeting(header.top_line),
    )
    .animate(
        Animation::new(Location::new().xs(
            7.col().as_x().with(5.row().as_y()),
            7.col().as_x().with(8.row().as_y()),
        ))
        .start(1750)
        .finish(3000)
        .targeting(header.pad_connector),
    )
}

struct HeaderEntities {
    name: Entity,
    top_desc: Entity,
    top_line: Entity,
    side_desc: Entity,
    pad_connector: Entity,
    pad_desc: Entity,
    desc: Entity,
}

/// Title, three live layout-measurement readouts (width/half-width/pad-height), and the
/// tagline -- everything anchored under one `name_container`.
fn header<T: EcsExtension>(tree: &mut T, root: Entity) -> HeaderEntities {
    let name_container = tree.branch(
        root,
        Leaf::sprout()
            .at(Location::new().xs(
                1.col().as_left().with(12.col().as_right()).max(600.0),
                4.row().as_top().with(8.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(12.col().gap(4), 12.row().gap(4))),
    );
    let name = tree.branch(
        name_container,
        Text::new("foliage.rs")
            .size(FontSize::new(44))
            .at(Location::new().xs(
                2.col().as_left().with(11.col().as_right()),
                1.row().as_top().with(3.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((
                HorizontalAlignment::Center,
                GlyphColors::new().add(7..10, Color::green(400)),
                Opacity::new(0.0),
            )),
    );
    let top_line = tree.branch(
        name_container,
        Line::new(2)
            .color(Color::gray(700))
            .at(Location::new().xs(
                4.col().as_x().with(5.row().as_y()),
                4.col().as_x().with(5.row().as_y()),
            ))
            .elevate(Elevation::up(1)),
    );
    let top_desc = measurement_readout(
        tree,
        name_container,
        top_line,
        "w",
        |s| s.width(),
        Location::new().xs(
            5.col().as_left().with(8.col().as_right()),
            4.row().as_top().with(4.row().as_bottom()),
        ),
    );
    let side_desc = measurement_readout(
        tree,
        name_container,
        top_line,
        "h",
        |s| s.width() * 0.5,
        Location::new().xs(
            9.col().as_left().with(11.col().as_right()),
            4.row().as_top().with(4.row().as_bottom()),
        ),
    );
    let pad_connector = tree.branch(
        name_container,
        Line::new(2)
            .color(Color::gray(700))
            .at(Location::new().xs(
                7.col().as_x().with(5.row().as_y()),
                7.col().as_x().with(5.row().as_y()),
            ))
            .elevate(Elevation::up(1)),
    );
    let pad_desc = measurement_readout(
        tree,
        name_container,
        pad_connector,
        "pad",
        |s| s.height(),
        Location::new().xs(
            8.col().as_left().with(11.col().as_right()),
            7.row().as_top().with(8.row().as_bottom()),
        ),
    );
    let desc = tree.branch(
        name_container,
        Text::new("native + web ui")
            .size(FontSize::new(24))
            .color(Color::gray(500))
            .at(Location::new().xs(
                1.col().as_left().with(12.col().as_right()),
                9.row().as_top().with(12.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with((
                HorizontalAlignment::Center,
                GlyphColors::new()
                    .add(7..8, Color::orange(700))
                    .add(13..15, Color::green(400)),
                Opacity::new(0.0),
            )),
    );
    HeaderEntities {
        name,
        top_desc,
        top_line,
        side_desc,
        pad_connector,
        pad_desc,
        desc,
    }
}

/// A faded-in `Text` that tracks `watch`'s `Section<Logical>` live, formatted as
/// `"<label>: <value>"` -- the shape all three of `header`'s readouts share, differing only in
/// which entity/dimension they read and their label.
fn measurement_readout<T: EcsExtension>(
    tree: &mut T,
    parent: Entity,
    watch: Entity,
    label: &'static str,
    read: impl Fn(&Section<Logical>) -> f32 + Send + Sync + 'static,
    at: Location,
) -> Entity {
    let desc = tree.branch(
        parent,
        Text::new(format!("{label}: 0.0"))
            .size(FontSize::new(14))
            .color(Color::gray(700))
            .at(at)
            .elevate(Elevation::up(1))
            .with(Opacity::new(0.0)),
    );
    tree.subscribe(
        watch,
        move |trigger: Trigger<Write<Section<Logical>>>,
              mut tree: Tree,
              sections: Query<&Section<Logical>>| {
            let value = read(sections.get(trigger.event_target()).unwrap());
            tree.write_to(desc, TextValue(format!("{label}: {value:.01}")));
        },
    );
    desc
}

// ===========================================================================
// GithubLink -- icon button + connecting line + highlighted label, entirely self-contained:
// it disables its own button at spawn, fades everything in on its own schedule, and
// re-enables itself once its own fade finishes. The caller supplies only where it goes.
// Used once, but the point isn't reuse -- it's that "button + line + label, one entity,
// nobody outside needs its children" is exactly what `Sprout` is for.
// ===========================================================================

#[component]
struct GithubLink {}

struct GithubLinkSprout {
    leaf: LeafSprout,
}
impl GithubLink {
    fn new() -> GithubLinkSprout {
        GithubLinkSprout {
            leaf: LeafSprout::default(),
        }
    }
}
impl Sprout for GithubLinkSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl foliage::Bundle {
        // matches root's own 12-column, row_size-tall grid exactly -- `.at()` (set by the
        // caller) spans root's full area, so column/row numbers below mean what they'd mean
        // spawned directly under root, including the desc text's `10.col()` reach.
        (GithubLink {}, Grid::new(12.col().gap(8), 40.px().gap(8)))
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        let button = tree.branch(
            this,
            icon_button(IconHandles::Github, Color::gray(200), Color::gray(800))
                .at(Location::new().xs(
                    1.col().as_left().with(48.px().as_width()),
                    1.row().as_top().with(48.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Opacity::new(0.0)),
        );
        tree.write_to(button, FontSize::new(16));
        tree.on_click(button, |_: Trigger<OnClick>| {
            HrefLink::new("https://github.com/eblack-leaf/foliage").navigate()
        });
        tree.disable(button);
        let line = tree.branch(
            this,
            Line::new(2)
                .color(Color::gray(700))
                .at(Location::new().xs(
                    anchor().right().as_x().adjust(16).with(1.row().as_y()),
                    anchor().right().as_x().adjust(16).with(1.row().as_y()),
                ))
                .elevate(Elevation::up(1))
                .with(Anchor::new(button)),
        );
        let desc = tree.branch(
            this,
            Text::new("on-click: github")
                .size(FontSize::new(14))
                .color(Color::gray(500))
                .at(Location::new().xs(
                    anchor()
                        .right()
                        .as_left()
                        .adjust(16)
                        .with(10.col().as_right()),
                    1.row().as_top().adjust(8).with(2.row().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .with((
                    GlyphColors::new().add(10..16, Color::green(300)),
                    Anchor::new(line),
                    Opacity::new(0.0),
                )),
        );

        Sequence::new(tree)
            .animate(
                Animation::new(Opacity::new(1.0))
                    .start(1000)
                    .finish(1500)
                    .targeting(button),
            )
            .animate(
                Animation::new(Opacity::new(1.0))
                    .start(2500)
                    .finish(3000)
                    .targeting(desc),
            )
            .animate(
                Animation::new(Location::new().xs(
                    anchor().right().as_x().adjust(16).with(1.row().as_y()),
                    anchor().right().as_x().adjust(64).with(1.row().as_y()),
                ))
                .start(1750)
                .finish(2500)
                .targeting(line),
            );
        // aesthetic pacing, not a GPU-readiness wait -- avoids the button becoming
        // interactive the instant its own fade finishes while its label is still settling.
        tree.timer(1500, move |_: Trigger<OnEnd>, mut tree: Tree| {
            tree.enable(button);
        });
    }
}

// ===========================================================================
// OptionRow -- the "on-click: usage/impl/docs" shape: icon button + callout line + label,
// self-contained the same way GithubLink is, but genuinely reused 3x with different
// icon/color/copy/geometry, so its variation rides a config component read back via `react`
// (build() has no access to `self` -- that's the actual reason this needs `react` instead of
// spawning directly like GithubLink does).
// ===========================================================================

#[component]
struct OptionRow {}

#[component]
#[derive(Clone)]
struct OptionRowConfig {
    icon: IconId,
    color: Color,
    desc_text: &'static str,
    desc_highlight: Range<usize>,
    desc_columns: (i32, i32),
    line_column: i32,
    line_target_columns: (i32, i32),
    /// when this row's own fade-in starts, ms from its own spawn.
    start_delay: u64,
    /// when this row's button becomes clickable -- a page-level decision (today: "once the
    /// whole page has settled"), so the caller supplies it rather than OptionRow guessing.
    enable_delay: u64,
}

struct OptionRowSprout {
    leaf: LeafSprout,
    config: OptionRowConfig,
}
impl OptionRow {
    #[allow(clippy::too_many_arguments)]
    fn new(
        icon: IconId,
        color: Color,
        desc_text: &'static str,
        desc_highlight: Range<usize>,
        desc_columns: (i32, i32),
        line_column: i32,
        line_target_columns: (i32, i32),
        start_delay: u64,
        enable_delay: u64,
    ) -> OptionRowSprout {
        OptionRowSprout {
            leaf: LeafSprout::default(),
            config: OptionRowConfig {
                icon,
                color,
                desc_text,
                desc_highlight,
                desc_columns,
                line_column,
                line_target_columns,
                start_delay,
                enable_delay,
            },
        }
    }
}
impl Sprout for OptionRowSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl foliage::Bundle {
        // `.at()` (set by the caller) spans exactly one row of options_container's own
        // 5-column grid -- matching that column split here, with a single local row, means
        // `3.col()`/`line_column.col()`/etc below resolve to the same pixels they would have
        // spanning the whole 3-row container directly.
        (
            OptionRow {},
            self.config,
            Grid::new(5.col().gap(4), 1.row()),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        tree.react::<OptionRowConfig, _>(
            this,
            move |trigger: Trigger<Insert, OptionRowConfig>,
                  configs: Query<&OptionRowConfig>,
                  mut tree: Tree| {
                let cfg = configs.get(trigger.event_target()).unwrap().clone();
                let button = tree.branch(
                    this,
                    icon_button(cfg.icon, cfg.color, Color::gray(900))
                        .outline(2)
                        .at(Location::new().xs(
                            3.col()
                                .as_left()
                                .with(3.col().as_right())
                                .max(48.0)
                                .min(48.0),
                            1.row()
                                .as_top()
                                .with(1.row().as_bottom())
                                .max(48.0)
                                .min(48.0),
                        ))
                        .elevate(Elevation::up(1))
                        .with(Opacity::new(0.0)),
                );
                tree.on_click(button, move |_: Trigger<OnClick>| {
                    HrefLink::new("https://eblack-leaf.github.io/foliage/book/").navigate()
                });
                tree.disable(button);
                let line = tree.branch(
                    this,
                    Line::new(2)
                        .color(cfg.color)
                        .at(Location::new().xs(
                            cfg.line_column.col().as_x().with(1.row().as_y()),
                            cfg.line_column.col().as_x().with(1.row().as_y()),
                        ))
                        .elevate(Elevation::up(1)),
                );
                let desc = tree.branch(
                    this,
                    Text::new(cfg.desc_text)
                        .size(FontSize::new(16))
                        .color(Color::gray(500))
                        .at(Location::new().xs(
                            cfg.desc_columns
                                .0
                                .col()
                                .as_left()
                                .with(cfg.desc_columns.1.col().as_right()),
                            1.row().as_top().with(1.row().as_bottom()),
                        ))
                        .elevate(Elevation::up(1))
                        .with((
                            HorizontalAlignment::Center,
                            VerticalAlignment::Middle,
                            GlyphColors::new().add(cfg.desc_highlight.clone(), cfg.color),
                            Opacity::new(0.0),
                        )),
                );

                let base = cfg.start_delay;
                Sequence::new(&mut tree)
                    .animate(
                        Animation::new(Opacity::new(1.0))
                            .targeting(button)
                            .start(base)
                            .finish(base + 500),
                    )
                    .animate(
                        Animation::new(Opacity::new(1.0))
                            .targeting(desc)
                            .start(base + 500)
                            .finish(base + 1000),
                    )
                    .animate(
                        Animation::new(Location::new().xs(
                            cfg.line_target_columns.0.col().as_x().with(1.row().as_y()),
                            cfg.line_target_columns.1.col().as_x().with(1.row().as_y()),
                        ))
                        .targeting(line)
                        .start(base)
                        .finish(base + 500),
                    );
                tree.timer(
                    cfg.enable_delay,
                    move |_: Trigger<OnEnd>, mut tree: Tree| {
                        tree.enable(button);
                    },
                );
            },
        );
    }
}

pub(crate) fn build<T: EcsExtension>(tree: &mut T) {
    let row_size = 40;
    let root = tree.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::abs(0))
            .with(Grid::new(12.col().gap(8), row_size.px().gap(8))),
    );
    tree.name(root, "home");

    let header = header(tree, root);
    tree.branch(
        root,
        GithubLink::new()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    let options_container = tree.branch(
        root,
        Leaf::sprout()
            .at(Location::new().xs(
                1.col().as_left().with(12.col().as_right()).max(600.0),
                10.row().as_top().with(13.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(Grid::new(5.col().gap(4), 3.row().gap(8))),
    );
    // row, icon, color, desc text, highlight range, desc columns, line column, line target,
    // start delay -- enable delay (3500) matches portfolio's own fade-in below: all three
    // rows wait for the whole page to settle, unlike GithubLink's early self-enable.
    let rows: [(
        i32,
        IconId,
        Color,
        &str,
        Range<usize>,
        (i32, i32),
        i32,
        (i32, i32),
        u64,
    ); 3] = [
        (
            1,
            IconHandles::Terminal.into(),
            Color::green(700),
            "on-click: usage",
            10..15,
            (4, 5),
            1,
            (1, 2),
            500,
        ),
        (
            2,
            IconHandles::Layers.into(),
            Color::green(500),
            "on-click: impl",
            10..14,
            (1, 2),
            5,
            (4, 5),
            1500,
        ),
        (
            3,
            IconHandles::BookOpen.into(),
            Color::green(300),
            "on-click: docs",
            10..14,
            (4, 5),
            1,
            (1, 2),
            2500,
        ),
    ];
    for (row, icon, color, desc_text, highlight, desc_columns, line_column, line_target, delay) in
        rows
    {
        tree.branch(
            options_container,
            OptionRow::new(
                icon,
                color,
                desc_text,
                highlight,
                desc_columns,
                line_column,
                line_target,
                delay,
                3500,
            )
            .at(Location::new().xs(
                1.col().as_left().with(5.col().as_right()),
                row.row().as_top().with(row.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
        );
    }

    let portfolio = tree.branch(
        root,
        foliage::Button::new()
            .icon(IconHandles::Code.into())
            .text("Portfolio")
            .rounding(Rounding::Sm)
            .colors(Color::orange(500), Color::gray(900))
            .outline(2)
            .at(Location::new().xs(
                3.col()
                    .as_left()
                    .with(10.col().as_right())
                    .min(175.0)
                    .max(350.0),
                15.row().as_top().with(48.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );
    tree.graft(portfolio)
        .write((FontSize::new(20), Opacity::new(0.0)))
        .on_click(
            move |_: Trigger<OnClick>, mut tree: Tree, keyring: Res<Keyring>| {
                tree.disable(root);
                portfolio::build(&mut tree, root, &keyring);
            },
        );

    let _spacing = tree.branch(
        root,
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                17.row().as_top().with(17.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    let seq = Sequence::new(tree);
    let seq = animate_header_entrance(seq, &header);
    seq.animate(
        Animation::new(Opacity::new(1.0))
            .start(3000)
            .finish(3500)
            .targeting(portfolio),
    )
    .end(move |_: Trigger<OnEnd>, mut tree: Tree| {
        tree.enable(portfolio);
    });
    tree.disable(portfolio);
}
