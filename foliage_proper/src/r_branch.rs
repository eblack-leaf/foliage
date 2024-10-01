use crate::anim::{Animate, Animation, AnimationRunner, AnimationTime, Sequence};
use crate::time::OnEnd;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::Commands;

pub trait CommandsExtension {
    fn spawn_sequence<'a, 'w, 's, SFN: FnOnce(&mut SequenceHandle<'a, 'w, 's>)>(
        &'a mut self,
        sfn: SFN,
    ) -> Entity;
}
impl<'w, 's> CommandsExtension for Commands<'w, 's> {
    fn spawn_sequence<'a, 'w, 's, SFN: FnOnce(&mut SequenceHandle<'a, 'w, 's>)>(
        &'a mut self,
        sfn: SFN,
    ) -> Entity {
        let sequence_entity = self.spawn_empty().id();
        let sequence = Sequence::default();
        let mut handle = SequenceHandle {
            cmds: self,
            sequence,
            sequence_entity,
        };
        sfn(&mut handle);
        self.entity(sequence_entity).insert(handle.sequence);
        sequence_entity
    }
}
pub struct SequenceHandle<'a, 'w, 's> {
    cmds: &'a mut Commands<'w, 's>,
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
        self.cmds.spawn(anim);
    }
    pub fn on_end(&mut self, on_end: OnEnd) {
        self.sequence.on_end.replace(on_end);
    }
}
