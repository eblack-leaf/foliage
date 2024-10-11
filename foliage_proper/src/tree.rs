use crate::anim::{Animate, Animation, AnimationRunner, AnimationTime, Sequence};
use crate::color::Color;
use crate::coordinate::elevation::Elevation;
use crate::grid::location::GridLocation;
use crate::grid::resolve::ResolveGridLocation;
use crate::grid::Grid;
use crate::leaf::{
    Leaf, Remove, ResolveElevation, ResolveVisibility, Stem, UpdateStem, Visibility,
};
use crate::opacity::{Opacity, ResolveOpacity};
use crate::time::OnEnd;
use crate::twig::{Branch, Twig};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::{Commands, EntityCommands};
use bevy_ecs::world::World;
use std::any::TypeId;
use bevy_ecs::observer::TriggerTargets;

pub type Tree<'w, 's> = Commands<'w, 's>;
pub trait EcsExtension {
    fn start_sequence<SFN: for<'w, 's> FnOnce(&mut SequenceHandle<'_, 'w, 's>)>(
        &mut self,
        sfn: SFN,
    ) -> Entity;
    fn branch<B: Branch>(&mut self, twig: Twig<B>) -> B::Handle;
    // fn add_leaf(&mut self) -> Entity;
    // fn location(&mut self, leaf: Entity, location: GridLocation);
    // fn flush_location(&mut self, tt: impl TriggerTargets);
    // fn opacity(&mut self, leaf: Entity, opacity: Opacity);
    // fn grid(&mut self, leaf: Entity, grid: Grid);
    // fn color(&mut self, leaf: Entity, color: Color);
    // fn elevation(&mut self, leaf: Entity, elevation: Elevation);
    // fn stem(&mut self, leaf: Entity, stem: Option<Entity>);
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
    fn remove(&mut self, leaf: Entity) {
        self.trigger_targets(Remove{}, leaf);
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
    fn remove(&mut self, leaf: Entity) {
        self.trigger_targets(Remove{}, leaf);
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
