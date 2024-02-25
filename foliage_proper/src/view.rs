use crate::aesthetic::Aesthetic;
use crate::conditional::{Branch, ConditionHandle, Conditional, SceneBranch, SpawnTarget};
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::InterfaceContext;
use crate::elm::config::CoreSet;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::{Disabled, Elm};
use crate::ginkgo::viewport::ViewportHandle;
use crate::layout::Layout;
use crate::scene::{Binder, ExtendTarget, Scene, SceneHandle};
use crate::segment::{MacroGrid, ResponsiveSegment};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Component, IntoSystemConfigs, Resource};
use bevy_ecs::system::{Commands, Query, ResMut};
use std::collections::{HashMap, HashSet};

pub struct ViewBuilder<'a, 'w, 's> {
    cmd: Option<&'a mut Commands<'w, 's>>,
    view_accumulator: ViewDescriptor,
    branch_handle: i32,
}
impl<'a, 'w, 's> ViewBuilder<'a, 'w, 's> {
    pub fn new(cmd: &'a mut Commands<'w, 's>) -> Self {
        Self {
            cmd: Some(cmd),
            view_accumulator: ViewDescriptor::default(),
            branch_handle: 0,
        }
    }
    fn cmd(&mut self) -> &mut Commands<'w, 's> {
        self.cmd.as_deref_mut().unwrap()
    }
    #[allow(unused)]
    pub fn apply_aesthetic<A: Aesthetic>(&mut self, a: A) -> ViewDescriptor {
        let cmd = self.cmd.take().unwrap();
        let mut sub_builder = Self::new(cmd);
        a.pigment(&mut sub_builder);
        self.cmd.replace(sub_builder.cmd.take().unwrap());
        sub_builder.finish()
    }
    pub fn add_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> SceneHandle {
        let desc = {
            let scene_desc = s.create(Binder::new(self.cmd.as_mut().unwrap(), None));
            self.cmd().entity(scene_desc.root()).insert(rs);
            scene_desc
        };
        self.view_accumulator.pool.0.insert(desc.root());
        desc
    }
    pub fn add<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity {
        let ent = { self.cmd().spawn(b).insert(rs).id() };
        self.view_accumulator.pool.0.insert(ent);
        ent
    }
    pub fn conditional<BR: Clone + Send + Sync + 'static>(
        &mut self,
        br: BR,
        rs: ResponsiveSegment,
    ) -> ConditionHandle {
        let desc = {
            let pre_spawned = self.cmd.as_mut().unwrap().spawn_empty().id();
            let branch_id = self
                .cmd()
                .spawn(Branch::new(br, SpawnTarget::This(pre_spawned), false))
                .insert(Conditional::new(rs, SpawnTarget::This(pre_spawned), false))
                .id();
            ConditionHandle::new(branch_id, pre_spawned)
        };
        self.view_accumulator
            .branches
            .insert(self.branch_handle, desc.this());
        self.branch_handle += 1;
        desc
    }
    pub fn conditional_scene<S: Scene + Clone>(
        &mut self,
        s: S,
        rs: ResponsiveSegment,
    ) -> ConditionHandle {
        let desc = {
            let pre_spawned = self.cmd.as_mut().unwrap().spawn_empty().id();
            let branch_id = self
                .cmd()
                .spawn(SceneBranch::new(s, SpawnTarget::This(pre_spawned), false))
                .insert(Conditional::new(rs, SpawnTarget::This(pre_spawned), false))
                .id();
            ConditionHandle::new(branch_id, pre_spawned)
        };
        self.view_accumulator
            .branches
            .insert(self.branch_handle, desc.this());
        self.branch_handle += 1;
        desc
    }
    pub fn extend<Ext: Bundle>(&mut self, entity: Entity, ext: Ext) {
        self.cmd().entity(entity).insert(ext);
    }
    pub fn conditional_extend<Ext: Bundle + Clone>(
        &mut self,
        ch: ConditionHandle,
        extend_target: ExtendTarget,
        ext: Ext,
    ) {
        match extend_target {
            ExtendTarget::This => {
                self.cmd().entity(ch.this()).insert(Conditional::<Ext>::new(
                    ext,
                    SpawnTarget::This(ch.target()),
                    true,
                ));
            }
            ExtendTarget::Binding(bind) => {
                self.cmd().entity(ch.this()).insert(Conditional::<Ext>::new(
                    ext,
                    SpawnTarget::BindingOf(ch.target(), bind),
                    true,
                ));
            }
        }
    }
    pub fn finish(self) -> ViewDescriptor {
        self.view_accumulator
    }
}
#[derive(Default)]
pub struct ViewDescriptor {
    pool: EntityPool,
    branches: HashMap<i32, Entity>,
}
pub type Create = fn(ViewBuilder) -> ViewDescriptor;
pub struct View {
    pub(crate) create: Box<Create>,
}
impl View {
    pub fn new(create: Create) -> Self {
        Self {
            create: Box::new(create),
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct Navigate(pub ViewHandle);
fn navigation(
    query: Query<(Entity, &Navigate)>,
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
) {
    if let Some((_e, n)) = query.iter().last() {
        if let Some(old) = compositor.current.take() {
            // despawn old
        }
        // call .create(cmd) ...
        let desc = (compositor.views.get(&n.0).expect("view").create)(ViewBuilder::new(&mut cmd));
        // set view
        compositor.current.replace(desc);
    }
    for (e, _) in query.iter() {
        cmd.entity(e).despawn();
    }
}
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct ViewHandle(pub i32);
impl From<i32> for ViewHandle {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
#[derive(Clone, Default)]
pub struct EntityPool(pub HashSet<Entity>);
#[derive(Resource, Default)]
pub struct Compositor {
    pub(crate) views: HashMap<ViewHandle, View>,
    current: Option<ViewDescriptor>,
    pub(crate) persistent: HashMap<ViewHandle, ViewDescriptor>,
}
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
        elm.main().add_systems((
            viewport_changed.in_set(CoreSet::Compositor),
            responsive_segment_changed.in_set(CoreSet::Compositor),
            navigation.in_set(CoreSet::Navigation),
        ));
    }
}