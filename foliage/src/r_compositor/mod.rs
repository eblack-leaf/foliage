use crate::animate::trigger::Trigger;
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::InterfaceContext;
use crate::differential::Despawn;
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::leaf::{EmptySetDescriptor, Leaf, Tag};
use crate::elm::{Disabled, Elm, EventStage};
use crate::ginkgo::viewport::ViewportHandle;
use crate::r_compositor::layout::Layout;
use crate::r_compositor::segment::ResponsiveSegment;
use crate::scene::{SceneCoordinator, SceneHandle};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{Event, EventReader};
use bevy_ecs::prelude::{IntoSystemConfigs, Resource};
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{Commands, Query, Res, ResMut};
use std::collections::{HashMap, HashSet};

pub mod layout;
pub mod segment;

#[derive(Resource)]
pub struct Compositor {
    current: ViewHandle,
    layout: Layout,
    views: HashMap<ViewHandle, View>,
    entity_to_view: HashMap<Entity, ViewHandle>,
    anchors: HashMap<ViewHandle, Position<InterfaceContext>>,
}
impl Compositor {
    pub fn new(area: Area<InterfaceContext>) -> Self {
        Self {
            current: ViewHandle::new(0, 0),
            layout: Layout::from_area(area),
            views: HashMap::new(),
            entity_to_view: HashMap::new(),
            anchors: HashMap::new(),
        }
    }
    pub fn layout(&self) -> Layout {
        self.layout
    }
    pub fn add_to_view<VH: Into<ViewHandle>>(&mut self, vh: VH, entity: Entity) {
        let handle = vh.into();
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
        self.anchors.insert(handle, Position::default());
        self.views.insert(handle, View::default());
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
        elm.main().add_systems((
            (responsive_changed, viewport_changed).in_set(CoreSet::Compositor),
            view_changed.in_set(CoreSet::TransitionView),
            despawn_triggered.in_set(ExternalSet::Spawn),
        ));
        elm.add_event::<ViewTransition>(EventStage::Process);
    }
}
fn responsive_changed(
    mut responsive: Query<
        (
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
) {
    for (segment, mut pos, mut area, mut layer, mut disabled, m_scene_handle) in
        responsive.iter_mut()
    {
        if let Some(coord) = segment.coordinate(compositor.layout(), viewport_handle.section()) {
            if let Some(sh) = m_scene_handle {
                coordinator.update_anchor(*sh, coord);
            }
            *pos = coord.section.position;
            *area = coord.section.area;
            *layer = coord.layer;
            if disabled.disabled() {
                *disabled = Disabled::inactive();
            }
        } else {
            if !disabled.disabled() {
                *disabled = Disabled::active();
            }
        }
    }
}
fn viewport_changed(
    mut compositor: ResMut<Compositor>,
    mut segments: Query<(
        &ResponsiveSegment,
        &mut Position<InterfaceContext>,
        &mut Area<InterfaceContext>,
        &mut Layer,
        &mut Disabled,
        Option<&SceneHandle>,
    )>,
    viewport_handle: Res<ViewportHandle>,
    mut coordinator: ResMut<SceneCoordinator>,
) {
    if viewport_handle.area_updated() {
        compositor.layout = Layout::from_area(viewport_handle.section().area);
        for (segment, mut pos, mut area, mut layer, mut disabled, m_scene_handle) in
            segments.iter_mut()
        {
            if let Some(coord) = segment.coordinate(compositor.layout(), viewport_handle.section())
            {
                if let Some(sh) = m_scene_handle {
                    coordinator.update_anchor(*sh, coord);
                }
                *pos = coord.section.position;
                *area = coord.section.area;
                *layer = coord.layer;
                if disabled.disabled() {
                    *disabled = Disabled::inactive();
                }
            } else {
                if !disabled.disabled() {
                    *disabled = Disabled::active();
                }
            }
        }
    }
}
#[derive(Event)]
pub struct ViewTransition(pub ViewHandle);
fn view_changed(
    mut compositor: ResMut<Compositor>,
    mut events: EventReader<ViewTransition>,
    mut cmd: Commands,
) {
    if let Some(event) = events.read().last() {
        let old = compositor.current;
        let new_anchor = *compositor.anchors.get(&event.0).unwrap();
        compositor.current = event.0;
        cmd.spawn((
            Trigger::default(),
            old,
            Tag::<ViewTransition>::new(), /* anchor-anim */
        ));
    }
}
fn despawn_triggered(
    mut compositor: ResMut<Compositor>,
    triggers: Query<(&Trigger, &ViewHandle), (With<Tag<ViewTransition>>, Changed<Trigger>)>,
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
            for e in entities {
                compositor.entity_to_view.remove(&e);
            }
        }
    }
}
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Debug, Component)]
pub struct ViewHandle(pub i32, pub i32);
impl ViewHandle {
    pub fn new(x: i32, y: i32) -> Self {
        Self(x, y)
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
