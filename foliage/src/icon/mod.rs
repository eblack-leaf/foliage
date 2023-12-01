use crate::color::Color;
use crate::coordinate::area::{Area, CReprArea};
use crate::coordinate::layer::Layer;
use crate::coordinate::position::{CReprPosition, Position};
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::{Differentiable, DifferentialBundle};
use crate::elm::{Elm, Leaf};
use crate::window::ScaleFactor;
#[allow(unused)]
use crate::{coordinate, differential_enable};
use bevy_ecs::component::Component;
#[allow(unused)]
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res};
use bundled_cov::BundledIcon;
use serde::{Deserialize, Serialize};

pub mod bundled_cov;
mod proc_gen;
mod renderer;
mod vertex;

#[derive(Bundle)]
pub struct Icon {
    position: Position<InterfaceContext>,
    area: Area<InterfaceContext>,
    scale: IconScale,
    icon_id: DifferentialBundle<IconId>,
    c_pos: DifferentialBundle<CReprPosition>,
    c_area: DifferentialBundle<CReprArea>,
    color: DifferentialBundle<Color>,
    differentiable: Differentiable,
}
impl Icon {
    pub fn new(
        icon_id: IconId,
        position: Position<InterfaceContext>,
        scale: IconScale,
        layer: Layer,
        color: Color,
    ) -> Self {
        let area = Area::new(scale.0, scale.0);
        Self {
            position,
            area,
            scale: IconScale(20.0),
            icon_id: DifferentialBundle::new(icon_id),
            c_pos: DifferentialBundle::new(CReprPosition::default()),
            c_area: DifferentialBundle::new(CReprArea::default()),
            color: DifferentialBundle::new(color),
            differentiable: Differentiable::new::<Self>(layer),
        }
    }
}
impl Leaf for Icon {
    fn attach(elm: &mut Elm) {
        differential_enable!(elm, CReprPosition, CReprArea, Color, IconId);
        // elm.job.main().add_systems((scale_icon
        //     .before(coordinate::area_set)
        //     .before(coordinate::position_set),));
    }
}
#[derive(Component, Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct IconId(pub u32);
impl IconId {
    pub fn new(bundled_icon: BundledIcon) -> Self {
        Self(bundled_icon as u32)
    }
}
#[derive(Component, Copy, Clone)]
pub struct IconScale(pub CoordinateUnit);
#[allow(unused)]
pub(crate) fn scale_icon(
    mut query: Query<
        (
            &mut Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
            &IconScale,
        ),
        Changed<IconScale>,
    >,
    scale_factor: Res<ScaleFactor>,
) {
    for (mut pos, mut area, scale) in query.iter_mut() {
        area.width = scale.0;
        area.height = scale.0;
        let scaled_area = area.to_device(scale_factor.factor());
        let diff = scaled_area.width - area.width;
        let diff = diff / scale_factor.factor();
        let half_diff = diff / 2f32;
        area.width -= half_diff;
        area.height -= half_diff;
        pos.x += half_diff;
        pos.y += half_diff;
    }
}
