mod pipeline;

use crate::ash::clip::ClipSection;
use crate::ash::differential::RenderQueue;
use crate::asset::AssetLoader;
use crate::foliage::DiffMarkers;
use crate::grid::AspectRatio;
use crate::opacity::BlendedOpacity;
use crate::Differential;
use crate::{asset_retrieval, AssetKey, AssetRetrieval, ClipContext};
use crate::{
    Area, Attachment, Component, Coordinates, Foliage, Layout, Logical, Numerical,
    ResolvedElevation, Section,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::prelude::{Entity, IntoSystemConfigs, Res};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Query, ResMut};
use bevy_ecs::world::DeferredWorld;
use wgpu::TextureFormat;

#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_insert = Self::on_insert)]
#[require(ImageView, ImageMetrics, ClipContext)]
#[require(Differential<Image, Section<Logical>>)]
#[require(Differential<Image, BlendedOpacity>)]
#[require(Differential<Image, ResolvedElevation>)]
#[require(Differential<Image, ClipSection>)]
pub struct Image {
    pub memory_id: MemoryId,
    pub key: AssetKey,
}
#[derive(Component, Copy, Clone, PartialEq, Default)]
#[component(on_insert = Self::on_insert)]
pub enum ImageView {
    #[default]
    Aspect,
    Crop,
    None,
}
impl ImageView {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let value = *world.get::<ImageView>(this).unwrap();
        let metrics = world.get::<ImageMetrics>(this).copied().unwrap_or_default();
        match value {
            ImageView::Aspect => {
                if metrics.extent != Area::default() {
                    let ratio =
                        AspectRatio::new().xs(metrics.extent.width() / metrics.extent.height());
                    world.commands().entity(this).insert(ratio);
                }
                world
                    .get_resource_mut::<RenderQueue<Image, CropAdjustment>>()
                    .unwrap()
                    .queue
                    .insert(this, CropAdjustment::default());
            }
            ImageView::None => {
                world.commands().entity(this).insert(AspectRatio::new());
                world
                    .get_resource_mut::<RenderQueue<Image, CropAdjustment>>()
                    .unwrap()
                    .queue
                    .insert(this, CropAdjustment::default());
            }
            _ => {
                world.commands().entity(this).insert(AspectRatio::new());
            }
        }
    }
}
#[derive(Component, Copy, Clone, PartialEq, Default)]
pub(crate) struct CropAdjustment {
    pub(crate) adjustments: Section<Numerical>,
}
impl Attachment for Image {
    fn attach(foliage: &mut Foliage) {
        foliage
            .world
            .insert_resource(RenderQueue::<Image, ImageWrite>::new());
        foliage
            .world
            .insert_resource(RenderQueue::<Image, ImageMemory>::new());
        foliage
            .world
            .insert_resource(RenderQueue::<Image, CropAdjustment>::new());
        foliage
            .diff
            .add_systems(Image::update.in_set(DiffMarkers::Finalize));
        foliage.remove_queue::<Image>();
        foliage.differential::<Image, Section<Logical>>();
        foliage.differential::<Image, ClipSection>();
        foliage.differential::<Image, BlendedOpacity>();
        foliage.differential::<Image, ResolvedElevation>();
    }
}
impl Image {
    pub const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    pub fn new(memory_id: MemoryId, key: AssetKey) -> Self {
        Self { memory_id, key }
    }
    pub fn memory<C: Into<Coordinates>>(id: MemoryId, coords: C) -> ImageMemory {
        ImageMemory {
            memory_id: id,
            extent: Area::from(coords),
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let value = *world.get::<Image>(this).unwrap();
        let write = if world
            .get_resource::<AssetLoader>()
            .unwrap()
            .assets
            .contains_key(&value.key)
        {
            let view = *world.get::<ImageView>(this).unwrap();
            let rgba_image = image::load_from_memory(
                world
                    .get_resource::<AssetLoader>()
                    .unwrap()
                    .assets
                    .get(&value.key)
                    .unwrap()
                    .data
                    .as_slice(),
            )
            .unwrap()
            .into_rgba8();
            let extent = Area::from((rgba_image.width(), rgba_image.height()));
            world
                .commands()
                .entity(this)
                .insert(ImageMetrics { extent })
                .insert(view);
            ImageWrite {
                image: value,
                data: rgba_image.to_vec(),
                extent,
            }
        } else {
            world
                .commands()
                .entity(this)
                .insert(AssetRetrieval::new(value.key))
                .observe(asset_retrieval(move |tree, entity, data| {
                    tree.entity(entity).insert(value);
                }));
            ImageWrite {
                image: value,
                data: vec![],
                extent: Default::default(),
            }
        };
        world
            .get_resource_mut::<RenderQueue<Image, ImageWrite>>()
            .unwrap()
            .queue
            .insert(this, write);
    }
    fn update(
        images: Query<
            (Entity, &ImageView, &ImageMetrics, &Section<Logical>),
            Or<(
                Changed<ImageView>,
                Changed<ImageMetrics>,
                Changed<Section<Logical>>,
            )>,
        >,
        layout: Res<Layout>,
        mut queue: ResMut<RenderQueue<Image, CropAdjustment>>,
    ) {
        for (entity, view, metrics, section) in images.iter() {
            match view {
                ImageView::Crop => {
                    let fitted = AspectRatio::new()
                        .xs(metrics.extent.width() / metrics.extent.height())
                        .fit(*section, *layout)
                        .unwrap();
                    if fitted != *section {
                        let x = (section.left() - fitted.left()) / fitted.width();
                        let y = (section.top() - fitted.top()) / fitted.height();
                        let w = (fitted.right() - section.right()) / fitted.width();
                        let h = (fitted.bottom() - section.bottom()) / fitted.height();
                        let adjustments = Section::numerical((x, y), (w, h));
                        queue.queue.insert(entity, CropAdjustment { adjustments });
                    }
                }
                _ => {}
            }
        }
    }
}
#[derive(Component, Copy, Clone, PartialEq, Default)]
pub struct ImageMetrics {
    pub extent: Area<Numerical>,
}
#[derive(Component, Clone, PartialEq)]
pub(crate) struct ImageWrite {
    pub(crate) image: Image,
    pub(crate) data: Vec<u8>,
    pub(crate) extent: Area<Numerical>,
}
#[derive(Component, Copy, Clone, Default)]
#[component(on_add = Self::on_add)]
pub struct ImageMemory {
    pub memory_id: MemoryId,
    pub extent: Area<Numerical>,
}
impl ImageMemory {
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let memory = *world.get::<ImageMemory>(this).unwrap();
        world
            .get_resource_mut::<RenderQueue<Image, ImageMemory>>()
            .unwrap()
            .queue
            .insert(this, memory);
    }
}
pub type MemoryId = i32;
