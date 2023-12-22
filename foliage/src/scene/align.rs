use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Changed, Or, Query};
#[derive(Component, Copy, Clone, Default)]
pub struct AlignmentDisable(pub bool);
pub(crate) fn calc_alignments_two(
    mut pos_aligned: Query<
        (
            &SceneAnchor,
            &mut Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &PositionAlignment,
            &AlignmentDisable,
        ),
        Or<(
            Changed<AlignmentDisable>,
            Changed<PositionAlignment>,
            Changed<SceneAnchor>,
            Changed<Position<InterfaceContext>>,
            Changed<Area<InterfaceContext>>,
        )>,
    >,
    mut layer_aligned: Query<
        (&SceneAnchor, &mut Layer, &LayerAlignment, &AlignmentDisable),
        Or<(
            Changed<AlignmentDisable>,
            Changed<LayerAlignment>,
            Changed<Layer>,
            Changed<SceneAnchor>,
        )>,
    >,
) {
    for (anchor, mut pos, area, alignment, disable) in pos_aligned.iter_mut() {
        if !disable.0 {
            let position = alignment.calc_pos(*anchor, *area);
            *pos = position;
        }
    }
    for (anchor, mut layer, alignment, disable) in layer_aligned.iter_mut() {
        if !disable.0 {
            *layer = alignment.calc_layer(anchor.0.layer);
        }
    }
}



#[derive(Copy, Clone, Component)]
pub struct SceneAnchor(pub Coordinate<InterfaceContext>);

impl From<Coordinate<InterfaceContext>> for SceneAnchor {
    fn from(value: Coordinate<InterfaceContext>) -> Self {
        Self(value)
    }
}