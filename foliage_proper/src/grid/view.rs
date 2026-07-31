use crate::ash::clip::ClipToViewport;
use crate::interaction::CurrentInteraction;
use crate::{Component, Logical, Moment, Position, Section, Stem, Tree};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, DetectChanges, Query, Ref, Res, Resource};
use std::collections::HashSet;

#[derive(Component, Copy, Clone, Debug, Default)]
pub(crate) struct ViewAdjustment(pub(crate) Position<Logical>);
/// Author-facing "please scroll here" request, as a 0..1 fraction of the entity's
/// *current* scrollable range per axis -- not a raw pixel offset. A raw offset would go
/// stale the instant `extent`/`Section` next shift (content added/removed, a resize), and
/// would let a caller park the view somewhere `ovrscrl`'s own clamp would never have
/// allowed a drag/wheel gesture to reach. `extent_check` resolves this the same way it
/// resolves a drag: recompute the real pixel delta from the live `View`/`Section` and feed
/// it through the same clamp `ViewAdjustment` already goes through, so a percent-driven
/// scrollbar can't desync from what dragging/wheeling the content directly would allow.
/// One-shot -- consumed and removed the same resolve pass it lands in, same "request,
/// not raw state" shape `Visibility` (vs. its internal `ResolvedVisibility`) already uses.
/// Either axis left `None` leaves that axis's current scroll position untouched.
#[derive(Component, Copy, Clone, Debug, Default)]
pub struct ScrollTo {
    pub x: Option<f32>,
    pub y: Option<f32>,
}
impl ScrollTo {
    /// Scroll horizontally to `percent` of the range, leaving the vertical position be.
    pub fn x(percent: f32) -> Self {
        Self {
            x: Some(percent.clamp(0.0, 1.0)),
            y: None,
        }
    }
    /// Scroll vertically to `percent` of the range, leaving the horizontal position be.
    pub fn y(percent: f32) -> Self {
        Self {
            x: None,
            y: Some(percent.clamp(0.0, 1.0)),
        }
    }
    /// Scroll both axes.
    pub fn xy(x: f32, y: f32) -> Self {
        Self {
            x: Some(x.clamp(0.0, 1.0)),
            y: Some(y.clamp(0.0, 1.0)),
        }
    }
}
#[derive(Component, Copy, Clone, Debug)]
/// Whether scroll this view cannot consume passes outward to an ancestor view.
///
/// On by default, which is what lets a drag on inner content still scroll the page behind
/// it once the inner view hits its own end. Turn it off to trap scrolling inside a region.
pub struct OverscrollPropagation(pub bool);
impl Default for OverscrollPropagation {
    fn default() -> Self {
        OverscrollPropagation(true)
    }
}
/// Wheel-scroll inertia, tracked per view: a multiplier applied to each wheel tick's raw
/// delta, separate from drag (which stays 1:1, no scaling) since a wheel tick is a
/// discrete pulse, not continuous tracking. Slow to start (base multiplier) so a single
/// tick stays a subtle nudge; grows toward a cap while ticks keep arriving close together,
/// resetting back to base once they stop -- pure state, no interpolation/animation
/// involved, since the delta itself (not a value being eased toward) is what scales.
#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct ScrollInertia {
    pub(crate) value: f32,
    pub(crate) last_tick: Option<Moment>,
}
impl Default for ScrollInertia {
    fn default() -> Self {
        Self {
            value: Self::BASE,
            last_tick: None,
        }
    }
}
impl ScrollInertia {
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
/// End-user-tunable knobs for drag/touch release momentum -- a real `Resource`, the same
/// "insert your own before `photosynthesize`" pattern [`Layout`](crate::Layout)'s own
/// breakpoints use, since the right feel here genuinely depends on the app (a dense list
/// vs. a wide gallery, say). Distinct from [`ScrollInertia`], which scales each *new*
/// wheel tick while ticks keep arriving -- this instead governs what happens once a
/// drag/touch release has *no more input to scale*: whether it just stops (a slow,
/// deliberate drag) or keeps coasting on its own release velocity (a flick, i.e. real
/// "momentum scrolling", the term most native/browser touch-scroll implementations use
/// for exactly this).
#[derive(Resource, Copy, Clone, Debug)]
pub struct ScrollMomentum {
    /// px/ms magnitude of release velocity below which a drag just stops -- reads as "the
    /// last motion before lifting off was already slow/settled", not a flick.
    pub velocity_threshold: f32,
    /// Multiplier applied to the release velocity when a coast starts. The only knob that
    /// changes how fast a coast begins; `decay` and `stop_epsilon` govern how it ends.
    pub launch: f32,
    /// Fraction of velocity retained per elapsed ms while *actively coasting* (exponential
    /// decay, applied as `decay.powf(elapsed_ms)` each tick so it's frame-rate independent).
    pub decay: f32,
    /// px/ms magnitude below which a coast is considered finished and stops.
    pub stop_epsilon: f32,
    /// ms since the last real move sample beyond which a release is read as "the pointer
    /// had already stopped," regardless of how fast it was moving before that -- a hard
    /// recency cutoff (velocity zeroed outright past this), not a continued exponential
    /// decay: a fast enough swipe followed by holding still for a couple of real seconds
    /// would otherwise still clear `velocity_threshold` even decayed by `decay` the whole
    /// time (`0.998.powf(2000)` is still only ~98% gone). Real touch-scroll implementations
    /// use a similarly short recency window for exactly this reason, not a slow decay --
    /// the same idea `ScrollInertia::WINDOW_MS` already uses for wheel-momentum reset.
    pub stillness_cutoff_ms: f32,
}
impl Default for ScrollMomentum {
    fn default() -> Self {
        Self {
            velocity_threshold: 0.15,
            launch: 2.0,
            decay: 0.998,
            stop_epsilon: 0.02,
            stillness_cutoff_ms: 150.0,
        }
    }
}
/// A view actively coasting on release velocity, decaying toward zero -- present only
/// while a coast is in flight (inserted by `interaction/mod.rs` on a fast-enough drag
/// release, removed here once it decays past [`ScrollMomentum::stop_epsilon`] or a fresh
/// drag interrupts it). `velocity` is px/ms, the same units/sign convention `ViewAdjustment`
/// itself already uses for a drag's own per-frame delta.
#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct Coasting {
    pub(crate) velocity: Position<Logical>,
    pub(crate) last_tick: Moment,
}
pub(crate) fn coast(
    mut coasting: Query<(Entity, &mut Coasting)>,
    momentum: Res<ScrollMomentum>,
    current: Res<CurrentInteraction>,
    mut tree: Tree,
) {
    for (entity, mut c) in coasting.iter_mut() {
        // One pointer, one momentum: a press anywhere ends every coast in flight. There is
        // only ever one gesture at a time, so there is only ever one coast worth keeping,
        // and putting a finger down is how a user says "stop."
        //
        // No relation is tested between the press and this view, because no useful one
        // exists to test. A coast attaches wherever the drag's own pan landed, which is
        // the first `View` walking up from the grab -- and since `Grid` requires `View`,
        // that is routinely a card holding an internal layout, reaching the real scroller
        // only through `OverscrollPropagation`. Two cards in the same list are siblings:
        // no ancestor walk from a press on one will ever reach a coast parked on the
        // other, so an ancestry test simply fails to stop it, and the coast keeps writing
        // its own decaying `ViewAdjustment` into the same scroller the new drag is pushing
        // the other way. Loosening the relation instead (any shared ancestor) degenerates
        // the opposite direction: `application/src/entry.rs` roots the whole app under one
        // `router`, so every pair of entities is "related" and every coast dies anyway.
        //
        // `current.pressed`, not `current.primary.is_some()`: `primary` is deliberately
        // left set *between* gestures (cleared only by the next `Start`, so a released
        // entity's own `Disengaged`/click can still be judged against it) -- its mere
        // presence is true immediately after every release too, which would kill a
        // just-started coast on its first tick, every time. `pressed` is the one field
        // that answers "is the pointer down right now."
        //
        // Read here in-place rather than trusting `interactive_elements`' own removal on
        // `Start`: that is a `Tree`-queued command, and both systems sit in
        // `MainMarkers::Process` with no ordering between them.
        if current.pressed {
            tree.entity(entity).remove::<Coasting>();
            continue;
        }
        let now = Moment::now();
        let elapsed_ms = now.duration_since(c.last_tick).as_secs_f32() * 1000.0;
        c.last_tick = now;
        tree.entity(entity)
            .insert(ViewAdjustment(c.velocity * elapsed_ms));
        let decayed = momentum.decay.powf(elapsed_ms);
        c.velocity = c.velocity * decayed;
        if c.velocity.left().hypot(c.velocity.top()) < momentum.stop_epsilon {
            tree.entity(entity).remove::<Coasting>();
        }
    }
}
#[derive(Component, Copy, Clone, Debug)]
#[require(ViewAdjustment, OverscrollPropagation, ScrollInertia, ScrollProgress)]
/// A scrollable window onto content larger than itself: how far it is scrolled, and how
/// far it may be.
///
/// Required by [`Grid`](crate::Grid), and universal for that reason. Two things depend on
/// every parent having one, whether or not it can actually scroll: a child's `Location`
/// resolve reads its parent's `offset` and folds it into every resolved coordinate, and
/// `ovrscrl` steps parent-by-parent through `View`s when passing unconsumed scroll
/// outward. A gap in either chain is not a skipped step -- it is a missing component the
/// code unwraps.
///
/// `extent` is grown from the children by `extent_check` each frame; `offset` is clamped
/// to it. Most views hold an offset of zero for their whole life.
///
/// Carrying a `View` therefore says nothing about being scrollable, and `With<View>` is
/// not a test for it.
pub struct View {
    pub(crate) offset: Position<Logical>,
    pub(crate) extent: Section<Logical>,
}
impl View {
    /// An unscrolled view. Its extent is computed from its children.
    pub fn new() -> View {
        View {
            offset: Default::default(),
            extent: Default::default(),
        }
    }
    /// Current pan, in px -- raw state, `pub(crate)`-write only (`extent_check`'s clamp
    /// is the one place that ever moves it); read-only from outside the crate. Most
    /// external callers want [`ScrollProgress`] instead (normalized, no `Section` needed
    /// to interpret it) -- this is for the rarer case that genuinely needs pixels (exact
    /// content height, which row sits at the top edge).
    pub fn offset(&self) -> Position<Logical> {
        self.offset
    }
    /// The scrollable content's bounds, in px -- see [`View::offset`]'s own doc for why
    /// this is read-only from outside the crate.
    pub fn extent(&self) -> Section<Logical> {
        self.extent
    }
}
impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}
/// Read-only readout of `View`'s own current scroll position, as a 0..1 fraction of its
/// scrollable range per axis -- the resolved counterpart to the [`ScrollTo`] request, the
/// same "author states intent, a resolved value is what everything else reads" split
/// `Visibility`/`ResolvedVisibility` already uses. Kept live by `extent_check` (a real
/// `Insert`, not a `Query`-mutation, so `tree.react::<ScrollProgress, _>(..)` sees every
/// update, not just the first) -- a scrollbar just queries this directly instead of
/// re-deriving it from `View`/`Section` itself. 0 on an axis with nothing to scroll.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct ScrollProgress {
    x: f32,
    y: f32,
}
impl ScrollProgress {
    /// Horizontal scroll position, 0..1 of the available range.
    pub fn x(&self) -> f32 {
        self.x
    }
    /// Vertical scroll position, 0..1 of the available range.
    pub fn y(&self) -> f32 {
        self.y
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
    scroll_requests: Query<(Entity, &ScrollTo), Changed<ScrollTo>>,
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
    for (entity, request) in scroll_requests.iter() {
        tracing::trace!(entity = ?entity, request = ?request, "grid::view: extent_check_v2 saw changed ScrollTo");
        to_check.insert(entity);
    }
    for (_entity, context) in contexts.iter() {
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
        // resolved the same way a drag/wheel pan is: recompute the real pixel target from
        // the *current* (already-regrown) extent, so a stale percent from before the last
        // content change can't be applied -- then consumed immediately, `ScrollTo` never
        // lingers to fight the next drag/wheel write over which one owns `offset`.
        if let Ok((_, request)) = scroll_requests.get(*entity) {
            let section = *sections.get(*entity).unwrap().1;
            let max_x = (view.extent.width() - section.right()).max(0.0);
            let max_y = (view.extent.height() - section.bottom()).max(0.0);
            if let Some(x) = request.x {
                view.offset.set_left(x * max_x);
            }
            if let Some(y) = request.y {
                view.offset.set_top(y * max_y);
            }
            tracing::trace!(entity = ?entity, request = ?request, after = ?view.offset, "grid::view: applied ScrollTo to offset");
            to_trigger.insert(*entity);
            tree.entity(*entity).remove::<ScrollTo>();
            // A `ScrollTo` states outright where this view belongs, so any momentum still
            // running on it is stale by definition -- one authority over `offset` at a
            // time, the same reason the request is consumed immediately just above. The
            // caller is typically nowhere near the view in the tree (a scrollbar has to
            // sit *outside* the view it drives -- see `application/src/toc.rs` -- so it is
            // neither the grabbed view nor inside it), which puts it out of reach of every
            // stem-walk that otherwise cancels a coast. Left running, the coast keeps
            // writing its own decaying `ViewAdjustment` into the same `offset` this
            // request just set, once per frame, and the two visibly fight.
            tree.entity(*entity).remove::<Coasting>();
        }
    }
    let to_check_final = to_check.clone();
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
    // a real `Insert`, not a `Query`-mutation -- `tree.react::<ScrollProgress, _>(..)`
    // needs to see every one of these, not just the first (see `ScrollProgress`'s own doc).
    for entity in to_check_final {
        let section = *sections.get(entity).unwrap().1;
        let view = *views.get(entity).unwrap();
        let max_x = (view.extent.width() - section.right()).max(0.0);
        let max_y = (view.extent.height() - section.bottom()).max(0.0);
        let progress = ScrollProgress {
            x: if max_x > 0.0 {
                (view.offset.left() / max_x).clamp(0.0, 1.0)
            } else {
                0.0
            },
            y: if max_y > 0.0 {
                (view.offset.top() / max_y).clamp(0.0, 1.0)
            } else {
                0.0
            },
        };
        tree.entity(entity).insert(progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimeDelta;
    use crate::{EcsExtension, Elevation, Foliage, Grid, GridExt, Leaf, Location, Sprout};
    use bevy_ecs::prelude::{ResMut, Resource};

    /// A parent sized `100px` tall with one child explicitly `300px` tall -- real
    /// overflow (not row/col-driven, just a child `Location` bigger than its parent's own
    /// box), so `View.extent` actually grows past the parent's own section instead of
    /// staying stuck equal to it (see `extent_check`'s "regrow from children" pass).
    /// `extent_check` sits in `Foliage`'s own `diff` schedule (`DiffMarkers::Prepare`),
    /// run once per real frame by `photosynthesize`'s event loop -- not by `world.flush()`
    /// (that only applies queued commands, which is all `Location`'s own `on_insert`-hook
    /// resolution needs). A headless test has no such loop, so it has to run that one
    /// schedule pass itself, same as `time.rs`'s own tests do for `main` via
    /// `foliage.main.run(&mut foliage.world)`.
    fn spawn_overflowing(foliage: &mut Foliage) -> Entity {
        let parent = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(100.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
        );
        foliage.world.branch(
            parent,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(300.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        foliage.world.flush();
        foliage.diff.run(&mut foliage.world);
        parent
    }

    #[test]
    fn a_freshly_spawned_overflowing_view_has_zero_scroll_progress() {
        let mut foliage = Foliage::new();
        let parent = spawn_overflowing(&mut foliage);

        let progress = foliage.world.get::<ScrollProgress>(parent).unwrap();
        assert_eq!(progress.x, 0.0);
        assert_eq!(progress.y, 0.0);
    }

    #[test]
    fn scroll_to_moves_progress_to_the_requested_percent() {
        let mut foliage = Foliage::new();
        let parent = spawn_overflowing(&mut foliage);

        foliage.write_to(parent, ScrollTo::y(0.5));
        foliage.world.flush();
        foliage.diff.run(&mut foliage.world);

        let progress = foliage.world.get::<ScrollProgress>(parent).unwrap();
        assert!(
            (progress.y - 0.5).abs() < 0.001,
            "expected ~0.5, got {}",
            progress.y
        );
        assert_eq!(progress.x, 0.0, "only the y request was set");
    }

    #[test]
    fn scroll_to_is_consumed_after_being_applied() {
        let mut foliage = Foliage::new();
        let parent = spawn_overflowing(&mut foliage);

        foliage.write_to(parent, ScrollTo::y(0.5));
        foliage.world.flush();
        foliage.diff.run(&mut foliage.world);

        assert!(
            foliage.world.get::<ScrollTo>(parent).is_none(),
            "a one-shot request should not linger on the entity once resolved"
        );
    }

    #[test]
    fn scroll_to_constructors_clamp_out_of_range_percent() {
        let mut foliage = Foliage::new();
        let parent = spawn_overflowing(&mut foliage);

        foliage.write_to(parent, ScrollTo::y(5.0));
        foliage.world.flush();
        foliage.diff.run(&mut foliage.world);

        let progress = foliage.world.get::<ScrollProgress>(parent).unwrap();
        assert!(
            (progress.y - 1.0).abs() < 0.001,
            "expected clamped to 1.0, got {}",
            progress.y
        );
    }

    #[test]
    fn scroll_to_on_an_axis_with_nothing_to_scroll_stays_zero() {
        let mut foliage = Foliage::new();
        // no overflow at all: parent and child are the same size.
        let parent = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(100.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
        );
        foliage.world.branch(
            parent,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(100.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        foliage.world.flush();
        foliage.diff.run(&mut foliage.world);

        foliage.write_to(parent, ScrollTo::y(1.0));
        foliage.world.flush();
        foliage.diff.run(&mut foliage.world);

        let progress = foliage.world.get::<ScrollProgress>(parent).unwrap();
        assert_eq!(progress.y, 0.0);
    }

    #[test]
    fn scroll_progress_is_a_real_insert_every_write_fires_a_reaction() {
        #[derive(Resource, Default)]
        struct Seen(Vec<f32>);
        fn mark(
            trigger: crate::Trigger<bevy_ecs::lifecycle::Insert, ScrollProgress>,
            progress: Query<&ScrollProgress>,
            mut r: ResMut<Seen>,
        ) {
            r.0.push(progress.get(trigger.entity).unwrap().y);
        }

        let mut foliage = Foliage::new();
        foliage.world.insert_resource(Seen::default());
        let parent = spawn_overflowing(&mut foliage);
        foliage.world.add_observer(mark);

        foliage.write_to(parent, ScrollTo::y(0.25));
        foliage.world.flush();
        foliage.diff.run(&mut foliage.world);
        foliage.write_to(parent, ScrollTo::y(0.75));
        foliage.world.flush();
        foliage.diff.run(&mut foliage.world);

        let seen = &foliage.world.resource::<Seen>().0;
        assert!(
            seen.len() >= 2,
            "expected a reaction fire per ScrollTo write, got {seen:?}"
        );
        assert!((seen[seen.len() - 2] - 0.25).abs() < 0.001);
        assert!((seen[seen.len() - 1] - 0.75).abs() < 0.001);
    }

    #[test]
    fn the_first_ever_tick_returns_the_base_multiplier() {
        let (multiplier, updated) = ScrollInertia::default().tick();
        assert_eq!(multiplier, ScrollInertia::BASE);
        assert!(updated.last_tick.is_some());
    }

    #[test]
    fn a_tick_within_the_window_grows_the_multiplier() {
        let inertia = ScrollInertia {
            value: ScrollInertia::BASE,
            last_tick: Some(Moment::now() - TimeDelta::from_millis(50)),
        };
        let (multiplier, _) = inertia.tick();
        assert_eq!(multiplier, ScrollInertia::BASE + ScrollInertia::GROWTH);
    }

    #[test]
    fn a_tick_after_the_window_resets_to_base_regardless_of_prior_value() {
        let inertia = ScrollInertia {
            value: 2.5,
            last_tick: Some(Moment::now() - TimeDelta::from_millis(200)),
        };
        let (multiplier, _) = inertia.tick();
        assert_eq!(
            multiplier,
            ScrollInertia::BASE,
            "a pause longer than WINDOW_MS should reset the ramp, not continue from where it left off"
        );
    }

    #[test]
    fn growth_is_capped_at_max() {
        let inertia = ScrollInertia {
            value: ScrollInertia::MAX - 0.05,
            last_tick: Some(Moment::now() - TimeDelta::from_millis(50)),
        };
        let (multiplier, _) = inertia.tick();
        assert_eq!(
            multiplier,
            ScrollInertia::MAX,
            "growth should clamp at MAX, not overshoot it"
        );
    }

    #[test]
    fn repeated_fast_ticks_accumulate_growth_across_calls() {
        let mut inertia = ScrollInertia::default();
        let (first, updated) = inertia.tick();
        inertia = updated;
        // simulate the next tick arriving well within the window, without a real sleep
        inertia.last_tick = Some(Moment::now() - TimeDelta::from_millis(50));
        let (second, _) = inertia.tick();
        assert!(
            second > first,
            "a second fast tick should ramp higher than the first"
        );
    }

    fn spawn_view(foliage: &mut Foliage) -> Entity {
        let e = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(200.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
        );
        foliage.world.flush();
        e
    }

    #[test]
    fn coast_writes_a_view_adjustment_from_its_decaying_velocity() {
        let mut foliage = Foliage::new();
        let parent = spawn_view(&mut foliage);
        // back-dated `last_tick`, same trick `repeated_fast_ticks_accumulate_growth_across_calls`
        // uses -- gives `coast` a real, nonzero elapsed time to decay across without an
        // actual `sleep`.
        foliage.world.entity_mut(parent).insert(Coasting {
            velocity: Position::logical((0.0, 1.0)),
            last_tick: Moment::now() - TimeDelta::from_millis(10),
        });
        foliage.world.flush();

        foliage.main.run(&mut foliage.world);

        let adjustment = foliage.world.get::<ViewAdjustment>(parent).unwrap().0;
        assert!(
            adjustment.top() > 0.0,
            "a downward-coasting view should have produced a downward pan, got {adjustment:?}"
        );
    }

    #[test]
    fn coast_removes_itself_once_velocity_decays_past_stop_epsilon() {
        let mut foliage = Foliage::new();
        let parent = spawn_view(&mut foliage);
        // an enormous elapsed time collapses `decay.powf(elapsed_ms)` to effectively zero
        // in a single tick, so this doesn't need many real frames (or a real sleep) to
        // observe the coast actually ending.
        foliage.world.entity_mut(parent).insert(Coasting {
            velocity: Position::logical((2.0, 0.0)),
            last_tick: Moment::now() - TimeDelta::from_secs(5),
        });
        foliage.world.flush();

        foliage.main.run(&mut foliage.world);

        assert!(
            foliage.world.get::<Coasting>(parent).is_none(),
            "velocity should have decayed past stop_epsilon and removed Coasting"
        );
    }

    #[test]
    fn coast_stops_immediately_once_the_same_view_is_grabbed_again() {
        // A fresh drag on a still-coasting view -- or on something inside it -- must not
        // leave the coast writing its own stale `ViewAdjustment` over the new drag's.
        let mut foliage = Foliage::new();
        let parent = spawn_view(&mut foliage);
        foliage.world.entity_mut(parent).insert(Coasting {
            velocity: Position::logical((2.0, 0.0)),
            last_tick: Moment::now(),
        });
        foliage.world.flush();
        {
            let mut current = foliage.world.resource_mut::<CurrentInteraction>();
            current.primary = Some(parent);
            current.pressed = true;
        }

        foliage.main.run(&mut foliage.world);

        assert!(
            foliage.world.get::<Coasting>(parent).is_none(),
            "grabbing the exact coasting view again should halt its own coast"
        );
    }

    #[test]
    fn coast_stops_when_something_inside_the_coasting_view_is_grabbed() {
        let mut foliage = Foliage::new();
        let list = spawn_view(&mut foliage);
        let row = foliage.world.branch(
            list,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(50.px().as_width()),
                    0.px().as_top().with(20.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        foliage.world.entity_mut(list).insert(Coasting {
            velocity: Position::logical((2.0, 0.0)),
            last_tick: Moment::now(),
        });
        foliage.world.flush();
        {
            let mut current = foliage.world.resource_mut::<CurrentInteraction>();
            current.primary = Some(row);
            current.pressed = true;
        }

        foliage.main.run(&mut foliage.world);

        assert!(
            foliage.world.get::<Coasting>(list).is_none(),
            "grabbing a row inside the list should walk up and halt the list's own coast, \
             same as `interactive_elements`' own walk-up finds the list to pan"
        );
    }

    #[test]
    fn clicking_a_nested_list_stops_the_outer_pages_own_coast() {
        // the outer page is coasting; the list is a real, nested `View` of its own
        // *inside* the page, and gets grabbed directly (not a plain child, an actual
        // scrollable region within a scrollable region). Walking up from the list still
        // reaches the page, so the page's own coast stops -- grabbing content contained
        // within a coasting view's own bounds should always stop it, regardless of
        // whether that content is a plain leaf or itself a `View`.
        let mut foliage = Foliage::new();
        let page = spawn_view(&mut foliage);
        let list = foliage.world.branch(
            page,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(150.px().as_width()),
                    0.px().as_top().with(150.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
        );
        foliage.world.flush();
        foliage.world.entity_mut(page).insert(Coasting {
            velocity: Position::logical((2.0, 0.0)),
            last_tick: Moment::now(),
        });
        foliage.world.flush();
        {
            let mut current = foliage.world.resource_mut::<CurrentInteraction>();
            current.primary = Some(list);
            current.pressed = true;
        }

        foliage.main.run(&mut foliage.world);

        assert!(
            foliage.world.get::<Coasting>(page).is_none(),
            "grabbing the nested list (inside the coasting page) should stop the page's own coast"
        );
    }

    #[test]
    fn a_scroll_to_request_cancels_the_views_own_coast() {
        // a scrollbar's own door: it drives the view by `ScrollTo` while deliberately
        // living *outside* that view in the tree (`application/src/toc.rs`'s own
        // `build_scrollbar` -- it can't be inside the thing it scrolls), so it is neither
        // the coasting view nor anything within it, and no stem-walk from the grab ever
        // reaches the coast. Dragging the knob against a live coast otherwise leaves both
        // writing to the same `offset` every frame.
        let mut foliage = Foliage::new();
        let view = spawn_view(&mut foliage);
        foliage.world.entity_mut(view).insert(Coasting {
            velocity: Position::logical((0.0, 1.0)),
            last_tick: Moment::now(),
        });
        foliage.world.flush();

        foliage.world.entity_mut(view).insert(ScrollTo::y(0.5));
        foliage.diff.run(&mut foliage.world);
        foliage.world.flush();

        assert!(
            foliage.world.get::<Coasting>(view).is_none(),
            "a ScrollTo states where the view belongs -- any momentum still running on it \
             is stale and must not keep writing over it"
        );
    }

    #[test]
    fn a_press_anywhere_stops_an_in_flight_coast() {
        // One pointer, one momentum. `unrelated` is deliberately in a separate subtree
        // from the coasting view -- not its parent, not its child, no useful relation at
        // all -- because the press still has to stop it. Anything narrower fails the case
        // that actually matters: two sibling cards in one list, where a coast parked on
        // the first (that is where the drag's own pan landed) is reachable from a press on
        // the second by no walk in either direction.
        let mut foliage = Foliage::new();
        let list = spawn_view(&mut foliage);
        let other_root = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(400.px().as_width()),
                    0.px().as_top().with(400.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
        );
        let unrelated = foliage.world.branch(
            other_root,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(200.px().as_width()),
                    0.px().as_top().with(200.px().as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Grid::new(1.col().gap(0), 1.row().gap(0))),
        );
        foliage.world.flush();
        foliage.world.entity_mut(list).insert(Coasting {
            velocity: Position::logical((2.0, 0.0)),
            last_tick: Moment::now(),
        });
        foliage.world.flush();
        {
            let mut current = foliage.world.resource_mut::<CurrentInteraction>();
            current.primary = Some(unrelated);
            current.pressed = true;
        }

        foliage.main.run(&mut foliage.world);

        assert!(
            foliage.world.get::<Coasting>(list).is_none(),
            "a press with the pointer down should end the coast"
        );
    }

    #[test]
    fn a_coast_survives_the_release_that_created_it() {
        // The counterpart to the rule above, and the reason it keys off `pressed` rather
        // than `primary`: `primary` stays set after release (cleared only by the next
        // `Start`, so the released entity's own click can still be judged), so a coast
        // would be killed on its very first tick by the gesture that just created it.
        let mut foliage = Foliage::new();
        let list = spawn_view(&mut foliage);
        foliage.world.entity_mut(list).insert(Coasting {
            velocity: Position::logical((2.0, 0.0)),
            last_tick: Moment::now(),
        });
        foliage.world.flush();
        {
            let mut current = foliage.world.resource_mut::<CurrentInteraction>();
            current.primary = Some(list);
            current.pressed = false;
        }

        foliage.main.run(&mut foliage.world);

        assert!(
            foliage.world.get::<Coasting>(list).is_some(),
            "with the pointer up, a fresh coast must keep running"
        );
    }

    #[test]
    fn a_slow_release_stays_under_the_default_velocity_threshold() {
        // sanity on the shipped defaults: a genuinely slow, deliberate drag (well under a
        // brisk flick) shouldn't clear `ScrollMomentum::default().velocity_threshold`.
        let slow: Position<Logical> = Position::logical((0.02, 0.0));
        let speed = slow.left().hypot(slow.top());
        assert!(speed < ScrollMomentum::default().velocity_threshold);
    }
}
