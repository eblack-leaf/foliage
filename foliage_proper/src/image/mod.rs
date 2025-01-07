mod pipeline;

use crate::ash::clip::ClipSection;
use crate::ash::differential::RenderQueue;
use crate::foliage::DiffMarkers;
use crate::grid::AspectRatio;
use crate::opacity::BlendedOpacity;
use crate::{
    Area, Attachment, Component, Coordinates, Foliage, Logical, Numerical, ResolvedElevation,
    Section,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::prelude::{Entity, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
use wgpu::TextureFormat;

#[derive(Component, Clone, PartialEq)]
#[component(on_insert = Self::on_insert)]
#[require(ImageView, CropAdjustment, ImageMetrics)]
pub struct Image {
    pub memory_id: MemoryId,
    pub data: Vec<u8>,
}
#[derive(Component, Copy, Clone, PartialEq, Default)]
pub enum ImageView {
    #[default]
    Aspect,
    Crop,
}
impl ImageView {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let value = *world.get::<ImageView>(this).unwrap();
        let metrics = *world.get::<ImageMetrics>(this).unwrap();
        match value {
            ImageView::Aspect => {
                world
                    .commands()
                    .entity(this)
                    .insert(AspectRatio::new().xs(metrics.extent.width() / metrics.extent.height()))
                    .insert(CropAdjustment::default());
            }
            ImageView::Crop => {
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
            .diff
            .add_systems(Image::update.in_set(DiffMarkers::Finalize));
        foliage.remove_queue::<Image>();
        foliage.differential::<Image, Section<Logical>>();
        foliage.differential::<Image, ClipSection>();
        foliage.differential::<Image, BlendedOpacity>();
        foliage.differential::<Image, ResolvedElevation>();
        foliage.differential::<Image, CropAdjustment>();
    }
}
impl Image {
    pub const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    pub fn new(memory_id: MemoryId, data: Vec<u8>) -> Self {
        Self { memory_id, data }
    }
    pub fn memory<C: Into<Coordinates>>(id: MemoryId, coords: C) -> ImageMemory {
        ImageMemory {
            memory_id: id,
            extent: Area::from(coords),
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let view = *world.get::<ImageView>(this).unwrap();
        let rgba_image = image::load_from_memory(world.get::<Image>(this).unwrap().data.as_slice())
            .unwrap()
            .into_rgba8();
        let extent = Area::from((rgba_image.width(), rgba_image.height()));
        world
            .commands()
            .entity(this)
            .insert(ImageMetrics { extent });
        let aspect = AspectRatio::new().xs(extent.width() / extent.height());
        match view {
            ImageView::Aspect => {
                world.commands().entity(this).insert(aspect);
            }
            ImageView::Crop => {}
        }
        let write = ImageWrite {
            image: Image::new(
                world.get::<Image>(this).unwrap().memory_id,
                rgba_image.to_vec(),
            ),
            extent,
        };
        world
            .get_resource_mut::<RenderQueue<Image, ImageWrite>>()
            .unwrap()
            .queue
            .insert(this, write);
    }
    fn update(
        mut images: Query<
            (
                &ImageView,
                &ImageMetrics,
                &Section<Logical>,
                &mut CropAdjustment,
            ),
            Or<(
                Changed<ImageView>,
                Changed<ImageMetrics>,
                Changed<Section<Logical>>,
            )>,
        >,
    ) {
        for (view, metrics, section, mut crop) in images.iter_mut() {
            match view {
                ImageView::Aspect => {
                    if crop.adjustments != Section::default() {
                        crop.adjustments = Section::default();
                    }
                }
                ImageView::Crop => {
                    let fitted = AspectRatio::new()
                        .xs(metrics.extent.width() / metrics.extent.height())
                        .fit(*section)
                        .unwrap();
                    if fitted != *section {
                        let x = (section.left() - fitted.left()) / fitted.width();
                        let y = (section.top() - fitted.top()) / fitted.height();
                        let w = (fitted.right() - section.right()) / fitted.width();
                        let h = (fitted.bottom() - section.bottom()) / fitted.height();
                        let adjustments = Section::numerical((x, y), (w, h));
                        *crop = CropAdjustment { adjustments };
                    }
                }
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
