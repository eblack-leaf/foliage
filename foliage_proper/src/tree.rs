use crate::anim::{Animate, Animation, AnimationRunner, AnimationTime, Sequence};
use crate::color::Color;
use crate::coordinate::elevation::Elevation;
use crate::grid::location::GridLocation;
use crate::grid::resolve::ResolveGridLocation;
use crate::grid::Grid;
use crate::leaf::{
    ChangeStem, Leaf, Remove, ResolveElevation, ResolveStem, ResolveVisibility, Stem, Visibility,
};
use crate::opacity::{Opacity, ResolveOpacity};
use crate::time::OnEnd;
use crate::twig::{Branch, Twig};
use bevy_ecs::entity::Entity;
use bevy_ecs::system::{Commands, EntityCommands};
use bevy_ecs::world::World;

pub type Tree<'w, 's> = Commands<'w, 's>;
pub trait EcsExtension {
    fn start_sequence<SFN: for<'w, 's> FnOnce(&mut SequenceHandle<'_, 'w, 's>)>(
        &mut self,
        sfn: SFN,
    ) -> Entity;
    fn branch<B: Branch>(&mut self, twig: Twig<B>) -> B::Handle;
    fn add_leaf<LFN: for<'a> FnOnce(LeafHandle<'a>)>(&mut self, lfn: LFN) -> Entity;
    fn update_leaf<LFN: for<'a> FnOnce(LeafHandle<'a>)>(&mut self, leaf: Entity, lfn: LFN);
    fn queue_remove(&mut self, leaf: Entity);
}
pub struct LeafHandle<'a> {
    pub(crate) repr: EntityCommands<'a>,
    pub(crate) from_add_leaf: bool,
}
impl<'a> LeafHandle<'a> {
    pub fn visibility(mut self, vis: bool) -> Self {
        self.repr
            .insert(Visibility::new(vis))
            .insert(ResolveVisibility {});
        self
    }
    pub fn located(mut self, loc: GridLocation) -> Self {
        self.repr.insert(loc).insert(ResolveGridLocation {});
        self
    }
    pub fn elevated<E: Into<Elevation>>(mut self, e: E) -> Self {
        self.repr.insert(e.into()).insert(ResolveElevation {});
        self
    }
    pub fn color<C: Into<Color>>(mut self, c: C) -> Self {
        self.repr.insert(c.into()).insert(ResolveOpacity {});
        self
    }
    pub fn stem_from(mut self, s: Option<Entity>) -> Self {
        if !self.from_add_leaf {
            panic!("please use change-stem to update existing Stem");
        }
        self.repr
            .insert(Stem(s))
            .insert(ResolveStem {})
            .insert(ResolveVisibility {})
            .insert(ResolveGridLocation {})
            .insert(ResolveElevation {})
            .insert(ResolveOpacity {});
        self
    }
    pub fn grid(mut self, grid: Grid) -> Self {
        self.repr.insert(grid).insert(ResolveGridLocation {});
        self
    }
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.repr
            .insert(Opacity::new(opacity))
            .insert(ResolveOpacity {});
        self
    }
    pub fn change_stem(mut self, stem: Option<Entity>) -> Self {
        if self.from_add_leaf {
            panic!("please use stem-from to declare Stem");
        }
        self.repr.insert(ChangeStem(stem)).insert(ResolveStem {});
        self
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
    fn add_leaf<LFN: for<'a> FnOnce(LeafHandle<'a>)>(&mut self, lfn: LFN) -> Entity {
        let id = self.spawn_empty().id();
        self.entity(id).insert(Leaf::new());
        self.update_leaf(id, lfn);
        id
    }
    fn update_leaf<LFN: for<'a> FnOnce(LeafHandle<'a>)>(&mut self, leaf: Entity, lfn: LFN) {
        let leaf_handle = LeafHandle {
            repr: self.entity(leaf),
            from_add_leaf: true,
        };
        lfn(leaf_handle);
    }
    fn queue_remove(&mut self, leaf: Entity) {
        self.entity(leaf).insert(Remove::queue_remove());
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
    fn add_leaf<LFN: for<'a> FnOnce(LeafHandle<'a>)>(&mut self, lfn: LFN) -> Entity {
        let mut cmds = self.commands();
        let e = cmds.add_leaf(lfn);
        e
    }
    fn update_leaf<LFN: for<'a> FnOnce(LeafHandle<'a>)>(&mut self, leaf: Entity, lfn: LFN) {
        let mut cmds = self.commands();
        cmds.update_leaf(leaf, lfn);
    }
    fn queue_remove(&mut self, leaf: Entity) {
        let mut cmds = self.commands();
        cmds.queue_remove(leaf);
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
