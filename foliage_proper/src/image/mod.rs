mod pipeline;

use crate::AsTree;
use crate::AssetKey;
use crate::Trigger;
use crate::ash::clip::ClipContext;
use crate::ash::differential::RenderQueue;
use crate::asset::AssetRetrieval;
use crate::asset::{AssetLoader, OnRetrieval};
use crate::foliage::DiffMarkers;
use crate::ginkgo::ScaleFactor;
use crate::grid::AspectRatio;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::rounding::CornerRadii;
use crate::{
    Area, Attachment, Author, Component, Foliage, Layout, LeafSprout, Logical, Numerical, Parent,
    Resolved, ResolvedElevation, ResolvedVisibility, Rounding, Section, Side,
};
use crate::{Differential, Tree, Visibility};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::{IntoScheduleConfigs, Res};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
use wgpu::TextureFormat;

/// Points at asset bytes; the GPU texture identity is derived from `key` internally (see
/// `image::pipeline::Resources::group_for`) -- there is no separate memory id to hand-assign
/// or keep in sync with anything.
#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
#[require(ImageView, ImageMetrics, Rounding, Side)]
#[require(Differential<Image, Section<Logical>>)]
#[require(Differential<Image, BlendedOpacity>)]
#[require(Differential<Image, ResolvedElevation>)]
#[require(Differential<Image, ClipContext>)]
#[require(CropAdjustment, Differential<Image, CropAdjustment>)]
#[require(CornerRadii, Differential<Image, CornerRadii>)]
pub struct Image {
    pub key: AssetKey,
}
/// How an image's own pixels are fitted into the box its `Location` resolved to.
#[derive(Component, Copy, Clone, PartialEq, Default)]
#[component(on_insert = Self::on_insert)]
pub enum ImageView {
    /// Scale to fit inside the box, preserving the image's aspect ratio and leaving the
    /// remainder of the box empty.
    #[default]
    Aspect,
    /// Fill the box, preserving aspect ratio and cropping the overflowing axis.
    Crop,
    /// Fill the box exactly, distorting the image where the ratios disagree.
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
                    world.tree().write_to(this, ratio);
                }
                world.tree().write_to(this, CropAdjustment::default());
            }
            ImageView::Stretch => {
                world.tree().write_to(this, AspectRatio::new());
                world.tree().write_to(this, CropAdjustment::default());
            }
            _ => {
                world.tree().write_to(this, AspectRatio::new());
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
        foliage.differential::<Image, CornerRadii>();
    }
}
impl Image {
    /// Pixel format every image is decoded into before upload.
    pub const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    /// Starts an [`Image`] entity drawing the asset behind `key`. The key is valid
    /// immediately; the image appears once the bytes load and decode.
    pub fn new(key: AssetKey) -> ImageSprout {
        ImageSprout {
            leaf: LeafSprout::default(),
            key,
            view: None,
            rounding: None,
            side: None,
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
                    tree.write_to(trigger.event_target(), *img);
                }
            }
        }
    }
    /// Fires once the pending asset fetch resolves (wasm) -- re-inserting `Image` re-runs
    /// `on_insert`, which now finds real bytes and finally allocates+uploads the texture.
    /// Until this point, `Section<Logical>`/`ResolvedElevation`/`Parent`/`BlendedOpacity`
    /// writes that already fired at spawn time were silently dropped (nothing in
    /// `entity_to_memory` to route them to yet), so re-fire each so the renderer catches up
    /// with wherever this entity already ended up.
    fn retrieve_img(trigger: Trigger<OnRetrieval>, mut tree: Tree, images: Query<&Image>) {
        let this = trigger.event_target();
        if let Ok(img) = images.get(this) {
            tree.write_to(this, *img);
            tree.refire::<(Section<Logical>,)>(this);
            tree.refire::<(ResolvedElevation,)>(this);
            tree.refire::<(Parent,)>(this);
            tree.refire::<(BlendedOpacity,)>(this);
            tree.refire::<(CornerRadii,)>(this);
        }
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let mut tree = world.tree();
        tree.subscribe(this, Self::retrieve_img);
        tree.subscribe(this, Self::visibility_trigger);
        tree.subscribe(this, Visibility::push_remove_packet::<Self>);
        tree.subscribe(this, Remove::push_remove_packet::<Self>);
    }
    /// Decodes once bytes are available (native: always; wasm: once the async fetch
    /// resolves) and pushes dimensions and pixel data together, so a size is never
    /// declared separately from the data it describes and no texture exists unfilled.
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let value = *world.get::<Image>(this).unwrap();
        let Some(asset) = world
            .get_resource::<AssetLoader>()
            .unwrap()
            .retrieve(value.key)
        else {
            world.tree().write_to(this, AssetRetrieval::new(value.key));
            return;
        };
        let view = *world.get::<ImageView>(this).unwrap();
        let rgba_image = image::load_from_memory(asset.data.as_slice())
            .unwrap()
            .into_rgba8();
        let extent = Area::from((rgba_image.width(), rgba_image.height()));
        world.tree().write_to(this, (ImageMetrics { extent }, view));
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
                &Rounding,
                &Side,
                &mut CropAdjustment,
                &mut CornerRadii,
            ),
            Or<(
                Changed<ImageView>,
                Changed<ImageMetrics>,
                Changed<Section<Logical>>,
                Changed<Rounding>,
                Changed<Side>,
            )>,
        >,
        layout: Res<Layout>,
        scale_factor: Res<ScaleFactor>,
    ) {
        // direct Query mutation, NOT commands: this runs at Finalize and the differential
        // senders run at Extract in the same frame -- crop must ship in the same frame as
        // the Section that caused it, or resize/scroll shows a frame of wrong crop
        for (view, metrics, section, rounding, side, mut crop, mut radii) in images.iter_mut() {
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
            let resolved = CornerRadii::resolve(
                // Unrounded, matching what `image/pipeline.rs` hands the shader as the
                // section -- the radii have to be measured against the same box.
                section.to_physical(scale_factor.value()),
                *rounding,
                *side,
                scale_factor.value(),
            );
            if *radii != resolved {
                *radii = resolved;
            }
        }
    }
}
/// Builder for an [`Image`] entity -- see [`Image::new`].
pub struct ImageSprout {
    leaf: LeafSprout,
    key: AssetKey,
    view: Option<ImageView>,
    rounding: Option<Rounding>,
    side: Option<Side>,
}
impl Author for ImageSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Image::new_marker(self.key),
            self.view.unwrap_or_default(),
            self.rounding.unwrap_or_default(),
            self.side.unwrap_or_default(),
        )
    }
}
impl ImageSprout {
    /// How the pixels fit the box. [`ImageView::Aspect`] by default.
    pub fn view(mut self, v: ImageView) -> Self {
        self.view = Some(v);
        self
    }
    /// Corner radius bracket, resolved exactly as [`Panel`](crate::Panel) resolves its own
    /// -- so `Image::new(k).view(ImageView::Crop).rounding(r)` on a panel of the same box
    /// and the same `r` is a full-bleed image whose curve matches the panel's.
    pub fn rounding(mut self, r: Rounding) -> Self {
        self.rounding = Some(r);
        self
    }
    /// Restricts [`rounding`](Self::rounding) to particular corners. Defaults to all four.
    pub fn side(mut self, s: Side) -> Self {
        self.side = Some(s);
        self
    }
}
/// The decoded image's own pixel dimensions, written once the asset resolves. Read it to
/// size a box to the real image rather than guessing.
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
