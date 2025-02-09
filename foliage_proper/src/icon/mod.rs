mod pipeline;
mod proc_gen;
use crate::ash::differential::RenderQueue;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::Stem;
use crate::{
    Attachment, Color, Component, Coordinates, Differential, Foliage, Logical, ResolvedElevation,
    Section, Visibility, Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::query::With;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

pub type IconId = i32;
#[derive(Component, Copy, Clone, PartialEq, Default)]
#[component(on_add = Self::on_add)]
#[require(Color, Differential<Icon, Color>)]
#[require(Differential<Icon, Stem>)]
#[require(Differential<Icon, Section<Logical>>)]
#[require(Differential<Icon, Icon>)]
#[require(Differential<Icon, ResolvedElevation>)]
#[require(Differential<Icon, BlendedOpacity>)]
pub struct Icon {
    pub id: IconId,
}
impl Attachment for Icon {
    fn attach(foliage: &mut Foliage) {
        foliage
            .world
            .insert_resource(RenderQueue::<Icon, IconMemory>::new());
        foliage.remove_queue::<Icon>();
        foliage.differential::<Icon, Icon>();
        foliage.differential::<Icon, Section<Logical>>();
        foliage.differential::<Icon, Stem>();
        foliage.differential::<Icon, ResolvedElevation>();
        foliage.differential::<Icon, Color>();
        foliage.differential::<Icon, BlendedOpacity>();
    }
}
impl Icon {
    pub const SCALE: Coordinates = Coordinates::new(24f32, 24f32);
    pub const TEXTURE_SCALE: Coordinates = Coordinates::new(96f32, 96f32);
    pub fn new<ID: Into<IconId>>(id: ID) -> Self {
        Self { id: id.into() }
    }
    pub fn memory<ID: Into<IconId>, M: AsRef<[u8]>>(mem: ID, bytes: M) -> IconMemory {
        IconMemory {
            id: mem.into(),
            bytes: bytes.as_ref().to_vec(),
        }
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .entity(this)
            .observe(Visibility::push_remove_packet::<Self>)
            .observe(Remove::push_remove_packet::<Self>)
            .observe(Self::only_24_px);
    }
    fn only_24_px(
        trigger: Trigger<Write<Section<Logical>>>,
        mut sections: Query<&mut Section<Logical>, With<Icon>>,
    ) {
        if let Ok(mut sec) = sections.get_mut(trigger.entity()) {
            if sec.area.coordinates != Self::SCALE {
                sec.area.coordinates = Self::SCALE;
            }
        }
    }
}
#[derive(Component, Clone, Default)]
#[component(on_add = Self::on_add)]
pub struct IconMemory {
    pub id: IconId,
    pub bytes: Vec<u8>,
}
impl IconMemory {
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let value = world.get::<IconMemory>(this).unwrap().clone();
        world
            .get_resource_mut::<RenderQueue<Icon, IconMemory>>()
            .unwrap()
            .queue
            .insert(this, value);
        world.commands().entity(this).despawn();
    }
}
