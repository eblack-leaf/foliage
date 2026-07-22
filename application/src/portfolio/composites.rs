//! A live tour of every composite the library ships, one labeled instance (or a couple of
//! meaningful variants) per row, stacked down a single tall column -- the page scrolls
//! rather than trying to fit everything on screen at once.

use crate::icons::IconHandles;
use foliage::{
    AssetKey, Carousel, CarouselPages, Checkbox, Color, DashPattern, Dropdown, EcsExtension,
    Elevation, Entity, FontSize, GridExt, HorizontalAlignment, Icon, InteractionPropagation,
    Location, Opacity, Pagination, PaginationMode, Panel, Polygon, Polyline, PolylineDrawProgress,
    Popover, PopoverPlacement, Position, Query, RadioGroup, Res, Rounding, SegmentedControl,
    Slider, Sprout, Tabs, TabsPages, Text, TextInput, Time, Toggle, Tree, VerticalAlignment,
    component,
};

/// Drives the demo's draw-in `Polyline` through a repeating "draw the path in" cycle. All
/// the actual path/segment math (which portion of the path is revealed at a given
/// progress) now lives in the library itself (`PolylineDrawProgress`, see its own docs) --
/// this is nothing more than a looping timer writing a single `f32` each frame.
#[component]
pub(crate) struct DrawProgress {
    elapsed: f32,
}
const DRAW_CYCLE_SECS: f32 = 1.5;

/// Registered onto `foliage.user` (the app-code schedule, see `Foliage::user`) from
/// `main.rs`.
pub(crate) fn drive_polyline_draw(
    time: Res<Time>,
    mut query: Query<(Entity, &mut DrawProgress)>,
    mut tree: Tree,
) {
    for (entity, mut progress) in query.iter_mut() {
        progress.elapsed += time.frame_diff().as_secs_f32();
        let t = (progress.elapsed % DRAW_CYCLE_SECS) / DRAW_CYCLE_SECS;
        tree.write_to(entity, PolylineDrawProgress(t));
    }
}

