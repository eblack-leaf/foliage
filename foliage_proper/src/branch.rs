use std::any::TypeId;
use std::collections::HashSet;

use crate::anim::{Animate, AnimationRunner, AnimationTime, Ease, Sequence, SequenceTimeRange};
use crate::differential::{Remove, RenderLink, RenderRemoveQueue, Visibility};
use crate::elm::{BranchLimiter, FilterAttrLimiter};
use crate::grid::{GridContext, ReferentialDependencies};
use crate::interaction::ClickInteractionListener;
use crate::layout::{Layout, LayoutFilter};
use crate::leaf::{BranchHandle, Dependents, IdTable, Leaf, LeafBundle, LeafHandle, OnEnd, Stem};
use crate::time::TimeDelta;
use crate::twig::Twig;
use bevy_ecs::change_detection::Mut;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Bundle, Changed, Commands, DetectChanges, Entity, Query, Resource, World};
use bevy_ecs::system::{Res, ResMut};
use bevy_ecs::world::Command;

pub struct Tree<'a> {
    pub(crate) world_handle: Option<&'a mut World>,
}

pub struct LeafPtr<'a> {
    pub(crate) world_handle: Option<&'a mut World>,
    pub(crate) handle: LeafHandle,
    pub(crate) entity: Entity,
}
pub struct FilteredAttributeConfig<A: Bundle + Send + Sync + 'static + Clone> {
    pub filter: LayoutFilter,
    pub a: A,
}
impl<A: Bundle + Send + Sync + 'static + Clone> FilteredAttributeConfig<A> {
    pub fn new(layout: Layout, a: A) -> Self {
        Self {
            filter: layout.into(),
            a,
        }
    }
}
#[derive(Component)]
pub struct FilteredAttribute<A: Bundle + Send + Sync + 'static + Clone> {
    filtered: Vec<FilteredAttributeConfig<A>>,
}
impl<A: Bundle + Send + Sync + 'static + Clone> Default for FilteredAttribute<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Bundle + Send + Sync + 'static + Clone> FilteredAttribute<A> {
    pub fn new() -> Self {
        Self { filtered: vec![] }
    }
    pub fn with(mut self, layout: Layout, a: A) -> Self {
        self.filtered.push(FilteredAttributeConfig::new(layout, a));
        self
    }
}
pub trait HasRenderLink {
    fn has_link() -> bool {
        false
    }
}
pub(crate) fn filter_attr_layout_change<A: Bundle + Send + Sync + 'static + Clone>(
    filtered: Query<(Entity, &FilteredAttribute<A>, Option<&RenderLink>)>,
    layout: Res<Layout>,
    cmd: Commands,
    render_remove_queue: ResMut<RenderRemoveQueue>,
) {
    if layout.is_changed() {
        for (entity, filter_attr, opt_link) in filtered.iter() {
            todo!()
            // if we have match then give else remove<A>
            // if removing + <A as HasRenderLink>::has_link() => send render-queue remove
        }
    }
}
pub(crate) fn filter_attr_changed<A: Bundle + Send + Sync + 'static + Clone>(
    filtered: Query<
        (Entity, &FilteredAttribute<A>, Option<&RenderLink>),
        Changed<FilteredAttribute<A>>,
    >,
    layout: Res<Layout>,
    cmd: Commands,
    render_remove_queue: ResMut<RenderRemoveQueue>,
) {
    for (entity, filtered_attr, opt_link) in filtered.iter() {
        todo!()
        // if we have match then give else remove<A>
        // if removing + <A as HasRenderLink>::has_link() => send render-queue remove
    }
}
impl<'a> LeafPtr<'a> {
    pub fn give<A: Bundle>(&mut self, a: A) {
        self.world_handle
            .as_mut()
            .unwrap()
            .entity_mut(self.entity)
            .insert(a);
    }
    pub fn get_mut<TH: Into<LeafHandle>, A: Component, AFN: FnOnce(&mut A)>(
        &mut self,
        th: TH,
        a_fn: AFN,
    ) {
        if TypeId::of::<A>() == TypeId::of::<Visibility>() {
            panic!("manually updating visibility does not affect dependents");
        }
        let handle = th.into();
        let entity = self
            .lookup_target_entity(handle)
            .expect("non-existent target handle");
        let mut comp = self
            .world_handle
            .as_mut()
            .unwrap()
            .get_mut::<A>(entity)
            .unwrap();
        a_fn(comp.as_mut());
    }
    pub fn give_filtered<A: Bundle + Send + Sync + 'static + Clone>(
        &mut self,
        filtered_attribute: FilteredAttribute<A>,
    ) {
        if !self
            .world_handle
            .as_ref()
            .unwrap()
            .contains_resource::<FilterAttrLimiter<A>>()
        {
            panic!("enable filtering for this attribute type")
        }
        self.world_handle
            .as_mut()
            .unwrap()
            .entity_mut(self.entity)
            .insert(filtered_attribute);
    }
    pub fn stem_from<RTH: Into<LeafHandle>>(&mut self, rth: RTH) {
        // lookup root
        let rth = rth.into();
        let root = self.lookup_target_entity(rth.clone()).unwrap();
        let root_vis = *self
            .world_handle
            .as_ref()
            .unwrap()
            .get::<Visibility>(root)
            .unwrap();
        self.world_handle
            .as_mut()
            .unwrap()
            .entity_mut(self.entity)
            .insert(root_vis);
        // give to that dependents
        self.world_handle
            .as_mut()
            .unwrap()
            .get_mut::<Dependents>(root)
            .unwrap()
            .0
            .insert(self.handle.clone());
        self.world_handle
            .as_mut()
            .unwrap()
            .get_mut::<Stem>(self.entity)
            .unwrap()
            .0
            .replace(rth);
    }
    fn lookup_target_entity<TH: Into<LeafHandle>>(&self, th: TH) -> Option<Entity> {
        self.world_handle
            .as_ref()
            .unwrap()
            .get_resource::<IdTable>()
            .unwrap()
            .lookup_leaf(th.into())
    }
}
pub struct SequenceHandle<'a> {
    world_handle: Option<&'a mut World>,
    sequence: Sequence,
    sequence_entity: Entity,
}
pub struct Animation<A: Animate> {
    leaf_handle: LeafHandle,
    a: A,
    sequence_time_range: SequenceTimeRange,
    ease: Ease,
}
impl<A: Animate> Animation<A> {
    pub fn new(a: A) -> Self {
        Self {
            leaf_handle: Default::default(),
            a,
            sequence_time_range: SequenceTimeRange::default(),
            ease: Ease::DECELERATE,
        }
    }
    pub fn targeting<LH: Into<LeafHandle>>(mut self, lh: LH) -> Self {
        self.leaf_handle = lh.into();
        self
    }
    pub fn start(mut self, s: u64) -> Self {
        self.sequence_time_range.start = TimeDelta::from_millis(s);
        self
    }
    pub fn end(mut self, e: u64) -> Self {
        self.sequence_time_range.end = TimeDelta::from_millis(e);
        self
    }
    pub fn eased(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
}
impl<'a> SequenceHandle<'a> {
    pub fn animate<A: Animate>(&mut self, animation: Animation<A>) {
        self.sequence.animations_to_finish += 1;
        let anim = AnimationRunner::new(
            animation.leaf_handle,
            animation.a,
            animation.ease,
            self.sequence_entity,
            AnimationTime::from(animation.sequence_time_range),
        );
        self.world_handle.as_mut().unwrap().spawn(anim);
    }
    pub fn on_end(&mut self, on_end: OnEnd) {
        self.sequence.on_end = on_end;
    }
    fn lookup_target_entity<TH: Into<LeafHandle>>(&self, th: TH) -> Option<Entity> {
        self.world_handle
            .as_ref()
            .unwrap()
            .get_resource::<IdTable>()
            .unwrap()
            .lookup_leaf(th.into())
    }
}
impl<'a> Tree<'a> {
    pub fn get_resource_mut<R: Resource>(&mut self) -> Mut<'_, R> {
        self.world_handle
            .as_mut()
            .unwrap()
            .get_resource_mut::<R>()
            .unwrap()
    }
    pub fn get_resource<R: Resource>(&self) -> &R {
        self.world_handle
            .as_ref()
            .unwrap()
            .get_resource::<R>()
            .unwrap()
    }
    pub fn add_resource<R: Resource>(&mut self, r: R) {
        self.world_handle.as_mut().unwrap().insert_resource(r);
    }
    pub fn update_attr_for<C: Component, CFN: FnOnce(&mut C), TH: Into<LeafHandle>>(
        &mut self,
        th: TH,
        c_fn: CFN,
    ) {
        if TypeId::of::<C>() == TypeId::of::<Visibility>() {
            panic!("manually updating visibility does not affect dependents");
        }
        let handle = th.into();
        let entity = self
            .lookup_target_entity(handle)
            .expect("non-existent target handle");
        let mut comp = self
            .world_handle
            .as_mut()
            .unwrap()
            .get_mut::<C>(entity)
            .unwrap();
        c_fn(comp.as_mut());
    }
    pub fn add_leaf<EFN: for<'b> FnOnce(&mut LeafPtr<'b>)>(&mut self, leaf: Leaf<EFN>) {
        let entity = self
            .world_handle
            .as_mut()
            .unwrap()
            .spawn(LeafBundle::default())
            .insert(leaf.name.clone())
            .insert(leaf.elevation)
            .insert(leaf.location)
            .id();
        if let Some(_old) = self
            .world_handle
            .as_mut()
            .unwrap()
            .get_resource_mut::<IdTable>()
            .unwrap()
            .add_target(leaf.name.clone(), entity)
        {
            panic!("overwriting leaf-handle") // TODO or warn deleting entity
                                              // *self
                                              //     .world_handle
                                              //     .as_mut()
                                              //     .unwrap()
                                              //     .get_mut::<Remove>(old)
                                              //     .unwrap() = Remove::queue_remove();
        }
        self.update_leaf(leaf.name.clone(), leaf.l_fn);
    }
    pub fn remove_leaf<LH: Into<LeafHandle>>(&mut self, lh: LH) {
        // queue remove of all dependents
        let handle = lh.into();
        let start = self
            .lookup_target_entity(handle.clone())
            .expect("attempting to remove non-existent element");
        self.world_handle
            .as_mut()
            .unwrap()
            .get_resource_mut::<IdTable>()
            .unwrap()
            .leafs
            .remove(&handle);
        self.world_handle
            .as_mut()
            .unwrap()
            .entity_mut(start)
            .insert(Remove::queue_remove());
        if let Some(root) = self
            .world_handle
            .as_ref()
            .unwrap()
            .get::<Stem>(start)
            .unwrap()
            .0
            .clone()
        {
            let entity = self.lookup_target_entity(root).unwrap();
            self.world_handle
                .as_mut()
                .unwrap()
                .get_mut::<Dependents>(entity)
                .unwrap()
                .0
                .remove(&handle);
        }
        let mut query =
            self.world_handle
                .as_mut()
                .unwrap()
                .query::<(Entity, &LeafHandle, &ReferentialDependencies)>();
        let mut referential = Vec::new();
        for (e, lh, ref_deps) in query.iter(self.world_handle.as_ref().unwrap()) {
            referential.push((e, lh, ref_deps));
        }
        let dependents = self.recursive_remove_leaf(handle, start, &referential);
        for (t, e) in dependents {
            self.world_handle
                .as_mut()
                .unwrap()
                .entity_mut(e)
                .insert(Remove::queue_remove());
            self.world_handle
                .as_mut()
                .unwrap()
                .get_resource_mut::<IdTable>()
                .unwrap()
                .leafs
                .remove(&t);
        }
    }
    fn recursive_remove_leaf(
        &self,
        leaf_handle: LeafHandle,
        current: Entity,
        referential: &Vec<(Entity, &LeafHandle, &ReferentialDependencies)>,
    ) -> HashSet<(LeafHandle, Entity)> {
        let mut removed_set = HashSet::new();
        if let Some(deps) = self
            .world_handle
            .as_ref()
            .unwrap()
            .get::<Dependents>(current)
        {
            for d in deps.0.iter() {
                let e = self.lookup_target_entity(d.clone()).unwrap();
                removed_set.insert((d.clone(), e));
                removed_set.extend(self.recursive_remove_leaf(d.clone(), e, referential));
            }
        }

        for (e, lh, ref_deps) in referential.iter() {
            if ref_deps
                .deps
                .contains(&GridContext::Named(leaf_handle.clone()))
            {
                removed_set.insert(((*lh).clone(), *e));
                removed_set.extend(self.recursive_remove_leaf((*lh).clone(), *e, referential));
            }
        }
        removed_set
    }
    pub fn update_leaf<LH: Into<LeafHandle>, LFN: for<'b> FnOnce(&mut LeafPtr<'b>)>(
        &mut self,
        lh: LH,
        l_fn: LFN,
    ) {
        let th = lh.into();
        let entity = self.lookup_target_entity(th.clone()).unwrap();
        let mut element_handle = LeafPtr {
            world_handle: self.world_handle.take(),
            entity,
            handle: th,
        };
        l_fn(&mut element_handle);
        self.world_handle = element_handle.world_handle.take();
    }
    pub fn change_leaf_stem<TH: Into<LeafHandle>>(&mut self, th: TH, new_root: Option<LeafHandle>) {
        let th = th.into();
        let this = self.lookup_target_entity(th.clone()).unwrap();
        if let Some(old) = self
            .world_handle
            .as_ref()
            .unwrap()
            .get::<Stem>(this)
            .unwrap()
            .0
            .as_ref()
        {
            let old_entity = self.lookup_target_entity(old.clone());
            if let Some(oe) = old_entity {
                self.world_handle
                    .as_mut()
                    .unwrap()
                    .get_mut::<Dependents>(oe)
                    .unwrap()
                    .0
                    .remove(&th);
            }
        }
        if let Some(rth) = new_root {
            let new_root_entity = self.lookup_target_entity(rth.clone()).unwrap();
            let new_root_visibility = self.get_visibility(rth.clone());
            self.update_visibility(th.clone(), new_root_visibility.visible());
            self.world_handle
                .as_mut()
                .unwrap()
                .get_mut::<Dependents>(new_root_entity)
                .unwrap()
                .0
                .insert(th.clone());
            self.world_handle
                .as_mut()
                .unwrap()
                .entity_mut(this)
                .insert(Stem::new(rth));
        } else {
            self.world_handle
                .as_mut()
                .unwrap()
                .entity_mut(this)
                .insert(Stem::default());
        }
    }
    pub fn get_visibility<TH: Into<LeafHandle>>(&self, th: TH) -> Visibility {
        *self
            .world_handle
            .as_ref()
            .unwrap()
            .get::<Visibility>(self.lookup_target_entity(th).unwrap())
            .unwrap()
    }
    pub fn update_visibility<TH: Into<LeafHandle>>(&mut self, th: TH, visibility: bool) {
        let handle = th.into();
        let entity = self.lookup_target_entity(handle.clone()).unwrap();
        let updated = self.recursive_visibility(entity);
        for entity in updated {
            self.world_handle
                .as_mut()
                .unwrap()
                .entity_mut(entity)
                .insert(Visibility::new(visibility));
        }
    }
    pub fn disable_interactions_for<TH: Into<LeafHandle>>(&mut self, th: TH) {
        let entity = self.lookup_target_entity(th).unwrap();
        self.world_handle
            .as_mut()
            .unwrap()
            .get_mut::<ClickInteractionListener>(entity)
            .unwrap()
            .disable();
    }
    pub fn enable_interactions_for<TH: Into<LeafHandle>>(&mut self, th: TH) {
        let entity = self.lookup_target_entity(th).unwrap();
        self.world_handle
            .as_mut()
            .unwrap()
            .get_mut::<ClickInteractionListener>(entity)
            .unwrap()
            .enable();
    }
    fn recursive_visibility(&self, current: Entity) -> HashSet<Entity> {
        let mut set = HashSet::new();
        set.insert(current);
        if let Some(deps) = self
            .world_handle
            .as_ref()
            .unwrap()
            .get::<Dependents>(current)
        {
            for d in deps.0.iter() {
                let e = self.lookup_target_entity(d.clone()).unwrap();
                set.extend(self.recursive_visibility(e));
            }
        }
        set
    }
    pub fn grow_branch<A: Branch>(&mut self, a: A) {
        let cmd = BranchCommand { data: a };
        cmd.apply(self.world_handle.as_mut().unwrap());
    }
    pub fn grow_twig<T: Clone>(&mut self, twig: Twig<T>)
    where
        Twig<T>: Branch,
    {
        self.grow_branch(twig);
    }
    pub fn run_sequence<SFN: FnOnce(&mut SequenceHandle<'a>)>(&mut self, seq_fn: SFN) {
        let se = self.world_handle.as_mut().unwrap().spawn_empty().id();
        let mut seq_handle = SequenceHandle {
            world_handle: self.world_handle.take(),
            sequence: Sequence::default(),
            sequence_entity: se,
        };
        seq_fn(&mut seq_handle);
        self.world_handle
            .replace(seq_handle.world_handle.take().unwrap());
        self.world_handle
            .as_mut()
            .unwrap()
            .entity_mut(se)
            .insert(seq_handle.sequence);
    }
    pub fn spawn<B: Bundle>(&mut self, b: B) {
        self.world_handle.as_mut().unwrap().spawn(b);
    }
    pub fn create_signaled_branch<A: Branch, AH: Into<BranchHandle>>(&mut self, ah: AH, a: A) {
        if !self
            .world_handle
            .as_ref()
            .unwrap()
            .contains_resource::<BranchLimiter<A>>()
        {
            panic!("please enable_signaled_action for this action type")
        }
        let signaler = self
            .world_handle
            .as_mut()
            .unwrap()
            .spawn(Signaler::new(a))
            .id();
        if let Some(o) = self
            .world_handle
            .as_mut()
            .unwrap()
            .get_resource_mut::<IdTable>()
            .unwrap()
            .add_branch(ah, signaler)
        {
            self.world_handle.as_mut().unwrap().despawn(o);
        }
    }
    pub(crate) fn lookup_target_entity<TH: Into<LeafHandle>>(&self, th: TH) -> Option<Entity> {
        self.world_handle
            .as_ref()
            .unwrap()
            .get_resource::<IdTable>()
            .unwrap()
            .lookup_leaf(th.into())
    }
}
pub trait Branch
where
    Self: Clone + Send + Sync + 'static,
{
    fn grow(self, tree: Tree);
}
#[derive(Clone)]
pub struct BranchCommand<A: Branch> {
    data: A,
}
impl<A: Branch> Command for BranchCommand<A> {
    fn apply(self, world: &mut World) {
        let branch = Tree {
            world_handle: Some(world),
        };
        self.data.grow(branch);
    }
}
#[derive(Component)]
pub struct Signal(pub bool);
impl Signal {
    pub fn active() -> Self {
        Self(true)
    }
    pub fn inactive() -> Self {
        Self(false)
    }
}
#[derive(Component)]
pub(crate) struct SignaledBranchCommand<A: Branch> {
    a: BranchCommand<A>,
}
pub(crate) fn signal_branch<A: Branch>(
    signals: Query<(&Signal, &SignaledBranchCommand<A>)>,
    mut cmd: Commands,
) {
    for (signal, signaled_branch) in signals.iter() {
        if signal.0 {
            let branch = signaled_branch.a.clone();
            cmd.add(branch);
        }
    }
}
pub(crate) fn clear_signal(mut signals: Query<&mut Signal, Changed<Signal>>) {
    for mut signal in signals.iter_mut() {
        signal.0 = false;
    }
}
#[derive(Bundle)]
pub(crate) struct Signaler<A: Branch> {
    branch: SignaledBranchCommand<A>,
    signal: Signal,
}
impl<A: Branch> Signaler<A> {
    pub fn new(a: A) -> Self {
        Self {
            branch: SignaledBranchCommand {
                a: BranchCommand { data: a },
            },
            signal: Signal(false),
        }
    }
}
