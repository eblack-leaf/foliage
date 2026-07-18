//! A live tour of every composite the library ships, one labeled instance (or a couple of
//! meaningful variants) per row, stacked down a single tall column -- the page scrolls
//! rather than trying to fit everything on screen at once.

use crate::icons::IconHandles;
use foliage::{
    AssetKey, Carousel, CarouselPages, Checkbox, Color, Dropdown, EcsExtension, Elevation, Entity,
    FontSize, GridExt, HorizontalAlignment, Icon, ImageView, InteractionPropagation, Location,
    Opacity, Pagination, PaginationMode, Panel, Popover, PopoverPlacement, RadioGroup, Rounding,
    SegmentedControl, Slider, Sprout, Tabs, TabsPages, Text, TextInput, Toggle, Tree,
    VerticalAlignment,
};

/// One section: a small gray label above whatever `.at()` box the caller gives the
/// composite instance -- returns that box's (top, bottom) in px so the caller only ever
/// picks a height, never computes an absolute position by hand.
fn section(tree: &mut Tree, parent: Entity, cursor: &mut i32, label: &str, height: i32) -> (i32, i32) {
    tree.branch(
        parent,
        Text::new(label)
            .size(FontSize::new(14))
            .color(Color::gray(500))
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                (*cursor).px().as_top().with((*cursor + 20).px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    *cursor += 28;
    let top = *cursor;
    let bottom = *cursor + height;
    *cursor = bottom + 28;
    (top, bottom)
}

pub(crate) fn build(tree: &mut Tree, app: Entity, artwork: [AssetKey; 3]) {
    tree.write_to(app, Opacity::new(1.0));

    // header -- row-based, same as the old artist_blog.rs (title row 1, subtitle row 2 of
    // the same 40px-row/8px-gap grid this page's root already has): each text gets its own
    // whole row, and the gap between rows is the grid's own, not hand-computed px math.
    // Row 3 is the first row that clears Modal's fixed close button (x/y:[16,56]).
    tree.branch(
        app,
        Icon::new(IconHandles::Grid)
            .color(Color::green(300))
            .at(Location::new().xs(
                8.px().as_left().with(24.px().as_width()),
                3.row().as_top().with(3.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    tree.branch(
        app,
        Text::new("composites")
            .size(FontSize::new(24))
            .color(Color::gray(200))
            .at(Location::new().xs(
                40.px().as_left().with(100.pct().as_right()),
                3.row().as_top().with(3.row().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(VerticalAlignment::Middle),
    );
    tree.branch(
        app,
        Text::new("every built-in widget, live")
            .size(FontSize::new(14))
            .color(Color::gray(500))
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right()),
                4.row().as_top().with(4.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    // row 4 ends at (4-1)*48+40 = 184px -- cursor picks up from there for everything else,
    // which still uses its own px-based section() helper.
    let mut cursor: i32 = 200;

    // Button
    let (top, bottom) = section(tree, app, &mut cursor, "Button", 44);
    tree.branch(
        app,
        foliage::Button::new()
            .icon(IconHandles::Code.into())
            .text("Button")
            .rounding(Rounding::Sm)
            .colors(Color::gray(900), Color::green(500))
            .at(Location::new().xs(
                8.px().as_left().with(160.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Toggle
    let (top, bottom) = section(tree, app, &mut cursor, "Toggle", 22);
    tree.branch(
        app,
        Toggle::new()
            .on(true)
            .colors(Color::green(500), Color::gray(700), Color::gray(200))
            .at(Location::new().xs(
                8.px().as_left().with(40.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Checkbox
    let (top, bottom) = section(tree, app, &mut cursor, "Checkbox", 24);
    tree.branch(
        app,
        Checkbox::new()
            .on(true)
            .colors(Color::gray(600), Color::green(500), Color::gray(900))
            .at(Location::new().xs(
                8.px().as_left().with(24.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // RadioGroup
    let (top, bottom) = section(tree, app, &mut cursor, "RadioGroup", 96);
    tree.branch(
        app,
        RadioGroup::new()
            .options(["One", "Two", "Three"])
            .selected(0)
            .colors(Color::green(500), Color::gray(600))
            .at(Location::new().xs(
                8.px().as_left().with(200.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // SegmentedControl
    let (top, bottom) = section(tree, app, &mut cursor, "SegmentedControl", 36);
    tree.branch(
        app,
        SegmentedControl::new()
            .options(["A", "B", "C"])
            .selected(0)
            .colors(Color::green(500), Color::gray(600))
            .rounding(Rounding::Sm)
            .at(Location::new().xs(
                8.px().as_left().with(240.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Slider
    let (top, bottom) = section(tree, app, &mut cursor, "Slider", 24);
    tree.branch(
        app,
        Slider::new()
            .progress(0.4)
            .colors(Color::gray(700), Color::green(300))
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // TextInput -- rounding/outline forwarded through to the backing panel this session
    let (top, bottom) = section(tree, app, &mut cursor, "TextInput", 40);
    tree.branch(
        app,
        TextInput::new()
            .hint_text("type here...")
            .foreground(Color::gray(200))
            .background(Color::gray(900))
            .accent(Color::green(600))
            .rounding(Rounding::Sm)
            .outline(1)
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Dropdown
    let (top, bottom) = section(tree, app, &mut cursor, "Dropdown", 36);
    tree.branch(
        app,
        Dropdown::new()
            .options(["Option 1", "Option 2", "Option 3"])
            .chevron(IconHandles::ChevronDown.into())
            .colors(Color::gray(200), Color::gray(900), Color::green(600))
            .at(Location::new().xs(
                8.px().as_left().with(220.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Pagination -- Dots variant
    let (top, bottom) = section(tree, app, &mut cursor, "Pagination (Dots)", 16);
    tree.branch(
        app,
        Pagination::new(5)
            .mode(PaginationMode::Dots)
            .colors(Color::green(300), Color::gray(600))
            .at(Location::new().xs(
                8.px().as_left().with(120.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Pagination -- Numbered variant with step icons
    let (top, bottom) = section(tree, app, &mut cursor, "Pagination (Numbered)", 40);
    tree.branch(
        app,
        Pagination::new(7)
            .mode(PaginationMode::Numbered)
            .step_icons(IconHandles::SkipLeft.into(), IconHandles::SkipRight.into())
            .step_colors(Color::gray(200), Color::gray(800))
            .colors(Color::green(300), Color::gray(600))
            .at(Location::new().xs(
                8.px().as_left().with(280.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Carousel
    let (top, bottom) = section(tree, app, &mut cursor, "Carousel", 180);
    tree.branch(
        app,
        Carousel::new()
            .pages(CarouselPages::new(
                artwork.len(),
                move |tree: &mut Tree, slot: Entity, i| {
                    tree.branch(
                        slot,
                        foliage::Image::new(artwork[i])
                            .view(ImageView::Crop)
                            .at(Location::new().xs(
                                0.pct().as_left().with(100.pct().as_right()),
                                0.pct().as_top().with(100.pct().as_bottom()),
                            ))
                            .elevate(Elevation::up(1)),
                    );
                },
            ))
            .pagination(PaginationMode::Dots)
            .colors(Color::green(300), Color::gray(600))
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Tabs -- header is a SegmentedControl composed in, not reimplemented
    let (top, bottom) = section(tree, app, &mut cursor, "Tabs", 160);
    tree.branch(
        app,
        Tabs::new()
            .pages(TabsPages::new(
                vec!["Tab 1".into(), "Tab 2".into(), "Tab 3".into()],
                |tree: &mut Tree, slot: Entity, i| {
                    // a real dark, square-cornered backing + a small icon, not just
                    // floating text -- makes it obvious the content actually changed per
                    // tab, without turning it into a bright color swatch.
                    let backing = [Color::gray(800), Color::gray(700), Color::gray(600)];
                    let icons = [IconHandles::Terminal, IconHandles::Layers, IconHandles::BookOpen];
                    tree.branch(
                        slot,
                        Panel::new()
                            .color(backing[i % backing.len()])
                            .at(Location::new().xs(
                                0.pct().as_left().with(100.pct().as_right()),
                                0.pct().as_top().with(100.pct().as_bottom()),
                            ))
                            .elevate(Elevation::up(1)),
                    );
                    tree.branch(
                        slot,
                        Icon::new(icons[i % icons.len()])
                            .color(Color::gray(200))
                            .at(Location::new().xs(
                                50.pct().as_center_x().with(32.px().as_width()),
                                12.px().as_top().with(32.px().as_height()),
                            ))
                            .elevate(Elevation::up(2)),
                    );
                    tree.branch(
                        slot,
                        Text::new(format!("content for tab {}", i + 1))
                            .size(FontSize::new(16))
                            .color(Color::gray(200))
                            .at(Location::new().xs(
                                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                                52.px().as_top().with(28.px().as_height()),
                            ))
                            .elevate(Elevation::up(2))
                            .with(HorizontalAlignment::Center),
                    );
                },
            ))
            .colors(Color::green(500), Color::gray(600))
            .rounding(Rounding::Sm)
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Popover -- tap-triggered (this interaction model has no hover concept), opens to the
    // right of its trigger; extent is the one thing only the author can know (here, a
    // plain px guess is fine since this demo's content is fixed and known)
    let (top, bottom) = section(tree, app, &mut cursor, "Popover", 44);
    tree.branch(
        app,
        Popover::new()
            .trigger(|tree: &mut Tree, slot: Entity| {
                tree.branch(
                    slot,
                    Icon::new(IconHandles::Grid)
                        .color(Color::gray(200))
                        .at(Location::new().xs(
                            0.px().as_left().with(24.px().as_width()),
                            50.pct().as_center_y().with(24.px().as_height()),
                        ))
                        .elevate(Elevation::up(1))
                        .with(InteractionPropagation::pass_through()),
                )
            })
            .content(|tree: &mut Tree, slot: Entity| {
                tree.branch(
                    slot,
                    Text::new("popover content")
                        .size(FontSize::new(14))
                        .color(Color::gray(200))
                        .at(Location::new().xs(
                            8.px().as_left().with(100.pct().as_right()),
                            0.pct().as_top().with(100.pct().as_bottom()),
                        ))
                        .elevate(Elevation::up(1))
                        .with((
                            HorizontalAlignment::Center,
                            VerticalAlignment::Middle,
                            InteractionPropagation::pass_through(),
                        )),
                )
            })
            .placement(PopoverPlacement::Right)
            .extent(160.px())
            .colors(Color::gray(600))
            .at(Location::new().xs(
                8.px().as_left().with(44.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // List -- a backing Panel behind it shows its scroll bounds (List itself has no
    // background opinion, same as Carousel/Dropdown). The row text already insets 8px from
    // its own row's edge, so the background doesn't recreate TextInput's cramped-border
    // issue (this isn't text flush against the panel's edge -- it's rows already padded).
    let (top, bottom) = section(tree, app, &mut cursor, "List", 150);
    tree.branch(
        app,
        Panel::new()
            .color(Color::gray(700))
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    tree.branch(
        app,
        foliage::List::new()
            .items(foliage::ListItems::new(20, |tree: &mut Tree, slot: Entity, i| {
                tree.branch(
                    slot,
                    Text::new(format!("row {}", i + 1))
                        .size(FontSize::new(16))
                        .color(Color::gray(300))
                        .at(Location::new().xs(
                            8.px().as_left().with(100.pct().as_right()),
                            0.pct().as_top().with(100.pct().as_bottom()),
                        ))
                        .elevate(Elevation::up(1))
                        .with(VerticalAlignment::Middle),
                );
            }))
            .row_height(28)
            .gap(4)
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(2)),
    );

    cursor += 40;
    let _spacer = tree.branch(
        app,
        foliage::Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                cursor.px().as_top().with(cursor.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
}
