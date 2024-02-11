use crate::ash::render_packet::RenderPacketStore;
use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Despawn, Differentiable, Differential, DifferentialBundle};
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::leaf::Leaf;
use crate::elm::Elm;
#[allow(unused)]
use crate::{coordinate, differential_enable};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Added, Query, SystemSet};
#[allow(unused)]
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs};
use bevy_ecs::query::Changed;
pub use bundled_cov::FeatherIcon;
use serde::{Deserialize, Serialize};
mod bundled_cov;
mod proc_gen;
mod renderer;
mod vertex;
#[derive(Default, Component, Clone, Deserialize, Serialize)]
pub(crate) struct RequestData(pub(crate) Option<Vec<u8>>);
#[derive(Default, Component, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WasRequest(pub(crate) bool);
#[derive(Bundle, Clone)]
pub struct Icon {
    scale: IconScale,
    icon_id: DifferentialBundle<IconId>,
    color: DifferentialBundle<Color>,
    data: RequestData,
    was_request: DifferentialBundle<WasRequest>,
    differentiable: Differentiable,
}
impl Icon {
    pub fn new<ID: Into<IconId>, C: Into<Color>>(icon_id: ID, scale: IconScale, color: C) -> Self {
        Self {
            scale,
            icon_id: DifferentialBundle::new(icon_id.into()),
            color: DifferentialBundle::new(color.into()),
            data: RequestData::default(),
            was_request: DifferentialBundle::new(WasRequest(false)),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
        }
    }
    pub fn storage<ID: Into<IconId>>(icon_id: ID, data: Vec<u8>) -> Self {
        Self {
            scale: IconScale::from_dim(12.0),
            icon_id: DifferentialBundle::new(icon_id.into()),
            color: DifferentialBundle::new(Color::default()),
            data: RequestData(Some(data)),
            was_request: DifferentialBundle::new(WasRequest(true)),
            differentiable: Differentiable::new::<Self>(
                Position::default(),
                Area::default(),
                Layer::default(),
            ),
        }
    }
}
#[derive(SystemSet, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum SetDescriptor {
    Area,
}
impl Leaf for Icon {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        differential_enable!(elm, CReprPosition, CReprArea, Color, IconId, WasRequest);
        elm.job.main().add_systems((
            scale_change.in_set(SetDescriptor::Area),
            id_changed.in_set(SetDescriptor::Area),
            clean_requests.after(CoreSet::RenderPacket),
            send_icon_data.in_set(CoreSet::Differential),
        ));
    }
}
fn send_icon_data(
    mut icon_requests: Query<(&mut RequestData, &mut RenderPacketStore), Changed<RequestData>>,
) {
    for (mut data, mut store) in icon_requests.iter_mut() {
        if data.0.is_some() {
            store.put(RequestData(Some(data.0.take().unwrap())));
        } else {
            store.put(RequestData(None));
        }
    }
}
#[derive(Component, Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct IconId(pub u32);
impl IconId {
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}
impl From<FeatherIcon> for IconId {
    fn from(value: FeatherIcon) -> Self {
        value.id()
    }
}
fn scale_change(
    mut query: Query<
        (&mut IconScale, &mut Area<InterfaceContext>),
        Changed<Area<InterfaceContext>>,
    >,
) {
    tracing::trace!("updating-icons");
    for (mut scale, mut area) in query.iter_mut() {
        *scale = IconScale::from_dim(area.width);
        let initial_px = scale.px();
        area.width = initial_px;
        area.height = initial_px;
    }
}
fn id_changed(
    mut icons: Query<
        (
            &mut Differential<Layer>,
            &mut Differential<CReprPosition>,
            &mut Differential<CReprArea>,
            &mut Differential<Color>,
        ),
        Changed<IconId>,
    >,
) {
    for (mut layer, mut pos, mut area, mut color) in icons.iter_mut() {
        layer.push_cached();
        pos.push_cached();
        area.push_cached();
        color.push_cached();
    }
}
fn clean_requests(mut query: Query<(&mut Despawn, &WasRequest), Added<WasRequest>>) {
    for (mut despawn, was_request) in query.iter_mut() {
        if was_request.0 {
            despawn.despawn();
        }
    }
}
#[derive(Component, Copy, Clone, Serialize, Deserialize, Debug)]
pub struct IconScale(pub(crate) CoordinateUnit);
impl IconScale {
    pub(crate) const UPPER_BOUND: u32 = 100;
    pub(crate) const LOWER_BOUND: u32 = 20;
    pub(crate) const INTERVAL: u32 = 4;
    pub fn px(self) -> CoordinateUnit {
        self.0
    }
    pub fn from_dim(r: CoordinateUnit) -> Self {
        let r = r - r % Self::INTERVAL as CoordinateUnit;
        Self(
            r.min(Self::UPPER_BOUND as f32)
                .max(Self::LOWER_BOUND as f32)
                .floor(),
        )
    }
}
