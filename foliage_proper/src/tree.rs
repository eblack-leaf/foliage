use crate::anim::{Animate, Animation, AnimationRunner, AnimationTime, Sequence};
use crate::leaf::{Remove, ResolveVisibility, Visibility};
use crate::time::OnEnd;
use crate::Branch;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::Commands;
use bevy_ecs::world::World;

pub type Tree<'w, 's> = Commands<'w, 's>;
pub trait EcsExtension {
    fn start_sequence<SFN: for<'w, 's> FnOnce(&mut SequenceHandle<'_, 'w, 's>)>(
        &mut self,
        sfn: SFN,
    ) -> Entity;
    fn branch<B: Branch>(&mut self, twig: B) -> B::Handle;
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
    fn branch<B: Branch>(&mut self, twig: B) -> B::Handle {
        B::grow(twig, self)
    }
    fn visibility(&mut self, leaf: Entity, visibility: bool) {
        self.entity(leaf).insert(Visibility::new(visibility));
        self.trigger_targets(ResolveVisibility(), leaf);
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
    fn branch<B: Branch>(&mut self, twig: B) -> B::Handle {
        let mut cmds = self.commands();
        let h = cmds.branch(twig);
        h
    }
    fn visibility(&mut self, leaf: Entity, visibility: bool) {
        self.entity_mut(leaf).insert(Visibility::new(visibility));
        self.trigger_targets(ResolveVisibility(), leaf);
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
    pub fn animate<A: Animate>(&mut self, animation: Animation<A>) {
        self.sequence.animations_to_finish += 1;
        let anim = AnimationRunner::new(
            animation.anim_target.unwrap(),
            animation.a,
            animation.ease,
            self.sequence_entity,
            AnimationTime::from(animation.sequence_time_range),
        );
        self.tree.spawn(anim);
    }
    pub fn on_end(&mut self, on_end: OnEnd) {
        self.sequence.on_end.replace(on_end);
    }
}
