use std::collections::HashMap;
use std::rc::Rc;
use bevy_ecs::entity::Entity;
use crate::elm::compact_string_type_id;
use bevy_ecs::prelude::Component;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Component)]
pub struct RenderId(pub CompactString);
pub trait RenderIdentification {
    fn id() -> RenderId;
}
impl<T: Render + 'static> RenderIdentification for T {
    fn id() -> RenderId {
        RenderId(compact_string_type_id::<T>())
    }
}
pub(crate) type PerRendererFn<T: Render> = Box<fn(&T::Resources) -> ()>;
pub enum RenderRecordBehavior<T: Render> {
    PerRenderer(PerRendererFn<T>),
    PerRenderPackage
}
pub struct Renderer<T: Render> {
    resources: T::Resources,
    packages: HashMap<Entity, RenderPackage<T::RenderPackage>>,
}
pub struct RenderPackage<T: Render> {
    instruction_handle: Option<RenderInstructionHandle>,
    package_data: T::RenderPackage,
}
pub(crate) struct RenderInstructionHandle(pub(crate) Rc<wgpu::RenderBundle>);
pub struct RenderInstructionGroup {
    // vec instruction handle
    // phase
}
pub enum RenderPhase {
    Opaque,
    Alpha(i32)
}
pub trait Render {
    type Resources;
    type RenderPackage;
    fn record(&self, )
}
