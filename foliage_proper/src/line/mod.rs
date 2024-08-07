use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use wgpu::RenderPass;

use crate::ash::{DrawRange, Render, Renderer};
use crate::coordinate::Coordinates;
use crate::elm::{Elm, RenderQueueHandle};
use crate::ginkgo::Ginkgo;
use crate::Root;

#[derive(Bundle)]
pub struct Line {}
#[derive(Component, Copy, Clone)]
pub(crate) struct LinePoints {
    pub(crate) start: Coordinates,
    pub(crate) end: Coordinates,
}
pub(crate) struct LineVertexOffsets {
    pub(crate) start: Coordinates,
    pub(crate) end: Coordinates,
}
pub struct Weight(pub(crate) f32);
impl Weight {
    pub fn new(w: u32) -> Self {
        Self(w as f32)
    }
}
pub struct LinePercent(pub(crate) f32);
pub(crate) struct PercentDrawn {
    pub(crate) percent_start: LinePercent,
    pub(crate) percent_end: LinePercent,
}
pub(crate) struct LineDescriptor {
    pub(crate) main: LinePoints,
    pub(crate) right: LinePoints,
    pub(crate) left: LinePoints,
    pub(crate) start: LinePoints,
    pub(crate) end: LinePoints,
}
pub(crate) struct JoinedLines {
    pub(crate) joined: Vec<LineJoin>,
}
pub enum LineJoinMethod {
    Start,
    Percent(LinePercent),
    End,
}
pub(crate) struct LineJoinAngle {
    angle: f32,
}
pub(crate) struct LineJoin {
    pub(crate) joined: Entity,
    pub(crate) method: LineJoinMethod,
    pub(crate) angle_to_joined: LineJoinAngle,
}
pub struct LineRenderResources {}
impl Root for Line {
    fn define(elm: &mut Elm) {
        todo!()
    }
}
impl Render for Line {
    type DirectiveGroupKey = Entity;
    type Resources = LineRenderResources;

    fn create_resources(ginkgo: &Ginkgo) -> Self::Resources {
        todo!()
    }

    fn prepare(
        renderer: &mut Renderer<Self>,
        queue_handle: &mut RenderQueueHandle,
        ginkgo: &Ginkgo,
    ) {
        todo!()
    }

    fn draw<'a>(
        renderer: &'a Renderer<Self>,
        group_key: Self::DirectiveGroupKey,
        draw_range: DrawRange,
        render_pass: &mut RenderPass<'a>,
    ) {
        todo!()
    }
}
