use crate::Trigger;
use crate::composite::PageChanged;
use crate::composite::{PageCount, PageIndex};
use crate::{
    Button, ButtonStyle, Color, Component, EcsExtension, Elevation, Entity, FocusBehavior, Grid,
    GridExt, HorizontalAlignment, IconId, IconValue, InteractionListener, InteractionPropagation,
    Leaf, LeafSprout, Location, OnClick, Outline, Panel, Rounding, Sprout, Text, TextValue, Tree,
    VerticalAlignment,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// Pagination is one entity: components in ([`PageIndex`], [`PageCount`],
/// [`PaginationStyle`]), [`PageChanged`] out. Indicator clicks, prev/next steps, and
/// programmatic `PageIndex` writes all go through the same door.
///
/// Indicator entities are built once per STRUCTURE change (count/style) and only patched --
/// color/label writes onto stable entities -- on page changes. Positions never move, and a
/// page change never spawns or removes anything, so event cycles through an enclosing
/// widget (Carousel keeps its embedded strip in sync by writing `PageIndex` back) re-enter
/// a pure patch, not a teardown.
///
/// Prev/next buttons exist only when `.step_icons(..)` supplies icons (the library ships
/// none); without them the indicator strip spans the whole widget.
#[derive(Component, Copy, Clone)]
pub struct Pagination {}
impl Pagination {
    pub fn new(count: usize) -> PaginationSprout {
        PaginationSprout {
            leaf: LeafSprout::default(),
            count,
            page: 0,
            style: PaginationStyle::default(),
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
pub enum PaginationMode {
    /// one dot per page -- the carousel indicator
    #[default]
    Dots,
    /// a fixed strip of at most 5 numbered pips showing a contiguous window of pages
    /// centered on the current one -- the window slides as you page, so any count fits
    /// any width without cramming or filler
    Numbered,
}

const NUMBERED_SLOTS: usize = 5;

/// First page of the visible window -- pure so the build pass, the patch pass, and every
/// click handler agree by construction.
fn window_start(current: usize, count: usize, shown: usize) -> usize {
    current
        .saturating_sub(shown / 2)
        .min(count.saturating_sub(shown))
}

/// Pagination's OWN config vocabulary, poked as one unit.
#[derive(Component, Copy, Clone, Default)]
pub struct PaginationStyle {
    /// current-page indicator color
    pub active: Color,
    /// every other indicator's color
    pub inactive: Color,
    pub mode: PaginationMode,
    /// prev/next button icons; `None` = no step buttons at all
    pub step_icons: Option<(IconId, IconId)>,
    /// prev/next button (icon, fill) colors; `None` falls back to (active, inactive)
    pub step_colors: Option<(Color, Color)>,
    /// `Dots` mode pip (width, height) in px -- a bar, not a circle; `None` falls back to
    /// (16, 4)
    pub dot_size: Option<(i32, i32)>,
}

/// Private child registry: the patch reaction and click handlers need the stable slot
/// entities the structure reaction built. Each slot is (the entity to despawn on rebuild,
/// the entity the patch reaction recolors/retexts) -- for `Dots` these differ (the outer,
/// invisible hit-region vs. the inner visual pip it parents); for `Numbered` one text
/// entity plays both roles. Both are tracked because a structural rebuild has to despawn
/// the hit region as well as the visual it parents; removing only the recolor target
/// leaks an `InteractionListener` per slot on every rebuild.
#[derive(Component, Clone)]
pub(crate) struct PaginationHandle {
    slots: Vec<(Entity, Entity)>,
    step: Option<(Entity, Entity)>,
}
impl PaginationHandle {
    fn empty() -> Self {
        Self {
            slots: Vec::new(),
            step: None,
        }
    }
}

pub struct PaginationSprout {
    leaf: LeafSprout,
    count: usize,
    page: usize,
    style: PaginationStyle,
}
impl PaginationSprout {
    pub fn page(mut self, p: usize) -> Self {
        self.page = p;
        self
    }
    pub fn mode(mut self, m: PaginationMode) -> Self {
        self.style.mode = m;
        self
    }
    pub fn step_icons(mut self, prev: IconId, next: IconId) -> Self {
        self.style.step_icons = Some((prev, next));
        self
    }
    pub fn step_colors(mut self, foreground: Color, background: Color) -> Self {
        self.style.step_colors = Some((foreground, background));
        self
    }
    pub fn dot_size(mut self, width: i32, height: i32) -> Self {
        self.style.dot_size = Some((width, height));
        self
    }
    pub fn colors(mut self, active: Color, inactive: Color) -> Self {
        self.style.active = active;
        self.style.inactive = inactive;
        self
    }
}
impl Sprout for PaginationSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Pagination {},
            PageIndex(self.page.min(self.count.saturating_sub(1))),
            PageCount(self.count),
            self.style,
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton: the indicator strip. Everything inside it (and the optional step
        // buttons beside it) depends on count/style, so it lands via the structure
        // reaction's first fire.
        let strip = tree.branch(
            this,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::default()),
        );
        tree.write_to(this, PaginationHandle::empty());

        // STRUCTURE: rebuild slot/step entities only when count or style changes -- never
        // on a page change, so no event cycle can remove what a same-flush fire spawned.
        tree.react_any::<(PageCount, PaginationStyle), _>(
            this,
            move |trigger: Trigger<Insert, (PageCount, PaginationStyle)>,
                  counts: Query<&PageCount>,
                  styles: Query<&PaginationStyle>,
                  pages: Query<&PageIndex>,
                  mut handles: Query<&mut PaginationHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let count = counts.get(e).unwrap().0;
                let current = pages.get(e).unwrap().0.min(count.saturating_sub(1));
                let style = *styles.get(e).unwrap();
                let mut handle = handles.get_mut(e).unwrap();
                for (root, _) in handle.slots.drain(..) {
                    tree.remove(root);
                }
                // The strip is one grid row; the step buttons (when configured) are its first
                // and last columns, the indicators the columns between. Every element -- step
                // buttons included -- is sized by its own grid cell, so nothing here computes a
                // pixel size. A step button is just a `Button`: it lays out its own icon, and
                // `AspectRatio(1.0)` squares the button to a circle within its cell.
                let shown = match style.mode {
                    PaginationMode::Dots => count,
                    PaginationMode::Numbered => count.min(NUMBERED_SLOTS),
                };
                let has_steps = style.step_icons.is_some();
                let lead = if has_steps { 1 } else { 0 };
                let cols = shown + if has_steps { 2 } else { 0 };
                tree.write_to(strip, Grid::new(cols.max(1).col(), 1.row()));
                match (style.step_icons, handle.step) {
                    (Some((prev_icon, next_icon)), existing) => {
                        let (prev, next) = existing.unwrap_or_else(|| {
                            let prev = tree.branch(
                                strip,
                                Button::new()
                                    .rounding(Rounding::Full)
                                    .elevate(Elevation::up(1))
                                    .with(crate::AspectRatio::new().xs(1.0)),
                            );
                            let next = tree.branch(
                                strip,
                                Button::new()
                                    .rounding(Rounding::Full)
                                    .elevate(Elevation::up(1))
                                    .with(crate::AspectRatio::new().xs(1.0)),
                            );
                            tree.on_click(
                                prev,
                                move |_: Trigger<OnClick>,
                                      pages: Query<&PageIndex>,
                                      mut tree: Tree| {
                                    let current = pages.get(e).unwrap().0;
                                    tree.write_to(e, PageIndex(current.saturating_sub(1)));
                                },
                            );
                            tree.on_click(
                                next,
                                move |_: Trigger<OnClick>,
                                      pages: Query<&PageIndex>,
                                      counts: Query<&PageCount>,
                                      mut tree: Tree| {
                                    let current = pages.get(e).unwrap().0;
                                    let count = counts.get(e).unwrap().0;
                                    tree.write_to(
                                        e,
                                        PageIndex((current + 1).min(count.saturating_sub(1))),
                                    );
                                },
                            );
                            (prev, next)
                        });
                        handle.step = Some((prev, next));
                        let (fg, bg) = style.step_colors.unwrap_or((style.active, style.inactive));
                        let button_style = ButtonStyle {
                            foreground: fg,
                            background: bg,
                            outline: Outline::default(),
                            rounding: Rounding::Full,
                        };
                        // just place the buttons in the first and last grid columns (their
                        // size is the cell; the button lays out its own icon). IconValue/style
                        // are config, not layout -- so those alone stay in the reaction.
                        tree.write_to(
                            prev,
                            (
                                IconValue(prev_icon),
                                button_style,
                                Location::new().xs(
                                    1.col().as_left().with(1.col().as_right()),
                                    1.row().as_top().with(1.row().as_bottom()),
                                ),
                            ),
                        );
                        tree.write_to(
                            next,
                            (
                                IconValue(next_icon),
                                button_style,
                                Location::new().xs(
                                    cols.col().as_left().with(cols.col().as_right()),
                                    1.row().as_top().with(1.row().as_bottom()),
                                ),
                            ),
                        );
                        if current == 0 {
                            tree.disable(prev);
                        } else {
                            tree.enable(prev);
                        }
                        if current + 1 >= count {
                            tree.disable(next);
                        } else {
                            tree.enable(next);
                        }
                    }
                    (None, Some((prev, next))) => {
                        tree.remove([prev, next]);
                        handle.step = None;
                    }
                    (None, None) => {}
                }
                let ws = window_start(current, count, shown);
                for s in 0..shown {
                    match style.mode {
                        PaginationMode::Dots => {
                            let (dot_w, dot_h) = style.dot_size.unwrap_or((16, 4));
                            // hit region is a comfortable minimum regardless of how thin the
                            // visual pip is -- a 4px-tall bar is a real bar to look at, not a
                            // real click target, so the two are separate entities: an
                            // invisible, generously-sized hit region carries the listener,
                            // the visual pip inside it is what's actually drawn.
                            let mut transparent = Color::default();
                            transparent.set_alpha(0.0);
                            let hit = tree.branch(
                                strip,
                                Panel::new()
                                    .color(transparent)
                                    .at(Location::new().xs(
                                        (s + 1 + lead)
                                            .col()
                                            .as_center_x()
                                            .with(dot_w.max(24).px().as_width()),
                                        50.pct().as_center_y().with(24.px().as_height()),
                                    ))
                                    .elevate(Elevation::up(1))
                                    .with((
                                        Grid::default(),
                                        InteractionListener::new(),
                                        FocusBehavior::ignore(),
                                    )),
                            );
                            let dot = tree.branch(
                                hit,
                                Panel::new()
                                    .rounding(Rounding::Full)
                                    .color(if s == current {
                                        style.active
                                    } else {
                                        style.inactive
                                    })
                                    .at(Location::new().xs(
                                        50.pct().as_center_x().with(dot_w.px().as_width()),
                                        50.pct().as_center_y().with(dot_h.px().as_height()),
                                    ))
                                    .elevate(Elevation::up(1))
                                    .with((
                                        InteractionPropagation::pass_through(),
                                        FocusBehavior::ignore(),
                                    )),
                            );
                            tree.on_click(hit, move |_: Trigger<OnClick>, mut tree: Tree| {
                                tree.write_to(e, PageIndex(s));
                            });
                            handle.slots.push((hit, dot));
                        }
                        PaginationMode::Numbered => {
                            let number = tree.branch(
                                strip,
                                Text::new(format!("{}", ws + s + 1))
                                    .color(if ws + s == current {
                                        style.active
                                    } else {
                                        style.inactive
                                    })
                                    .at(Location::new().xs(
                                        (s + 1 + lead)
                                            .col()
                                            .as_left()
                                            .with((s + 1 + lead).col().as_right()),
                                        1.row().as_top().with(1.row().as_bottom()),
                                    ))
                                    .elevate(Elevation::up(1))
                                    .with((
                                        HorizontalAlignment::Center,
                                        VerticalAlignment::Middle,
                                        InteractionListener::new(),
                                        FocusBehavior::ignore(),
                                    )),
                            );
                            // which page this slot means depends on where the window sits
                            // at click time
                            tree.on_click(
                                number,
                                move |_: Trigger<OnClick>,
                                      pages: Query<&PageIndex>,
                                      counts: Query<&PageCount>,
                                      mut tree: Tree| {
                                    let count = counts.get(e).unwrap().0;
                                    let current =
                                        pages.get(e).unwrap().0.min(count.saturating_sub(1));
                                    let ws =
                                        window_start(current, count, count.min(NUMBERED_SLOTS));
                                    tree.write_to(e, PageIndex(ws + s));
                                },
                            );
                            handle.slots.push((number, number));
                        }
                    }
                }
            },
        );
        // PATCH: page changes recolor (and, for Numbered, slide the window's labels across)
        // the stable indicators and announce -- no spawns, no removes; safe to re-enter
        // from enclosing widgets' sync writes.
        tree.react::<PageIndex, _>(
            this,
            move |trigger: Trigger<Insert, PageIndex>,
                  pages: Query<&PageIndex>,
                  counts: Query<&PageCount>,
                  styles: Query<&PaginationStyle>,
                  handles: Query<&PaginationHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let count = counts.get(e).unwrap().0;
                let current = pages.get(e).unwrap().0.min(count.saturating_sub(1));
                let style = *styles.get(e).unwrap();
                let handle = handles.get(e).unwrap().clone();
                if let Some((prev, next)) = handle.step {
                    if current == 0 {
                        tree.disable(prev);
                    } else {
                        tree.enable(prev);
                    }
                    if current + 1 >= count {
                        tree.disable(next);
                    } else {
                        tree.enable(next);
                    }
                }
                match style.mode {
                    PaginationMode::Dots => {
                        for (s, (_, slot)) in handle.slots.iter().enumerate() {
                            tree.write_to(
                                *slot,
                                if s == current {
                                    style.active
                                } else {
                                    style.inactive
                                },
                            );
                        }
                    }
                    PaginationMode::Numbered => {
                        let ws = window_start(current, count, handle.slots.len());
                        for (s, (_, slot)) in handle.slots.iter().enumerate() {
                            tree.write_to(
                                *slot,
                                (
                                    TextValue(format!("{}", ws + s + 1)),
                                    if ws + s == current {
                                        style.active
                                    } else {
                                        style.inactive
                                    },
                                ),
                            );
                        }
                    }
                }
                tree.trigger_targets(PageChanged::new(current), e);
            },
        );
    }
}
