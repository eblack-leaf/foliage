use crate::Trigger;
use crate::composite::pagination::{PageChanged, Pagination, PaginationMode};
use crate::composite::{IndexedSlotFn, PageCount, PageIndex};
use crate::{
    Animation, Color, Component, CurrentInteraction, Disengaged, Ease, EcsExtension, Elevation,
    Entity, Grid, GridExt, InteractionListener, InteractionPropagation, Leaf, LeafSprout, Location,
    Logical, Section, Sequence, Sprout, Tree,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::Res;
use bevy_ecs::system::Query;
use std::sync::Arc;

/// A carousel is one entity: components in ([`CarouselPages`], [`PageIndex`],
/// [`CarouselStyle`], [`CarouselConfig`]), [`PageChanged`] out (the same event Pagination
/// defines -- one paging vocabulary). Pages are author content via the slot convention
/// (see [`crate::composite::IndexedSlotFn`]): each page gets a viewport-sized slot laid
/// side-by-side, the strip slides on page change, and ancestor clipping hides everything
/// off-viewport. Swipe (drag past a fifth of the width), dot-click on the optional embedded
/// [`Pagination`], and programmatic `PageIndex` writes all go through the same door.
#[derive(Component, Copy, Clone)]
pub struct Carousel {}
impl Carousel {
    pub fn new() -> CarouselSprout {
        CarouselSprout::default()
    }
}

/// The page set: count + per-index builder, rewritten as one unit (same shape as
/// [`crate::ListItems`]).
#[derive(Component, Clone)]
pub struct CarouselPages {
    pub count: usize,
    pub(crate) builder: IndexedSlotFn,
}
impl CarouselPages {
    pub fn new(
        count: usize,
        builder: impl Fn(&mut Tree, Entity, usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            count,
            builder: Arc::new(builder),
        }
    }
}

/// Dot colors for the embedded pagination -- carousel's whole visual surface is otherwise
/// the author's pages.
#[derive(Component, Copy, Clone, Default)]
pub struct CarouselStyle {
    pub active: Color,
    pub inactive: Color,
}
/// Structural config, separate from [`CarouselStyle`] so restyling doesn't re-state it.
#[derive(Component, Copy, Clone, Default)]
pub struct CarouselConfig {
    /// embed a [`Pagination`] strip pinned bottom-center; `None` = pages + swipe only
    pub pagination: Option<PaginationMode>,
    /// forwarded straight to the embedded `Pagination`'s own `.dot_size(..)`; `None` falls
    /// back to `Pagination`'s own default (16, 4)
    pub dot_size: Option<(i32, i32)>,
}

/// Private child registry.
#[derive(Component, Clone)]
pub(crate) struct CarouselHandle {
    viewport: Entity,
    slots: Vec<Entity>,
    pagination: Option<Entity>,
}

fn slot_location(slot_index: usize, current: usize) -> Location {
    let offset = (slot_index as i64 - current as i64) * 100;
    Location::new().xs(
        (offset as i32)
            .pct()
            .as_left()
            .with(((offset + 100) as i32).pct().as_right()),
        0.pct().as_top().with(100.pct().as_bottom()),
    )
}

#[derive(Default)]
pub struct CarouselSprout {
    leaf: LeafSprout,
    pages: Option<CarouselPages>,
    page: usize,
    style: CarouselStyle,
    config: CarouselConfig,
}
impl CarouselSprout {
    /// Takes the same [`CarouselPages`] a rebuild writes -- spawn and re-render share one
    /// construction form.
    pub fn pages(mut self, pages: CarouselPages) -> Self {
        self.pages = Some(pages);
        self
    }
    pub fn page(mut self, p: usize) -> Self {
        self.page = p;
        self
    }
    pub fn pagination(mut self, mode: PaginationMode) -> Self {
        self.config.pagination = Some(mode);
        self
    }
    pub fn dot_size(mut self, width: i32, height: i32) -> Self {
        self.config.dot_size = Some((width, height));
        self
    }
    pub fn colors(mut self, active: Color, inactive: Color) -> Self {
        self.style = CarouselStyle { active, inactive };
        self
    }
}
impl Sprout for CarouselSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        let pages = self.pages.expect("Carousel::pages(..) is required");
        let count = pages.count;
        (
            Carousel {},
            PageIndex(self.page.min(count.saturating_sub(1))),
            PageCount(count),
            pages,
            self.style,
            self.config,
            Grid::default(),
            InteractionListener::new(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton: the viewport every page slot lives in. Its listener is the swipe
        // surface -- author pages with their own listeners take precedence over it, so swipe
        // works from any non-interactive part of a page.
        // no listener here: the slots overflow this entity's implicit View (Grid requires
        // one). `disable_drag` keeps that View from ever accepting a raw drag-panning
        // ViewAdjustment -- paging is driven entirely by the Disengaged subscribe below,
        // sliding each slot to its own animated `slot_location`; a stray per-tick drag
        // delta landing on the viewport's own offset too would fight that animation and
        // drift the strip out of sync with where paging thinks it put it (the extent
        // spans every slot, on- and off-screen, so a pan attempt here is not the no-op it
        // used to look like once the interaction walk-up was taught to skip past disabled
        // views and keep going instead of stopping dead at the first one it finds).
        let viewport = tree.branch(
            this,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .with((
                    Grid::default(),
                    InteractionPropagation::grab().disable_drag(),
                )),
        );
        tree.write_to(
            this,
            CarouselHandle {
                viewport,
                slots: Vec::new(),
                pagination: None,
            },
        );
        // swipe: horizontal drag past a fifth of the width steps a page.
        tree.subscribe(
            this,
            move |_: Trigger<Disengaged>,
                  interaction: Res<CurrentInteraction>,
                  sections: Query<&Section<Logical>>,
                  pages: Query<&PageIndex>,
                  counts: Query<&PageCount>,
                  mut tree: Tree| {
                let click = interaction.click();
                let released = click.end.unwrap_or(click.current);
                let dx = released.left() - click.start.left();
                let width = sections.get(this).unwrap().width();
                if dx.abs() < width * 0.2 {
                    return;
                }
                let current = pages.get(this).unwrap().0;
                let count = counts.get(this).unwrap().0;
                let next = if dx < 0.0 {
                    (current + 1).min(count.saturating_sub(1))
                } else {
                    current.saturating_sub(1)
                };
                if next != current {
                    tree.write_to(this, PageIndex(next));
                }
            },
        );
        // the page set: teardown/respawn slots, sync PageCount (root + embedded pagination).
        tree.react::<CarouselPages, _>(
            this,
            move |trigger: Trigger<Insert, CarouselPages>,
                  pages: Query<&CarouselPages>,
                  page_index: Query<&PageIndex>,
                  mut handles: Query<&mut CarouselHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let pages = pages.get(e).unwrap().clone();
                let current = page_index
                    .get(e)
                    .unwrap()
                    .0
                    .min(pages.count.saturating_sub(1));
                let mut handle = handles.get_mut(e).unwrap();
                for slot in handle.slots.drain(..) {
                    tree.remove(slot);
                }
                let viewport = handle.viewport;
                for i in 0..pages.count {
                    // `disable_drag` (the same primitive Slider's knob already uses) stops
                    // drag from auto-panning this slot's own View at all -- there's nothing
                    // left to skew the carousel with, since paging is driven entirely by the
                    // Disengaged subscribe below, not View panning. Unlike
                    // `OverscrollPropagation(false)` (what used to be here), this doesn't
                    // also block wheel-scroll from reaching the page behind the carousel --
                    // wheel bypasses `disable_drag` unconditionally (see the fix already in
                    // `interaction/mod.rs` for the same class of problem on text-input
                    // cursors/slider knobs).
                    let slot = tree.branch(
                        viewport,
                        Leaf::sprout()
                            .at(slot_location(i, current))
                            .elevate(Elevation::up(1))
                            .with((
                                Grid::default(),
                                InteractionPropagation::grab().disable_drag(),
                            )),
                    );
                    (pages.builder)(&mut tree, slot, i);
                    handle.slots.push(slot);
                }
                tree.write_to(e, PageCount(pages.count));
                if let Some(p) = handle.pagination {
                    tree.write_to(p, PageCount(pages.count));
                }
            },
        );
        // page change: slide every slot to its new offset, sync the embedded pagination,
        // announce at the root.
        tree.react::<PageIndex, _>(
            this,
            move |trigger: Trigger<Insert, PageIndex>,
                  page_index: Query<&PageIndex>,
                  counts: Query<&PageCount>,
                  handles: Query<&CarouselHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let count = counts.get(e).unwrap().0;
                let current = page_index.get(e).unwrap().0.min(count.saturating_sub(1));
                let handle = handles.get(e).unwrap().clone();
                let mut seq = Sequence::new(&mut tree);
                for (i, slot) in handle.slots.iter().enumerate() {
                    seq = seq.animate(
                        Animation::new(slot_location(i, current))
                            .targeting(*slot)
                            .start(0)
                            .finish(300)
                            .eased(Ease::EMPHASIS),
                    );
                }
                if let Some(p) = handle.pagination {
                    tree.write_to(p, PageIndex(current));
                }
                tree.trigger_targets(PageChanged::new(current), e);
            },
        );
        // the embedded pagination strip: exists only while configured.
        tree.react_any::<(CarouselStyle, CarouselConfig), _>(
            this,
            move |trigger: Trigger<Insert, (CarouselStyle, CarouselConfig)>,
                  styles: Query<&CarouselStyle>,
                  configs: Query<&CarouselConfig>,
                  counts: Query<&PageCount>,
                  page_index: Query<&PageIndex>,
                  mut handles: Query<&mut CarouselHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let style = *styles.get(e).unwrap();
                let config = *configs.get(e).unwrap();
                let mut handle = handles.get_mut(e).unwrap();
                // strip height derives from the actual configured pip height + a small
                // breathing-room margin (the same 8px gap this codebase uses everywhere else
                // for "a little room around a compact element" -- Button's icon-to-text gap,
                // the chevron's own edge inset, etc.) -- never a flat guess independent of
                // whatever pip size is actually configured.
                let strip_height = config.dot_size.map(|(_, h)| h).unwrap_or(4) + 8;
                match (config.pagination, handle.pagination) {
                    (Some(_), Some(existing)) => {
                        tree.write_to(
                            existing,
                            crate::composite::pagination::PaginationStyle {
                                active: style.active,
                                inactive: style.inactive,
                                mode: config.pagination.unwrap_or_default(),
                                step_icons: None,
                                step_colors: None,
                                dot_size: config.dot_size,
                            },
                        );
                        tree.write_to(
                            existing,
                            Location::new().xs(
                                50.pct().as_center_x().with(50.pct().as_width()),
                                100.pct()
                                    .as_bottom()
                                    .adjust(-8)
                                    .with(strip_height.px().as_height()),
                            ),
                        );
                    }
                    (Some(mode), None) => {
                        let mut pagination = Pagination::new(counts.get(e).unwrap().0)
                            .page(page_index.get(e).unwrap().0)
                            .mode(mode)
                            .colors(style.active, style.inactive);
                        if let Some((w, h)) = config.dot_size {
                            pagination = pagination.dot_size(w, h);
                        }
                        let strip = tree.branch(
                            e,
                            pagination
                                .at(Location::new().xs(
                                    50.pct().as_center_x().with(50.pct().as_width()),
                                    100.pct()
                                        .as_bottom()
                                        .adjust(-8)
                                        .with(strip_height.px().as_height()),
                                ))
                                .elevate(Elevation::up(2)),
                        );
                        // dot clicks come back as the strip's PageChanged; only a genuine
                        // difference re-enters the carousel's own PageIndex door (the write
                        // it makes flows back down to the strip -- the equality check is
                        // what keeps that cycle from being infinite).
                        tree.subscribe(
                            strip,
                            move |trigger: Trigger<PageChanged>,
                                  page_index: Query<&PageIndex>,
                                  mut tree: Tree| {
                                let index = trigger.event().index;
                                if page_index.get(e).unwrap().0 != index {
                                    tree.write_to(e, PageIndex(index));
                                }
                            },
                        );
                        handle.pagination = Some(strip);
                    }
                    (None, Some(existing)) => {
                        tree.remove(existing);
                        handle.pagination = None;
                    }
                    (None, None) => {}
                }
            },
        );
    }
}
