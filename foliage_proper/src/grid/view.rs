use crate::ash::clip::{ClipToViewport, InheritedClip, ResolvedClip, clip_of};
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::location::Resolution;
use crate::interaction::CurrentInteraction;
use crate::{
    AnchorDeps, Children, Component, LayoutSection, Location, Logical, Moment, Parent, Points,
    Position, Resolve, Section, Tree,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, DetectChanges, Query, Ref, Res, ResMut, Resource};
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
/// vs. a wide gallery, say). Distinct from `ScrollInertia` (engine-internal), which scales each *new*
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
            tree.strip::<Coasting>(entity);
            continue;
        }
        let now = Moment::now();
        let elapsed_ms = now.duration_since(c.last_tick).as_secs_f32() * 1000.0;
        c.last_tick = now;
        tree.write_to(entity, ViewAdjustment(c.velocity * elapsed_ms));
        let decayed = momentum.decay.powf(elapsed_ms);
        c.velocity = c.velocity * decayed;
        if c.velocity.left().hypot(c.velocity.top()) < momentum.stop_epsilon {
            tree.strip::<Coasting>(entity);
        }
    }
}
#[derive(Component, Copy, Clone, Debug)]
#[require(ViewAdjustment, OverscrollPropagation, ScrollInertia, ScrollProgress)]
/// A scrollable window onto content larger than itself: how far it is scrolled, and how
/// far it may be.
///
/// Required by [`Grid`](crate::Grid), and universal for that reason. Two things depend on
/// every parent having one, whether or not it can actually scroll: `accumulated_offset` is
/// read off the parent to place each child on screen, and `ovrscrl` steps parent-by-parent
/// through `View`s when passing unconsumed scroll outward. A gap in either chain is not a
/// skipped step -- it is a missing component the code unwraps.
///
/// `extent` is grown from the children by `extent_check` each frame; `offset` is clamped
/// to it. Most views hold an offset of zero for their whole life.
///
/// Carrying a `View` therefore says nothing about being scrollable, and `With<View>` is
/// not a test for it.
///
/// Scrolling never re-resolves anything. An offset is a translation of a whole subtree,
/// which is why it is kept out of the layout entirely: `Location` resolves into
/// [`LayoutSection`], and `propagate_offsets` subtracts
/// `accumulated_offset` from it to produce the on-screen [`Section`]. Moving a view rewrites
/// its descendants' `Section`s and nothing else -- no `Location` is resolved, no glyph is
/// laid out again.
///
/// The cost that remains is one write per descendant, plus the differential and the instance
/// buffer behind it. That floor is deliberate: pushing the offset past the differential (a
/// per-clip-group translation applied while drawing) would leave the renderer without
/// clipping known ahead of time, which is what drives visibility -- and a clip group holds
/// many instances at many offsets, so there is no single translation to hand it anyway.
pub struct View {
    pub(crate) offset: Position<Logical>,
    pub(crate) extent: Section<Logical>,
    /// This view's own [`offset`](Self::offset) plus every ancestor view's -- the total a
    /// child of this view subtracts from its [`LayoutSection`] to
    /// land on screen.
    ///
    /// Not the same number as `offset` whenever views nest: a card scrolled 40px inside a
    /// page scrolled 300px has an `offset` of 40 and hands 340 down to its children. Kept
    /// here rather than walked per entity so deriving a child's `Section` is one lookup.
    pub(crate) accumulated_offset: Position<Logical>,
}
impl View {
    /// An unscrolled view. Its extent is computed from its children.
    pub fn new() -> View {
        View {
            offset: Default::default(),
            extent: Default::default(),
            accumulated_offset: Default::default(),
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
/// `Insert` on every change, not a `Query`-mutation, so nothing reacting to it misses an
/// update) -- read it with [`Canopy::sample`](crate::Canopy::sample)/
/// [`Canopy::scroll_offset`](crate::Canopy::scroll_offset)-style sampling rather than
/// re-deriving it from `View`/`Section` yourself. 0 on an axis with nothing to scroll.
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
    contexts: &Query<(Entity, Ref<Parent>)>,
    sections: &Query<(Entity, Ref<Section<Logical>>)>,
    to_trigger: &mut HashSet<Entity>,
) -> (Option<Entity>, Position<Logical>) {
    let propagation = propagations.get(entity).unwrap();
    let old_offset = views.get(entity).unwrap().offset;
    let mut view = views.get_mut(entity).unwrap();
    // Whole units, at the one place every write to `offset` passes through -- a drag delta, a
    // wheel tick scaled by its inertia, a `ScrollTo`, and an overscroll remainder handed down
    // from a parent all arrive here before anything reads them.
    //
    // A fractional offset is what makes a scrolling box flicker. `Section::rounded` snaps the
    // four edges before rasterizing, deliberately, so that two boxes sharing a coordinate cannot
    // land a pixel apart; the cost is that a box's snapped size is `round(left + width) -
    // round(left)`, which depends on `left`'s fractional part. Scrolling by a fraction walks that
    // part, and anything whose width or height is not whole gains and loses a pixel as it moves.
    // What matters is that the fraction stops *changing*, not that it is zero -- an offset in
    // whole units leaves every element the same fraction it was laid out with.
    //
    // Snapped here rather than on the way to the screen so there is one authority over `offset`:
    // the clamp below, momentum, and `ScrollProgress` all read the number the reader is looking
    // at. The cost is that a scroll cannot advance by less than a unit, which is not a distance
    // anyone can ask for.
    view.offset = (view.offset + ovr).rounded();
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
/// Recomputes each view's scrollable extent from its children and clamps `offset` to it.
///
/// TODO: this walks the whole tree twice every frame regardless of what changed. Both
/// `contexts.iter()` and `sections.iter()` are full scans, filtered afterwards by
/// `is_changed()` -- so the cost is paid on an idle frame the same as a busy one, and it
/// scales with the size of the tree rather than with how much of it moved.
///
/// It stands out only because the work around it has gone: with layout resolution and the
/// clip cascade off the scroll path, this is the only part of the framework left above 1% in
/// a scroll profile, under the texture atlas (whose own inner cost is syscalls, not
/// framework code). At that size it is not worth changing.
///
/// The shape of the fix, when it is worth it: the same set the walk below already builds.
/// `propagate_offsets` knows exactly which entities moved, and a `Changed`-filtered query
/// (`Query<(Entity, &Parent), Changed<Section<Logical>>>`) would hand back the changed
/// entities directly instead of every entity plus a test. The reason it is written this way
/// is that the check needs the *parents* of changed entities, not the entities themselves,
/// and the parent lookup is what the scan currently provides.
pub(crate) fn extent_check(
    adjustments: Query<(Entity, &ViewAdjustment), Changed<ViewAdjustment>>,
    scroll_requests: Query<(Entity, &ScrollTo), Changed<ScrollTo>>,
    mut views: Query<&mut View>,
    propagations: Query<&OverscrollPropagation>,
    contexts: Query<(Entity, Ref<Parent>)>,
    sections: Query<(Entity, Ref<Section<Logical>>)>,
    clip_to_viewport: Query<&ClipToViewport>,
    mut scrolled: ResMut<ScrolledViews>,
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
            tree.strip::<ScrollTo>(*entity);
            // A `ScrollTo` states outright where this view belongs, so any momentum still
            // running on it is stale by definition -- one authority over `offset` at a
            // time, the same reason the request is consumed immediately just above. The
            // caller is typically nowhere near the view in the tree (a scrollbar has to
            // sit *outside* the view it drives -- see `application/src/toc.rs` -- so it is
            // neither the grabbed view nor inside it), which puts it out of reach of every
            // stem-walk that otherwise cancels a coast. Left running, the coast keeps
            // writing its own decaying `ViewAdjustment` into the same `offset` this
            // request just set, once per frame, and the two visibly fight.
            tree.strip::<Coasting>(*entity);
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
        scrolled.0.insert(*entity);
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
        tree.write_to(entity, progress);
    }
}

/// Views whose `offset` moved this frame, handed from [`extent_check`] to
/// [`propagate_offsets`].
///
/// Separate systems because the clamp that settles an offset and the pass that moves a
/// subtree to match it are separate jobs, and the second needs the first to have finished
/// for every view before it walks anything -- a nested view's own clamp changes what its
/// children end up subtracting.
#[derive(Resource, Default)]
pub(crate) struct ScrolledViews(pub(crate) HashSet<Entity>);
/// Re-derives the on-screen `Section` of everything under a view that just scrolled.
///
/// Top-down, because [`View::accumulated_offset`] is built up on the way: each level hands
/// its children its own accumulated total, and a nested view adds its own `offset` to what
/// it was handed before passing it on. The subtraction itself is the whole of the work per
/// entity -- no `Location` is resolved, no `LayoutSection` is written, and nothing here
/// re-enters the layout solver.
///
/// `Section` is *mutated*, not inserted, and that is the point: an insert is a queued
/// command, and every one of them costs an archetype touch plus whatever hooks and observers
/// hang off it. A scroll issuing one per descendant put the frame's cost in the flush rather
/// than in any computation.
///
/// What a mutation still reaches: everything written as a system over
/// `Changed<Section<Logical>>` -- the render differentials, image cropping -- because change
/// detection fires on mutation. What it does not reach is the insert-only
/// `Resolved<Section<Logical>>` event, so its two consumers that matter while scrolling are
/// handled directly. `ResolvedClip` is computed here, top-down, where a parent's clip is
/// guaranteed to be settled before its children read it; the text scissor rides
/// `Text::update_from_section`, which is a `Changed`-driven system for exactly this reason
/// and is ordered `.after` this one -- it reads the `Section` written here, so sharing
/// `DiffMarkers::Prepare` without that edge would let it run first and trail a frame.
/// The third consumer, `Panel::update_from_section`, recomputes corner radii from the box's
/// *size*, which a scroll never changes -- not firing it is the correct outcome, not a gap.
pub(crate) fn propagate_offsets(
    mut scrolled: ResMut<ScrolledViews>,
    stems: Query<&Parent>,
    branches: Query<&Children>,
    mut views: Query<&mut View>,
    layouts: Query<&LayoutSection>,
    resolutions: Query<&Resolution>,
    anchor_deps: Query<&AnchorDeps>,
    mut sections: Query<&mut Section<Logical>>,
    mut points: Query<&mut Points<Logical>>,
    mut clips: Query<&mut ResolvedClip>,
    mut inherited_clips: Query<&mut InheritedClip>,
    marked: Query<&ClipToViewport>,
    viewport: Res<ViewportHandle>,
    mut tree: Tree,
) {
    if scrolled.0.is_empty() {
        return;
    }
    let roots = scrolled.0.drain().collect::<Vec<_>>();
    let viewport = viewport.section();
    let mut touched = HashSet::new();
    let mut anchored = HashSet::new();
    for root in roots {
        let inherited = stems
            .get(root)
            .ok()
            .and_then(|s| s.id)
            .and_then(|p| views.get(p).ok().map(|v| v.accumulated_offset))
            .unwrap_or_default();
        let accumulated = {
            let mut view = views.get_mut(root).unwrap();
            view.accumulated_offset = inherited + view.offset;
            view.accumulated_offset
        };
        // The root's own box does not move when the root scrolls -- only what is inside it --
        // so its clip is already right, and what descends is the base its children clip
        // against.
        let base = clip_of(
            *sections.get(root).unwrap(),
            inherited_clips.get(root).unwrap().0,
            marked.get(root).is_ok(),
            viewport,
        )
        .1;
        let mut stack = vec![(root, accumulated, base)];
        while let Some((entity, accumulated, base)) = stack.pop() {
            let Ok(branch) = branches.get(entity) else {
                continue;
            };
            for child in branch.ids.iter().copied() {
                touched.insert(child);
                let mut moved = None;
                if let Ok(layout) = layouts.get(child) {
                    let mut section = layout.0;
                    section.position -= accumulated;
                    if let Ok(mut current) = sections.get_mut(child) {
                        *current = section;
                    }
                    moved = Some(section);
                }
                if let Ok(resolution) = resolutions.get(child) {
                    if resolution.from_points {
                        let mut translated = resolution.points;
                        for pt in translated.data.iter_mut() {
                            *pt -= accumulated;
                        }
                        if let Ok(mut current) = points.get_mut(child) {
                            *current = translated;
                        }
                    }
                }
                if let Ok(deps) = anchor_deps.get(child) {
                    anchored.extend(deps.ids.iter().copied());
                }
                // `InheritedClip` is kept in step as well as `ResolvedClip`: it is what the
                // observer path reads the next time this entity's box changes for a reason
                // other than scrolling, and a stale one there would resolve against the box
                // this walk already moved past.
                let child_base = if let Some(section) = moved {
                    let (resolved, next) =
                        clip_of(section, Some(base), marked.get(child).is_ok(), viewport);
                    if let Ok(mut current) = clips.get_mut(child) {
                        if *current != resolved {
                            *current = resolved;
                        }
                    }
                    if let Ok(mut current) = inherited_clips.get_mut(child) {
                        current.0.replace(base);
                    }
                    next
                } else {
                    base
                };
                let inherited = if let Ok(mut view) = views.get_mut(child) {
                    view.accumulated_offset = accumulated + view.offset;
                    view.accumulated_offset
                } else {
                    accumulated
                };
                stack.push((child, inherited, child_base));
            }
        }
    }
    // Anchored from *outside* the subtree that moved: the two are no longer the same
    // distance apart, which is the one case a translation cannot express -- so those really
    // do have to resolve again. Anything anchored from inside moved with the rest of the
    // subtree already.
    let outside = anchored.difference(&touched).copied().collect::<Vec<_>>();
    if !outside.is_empty() {
        tracing::trace!(entities = ?outside, "grid::view: re-resolving anchors across the scroll boundary");
        tree.send_to(Resolve::<Location>::new(), outside);
    }
}
