use crate::animate::trigger::Trigger;
use crate::compositor::layout::Layout;
use crate::compositor::segment::{ResponsiveGrid, ResponsiveSegment};
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::leaf::{EmptySetDescriptor, Leaf, Tag};
use crate::elm::{Disabled, Elm};
use crate::ginkgo::viewport::ViewportHandle;
use crate::scene::{Anchor, Scene, SceneCoordinator, SceneHandle};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{DetectChanges, IntoSystemConfigs, Resource};
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{Commands, Query, Res, ResMut, SystemParamItem};
use std::collections::{HashMap, HashSet};

pub mod layout;
pub mod segment;

#[derive(Resource, Copy, Clone)]
pub struct CurrentView(pub(crate) ViewHandle);
impl CurrentView {
    pub fn change_view<VH: Into<ViewHandle>>(&mut self, vh: VH) {
        self.0 = vh.into();
    }
}
#[derive(Resource)]
pub struct Compositor {
    current: ViewHandle,
    layout: Layout,
    views: HashMap<ViewHandle, View>,
    entity_to_view: HashMap<Entity, ViewHandle>,
}
impl Compositor {
    pub fn add_responsive<B: Bundle>(
        &mut self,
        b: B,
        view_handle: ViewHandle,
        responsive_segment: ResponsiveSegment,
        cmd: &mut Commands,
    ) {
        let entity = cmd
            .spawn(b)
            .insert(Segmental::new(responsive_segment.viewed_at(view_handle)))
            .id();
        self.add_to_view(view_handle, entity);
    }
    pub fn add_responsive_scene<S: Scene>(
        &mut self,
        args: S,
        view_handle: ViewHandle,
        responsive_segment: ResponsiveSegment,
        external_args: &SystemParamItem<S::ExternalArgs>,
        coordinator: &mut SceneCoordinator,
        cmd: &mut Commands,
    ) {
        let (_handle, entity) =
            coordinator.spawn_scene::<S>(Anchor::default(), args, external_args, cmd);
        cmd.entity(entity)
            .insert(Segmental::new(responsive_segment.viewed_at(view_handle)));
        self.add_to_view(view_handle, entity);
    }
    pub fn new(area: Area<InterfaceContext>) -> Self {
        Self {
            current: ViewHandle::new(0, 0),
            layout: Layout::from_area(area),
            views: HashMap::new(),
            entity_to_view: HashMap::new(),
        }
    }
    pub fn layout(&self) -> Layout {
        self.layout
    }
    pub fn add_to_view<VH: Into<ViewHandle>>(&mut self, vh: VH, entity: Entity) {
        let handle = vh.into();
        if self.views.get(&handle).is_none() {
            self.views.insert(handle, View::default());
        }
        self.views.get_mut(&handle).unwrap().add(entity);
        self.entity_to_view.insert(entity, handle);
    }
    pub fn remove_from_view<VH: Into<ViewHandle>>(&mut self, vh: VH, entity: Entity) {
        self.views
            .get_mut(&vh.into())
            .unwrap()
            .entities
            .remove(&entity);
        self.entity_to_view.remove(&entity);
    }
    pub fn add_view<VH: Into<ViewHandle>>(&mut self, vh: VH) {
        let handle = vh.into();
        if self.views.get(&handle).is_none() {
            self.views.insert(handle, View::default());
        }
    }
    pub fn remove_view<VH: Into<ViewHandle>>(&mut self, vh: VH) {
        let handle = vh.into();
        if self.views.get(&handle).is_none() {
            return;
        }
        for ent in self.views.remove(&handle).unwrap().entities {
            self.entity_to_view.remove(&ent);
        }
    }
}
#[derive(Bundle)]
pub struct Segmental {
    segment: ResponsiveSegment,
    disabled: Disabled,
}
impl Segmental {
    pub fn new(segment: ResponsiveSegment) -> Self {
        Self {
            segment,
            disabled: Disabled::default(),
        }
    }
}
impl Leaf for Compositor {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.job
            .container
            .insert_resource(CurrentView(ViewHandle::new(0, 0)));
        elm.container().insert_resource(ResponsiveGrid::default());
        elm.main().add_systems((
            responsive_changed.in_set(CoreSet::Compositor),
            viewport_changed
                .in_set(CoreSet::Compositor)
                .run_if(|vh: Res<ViewportHandle>| -> bool { vh.is_changed() }),
            view_changed
                .in_set(CoreSet::TransitionView)
                .run_if(|cv: Res<CurrentView>| -> bool { cv.is_changed() }),
            despawn_triggered.in_set(ExternalSet::Spawn),
        ));
    }
}
fn responsive_changed(
    mut responsive: Query<
        (
            Entity,
            &ResponsiveSegment,
            &mut Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
            &mut Layer,
            &mut Disabled,
            Option<&SceneHandle>,
        ),
        Or<(Changed<ResponsiveSegment>, Changed<Disabled>)>,
    >,
    viewport_handle: Res<ViewportHandle>,
    compositor: Res<Compositor>,
    mut coordinator: ResMut<SceneCoordinator>,
    grid: Res<ResponsiveGrid>,
) {
    for (entity, segment, mut pos, mut area, mut layer, mut disabled, m_scene_handle) in
        responsive.iter_mut()
    {
        if let Some(coord) =
            segment.coordinate(compositor.layout(), viewport_handle.section(), &grid)
        {
            tracing::trace!("responsive-changed: {:?}", entity);
            if let Some(sh) = m_scene_handle {
                tracing::trace!(
                    "updating-anchor-from-compositor(responsive): {:?}:{:?}",
                    sh,
                    coord
                );
                coordinator.update_anchor(*sh, coord);
            }
            *pos = coord.section.position;
            *area = coord.section.area;
            *layer = coord.layer;
            if disabled.disabled() {
                *disabled = Disabled::inactive();
            }
        } else if !disabled.disabled() {
            *disabled = Disabled::active();
        }
    }
}
fn viewport_changed(
    mut compositor: ResMut<Compositor>,
    mut segments: Query<(
        Entity,
        &ResponsiveSegment,
        &mut Position<InterfaceContext>,
        &mut Area<InterfaceContext>,
        &mut Layer,
        &mut Disabled,
        Option<&SceneHandle>,
    )>,
    mut viewport_handle: ResMut<ViewportHandle>,
    mut coordinator: ResMut<SceneCoordinator>,
    grid: Res<ResponsiveGrid>,
) {
    if viewport_handle.area_updated() {
        let new_area = viewport_handle.section().area;
        compositor.layout = Layout::from_area(new_area);
        viewport_handle.set_position(compositor.current.anchor(new_area));
        for (entity, segment, mut pos, mut area, mut layer, mut disabled, m_scene_handle) in
            segments.iter_mut()
        {
            if let Some(coord) =
                segment.coordinate(compositor.layout(), viewport_handle.section(), &grid)
            {
                tracing::trace!("viewport-changed: {:?}", entity);
                if let Some(sh) = m_scene_handle {
                    tracing::trace!(
                        "updating-anchor-from-compositor(viewport): {:?}:{:?}",
                        sh,
                        coord
                    );
                    coordinator.update_anchor(*sh, coord);
                }
                *pos = coord.section.position;
                *area = coord.section.area;
                *layer = coord.layer;
                if disabled.disabled() {
                    *disabled = Disabled::inactive();
                }
            } else if !disabled.disabled() {
                *disabled = Disabled::active();
            }
        }
    }
}
fn view_changed(
    mut compositor: ResMut<Compositor>,
    current_view: ResMut<CurrentView>,
    mut viewport_handle: ResMut<ViewportHandle>,
    mut cmd: Commands,
) {
    if current_view.is_changed() {
        let old = compositor.current;
        let _new_anchor = current_view.0.anchor(viewport_handle.section().area);
        viewport_handle.set_position((_new_anchor.x, _new_anchor.y).into());
        compositor.current = current_view.0;
        cmd.spawn((
            Trigger(true),
            old,
            Tag::<View>::new(), /* TODO anchor-anim */
        ));
    }
}
fn despawn_triggered(
    mut compositor: ResMut<Compositor>,
    triggers: Query<(&Trigger, &ViewHandle), (With<Tag<View>>, Changed<Trigger>)>,
    mut despawn: Query<&mut Despawn>,
) {
    for (trigger, handle) in triggers.iter() {
        if trigger.triggered() {
            let entities = compositor
                .views
                .get_mut(handle)
                .unwrap()
                .entities
                .drain()
                .map(|e| {
                    *despawn.get_mut(e).unwrap() = Despawn::signal_despawn();
                    e
                })
                .collect::<Vec<Entity>>();
            tracing::trace!("triggered-despawn: {:?}", entities);
            for e in entities {
                compositor.entity_to_view.remove(&e);
            }
        }
    }
}
pub type ViewHandleOffset = i32;
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Debug, Component)]
pub struct ViewHandle(pub ViewHandleOffset, pub ViewHandleOffset);
impl ViewHandle {
    pub const fn new(x: ViewHandleOffset, y: ViewHandleOffset) -> Self {
        Self(x, y)
    }
    pub fn anchor(&self, area: Area<InterfaceContext>) -> Position<InterfaceContext> {
        
        (
            self.0 as CoordinateUnit * area.width,
            -self.1 as CoordinateUnit * area.height,
        )
            .into()
    }
}
#[derive(Default)]
pub struct View {
    entities: HashSet<Entity>,
}
impl View {
    pub fn add(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }
}
