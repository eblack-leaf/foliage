use crate::ash::clip::ClipToViewport;
use crate::{Component, Logical, Moment, Position, Section, Stem, Tree};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, DetectChanges, Query, Ref};
use std::collections::HashSet;

#[derive(Component, Copy, Clone, Debug, Default)]
pub(crate) struct ViewAdjustment(pub(crate) Position<Logical>);
#[derive(Component, Copy, Clone, Debug)]
pub struct OverscrollPropagation(pub bool);
impl Default for OverscrollPropagation {
    fn default() -> Self {
        OverscrollPropagation(true)
    }
}
/// Wheel-scroll momentum, tracked per view: a multiplier applied to each wheel tick's raw
/// delta, separate from drag (which stays 1:1, no momentum) since a wheel tick is a
/// discrete pulse, not continuous tracking. Slow to start (base multiplier) so a single
/// tick stays a subtle nudge; grows toward a cap while ticks keep arriving close together,
/// resetting back to base once they stop -- pure state, no interpolation/animation
/// involved, since the delta itself (not a value being eased toward) is what scales.
#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct ScrollMomentum {
    pub(crate) value: f32,
    pub(crate) last_tick: Option<Moment>,
}
impl Default for ScrollMomentum {
    fn default() -> Self {
        Self {
            value: Self::BASE,
            last_tick: None,
        }
    }
}
impl ScrollMomentum {
    const BASE: f32 = 1.0;
    const GROWTH: f32 = 0.15;
    const MAX: f32 = 3.0;
    const WINDOW_MS: u128 = 150;
    /// Given the current state, returns (the multiplier to scale this tick's delta by, the
    /// updated state to write back).
    pub(crate) fn tick(self) -> (f32, Self) {
        let now = Moment::now();
        let value = match self.last_tick {
            Some(last) if now.duration_since(last).as_millis() < Self::WINDOW_MS => {
                (self.value + Self::GROWTH).min(Self::MAX)
            }
            _ => Self::BASE,
        };
        (
            value,
            Self {
                value,
                last_tick: Some(now),
            },
        )
    }
}
#[derive(Component, Copy, Clone, Debug)]
#[require(ViewAdjustment, OverscrollPropagation, ScrollMomentum)]
pub struct View {
    pub offset: Position<Logical>,
    pub extent: Section<Logical>,
}
impl View {
    pub fn new() -> View {
        View {
            offset: Default::default(),
            extent: Default::default(),
        }
    }
}
impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}
fn ovrscrl(
    entity: Entity,
    ovr: Position<Logical>,
    views: &mut Query<&mut View>,
    propagations: &Query<&OverscrollPropagation>,
    contexts: &Query<(Entity, Ref<Stem>)>,
    sections: &Query<(Entity, Ref<Section<Logical>>)>,
    to_trigger: &mut HashSet<Entity>,
) -> (Option<Entity>, Position<Logical>) {
    let propagation = propagations.get(entity).unwrap();
    let old_offset = views.get(entity).unwrap().offset;
    let mut view = views.get_mut(entity).unwrap();
    view.offset += ovr;
    let section = *sections.get(entity).unwrap().1;
    let mut over = Position::default();
    let over_right = section.right() + view.offset.left();
    if over_right > view.extent.width() {
        let val = view.extent.width() - section.right();
        over.set_left(view.offset.left() - val);
        view.offset.set_left(val);
    }
    let over_bottom = section.bottom() + view.offset.top();
    if over_bottom > view.extent.height() {
        let val = view.extent.height() - section.bottom();
        over.set_top(view.offset.top() - val);
        view.offset.set_top(val);
    }
    let over_left = section.left() + view.offset.left();
    if over_left < view.extent.left() {
        let val = view.extent.left() - section.left();
        over.set_left(view.offset.left() - val);
        view.offset.set_left(val);
    }
    let over_top = section.top() + view.offset.top();
    if over_top < view.extent.top() {
        let val = view.extent.top() - section.top();
        over.set_top(view.offset.top() - val);
        view.offset.set_top(val);
    }
    if old_offset != view.offset {
        to_trigger.insert(entity);
    }
    if !propagation.0 {
        return (None, Position::default());
    }
    (contexts.get(entity).unwrap().1.id, over)
}
pub(crate) fn extent_check(
    adjustments: Query<(Entity, &ViewAdjustment), Changed<ViewAdjustment>>,
    mut views: Query<&mut View>,
    propagations: Query<&OverscrollPropagation>,
    contexts: Query<(Entity, Ref<Stem>)>,
    sections: Query<(Entity, Ref<Section<Logical>>)>,
    clip_to_viewport: Query<&ClipToViewport>,
    mut tree: Tree,
) {
    let mut to_check = HashSet::new();
    for (entity, adjustment) in adjustments.iter() {
        tracing::trace!(entity = ?entity, adjustment = ?adjustment.0, "grid::view: extent_check_v2 saw changed ViewAdjustment");
        to_check.insert(entity);
    }
    for (entity, context) in contexts.iter() {
        if context.is_changed() {
            if let Some(id) = context.id {
                to_check.insert(id);
            }
        }
    }
    for (entity, section) in sections.iter() {
        if section.is_changed() {
            if let Ok((_, context)) = contexts.get(entity) {
                if let Some(id) = context.id {
                    to_check.insert(id);
                }
            }
        }
    }
    if to_check.is_empty() {
        return;
    }
    for entity in to_check.iter() {
        let section = *sections.get(*entity).unwrap().1;
        let old_extent = views.get(*entity).unwrap().extent;
        let old_offset = views.get(*entity).unwrap().offset;
        views.get_mut(*entity).unwrap().extent =
            Section::new(section.position, (section.right(), section.bottom()));
        tracing::trace!(
            entity = ?entity,
            own_section = ?section,
            old_extent = ?old_extent,
            reset_extent = ?views.get(*entity).unwrap().extent,
            current_offset = ?old_offset,
            "grid::view: extent reset to own section (pre-regrow)"
        );
    }
    for (entity, context) in contexts.iter() {
        if let Some(id) = context.id {
            if to_check.contains(&id) {
                // a `ClipToViewport` child is a floating overlay, not really "contained
                // scrollable content" of its structural parent (same reasoning as why it
                // already escapes the parent's own clip instead of intersecting with it) --
                // without this, a small trigger entity that merely needs a `Grid` for its own
                // unrelated layout (hence a `View`, via `Grid`'s own requirement) would have
                // its View extent inflated by wherever its open overlay child currently
                // renders, falsely reporting scrollable room it was never meant to have
                // (concretely: Dropdown's trigger vs. its own option-list surface).
                if clip_to_viewport.get(entity).is_ok() {
                    continue;
                }
                if let Ok(mut view) = views.get_mut(id) {
                    if let Ok((_, section)) = sections.get(entity) {
                        let mut relative = *section;
                        relative.position += view.offset;
                        if relative.left() < view.extent.left() {
                            view.extent.set_left(relative.left());
                        }
                        if relative.right() > view.extent.width() {
                            view.extent.set_width(relative.right());
                        }
                        if relative.top() < view.extent.top() {
                            view.extent.set_top(relative.top());
                        }
                        if relative.bottom() > view.extent.height() {
                            view.extent.set_height(relative.bottom());
                        }
                    }
                }
            }
        }
    }
    for entity in to_check.iter() {
        tracing::trace!(
            entity = ?entity,
            grown_extent = ?views.get(*entity).unwrap().extent,
            "grid::view: extent after regrow-from-children pass"
        );
    }
    let mut to_trigger = HashSet::new();
    for entity in to_check.iter() {
        let before = views.get(*entity).unwrap().offset;
        let _ovr = ovrscrl(
            *entity,
            Position::default(),
            &mut views,
            &propagations,
            &contexts,
            &sections,
            &mut to_trigger,
        );
        let after = views.get(*entity).unwrap().offset;
        tracing::trace!(
            entity = ?entity,
            before_offset = ?before,
            after_offset = ?after,
            "grid::view: initial ovrscrl (zero-delta clamp) pass"
        );
    }
    for entity in to_check.iter() {
        let mut view = views.get_mut(*entity).unwrap();
        if let Ok((_, adjustment)) = adjustments.get(*entity) {
            let before = view.offset;
            view.offset += adjustment.0;
            tracing::trace!(entity = ?entity, before = ?before, after = ?view.offset, "grid::view: applied ViewAdjustment to offset");
            to_trigger.insert(*entity);
        }
    }
    for entity in to_check {
        let mut overscroll = ovrscrl(
            entity,
            Position::default(),
            &mut views,
            &propagations,
            &contexts,
            &sections,
            &mut to_trigger,
        );
        while overscroll.0.is_some() && overscroll.1 != Position::default() {
            let id = overscroll.0.unwrap();
            overscroll = ovrscrl(
                id,
                overscroll.1,
                &mut views,
                &propagations,
                &contexts,
                &sections,
                &mut to_trigger,
            );
        }
    }
    let mut in_chain = HashSet::new();
    for entity in to_trigger.iter() {
        let mut stem = *contexts.get(*entity).unwrap().1;
        while stem.id.is_some() {
            let id = stem.id.unwrap();
            if to_trigger.contains(&id) {
                in_chain.insert(*entity);
                break;
            }
            stem = *contexts.get(id).unwrap().1;
        }
    }
    for entity in to_trigger.difference(&in_chain) {
        let section = *sections.get(*entity).unwrap().1;
        tracing::trace!(entity = ?entity, section = ?section, "grid::view: re-inserting Section<Logical> to cascade");
        tree.entity(*entity).insert(section);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimeDelta;

    #[test]
    fn the_first_ever_tick_returns_the_base_multiplier() {
        let (multiplier, updated) = ScrollMomentum::default().tick();
        assert_eq!(multiplier, ScrollMomentum::BASE);
        assert!(updated.last_tick.is_some());
    }

    #[test]
    fn a_tick_within_the_window_grows_the_multiplier() {
        let momentum = ScrollMomentum {
            value: ScrollMomentum::BASE,
            last_tick: Some(Moment::now() - TimeDelta::from_millis(50)),
        };
        let (multiplier, _) = momentum.tick();
        assert_eq!(multiplier, ScrollMomentum::BASE + ScrollMomentum::GROWTH);
    }

    #[test]
    fn a_tick_after_the_window_resets_to_base_regardless_of_prior_value() {
        let momentum = ScrollMomentum {
            value: 2.5,
            last_tick: Some(Moment::now() - TimeDelta::from_millis(200)),
        };
        let (multiplier, _) = momentum.tick();
        assert_eq!(
            multiplier,
            ScrollMomentum::BASE,
            "a pause longer than WINDOW_MS should reset the ramp, not continue from where it left off"
        );
    }

    #[test]
    fn growth_is_capped_at_max() {
        let momentum = ScrollMomentum {
            value: ScrollMomentum::MAX - 0.05,
            last_tick: Some(Moment::now() - TimeDelta::from_millis(50)),
        };
        let (multiplier, _) = momentum.tick();
        assert_eq!(
            multiplier,
            ScrollMomentum::MAX,
            "growth should clamp at MAX, not overshoot it"
        );
    }

    #[test]
    fn repeated_fast_ticks_accumulate_growth_across_calls() {
        let mut momentum = ScrollMomentum::default();
        let (first, updated) = momentum.tick();
        momentum = updated;
        // simulate the next tick arriving well within the window, without a real sleep
        momentum.last_tick = Some(Moment::now() - TimeDelta::from_millis(50));
        let (second, _) = momentum.tick();
        assert!(
            second > first,
            "a second fast tick should ramp higher than the first"
        );
    }
}
