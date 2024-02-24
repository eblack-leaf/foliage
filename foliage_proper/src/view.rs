use crate::animate::trigger::Trigger;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::InterfaceContext;
use crate::differential::Despawn;
use crate::elm::config::CoreSet;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::{Disabled, Elm};
use crate::ginkgo::viewport::ViewportHandle;
use crate::layout::Layout;
use crate::scene::{
    Binder, Bindings, ExtendTarget, Scene, SceneBinding, SceneComponents, SceneDesc,
};
use crate::segment::{MacroGrid, ResponsiveSegment};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Component, IntoSystemConfigs};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{
    Commands, Query, Res, ResMut, Resource, StaticSystemParam, SystemParam, SystemParamItem,
};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

#[derive(Component, Copy, Clone)]
pub struct Photosynthesize<V>(PhantomData<V>);
impl<V> Photosynthesize<V> {
    pub fn new() -> Self {
        Self { 0: PhantomData }
    }
}
pub(crate) fn photosynthesize<V: Photosynthesis + Send + Sync + 'static>(
    mut compositor: ResMut<Compositor>,
    navigation: Query<(Entity, &Photosynthesize<V>), Changed<Photosynthesize<V>>>,
    mut cmd: Commands,
    mut ext: StaticSystemParam<V::Chlorophyll>,
    mut grid: ResMut<MacroGrid>,
) {
    if let Some((_, _n)) = navigation.iter().last() {
        // TODO despawn current tree + all in pool
        // or anim-out && @-end trigger despawn

        *grid = V::GRID;
        let view = V::photosynthesize(&mut cmd, &mut ext);
        compositor.current.replace(view);
    }
    for (e, _) in navigation.iter() {
        cmd.entity(e).despawn();
    }
}
pub trait Photosynthesis {
    const GRID: MacroGrid;
    type Chlorophyll: SystemParam + 'static;
    fn photosynthesize(cmd: &mut Commands, res: &mut SystemParamItem<Self::Chlorophyll>) -> View;
}
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct BranchHandle(pub i32);
impl From<i32> for BranchHandle {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
#[derive(Component)]
pub struct Conditional<T: Clone> {
    wrapped: T,
    target: BranchExtendTarget,
    is_extension: bool,
}
impl<T: Clone> Conditional<T> {
    pub fn new(e: BranchExtendTarget, t: T, is_extension: bool) -> Self {
        Self {
            wrapped: t,
            target: e,
            is_extension,
        }
    }
}
#[derive(Component)]
pub struct ConditionalScene<S: Scene + Clone> {
    wrapped: S,
    target: BranchExtendTarget,
    is_extension: bool,
}
impl<S: Scene + Clone> ConditionalScene<S> {
    pub fn new(e: BranchExtendTarget, t: S, is_extension: bool) -> Self {
        Self {
            wrapped: t,
            target: e,
            is_extension,
        }
    }
}
//
#[derive(Bundle)]
pub struct Branch<T: Clone + Send + Sync + 'static> {
    conditional: Conditional<T>,
    trigger: Trigger,
}
impl<T: Clone + Send + Sync + 'static> Branch<T> {
    pub fn new(t: T, e: BranchExtendTarget, is_extension: bool) -> Self {
        Self {
            conditional: Conditional::<T>::new(e, t, is_extension),
            trigger: Trigger::default(),
        }
    }
}
#[derive(Bundle)]
pub struct SceneBranch<T: Clone + Scene + Send + Sync + 'static> {
    conditional: ConditionalScene<T>,
    trigger: Trigger,
}
impl<S: Scene + Clone> SceneBranch<S> {
    pub fn new(t: S, e: BranchExtendTarget, is_extension: bool) -> Self {
        Self {
            conditional: ConditionalScene::<S>::new(e, t, is_extension),
            trigger: Trigger::default(),
        }
    }
}
#[derive(Component, Clone, Default)]
pub struct View(pub HashSet<Entity>, HashMap<BranchHandle, Entity>);
pub struct Aesthetics<'a, 'w, 's> {
    cmd: &'a mut Commands<'w, 's>,
    view: View,
}
impl<'a, 'w, 's> Aesthetics<'a, 'w, 's> {
    pub fn new(cmd: &'a mut Commands<'w, 's>) -> Self {
        Self {
            cmd,
            view: View::default(),
        }
    }
    pub fn view(self) -> View {
        self.view
    }
    pub fn add_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> SceneDesc {
        let desc = {
            let scene_desc = s.create(Binder::new(self.cmd, None));
            self.cmd.entity(scene_desc.root()).insert(rs);
            scene_desc
        };
        self.view.0.insert(desc.root());
        desc
    }
    pub fn add<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity {
        let ent = { self.cmd.spawn(b).insert(rs).id() };
        self.view.0.insert(ent);
        ent
    }
    pub fn conditional<BR: Clone + Send + Sync + 'static, BH: Into<BranchHandle>>(
        &mut self,
        bh: BH,
        br: BR,
        rs: ResponsiveSegment,
    ) -> ConditionDesc {
        let desc = {
            let pre_spawned = self.cmd.spawn_empty().id();
            let branch_id = self
                .cmd
                .spawn(Branch::new(
                    br,
                    BranchExtendTarget::This(pre_spawned),
                    false,
                ))
                .insert(Conditional::new(
                    BranchExtendTarget::This(pre_spawned),
                    rs,
                    false,
                ))
                .id();
            ConditionDesc::new(branch_id, pre_spawned)
        };
        self.view.1.insert(bh.into(), desc.branch_entity);
        desc
    }
    pub fn conditional_scene<S: Scene + Clone, BH: Into<BranchHandle>>(
        &mut self,
        bh: BH,
        s: S,
        rs: ResponsiveSegment,
    ) -> ConditionDesc {
        let desc = {
            let pre_spawned = self.cmd.spawn_empty().id();
            let branch_id = self
                .cmd
                .spawn(SceneBranch::new(
                    s,
                    BranchExtendTarget::This(pre_spawned),
                    false,
                ))
                .insert(Conditional::new(
                    BranchExtendTarget::This(pre_spawned),
                    rs,
                    false,
                ))
                .id();
            ConditionDesc::new(branch_id, pre_spawned)
        };
        self.view.1.insert(bh.into(), desc.branch_entity);
        desc
    }
    pub fn extend<Ext: Bundle>(&mut self, entity: Entity, ext: Ext) {
        self.cmd.entity(entity).insert(ext);
    }
    pub fn conditional_extend<Ext: Bundle + Clone>(
        &mut self,
        branch_desc: ConditionDesc,
        extend_target: ExtendTarget,
        ext: Ext,
    ) {
        match extend_target {
            ExtendTarget::This => {
                self.cmd
                    .entity(branch_desc.branch_entity)
                    .insert(Conditional::<Ext>::new(
                        BranchExtendTarget::This(branch_desc.pre_spawned),
                        ext,
                        true,
                    ));
            }
            ExtendTarget::Binding(bind) => {
                self.cmd
                    .entity(branch_desc.branch_entity)
                    .insert(Conditional::<Ext>::new(
                        BranchExtendTarget::BindingOf(branch_desc.pre_spawned, bind),
                        ext,
                        true,
                    ));
            }
        }
    }
}
#[derive(Default, Resource)]
pub struct Compositor {
    current: Option<View>,
    persistent: Vec<View>,
}
impl Compositor {
    pub fn photosynthesize<V: Photosynthesis + Send + Sync + 'static>(cmd: &mut Commands) {
        // TODO add transition logic here then spawn
        cmd.spawn(Photosynthesize::<V>::new());
    }
}
#[derive(Component, Copy, Clone)]
pub struct BranchSet(pub BranchHandle, pub bool);
pub(crate) fn set_branch(
    query: Query<(Entity, &BranchSet)>,
    mut cmd: Commands,
    compositor: Res<Compositor>,
) {
    if compositor.current.is_some() {
        for (entity, branch_request) in query.iter() {
            let trigger = if !branch_request.1 {
                Trigger::inverse()
            } else {
                Trigger::active()
            };
            cmd.entity(
                *compositor
                    .current
                    .as_ref()
                    .unwrap()
                    .1
                    .get(&branch_request.0)
                    .expect("invalid-condition-set"),
            )
            .insert(trigger);
            cmd.entity(entity).despawn();
        }
    }
}
pub enum BranchExtendTarget {
    This(Entity),
    BindingOf(Entity, SceneBinding),
}
pub struct ConditionDesc {
    branch_entity: Entity,
    pre_spawned: Entity,
}
impl ConditionDesc {
    fn new(branch_entity: Entity, pre_spawned: Entity) -> Self {
        Self {
            branch_entity,
            pre_spawned,
        }
    }
}
// TODO Derived-Value handler + other
// pub struct OnEnter<T> {}

