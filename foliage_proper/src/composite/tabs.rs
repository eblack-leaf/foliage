use crate::Trigger;
use crate::composite::pagination::PageChanged;
use crate::composite::segmented_control::{SegmentChanged, SegmentedControl};
use crate::composite::{IndexedSlotFn, PageCount, PageIndex, Root};
use crate::{
    Color, Component, EcsExtension, Elevation, Entity, Grid, GridExt, LeafSprout, Location,
    Rounding, Sprout, Tree, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;
use std::sync::Arc;

/// A tabs widget is one entity: components in ([`TabsPages`], [`PageIndex`], [`PageCount`],
/// [`TabsStyle`]), [`PageChanged`] out (the same paging vocabulary `Carousel` already
/// shares with `Pagination` -- one active index among a known count, one event). The header
/// row IS a `SegmentedControl` (branched as a child, not reimplemented) driving the same
/// `PageIndex`; the content area holds one slot per tab (the same `IndexedSlotFn`
/// convention as `Carousel`'s pages / `List`'s rows), all built once and shown/hidden by
/// `Visibility` on switch rather than torn down and rebuilt -- switching tabs doesn't re-run
/// the author's builder, only the currently-active one is ever visible.
///
/// Element-level, not engine-level: like `List`/`Carousel`, tab content lifecycle is the
/// author's to manage (their builder closure owns whatever state each tab needs) -- this
/// widget only owns which one is currently shown.
#[derive(Component, Copy, Clone)]
pub struct Tabs {}
impl Tabs {
    pub fn new() -> TabsSprout {
        TabsSprout {
            leaf: LeafSprout::default(),
            pages: None,
            page: 0,
            style: TabsStyle::default(),
        }
    }
}

/// The tab set: labels + per-index content builder, rewritten as one unit (same shape as
/// [`crate::composite::carousel::CarouselPages`]).
#[derive(Component, Clone)]
pub struct TabsPages {
    pub labels: Vec<String>,
    pub(crate) builder: IndexedSlotFn,
}
impl TabsPages {
    pub fn new(
        labels: Vec<String>,
        builder: impl Fn(&mut Tree, Entity, usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            labels,
            builder: Arc::new(builder),
        }
    }
}

/// Tabs' OWN config vocabulary, poked as one unit. Colors/rounding are forwarded straight
/// to the header `SegmentedControl`.
#[derive(Component, Copy, Clone)]
pub struct TabsStyle {
    pub active: Color,
    pub inactive: Color,
    /// header label color, in both states -- separate from the two fills so a label is
    /// never the same color as the segment sitting directly behind it.
    pub foreground: Color,
    pub rounding: Rounding,
    pub header_height: i32,
}
impl Default for TabsStyle {
    fn default() -> Self {
        Self {
            active: Color::default(),
            inactive: Color::default(),
            foreground: Color::default(),
            rounding: Rounding::default(),
            header_height: 44,
        }
    }
}

/// Private child registry.
#[derive(Component, Clone)]
pub(crate) struct TabsHandle {
    header: Entity,
    content_slots: Vec<Entity>,
}

pub struct TabsSprout {
    leaf: LeafSprout,
    pages: Option<TabsPages>,
    page: usize,
    style: TabsStyle,
}
impl TabsSprout {
    pub fn pages(mut self, pages: TabsPages) -> Self {
        self.pages = Some(pages);
        self
    }
    pub fn page(mut self, p: usize) -> Self {
        self.page = p;
        self
    }
    pub fn colors(mut self, active: Color, inactive: Color, foreground: Color) -> Self {
        self.style.active = active;
        self.style.inactive = inactive;
        self.style.foreground = foreground;
        self
    }
    pub fn rounding(mut self, r: Rounding) -> Self {
        self.style.rounding = r;
        self
    }
    pub fn header_height(mut self, px: i32) -> Self {
        self.style.header_height = px;
        self
    }
}
impl Sprout for TabsSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        let pages = self.pages.expect("Tabs::pages(..) is required");
        let count = pages.labels.len();
        (
            Tabs {},
            PageIndex(self.page.min(count.saturating_sub(1))),
            PageCount(count),
            pages,
            self.style,
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // header: a SegmentedControl child, not reimplemented -- driven by the same
        // PageIndex the content area watches. No Location yet: header_height lives in
        // TabsStyle, not known until the structure reaction's first fire below.
        let header = tree.branch(this, SegmentedControl::new().elevate(Elevation::up(1)));
        tree.subscribe(
            header,
            move |trigger: Trigger<SegmentChanged>,
                  page_index: Query<&PageIndex>,
                  mut tree: Tree| {
                // equality guard breaks the write cycle this creates: this write's own
                // downstream (Tabs' PageIndex patch) writes SegmentedSelected back onto the
                // header, which would otherwise re-fire this same observer -- matches
                // Carousel's identical guard on its embedded Pagination sync.
                let index = trigger.event().index;
                if page_index.get(this).unwrap().0 != index {
                    tree.write_to(this, PageIndex(index));
                }
            },
        );
        tree.write_to(
            this,
            TabsHandle {
                header,
                content_slots: Vec::new(),
            },
        );

        // structure: rebuild the header's options and the content slots only when the page
        // set or style changes -- mirrors Carousel's own CarouselPages reaction exactly.
        tree.react_any::<(TabsPages, TabsStyle), _>(
            this,
            move |trigger: Trigger<Insert, (TabsPages, TabsStyle)>,
                  pages: Query<&TabsPages>,
                  styles: Query<&TabsStyle>,
                  page_index: Query<&PageIndex>,
                  mut handles: Query<&mut TabsHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let tabs_pages = pages.get(e).unwrap().clone();
                let style = *styles.get(e).unwrap();
                let current = page_index
                    .get(e)
                    .unwrap()
                    .0
                    .min(tabs_pages.labels.len().saturating_sub(1));
                let mut handle = handles.get_mut(e).unwrap();
                // kept in sync with the actual label count on every rebuild -- otherwise a
                // `TabsPages` rewrite to a different length leaves `PageCount` stale from
                // spawn, and the PATCH reaction below clamps `PageIndex` against that wrong
                // bound (Carousel's own `CarouselPages` reaction already does this same
                // write for the identical reason).
                tree.write_to(e, PageCount(tabs_pages.labels.len()));
                tree.write_to(
                    handle.header,
                    (
                        crate::composite::segmented_control::SegmentedOptions(
                            tabs_pages.labels.clone(),
                        ),
                        crate::composite::segmented_control::SegmentedStyle {
                            active: style.active,
                            inactive: style.inactive,
                            foreground: style.foreground,
                            rounding: style.rounding,
                            // top only on each end: the header sits flush over the content
                            // area below it, so the whole bottom edge stays flat instead of
                            // a capsule.
                            first_end: crate::Side::corners(true, false, false, false),
                            last_end: crate::Side::corners(false, true, false, false),
                        },
                        crate::composite::segmented_control::SegmentedSelected(current),
                        Location::new().xs(
                            0.pct().as_left().with(100.pct().as_right()),
                            0.px().as_top().with(style.header_height.px().as_height()),
                        ),
                    ),
                );
                for slot in handle.content_slots.drain(..) {
                    tree.remove(slot);
                }
                for i in 0..tabs_pages.labels.len() {
                    let slot = tree.branch(
                        e,
                        crate::Leaf::sprout()
                            .at(Location::new().xs(
                                0.pct().as_left().with(100.pct().as_right()),
                                style
                                    .header_height
                                    .px()
                                    .as_top()
                                    .with(100.pct().as_bottom()),
                            ))
                            .elevate(Elevation::up(1))
                            .with((Grid::default(), Root(e), Visibility::new(i == current))),
                    );
                    (tabs_pages.builder)(&mut tree, slot, i);
                    handle.content_slots.push(slot);
                }
            },
        );
        // patch: page changes just toggle visibility -- no rebuild, no re-running the
        // author's content builder.
        tree.react::<PageIndex, _>(
            this,
            move |trigger: Trigger<Insert, PageIndex>,
                  page_index: Query<&PageIndex>,
                  counts: Query<&PageCount>,
                  handles: Query<&TabsHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let count = counts.get(e).unwrap().0;
                let current = page_index.get(e).unwrap().0.min(count.saturating_sub(1));
                let handle = handles.get(e).unwrap();
                for (i, slot) in handle.content_slots.iter().enumerate() {
                    tree.write_to(*slot, Visibility::new(i == current));
                }
                tree.write_to(
                    handle.header,
                    crate::composite::segmented_control::SegmentedSelected(current),
                );
                tree.trigger_targets(PageChanged::new(current), e);
            },
        );
    }
}
