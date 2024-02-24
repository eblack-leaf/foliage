use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::InterfaceContext;
use crate::elm::Disabled;
use crate::ginkgo::viewport::ViewportHandle;
use crate::layout::Layout;
use crate::segment::{MacroGrid, ResponsiveSegment};
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Component, Resource};
use bevy_ecs::system::{Commands, Query, ResMut};
use std::collections::{HashMap, HashSet};

pub struct ViewBuilder<'a, 'w, 's> {
    cmd: &'a mut Commands<'w, 's>,
}
impl<'a, 'w, 's> ViewBuilder<'a, 'w, 's> {
    pub fn new(cmd: &mut Commands) -> Self {
        Self { cmd }
    }
    pub fn finish(self) -> ViewDescriptor {
        todo!()
    }
}
#[derive(Default)]
pub struct ViewDescriptor {
    pool: EntityPool,
    // branch + specific entity collection
}
pub type Create = fn(ViewBuilder) -> ViewDescriptor;
pub struct View {
    create: Box<Create>,
    desc: ViewDescriptor,
}
impl View {
    pub fn new(create: Create) -> Self {
        Self {
            create: Box::new(create),
            desc: ViewDescriptor::default(),
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct Navigate(pub ViewHandle);
fn navigation(
    query: Query<(Entity, Navigate)>,
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
) {
    if let Some(l) = query.iter().last() {
        // despawn old
        // call .create(cmd) ...
        // set view
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
    views: HashMap<ViewHandle, View>,
    current: Option<ViewHandle>,
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