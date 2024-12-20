use crate::anim::{Animate, Animation, AnimationRunner, AnimationTime, Sequence};
use crate::grid::responsive::anim::{
    ResponsiveAnimationHook, ResponsiveLocationAnimPackage, ResponsivePointsAnimPackage,
    ResponsivePointsAnimationHook,
};
use crate::grid::responsive::{ResponsiveLocation, ResponsivePointBundle};
use crate::leaf::{EvaluateVisibility, Remove, Visibility};
use crate::time::OnEnd;
use crate::twig::{Branch, Twig};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::{Commands, IntoObserverSystem};
use bevy_ecs::world::World;
use std::any::TypeId;

pub type Tree<'w, 's> = Commands<'w, 's>;
pub trait EcsExtension {
    fn start_sequence<SFN: for<'w, 's> FnOnce(&mut SequenceHandle<'_, 'w, 's>)>(
        &mut self,
        sfn: SFN,
    ) -> Entity;
    fn branch<B: Branch>(&mut self, twig: Twig<B>) -> B::Handle;
    fn visibility(&mut self, leaf: Entity, visibility: bool);
    fn remove(&mut self, entity: Entity);
}
impl<'w, 's> EcsExtension for Tree<'w, 's> {
    fn start_sequence<SFN: FnOnce(&mut SequenceHandle<'_, 'w, 's>)>(&mut self, sfn: SFN) -> Entity {
        let sequence_entity = self.spawn_empty().id();
        let mut sequence = Sequence::default();
        let mut handle = SequenceHandle {
            tree: self,
            sequence,
            sequence_entity,
        };
        sfn(&mut handle);
        sequence = handle.sequence;
        drop(handle);
        self.entity(sequence_entity).insert(sequence);
        sequence_entity
    }
    fn branch<B: Branch>(&mut self, twig: Twig<B>) -> B::Handle {
        B::grow(twig, self)
    }
    fn visibility(&mut self, leaf: Entity, visibility: bool) {
        self.entity(leaf)
            .insert(Visibility::new(visibility))
            .insert(EvaluateVisibility::recursive());
    }
    fn remove(&mut self, leaf: Entity) {
        self.trigger_targets(Remove {}, leaf);
    }
}
impl EcsExtension for World {
    fn start_sequence<SFN: for<'w, 's> FnOnce(&mut SequenceHandle<'_, 'w, 's>)>(
        &mut self,
        sfn: SFN,
    ) -> Entity {
        let mut cmds = self.commands();
        let e = cmds.start_sequence(sfn);
        e
    }
    fn branch<B: Branch>(&mut self, twig: Twig<B>) -> B::Handle {
        let mut cmds = self.commands();
        let h = cmds.branch(twig);
        h
    }
    fn visibility(&mut self, leaf: Entity, visibility: bool) {
        self.entity_mut(leaf)
            .insert(Visibility::new(visibility))
            .insert(EvaluateVisibility::recursive());
    }

    fn remove(&mut self, leaf: Entity) {
        self.trigger_targets(Remove {}, leaf);
    }
}
pub struct SequenceHandle<'a, 'w, 's> {
    tree: &'a mut Tree<'w, 's>,
    sequence: Sequence,
    sequence_entity: Entity,
}
impl<'a, 'w, 's> SequenceHandle<'a, 'w, 's> {
    pub fn animate<A: Animate>(&mut self, animation: Animation<A>) -> Entity {
        debug_assert_ne!(TypeId::of::<ResponsiveLocation>(), TypeId::of::<A>());
        debug_assert_ne!(TypeId::of::<ResponsivePointBundle>(), TypeId::of::<A>());
        self.sequence.animations_to_finish += 1;
        let anim = AnimationRunner::new(
            animation.anim_target.unwrap(),
            animation.a,
            animation.ease,
            self.sequence_entity,
            AnimationTime::from(animation.sequence_time_range),
        );
        self.tree.spawn(anim).id()
    }
    pub fn animate_location(&mut self, animation: Animation<ResponsiveLocation>) -> Entity {
        let mut converted = Animation::new(ResponsiveAnimationHook::default());
        converted.anim_target = animation.anim_target;
        converted.sequence_time_range = animation.sequence_time_range;
        converted.ease = animation.ease;
        let anim = self.animate(converted);
        self.tree
            .entity(anim)
            .insert(ResponsiveLocationAnimPackage {
                base: animation.a.base,
                exceptions: animation.a.exceptions,
            });
        anim
    }
    pub fn animate_points(&mut self, animation: Animation<ResponsivePointBundle>) -> Entity {
        let mut converted = Animation::new(ResponsivePointsAnimationHook::default());
        converted.anim_target = animation.anim_target;
        converted.sequence_time_range = animation.sequence_time_range;
        converted.ease = animation.ease;
        let anim = self.animate(converted);
        self.tree.entity(anim).insert(ResponsivePointsAnimPackage {
            base_points: animation.a.base_points,
            exceptions: animation.a.exceptions,
        });
        anim
    }
    pub fn on_end<O: IntoObserverSystem<OnEnd, B, M>, B: Bundle, M>(&mut self, o: O) {
        self.tree.entity(self.sequence_entity).observe(o);
    }
}
