use crate::anim::ease::Easement;
use crate::anim::interpolation::Interpolations;
use crate::anim::sequence::{AnimationTime, Sequence};
use crate::anim::Animate;
use crate::Component;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
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
}

impl<A: Animate> AnimationRunner<A> {
    pub(crate) fn new<EASE: Into<Easement>>(
        target: Entity,
        finish: A,
        ease: EASE,
        se: Entity,
        animation_time: AnimationTime,
    ) -> Self {
        Self {
            started: false,
            finish: Some(finish),
            interpolations: Interpolations::default(),
            easement: ease.into(),
            sequence_entity: se,
            animation_time,
            animation_target: target,
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let value = world.get::<Self>(this).unwrap();
        world
            .get_mut::<Sequence>(value.sequence_entity)
            .unwrap()
            .animations_to_finish += 1;
    }
}
