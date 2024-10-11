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

pub type Tree<'w, 's> = Commands<'w, 's>;
pub trait EcsExtension {
    fn start_sequence<SFN: for<'w, 's> FnOnce(&mut SequenceHandle<'_, 'w, 's>)>(
        &mut self,
        sfn: SFN,
    ) -> Entity;
    fn branch<B: Branch>(&mut self, twig: Twig<B>) -> B::Handle;
    fn add_leaf<LFN: for<'a> FnOnce(&mut LeafHandle<'a>)>(&mut self, lfn: LFN) -> Entity;
    fn update_leaf<LFN: for<'a> FnOnce(&mut LeafHandle<'a>)>(&mut self, leaf: Entity, lfn: LFN);
    fn queue_remove(&mut self, leaf: Entity);
}
pub struct LeafHandle<'a> {
    pub(crate) repr: EntityCommands<'a>,
    pub(crate) from_add_leaf: bool,
}
impl<'a> LeafHandle<'a> {
    pub fn visibility(&mut self, vis: bool) {
        self.repr
            .insert(Visibility::new(vis))
            .insert(ResolveVisibility {});
    }
    pub fn location(&mut self, loc: GridLocation) {
        self.repr.insert(loc).insert(ResolveGridLocation {});
    }
    pub fn elevation<E: Into<Elevation>>(&mut self, e: E) {
        self.repr.insert(e.into()).insert(ResolveElevation {});
    }
    pub fn color<C: Into<Color>>(&mut self, c: C) {
        self.repr.insert(c.into()).insert(ResolveOpacity {});
    }
    pub fn stem_from(&mut self, s: Option<Entity>) {
        self.repr
            .insert(Stem(s))
            .insert(ResolveStem {})
            .insert(ResolveVisibility {})
            .insert(ResolveGridLocation {})
            .insert(ResolveElevation {})
            .insert(ResolveOpacity {});
    }
    pub fn grid(&mut self, grid: Grid) {
        self.repr.insert(grid).insert(ResolveGridLocation {});
    }
    pub fn opacity(&mut self, opacity: f32) {
        self.repr
            .insert(Opacity::new(opacity))
            .insert(ResolveOpacity {});
    }
    pub fn change_stem(&mut self, stem: Option<Entity>) {
        if self.from_add_leaf {
            panic!("please use stem-from to declare Stem");
        }
        self.repr.insert(UpdateStem(stem)).insert(ResolveStem {});
    }
    pub fn give<A: Bundle>(&mut self, a: A) {
        debug_assert_ne!(TypeId::of::<A>(), TypeId::of::<Color>());
        debug_assert_ne!(TypeId::of::<A>(), TypeId::of::<Opacity>());
        debug_assert_ne!(TypeId::of::<A>(), TypeId::of::<Stem>());
        debug_assert_ne!(TypeId::of::<A>(), TypeId::of::<GridLocation>());
        debug_assert_ne!(TypeId::of::<A>(), TypeId::of::<UpdateStem>());
        debug_assert_ne!(TypeId::of::<A>(), TypeId::of::<Visibility>());
        debug_assert_ne!(TypeId::of::<A>(), TypeId::of::<Elevation>());
        debug_assert_ne!(TypeId::of::<A>(), TypeId::of::<Grid>());
        self.repr.insert(a);
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
    fn add_leaf<LFN: for<'a> FnOnce(&mut LeafHandle<'a>)>(&mut self, lfn: LFN) -> Entity {
        let id = self.spawn_empty().id();
        self.entity(id).insert(Leaf::default());
        let mut leaf_handle = LeafHandle {
            repr: self.entity(id),
            from_add_leaf: true,
        };
        lfn(&mut leaf_handle);
        id
    }
    fn update_leaf<LFN: for<'a> FnOnce(&mut LeafHandle<'a>)>(&mut self, leaf: Entity, lfn: LFN) {
        let mut leaf_handle = LeafHandle {
            repr: self.entity(leaf),
            from_add_leaf: false,
        };
        lfn(&mut leaf_handle);
    }
    fn queue_remove(&mut self, leaf: Entity) {
        self.entity(leaf).insert(Remove::remove());
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
    fn add_leaf<LFN: for<'a> FnOnce(&mut LeafHandle<'a>)>(&mut self, lfn: LFN) -> Entity {
        let mut cmds = self.commands();
        let e = cmds.add_leaf(lfn);
        e
    }
    fn update_leaf<LFN: for<'a> FnOnce(&mut LeafHandle<'a>)>(&mut self, leaf: Entity, lfn: LFN) {
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