/// One section: a small gray label above whatever `.at()` box the caller gives the
/// composite instance -- returns that box's (top, bottom) in px so the caller only ever
/// picks a height, never computes an absolute position by hand.
fn section(
    tree: &mut Tree,
    parent: Entity,
    cursor: &mut i32,
    label: &str,
    height: i32,
) -> (i32, i32) {
    tree.branch(
        parent,
        Text::new(label)
            .size(FontSize::new(14))
            .color(Color::gray(500))
            .at(Location::new().xs(
                8.px().as_left().with(100.pct().as_right().adjust(-8)),
                (*cursor)
                    .px()
                    .as_top()
                    .with((*cursor + 20).px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    *cursor += 28;
    let top = *cursor;
    let bottom = *cursor + height;
    *cursor = bottom + 28;
    (top, bottom)
}

pub(crate) fn build(tree: &mut Tree, app: Entity, _artwork: [AssetKey; 3]) {
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
            .check_icon(IconHandles::Check)
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
            .colors(Color::green(500), Color::gray(600), Color::gray(900))
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
            .outline(3)
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
                3,
                move |tree: &mut Tree, slot: Entity, i| {
                    // placeholder pages -- a plain colored backing + a page-number label,
                    // not a real image (the artwork crops looked wrong here). avoid green --
                    // that's the brand/active accent (the dots' active color, the Tabs
                    // indicator) everywhere else on this page. also avoid gray(600) (the
                    // dots' own inactive color) and gray(800) (the modal's own backdrop).
                    let backing = [Color::blue(700), Color::orange(700), Color::gray(500)];
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
                        Text::new(format!("page {}", i + 1))
                            .size(FontSize::new(16))
                            .color(Color::gray(200))
                            .at(Location::new().xs(
                                0.pct().as_left().with(100.pct().as_right()),
                                0.pct().as_top().with(100.pct().as_bottom()),
                            ))
                            .elevate(Elevation::up(2))
                            .with((HorizontalAlignment::Center, VerticalAlignment::Middle)),
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
                    // tab, without turning it into a bright color swatch. a couple distinct
                    // hues here (unlike Carousel's backing, this one never sits under the
                    // Tabs colors() config -- that only styles the header SegmentedControl)
                    // -- still avoid gray(800), the modal's own backdrop, same mistake as
                    // the Popover surface earlier.
                    let backing = [Color::teal(700), Color::indigo(700), Color::gray(500)];
                    let icons = [
                        IconHandles::Terminal,
                        IconHandles::Layers,
                        IconHandles::BookOpen,
                    ];
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
            .colors(Color::green(500), Color::gray(600), Color::gray(900))
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
                // looks like this app's own icon_button (filled circle) so it actually
                // reads as clickable -- can't *be* a real Button here though, since a real
                // Button's own InteractionListener would grab the click before it ever
                // reaches Popover's root (the same constraint documented on Popover
                // itself): a plain circular Panel + Icon, both passed through, instead.
                tree.branch(
                    slot,
                    Panel::new()
                        .rounding(Rounding::Full)
                        .color(Color::gray(700))
                        .at(Location::new().xs(
                            0.px().as_left().with(44.px().as_width()),
                            50.pct().as_center_y().with(44.px().as_height()),
                        ))
                        .elevate(Elevation::up(1))
                        .with(InteractionPropagation::pass_through()),
                );
                tree.branch(
                    slot,
                    Icon::new(IconHandles::Grid)
                        .color(Color::gray(200))
                        .at(Location::new().xs(
                            10.px().as_left().with(24.px().as_width()),
                            50.pct().as_center_y().with(24.px().as_height()),
                        ))
                        .elevate(Elevation::up(2))
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
                20.px().as_left().with(100.pct().as_right().adjust(-20)),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    tree.branch(
        app,
        foliage::List::new()
            .items(foliage::ListItems::new(
                20,
                |tree: &mut Tree, slot: Entity, i| {
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
                },
            ))
            .row_height(28)
            .gap(4)
            .at(Location::new().xs(
                20.px().as_left().with(100.pct().as_right().adjust(-20)),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(2)),
    );

    // Polygon -- the "expressive shape" primitive: sides/rounding/rotation are plain
    // animatable scalars, not a Panel variant (Panel keeps arbitrary-aspect rects with
    // independent per-corner radii; this is regular, roughly-square shapes only). One
    // color throughout so the row reads as "compare shapes," not "compare colors" --
    // sharp triangle, lightly-rounded pentagon, more-rounded hexagon, and full rounding
    // (any side count converges to a true circle once rounding hits 1.0).
    let (top, bottom) = section(tree, app, &mut cursor, "Polygon", 60);
    let polygons: [(f32, f32); 4] = [(3.0, 0.0), (5.0, 0.15), (6.0, 0.4), (8.0, 1.0)];
    for (i, (sides, rounding)) in polygons.into_iter().enumerate() {
        let left = 8 + i as i32 * 68;
        tree.branch(
            app,
            Polygon::new()
                .sides(sides)
                .rounding(rounding)
                .color(Color::gray(600))
                .at(Location::new().xs(
                    left.px().as_left().with(60.px().as_width()),
                    top.px().as_top().with(bottom.px().as_bottom()),
                ))
                .elevate(Elevation::up(1)),
        );
    }

    // Polyline -- built from Line + Polygon, not its own render pipeline (see the type's own
    // docs). Points are authored in the polyline's own local px space, same zigzag shape on
    // both sides so the row reads as "compare line style" -- plain vs. dashed -- not "compare
    // shapes." One color throughout, same reasoning as the Polygon row above.
    let (top, bottom) = section(tree, app, &mut cursor, "Polyline", 60);
    let zigzag: Vec<Position<foliage::Logical>> = vec![
        (10, 50).into(),
        (50, 10).into(),
        (90, 50).into(),
        (130, 10).into(),
        (170, 40).into(),
    ];
    tree.branch(
        app,
        Polyline::new()
            .points(zigzag.clone())
            .weight(1)
            .color(Color::gray(400))
            .at(Location::new().xs(
                8.px().as_left().with(180.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .with(DrawProgress { elapsed: 0.0 }),
    );
    tree.branch(
        app,
        Polyline::new()
            .points(zigzag)
            .weight(3)
            .color(Color::gray(400))
            .dash(DashPattern::new(10.0, 6.0))
            .at(Location::new().xs(
                208.px().as_left().with(180.px().as_width()),
                top.px().as_top().with(bottom.px().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // Icon -- one 48px MTSDF field per icon serves every on-screen size, so these rows exist
    // to eyeball the scaling quality: chasms at stroke intersections, wobbly curves, and
    // rounded-off corners are the field's failure modes, and they're most visible well above
    // the field's own resolution. Icons chosen for maximum failure visibility: x (crossing
    // strokes), box (Y-junctions), github (dense curves + junctions), search (a true circle,
    // where any polygonization would show). Bottom-aligned so the size progression reads.
    let inspect = [
        ("Icon (x)", IconHandles::X),
        ("Icon (box)", IconHandles::Box),
        ("Icon (github)", IconHandles::Github),
        ("Icon (search)", IconHandles::Search),
    ];
    let sizes = [16, 24, 48, 96, 144];
    for (label, handle) in inspect {
        let (_top, bottom) = section(tree, app, &mut cursor, label, 144);
        let mut left = 8;
        for size in sizes {
            tree.branch(
                app,
                Icon::new(handle)
                    .color(Color::gray(200))
                    .at(Location::new().xs(
                        left.px().as_left().with(size.px().as_width()),
                        (bottom - size).px().as_top().with(bottom.px().as_bottom()),
                    ))
                    .elevate(Elevation::up(1)),
            );
            left += size + 16;
        }
    }

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
