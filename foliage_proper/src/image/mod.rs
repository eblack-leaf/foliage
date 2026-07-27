mod pipeline;

use crate::Trigger;
use crate::ash::clip::ClipContext;
use crate::ash::differential::RenderQueue;
use crate::asset::{AssetLoader, OnRetrieval};
use crate::foliage::DiffMarkers;
use crate::grid::AspectRatio;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::{
    Area, Attachment, Component, Foliage, Layout, LeafSprout, Logical, Numerical,
    ResolvedElevation, ResolvedVisibility, Section, Sprout, Stem, Resolved,
};
use crate::{AssetKey, AssetRetrieval};
use crate::{Differential, EcsExtension, Tree, Visibility};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::ComponentId;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::{Entity, IntoScheduleConfigs, Res};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Query, ResMut};
use bevy_ecs::world::DeferredWorld;
use wgpu::TextureFormat;

/// Points at asset bytes; the GPU texture identity is derived from `key` internally (see
/// `image::pipeline::Resources::group_for`) -- there is no separate memory id to hand-assign
/// or keep in sync with anything.
#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
#[require(ImageView, ImageMetrics)]
#[require(Differential<Image, Section<Logical>>)]
#[require(Differential<Image, BlendedOpacity>)]
#[require(Differential<Image, ResolvedElevation>)]
#[require(Differential<Image, ClipContext>)]
#[require(CropAdjustment, Differential<Image, CropAdjustment>)]
pub struct Image {
    pub key: AssetKey,
}
#[derive(Component, Copy, Clone, PartialEq, Default)]
#[component(on_insert = Self::on_insert)]
pub enum ImageView {
    #[default]
    Aspect,
    Crop,
    Stretch,
}
impl ImageView {
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
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
                    .commands()
                    .entity(this)
                    .insert(CropAdjustment::default());
            }
            ImageView::Stretch => {
                world.commands().entity(this).insert(AspectRatio::new());
                world
                    .commands()
                    .entity(this)
                    .insert(CropAdjustment::default());
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
            .diff
            .add_systems(Image::update.in_set(DiffMarkers::Finalize));
        foliage.remove_queue::<Image>();
        foliage.differential::<Image, Section<Logical>>();
        foliage.differential::<Image, ClipContext>();
        foliage.differential::<Image, BlendedOpacity>();
        foliage.differential::<Image, ResolvedElevation>();
        foliage.differential::<Image, CropAdjustment>();
    }
}
impl Image {
    pub const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    pub fn new(key: AssetKey) -> ImageSprout {
        ImageSprout {
            leaf: LeafSprout::default(),
            key,
            view: None,
        }
    }
    pub(crate) fn new_marker(key: AssetKey) -> Self {
        Self { key }
    }
    fn visibility_trigger(
        trigger: Trigger<Resolved<Visibility>>,
        images: Query<&Image>,
        mut tree: Tree,
        vis: Query<&ResolvedVisibility>,
    ) {
        if let Ok(img) = images.get(trigger.event_target()) {
            if let Ok(v) = vis.get(trigger.event_target()) {
                if v.visible() {
                    tree.entity(trigger.event_target()).insert(*img);
                }
            }
        }
    }
    /// Fires once the pending asset fetch resolves (wasm) -- re-inserting `Image` re-runs
    /// `on_insert`, which now finds real bytes and finally allocates+uploads the texture.
    /// Until this point, `Section<Logical>`/`ResolvedElevation`/`Stem`/`BlendedOpacity`
    /// writes that already fired at spawn time were silently dropped (nothing in
    /// `entity_to_memory` to route them to yet), so re-fire each so the renderer catches up
    /// with wherever this entity already ended up.
    fn retrieve_img(trigger: Trigger<OnRetrieval>, mut tree: Tree, images: Query<&Image>) {
        let this = trigger.event_target();
        if let Ok(img) = images.get(this) {
            tree.entity(this).insert(*img);
            tree.refire::<(Section<Logical>,)>(this);
            tree.refire::<(ResolvedElevation,)>(this);
            tree.refire::<(Stem,)>(this);
            tree.refire::<(BlendedOpacity,)>(this);
        }
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Self::retrieve_img)
            .observe(Self::visibility_trigger)
            .observe(Visibility::push_remove_packet::<Self>)
            .observe(Remove::push_remove_packet::<Self>);
    }
    /// Decodes once bytes are available (native: always; wasm: once the async fetch
    /// resolves) and pushes real dimensions + pixel data together -- there's no longer a
    /// separately-declared size to keep in sync, and no window where a texture exists
    /// without real data in it.
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let value = *world.get::<Image>(this).unwrap();
        let Some(asset) = world
            .get_resource::<AssetLoader>()
            .unwrap()
            .retrieve(value.key)
        else {
            world
                .commands()
                .entity(this)
                .insert(AssetRetrieval::new(value.key));
            return;
        };
        let view = *world.get::<ImageView>(this).unwrap();
        let rgba_image = image::load_from_memory(asset.data.as_slice())
            .unwrap()
            .into_rgba8();
        let extent = Area::from((rgba_image.width(), rgba_image.height()));
        world
            .commands()
            .entity(this)
            .insert(ImageMetrics { extent })
            .insert(view);
        world
            .get_resource_mut::<RenderQueue<Image, ImageWrite>>()
            .unwrap()
            .queue
            .insert(
                this,
                ImageWrite {
                    key: value.key,
                    data: rgba_image.to_vec(),
                    extent,
                },
            );
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
        layout: Res<Layout>,
    ) {
        // direct Query mutation, NOT commands: this runs at Finalize and the differential
        // senders run at Extract in the same frame -- crop must ship in the same frame as
        // the Section that caused it, or resize/scroll shows a frame of wrong crop
        for (view, metrics, section, mut crop) in images.iter_mut() {
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
                        *crop = CropAdjustment { adjustments };
                    } else {
                        *crop = CropAdjustment::default();
                    }
                }
                _ => {}
            }
        }
    }
}
pub struct ImageSprout {
    leaf: LeafSprout,
    key: AssetKey,
    view: Option<ImageView>,
}
impl Sprout for ImageSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (Image::new_marker(self.key), self.view.unwrap_or_default())
    }
}
impl ImageSprout {
    pub fn view(mut self, v: ImageView) -> Self {
        self.view = Some(v);
        self
    }
}
#[derive(Component, Copy, Clone, PartialEq, Default)]
pub struct ImageMetrics {
    pub extent: Area<Numerical>,
}
#[derive(Clone, PartialEq)]
pub(crate) struct ImageWrite {
    pub(crate) key: AssetKey,
    pub(crate) data: Vec<u8>,
    pub(crate) extent: Area<Numerical>,
}
