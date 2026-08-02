use crate::Component;
use crate::anim::Animate;
use crate::anim::Repeat;
use crate::anim::ease::Easement;
use crate::anim::interpolation::Interpolations;
use crate::anim::sequence::{AnimationTime, SequenceMarker};
use crate::time::TimeDelta;
use bevy_ecs::entity::Entity;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::world::DeferredWorld;

#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub(crate) struct AnimationRunner<A: Animate> {
    pub(crate) started: bool,
    pub(crate) finish: Option<A>,
    pub(crate) interpolations: Interpolations,
    pub(crate) easement: Easement,
    pub(crate) sequence_entity: Entity,
    pub(crate) animation_time: AnimationTime,
    pub(crate) animation_target: Entity,
    /// Set on this animation's first tick in `animate::<A>` (reusing `Time`'s own elapsed
    /// clock, not a new counter) -- lets a same-tick supersede check tell which of two
    /// animations targeting the same entity started more recently. `Entity` ordering looks
    /// like it should answer that but doesn't: its `Ord` compares an internal bit-packed
    /// representation that doesn't correlate with spawn order at all (confirmed directly --
    /// a later-spawned entity compared as *less than* an earlier one).
    pub(crate) created_at: Option<TimeDelta>,
    /// Passes still to run after the current one. See [`Repeat`].
    pub(crate) repeat: Repeat,
    /// Whether each replay runs back the way it came -- see
    /// [`Animation::backtrack`](crate::Animation::backtrack).
    pub(crate) backtrack: bool,
    /// Whether this animation has already decremented its sequence. A looping animation
    /// settles its sequence on its *first* completed pass and then keeps running, so every
    /// later teardown must skip the decrement -- `despawn_and_update_sequence` unwraps the
    /// sequence entity, which is despawned once its count reaches zero.
    pub(crate) sequence_settled: bool,
}

impl<A: Animate> AnimationRunner<A> {
    pub(crate) fn new<EASE: Into<Easement>>(
        target: Entity,
        finish: A,
        ease: EASE,
        se: Entity,
        animation_time: AnimationTime,
        repeat: Repeat,
        backtrack: bool,
    ) -> Self {
        Self {
            started: false,
            finish: Some(finish),
            interpolations: Interpolations::default(),
            easement: ease.into(),
            sequence_entity: se,
            animation_time,
            animation_target: target,
            created_at: None,
            repeat,
            backtrack,
            sequence_settled: false,
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let value = world.get::<Self>(this).unwrap();
        let sequence_entity = value.sequence_entity;
        world
            .get_mut::<SequenceMarker>(sequence_entity)
            .unwrap_or_else(|| {
                panic!(
                    "this `Animation`'s sequence {sequence_entity:?} has no `SequenceMarker` -- \
                     every animation belongs to a sequence, so it needs \
                     `.during(tree.sequence())`. Without it the sequence entity is left as a \
                     placeholder and there is nothing to count this animation against (see \
                     `Animation::during`)"
                )
            })
            .animations_to_finish += 1;
    }
}
