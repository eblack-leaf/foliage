use crate::coordinate::points::Points;
use crate::foliage::DiffMarkers;
use crate::ginkgo::ScaleFactor;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::Differential;
use crate::Stem;
use crate::{
    Attachment, Color, Component, Coordinates, Foliage, Logical, Position, ResolvedElevation,
    Visibility,
};
use bevy_ecs::change_detection::Res;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, IntoSystemConfigs, Or, Query};
use bevy_ecs::world::DeferredWorld;
use bytemuck::{Pod, Zeroable};

mod pipeline;
#[derive(Component, Copy, Clone)]
#[require(Shape)]
pub struct Line {
    pub weight: i32,
}
impl Attachment for Shape {
    fn attach(foliage: &mut Foliage) {
        foliage
            .diff
            .add_systems(Line::distill_descriptor.in_set(DiffMarkers::Finalize));
        foliage.remove_queue::<Shape>();
        foliage.differential::<Shape, Shape>();
        foliage.differential::<Shape, BlendedOpacity>();
        foliage.differential::<Shape, ResolvedElevation>();
        foliage.differential::<Shape, Stem>();
        foliage.differential::<Shape, Color>();
    }
}
impl Line {
    pub fn new(w: i32) -> Self {
        Self { weight: w.max(1) }
    }
    pub(crate) fn distill_descriptor(
        mut lines: Query<
            (&Points<Logical>, &Line, &mut Shape),
            Or<(Changed<Line>, Changed<Points<Logical>>)>,
        >,
        scale_factor: Res<ScaleFactor>,
    ) {
        for (points, line, mut shape) in lines.iter_mut() {
            let pts = (points.data[0].coordinates, points.data[1].coordinates);
            let x_diff = pts.1.a() - pts.0.a();
            let y_diff = pts.1.b() - pts.0.b();
            let slope = y_diff / x_diff;
            let normal_slope = 1.0 / slope;
            let angle = normal_slope.atan();
            let half_weight = line.weight as f32 / 2.0;
            let factor = f32::from(x_diff.abs() > 0.0 && y_diff.abs() > 0.0);
            let angle_bias = 0.5 * factor;
            // println!(
            //     "angle-bias: {} with {} for {}-{}",
            //     angle_bias, factor, x_diff, y_diff
            // );
            let x_adjust = angle.cos() * (half_weight + angle_bias);
            let y_adjust = angle.sin() * (half_weight + angle_bias);
            let left_top = Position::logical((pts.0.a() + x_adjust, pts.0.b() - y_adjust));
            let left_bottom = Position::logical((pts.0.a() - x_adjust, pts.0.b() + y_adjust));
            let right_top = Position::logical((pts.1.a() + x_adjust, pts.1.b() - y_adjust));
            let right_bottom = Position::logical((pts.1.a() - x_adjust, pts.1.b() + y_adjust));
            *shape = Shape::new(
                EdgePoints::new(
                    left_bottom.to_physical(scale_factor.value()).coordinates,
                    left_top.to_physical(scale_factor.value()).coordinates,
                ),
                EdgePoints::new(
                    right_bottom.to_physical(scale_factor.value()).coordinates,
                    right_top.to_physical(scale_factor.value()).coordinates,
                ),
            );
        }
    }
}
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, Default, PartialEq)]
pub struct EdgePoints {
    pub start: Coordinates,
    pub end: Coordinates,
}
impl EdgePoints {
    pub fn new(start: Coordinates, end: Coordinates) -> Self {
        Self { start, end }
    }
}
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, Default, PartialEq)]
#[require(Differential<Shape, Shape>)]
#[require(Differential<Shape, Stem>)]
#[require(Color, Differential<Shape, Color>)]
#[require(Differential<Shape, ResolvedElevation>)]
#[require(Differential<Shape, BlendedOpacity>)]
#[require(Points<Logical>)]
#[component(on_add = Self::on_add)]
pub struct Shape {
    pub left: EdgePoints,
    pub right: EdgePoints,
}
impl Shape {
    pub fn new(left: EdgePoints, right: EdgePoints) -> Self {
        Self { left, right }
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .entity(this)
            .observe(Remove::push_remove_packet::<Self>)
            .observe(Visibility::push_remove_packet::<Self>);
    }
}
