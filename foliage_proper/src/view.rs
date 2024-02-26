use crate::aesthetic::Aesthetic;
use crate::conditional::{Branch, ConditionHandle, Conditional, SceneBranch, SpawnTarget};
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
use crate::scene::{Binder, ExtendTarget, Scene, SceneHandle};
use crate::segment::{MacroGrid, ResponsiveSegment};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Component, IntoSystemConfigs, Resource};
use bevy_ecs::system::{Commands, Query, ResMut};
use std::collections::{HashMap, HashSet};
#[derive(Copy, Clone, Component)]
pub(crate) struct PersistentView(pub ViewHandle);
impl PersistentView {
    pub fn new<VH: Into<ViewHandle>>(vh: VH) -> Self {
        Self(vh.into())
    }
}
pub trait Viewable {
    const GRID: MacroGrid;
    fn view(view_builder: ViewBuilder) -> ViewDescriptor;
}
pub struct ViewBuilder<'a, 'w, 's> {
    cmd: Option<&'a mut Commands<'w, 's>>,
    view_descriptor: ViewDescriptor,
    branch_handle: i32,
}
impl<'a, 'w, 's> ViewBuilder<'a, 'w, 's> {
    pub fn new(cmd: &'a mut Commands<'w, 's>) -> Self {
        Self {
            cmd: Some(cmd),
            view_descriptor: ViewDescriptor::default(),
            branch_handle: 0,
        }
    }
    fn cmd(&mut self) -> &mut Commands<'w, 's> {
        self.cmd.as_deref_mut().unwrap()
    }
    pub fn apply_aesthetic<A: Aesthetic>(&mut self, a: A) -> ViewDescriptor {
        let cmd = self.cmd.take().unwrap();
        let mut sub_builder = Self::new(cmd);
        a.pigment(&mut sub_builder);
        self.cmd.replace(sub_builder.cmd.take().unwrap());
        let desc = sub_builder.finish();
        self.view_descriptor.pool.0.extend(&desc.pool.0);
        for b in desc.branches.iter() {
            self.view_descriptor
                .branches
                .insert(b.0 + self.branch_handle, *b.1);
            self.branch_handle += 1;
        }
        desc
    }
    pub fn add_scene<S: Scene>(&mut self, s: S, rs: ResponsiveSegment) -> SceneHandle {
        let desc = {
            let scene_desc = s.create(Binder::new(self.cmd.as_mut().unwrap(), None));
            self.cmd().entity(scene_desc.root()).insert(rs);
            scene_desc
        };
        self.view_descriptor.pool.0.insert(desc.root());
        desc
    }
    pub fn add<B: Bundle>(&mut self, b: B, rs: ResponsiveSegment) -> Entity {
        let ent = { self.cmd().spawn(b).insert(rs).id() };
        self.view_descriptor.pool.0.insert(ent);
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
        self.view_descriptor
            .branches
            .insert(self.branch_handle, desc);
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
        self.view_descriptor
            .branches
            .insert(self.branch_handle, desc);
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
        self.view_descriptor
    }
}
pub type Branches = HashMap<i32, ConditionHandle>;
#[derive(Default)]
pub struct ViewDescriptor {
    pool: EntityPool,
    branches: Branches,
}
impl ViewDescriptor {
    pub fn pool(&self) -> &EntityPool {
        &self.pool
    }
    pub fn branches(&self) -> &Branches {
        &self.branches
    }
}
pub type Create = fn(ViewBuilder) -> ViewDescriptor;
pub struct View {
    pub(crate) create: Box<Create>,
    pub(crate) grid: MacroGrid,
}
impl View {
    pub(crate) fn new(create: Create, macro_grid: MacroGrid) -> Self {
        Self {
            create: Box::new(create),
            grid: macro_grid,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct Navigate(pub ViewHandle);
fn navigation(
    query: Query<(Entity, &Navigate)>,
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
    mut macro_grid: ResMut<MacroGrid>,
) {
    if let Some((_e, n)) = query.iter().last() {
        if let Some(old) = compositor.current.take() {
            // despawn old
            for e in old.pool.0 {
                cmd.entity(e).insert(Despawn::signal_despawn());
            }
        }
        // call .create(cmd) ...

        let v = compositor.views.get(&n.0).expect("view");
        *macro_grid = v.grid;
        let desc = (v.create)(ViewBuilder::new(&mut cmd));
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
    pub(crate) persistent: HashMap<ViewHandle, (MacroGrid, ViewDescriptor)>,
}
fn viewport_changed(
    mut query: Query<(
        &ResponsiveSegment,
        &mut Position<InterfaceContext>,
        &mut Area<InterfaceContext>,
        &mut Layer,
        &mut Disabled,
        Option<&PersistentView>,
    )>,
    viewport_handle: Res<ViewportHandle>,
    grid: Res<MacroGrid>,
    mut layout: ResMut<Layout>,
    compositor: Res<Compositor>,
) {
    if viewport_handle.area_updated() {
        *layout = Layout::from_area(viewport_handle.section().area);
        for (res_seg, mut pos, mut area, mut layer, mut disabled, pv) in query.iter_mut() {
            // calc
            let g = if let Some(pg) = pv {
                &compositor
                    .persistent
                    .get(&pg.0)
                    .expect("invalid-persistent-link")
                    .0
            } else {
                &grid
            };
            if let Some(coord) = res_seg.coordinate(*layout, viewport_handle.section(), g) {
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
            Option<&PersistentView>,
        ),
        Changed<ResponsiveSegment>,
    >,
    viewport_handle: Res<ViewportHandle>,
    layout: Res<Layout>,
    grid: Res<MacroGrid>,
    compositor: Res<Compositor>,
) {
    for (res_seg, mut pos, mut area, mut layer, mut disabled, pv) in query.iter_mut() {
        // calc
        let g = if let Some(pg) = pv {
            &compositor
                .persistent
                .get(&pg.0)
                .expect("invalid-persistent-link")
                .0
        } else {
            &grid
        };
        if let Some(coord) = res_seg.coordinate(*layout, viewport_handle.section(), g) {
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
        elm.container().insert_resource(MacroGrid::new(8, 8));
        elm.main().add_systems((
            viewport_changed.in_set(CoreSet::Compositor),
            responsive_segment_changed.in_set(CoreSet::Compositor),
            navigation.in_set(CoreSet::Navigation),
        ));
    }
}