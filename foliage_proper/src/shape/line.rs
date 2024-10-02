use crate::color::Color;
use crate::coordinate::points::Points;
use crate::coordinate::position::Position;
use crate::coordinate::{CoordinateUnit, LogicalContext};
use crate::elm::{Elm, InternalStage};
use crate::ginkgo::ScaleFactor;
use crate::shape::{EdgePoints, Shape, ShapeDescriptor};
use crate::Root;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Query, Res};

#[derive(Bundle)]
pub struct Line {
    shape: Shape,
    line_weight: LineWeight,
}
impl From<i32> for LineWeight {
    fn from(line_weight: i32) -> Self {
        Self(line_weight as f32)
    }
}
impl Root for Line {
    fn attach(elm: &mut Elm) {
        elm.scheduler
            .main
            .add_systems(distill_descriptor.in_set(InternalStage::Prepare));
    }
}
#[derive(Component)]
pub struct LineWeight(pub CoordinateUnit);
impl LineWeight {
    pub fn new(line_weight: i32) -> Self {
        Self(line_weight as f32)
    }
}
pub(crate) fn distill_descriptor(
    mut lines: Query<
        (&Points<LogicalContext>, &LineWeight, &mut ShapeDescriptor),
        Or<(Changed<LineWeight>, Changed<Points<LogicalContext>>)>,
    >,
    scale_factor: Res<ScaleFactor>,
) {
    for (points, line_weight, mut shape) in lines.iter_mut() {
        let line = (points.data[0].coordinates, points.data[1].coordinates);
        let slope =
            (line.1.vertical() - line.0.vertical()) / (line.1.horizontal() - line.0.horizontal());
        let normal_slope = 1.0 / slope;
        let angle = normal_slope.atan();
        let half_weight = line_weight.0 / 2.0;
        let x_adjust = angle.cos() * half_weight;
        let y_adjust = angle.sin() * half_weight;
        let left_top =
            Position::logical((line.0.horizontal() + x_adjust, line.0.vertical() - y_adjust));
        let left_bottom =
            Position::logical((line.0.horizontal() - x_adjust, line.0.vertical() + y_adjust));
        let right_top =
            Position::logical((line.1.horizontal() + x_adjust, line.1.vertical() - y_adjust));
        let right_bottom =
            Position::logical((line.1.horizontal() - x_adjust, line.1.vertical() + y_adjust));
        *shape = ShapeDescriptor::new(
            EdgePoints::new(
                left_bottom
                    .to_device(scale_factor.value())
                    .rounded()
                    .coordinates,
                left_top
                    .to_device(scale_factor.value())
                    .rounded()
                    .coordinates,
            ),
            EdgePoints::new(
                right_bottom
                    .to_device(scale_factor.value())
                    .rounded()
                    .coordinates,
                right_top
                    .to_device(scale_factor.value())
                    .rounded()
                    .coordinates,
            ),
        );
    }
}
impl Line {
    pub fn new<C: Into<Color>, W: Into<LineWeight>>(w: W, c: C) -> Self {
        Line {
            shape: Shape::new(ShapeDescriptor::default(), c.into()),
            line_weight: w.into(),
        }
    }
}
