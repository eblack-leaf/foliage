use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;
use crate::{Attachment, Component, Coordinates, Differential, Foliage};
use crate::ash::differential::RenderQueue;

pub type IconId = i32;
#[derive(Component, Copy, Clone, PartialEq, Default)]
#[require(Color, Differential<Icon, Color>)]
#[require(ClipContext, Differential<Icon, ClipSection>)]
#[require(Differential<Icon, Section<Logical>>)]
#[require(Differential<Icon, Icon>)]
#[require(Differential<Icon, ResolvedElevation>)]
#[require(Differential<Icon, BlendedOpacity>)]
pub struct Icon {
    pub id: IconId
}
impl Attachment for Icon {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(RenderQueue::<Icon, IconMemory>::new());
        foliage.differential::<Icon, Icon>();
    }
}
impl Icon {
    pub const SCALE: Coordinates = Coordinates::new(24f32, 24f32);
    pub const TEXTURE_SCALE: Coordinates = Coordinates::new(96f32, 96f32);
    pub fn new(id: IconId) -> Self {
        Self { id }
    }
    pub fn memory(mem: MemoryId, bytes: Vec<u8>) -> IconMemory {
        IconMemory {
            mem,
            bytes,
        }
    }
}
#[derive(Component, Clone, Default)]
#[component(on_add = Self::on_add)]
pub struct IconMemory {
    pub mem: MemoryId,
    pub bytes: Vec<u8>
}
impl IconMemory {
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let value = world.get::<IconMemory>(this).unwrap();
        world.get_resource_mut::<RenderQueue<Icon, IconMemory>>().unwrap().queue.insert(this, value.clone());
    }
}
pub type MemoryId = i32;