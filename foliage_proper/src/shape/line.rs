use crate::color::Color;
use crate::coordinate::points::Points;
use crate::coordinate::position::Position;
use crate::coordinate::{CoordinateUnit, LogicalContext};
use crate::elm::{Elm, InternalStage};
use crate::ginkgo::ScaleFactor;
use crate::panel::{Panel, Rounding};
use crate::shape::{EdgePoints, Shape, ShapeDescriptor};
use crate::Root;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Query, Res};

#[derive(Bundle)]
pub struct Line {
    shape: Shape,
    line_weight: LineWeight,
}
impl Line {
    pub const MINIMUM_WEIGHT_THRESHOLD: f32 = 3.0;
}
impl From<i32> for LineWeight {
    fn from(line_weight: i32) -> Self {
        Self((line_weight as f32).max(Line::MINIMUM_WEIGHT_THRESHOLD))
    }
}
impl Root for Line {
    fn attach(elm: &mut Elm) {
        elm.scheduler
            .main
            .add_systems(distill_descriptor.in_set(InternalStage::Prepare));
    }
}
#[derive(Bundle)]
pub struct LineJoin {
    panel: Panel,
}
impl LineJoin {
    pub fn new<C: Into<Color>>(c: C) -> Self {
        Self {
            panel: Panel::new(Rounding::all(1.0), c.into()),
        }
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
        (
            Entity,
            &Points<LogicalContext>,
            &LineWeight,
            &mut ShapeDescriptor,
        ),
        Or<(Changed<LineWeight>, Changed<Points<LogicalContext>>)>,
    >,
    scale_factor: Res<ScaleFactor>,
) {
    for (entity, points, line_weight, mut shape) in lines.iter_mut() {
        let line = (points.data[0].coordinates, points.data[1].coordinates);
        let x_diff = line.1.horizontal() - line.0.horizontal();
        let y_diff = line.1.vertical() - line.0.vertical();
        let slope = y_diff / x_diff;
        let normal_slope = 1.0 / slope;
        let angle = normal_slope.atan();
        let half_weight = line_weight.0 / 2.0;
        let factor = f32::from(x_diff.abs() > 0.0 && y_diff.abs() > 0.0);
        let angle_bias = 0.25 * factor;
        println!(
            "angle-bias: {} with {} for {}-{}",
            angle_bias, factor, x_diff, y_diff
        );
        let x_adjust = angle.cos() * (half_weight + angle_bias);
        let y_adjust = angle.sin() * (half_weight + angle_bias);
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