// // NOTE: type inference fails here, so annotations are required on the closure.
// commands.add(|w: &mut World| {
// // Mutate the world however you want...
// });
fn viewport_changed(
    mut query: Query<(
        &ResponsiveSegment,
        &mut Position<InterfaceContext>,
        &mut Area<InterfaceContext>,
        &mut Layer,
        &mut Disabled,
    )>,
    viewport_handle: Res<ViewportHandle>,
    grid: Res<MacroGrid>,
    mut layout: ResMut<Layout>,
) {
    if viewport_handle.area_updated() {
        *layout = Layout::from_area(viewport_handle.section().area);
        for (res_seg, mut pos, mut area, mut layer, mut disabled) in query.iter_mut() {
            // calc
            if let Some(coord) = res_seg.coordinate(*layout, viewport_handle.section(), &grid) {
                *pos = coord.section.position;
                *area = coord.section.area;
                *layer = coord.layer;
                if disabled.is_disabled() {
                    *disabled = Disabled::not_disabled();
                }
            } else {
                *disabled = Disabled::disabled();
            }
        }
    }
}
fn responsive_segment_changed(
    mut query: Query<
        (
            &ResponsiveSegment,
            &mut Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
            &mut Layer,
            &mut Disabled,
        ),
        Changed<ResponsiveSegment>,
    >,
    viewport_handle: Res<ViewportHandle>,
    layout: Res<Layout>,
    grid: Res<MacroGrid>,
) {
    for (res_seg, mut pos, mut area, mut layer, mut disabled) in query.iter_mut() {
        // calc
        if let Some(coord) = res_seg.coordinate(*layout, viewport_handle.section(), &grid) {
            *pos = coord.section.position;
            *area = coord.section.area;
            *layer = coord.layer;
            if disabled.is_disabled() {
                *disabled = Disabled::not_disabled();
            }
        } else {
            *disabled = Disabled::disabled();
        }
    }
}
impl Leaf for View {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.container().insert_resource(Compositor::default());
        elm.container().insert_resource(Layout::PORTRAIT_MOBILE);
        elm.container().insert_resource(MacroGrid::default());
        elm.main().add_systems((
            viewport_changed.in_set(CoreSet::Compositor),
            responsive_segment_changed.in_set(CoreSet::Compositor),
            set_branch.in_set(CoreSet::ConditionPrepare),
        ));
    }
}