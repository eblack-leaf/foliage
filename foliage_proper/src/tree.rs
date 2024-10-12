use crate::anim::{Animate, Animation, AnimationRunner, AnimationTime, Sequence};
use crate::color::Color;
use crate::coordinate::elevation::Elevation;
use crate::grid::location::GridLocation;
use crate::grid::resolve::ResolveGridLocation;
use crate::grid::Grid;
use crate::leaf::{Leaf, Remove, ResolveElevation, ResolveVisibility, UpdateStem, Visibility};
use crate::opacity::{Opacity, ResolveOpacity};
use crate::time::OnEnd;
use crate::twig::{Branch, Twig};
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::system::Commands;
use bevy_ecs::world::World;

pub type Tree<'w, 's> = Commands<'w, 's>;
pub trait EcsExtension {
    fn start_sequence<SFN: for<'w, 's> FnOnce(&mut SequenceHandle<'_, 'w, 's>)>(
        &mut self,
        sfn: SFN,
    ) -> Entity;
    fn branch<B: Branch>(&mut self, twig: Twig<B>) -> B::Handle;
    fn add_leaf(&mut self) -> Entity;
    fn leaves(&mut self, n: usize) -> Vec<Entity>;
    fn location(&mut self, leaf: Entity, location: GridLocation);
    fn flush_location(&mut self, tt: impl TriggerTargets);
    fn opacity<O: Into<Opacity>>(&mut self, leaf: Entity, opacity: O);
    fn flush_opacity(&mut self, leaves: impl TriggerTargets);
    fn grid(&mut self, leaf: Entity, grid: Grid);
    fn color(&mut self, leaf: Entity, color: Color);
    fn elevation<E: Into<Elevation>>(&mut self, leaf: Entity, elevation: E);
    fn flush_elevation(&mut self, leaves: impl TriggerTargets);
    fn stem(&mut self, leaf: Entity, stem: Option<Entity>);
    fn visibility(&mut self, leaf: Entity, visibility: bool);
    fn remove(&mut self, entity: Entity);
    fn flush<F>(&mut self, leaves: F)
    where
        F: TriggerTargets + Clone,
    {
        self.flush_elevation(leaves.clone());
        self.flush_location(leaves.clone());
        self.flush_opacity(leaves);
    }
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
    fn add_leaf(&mut self) -> Entity {
        self.spawn(Leaf::default()).id()
    }
    fn leaves(&mut self, n: usize) -> Vec<Entity> {
        self.spawn_batch(vec![Leaf::default(); n])
            .entities()
            .collect()
    }
    fn location(&mut self, leaf: Entity, location: GridLocation) {
        self.entity(leaf).insert(location);
    }
    fn flush_location(&mut self, tt: impl TriggerTargets) {
        self.trigger_targets(ResolveGridLocation {}, tt);
    }
    fn opacity<O: Into<Opacity>>(&mut self, leaf: Entity, opacity: O) {
        self.entity(leaf).insert(opacity.into());
    }
    fn flush_opacity(&mut self, leaves: impl TriggerTargets) {
        self.trigger_targets(ResolveOpacity {}, leaves);
    }
    fn grid(&mut self, leaf: Entity, grid: Grid) {
        self.entity(leaf).insert(grid);
    }
    fn color(&mut self, leaf: Entity, color: Color) {
        self.entity(leaf).insert(color);
    }
    fn elevation<E: Into<Elevation>>(&mut self, leaf: Entity, elevation: E) {
        self.entity(leaf).insert(elevation.into());
    }
    fn flush_elevation(&mut self, leaves: impl TriggerTargets) {
        self.trigger_targets(ResolveElevation {}, leaves);
    }
    fn stem(&mut self, leaf: Entity, stem: Option<Entity>) {
        self.trigger_targets(UpdateStem(stem), leaf);
    }
    fn visibility(&mut self, leaf: Entity, visibility: bool) {
        self.entity(leaf).insert(Visibility::new(visibility));
        self.trigger_targets(ResolveVisibility(visibility), leaf);
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
    fn add_leaf(&mut self) -> Entity {
        self.spawn(Leaf::default()).id()
    }

    fn leaves(&mut self, n: usize) -> Vec<Entity> {
        self.spawn_batch(vec![Leaf::default(); n]).collect()
    }

    fn location(&mut self, leaf: Entity, location: GridLocation) {
        self.entity_mut(leaf).insert(location);
    }
    fn flush_location(&mut self, tt: impl TriggerTargets) {
        self.trigger_targets(ResolveGridLocation {}, tt);
    }
    fn opacity<O: Into<Opacity>>(&mut self, leaf: Entity, opacity: O) {
        self.entity_mut(leaf).insert(opacity.into());
    }
    fn flush_opacity(&mut self, leaves: impl TriggerTargets) {
        self.trigger_targets(ResolveOpacity {}, leaves);
    }
    fn grid(&mut self, leaf: Entity, grid: Grid) {
        self.entity_mut(leaf).insert(grid);
    }
    fn color(&mut self, leaf: Entity, color: Color) {
        self.entity_mut(leaf).insert(color);
    }
    fn elevation<E: Into<Elevation>>(&mut self, leaf: Entity, elevation: E) {
        self.entity_mut(leaf).insert(elevation.into());
    }
    fn flush_elevation(&mut self, leaves: impl TriggerTargets) {
        self.trigger_targets(ResolveElevation {}, leaves);
    }
    fn stem(&mut self, leaf: Entity, stem: Option<Entity>) {
        self.trigger_targets(UpdateStem(stem), leaf);
    }
    fn visibility(&mut self, leaf: Entity, visibility: bool) {
        self.entity_mut(leaf).insert(Visibility::new(visibility));
        self.trigger_targets(ResolveVisibility(visibility), leaf);
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
