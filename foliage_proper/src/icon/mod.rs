mod pipeline;
use crate::ash::differential::RenderQueue;
use crate::asset::{AssetLoader, AssetRetrieval, OnRetrieval};
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::Stem;
use crate::Trigger;
use crate::{
    AssetKey, Attachment, Color, Component, Coordinates, Differential, Foliage,
    HorizontalAlignment, LeafSprout, Logical, ResolvedElevation, Section, Sprout, Tree,
    VerticalAlignment, Visibility, Write,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::{HookContext, Insert};
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Query, Res, ResMut};
use bevy_ecs::world::DeferredWorld;
use std::collections::HashMap;

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
        foliage.define(Icon::apply_icon_value);
        foliage
            .world
            .insert_resource(RenderQueue::<Icon, IconMemory>::new());
        foliage.world.insert_resource(IconRenderSizes::default());
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
    /// Default logical render size -- the fallback when an `IconId` has no registered
    /// [`IconMemory`] yet, and the value [`Icon::memory`]'s two-arg constructor uses.
    pub const SCALE: Coordinates = Coordinates::new(24f32, 24f32);
    /// Default texture scale (the largest/mip-0 level) -- matches [`Icon::SCALE`] at
    /// [`Icon::MIP_COUNT`] mip levels: `24 << (3-1) == 96`.
    pub const TEXTURE_SCALE: Coordinates = Coordinates::new(96f32, 96f32);
    pub const MIP_COUNT: u32 = 3;
    pub fn new<ID: Into<IconId>>(id: ID) -> IconSprout {
        IconSprout {
            id: id.into(),
            ..Default::default()
        }
    }
    pub(crate) fn new_marker<ID: Into<IconId>>(id: ID) -> Self {
        Self { id: id.into() }
    }
    /// Registers `bytes` at [`Icon::TEXTURE_SCALE`]/[`Icon::SCALE`]/[`Icon::MIP_COUNT`] --
    /// today's fixed shape. Use [`Icon::memory_sized`] for any other bucket configuration.
    pub fn memory<ID: Into<IconId>, M: AsRef<[u8]>>(mem: ID, bytes: M) -> IconMemory {
        IconMemory {
            id: mem.into(),
            source: IconSource::Ready(bytes.as_ref().to_vec()),
            texture_scale: Self::TEXTURE_SCALE,
            render_size: Self::SCALE,
            mip_count: Self::MIP_COUNT,
        }
    }
    /// Registers `bytes` (a flat, largest-mip-first byte buffer -- see `foliage_icons`) at an
    /// explicit texture/render shape, for icon sets generated at other than the default
    /// 24px/96px/3-mip configuration.
    pub fn memory_sized<ID: Into<IconId>, M: AsRef<[u8]>>(
        mem: ID,
        bytes: M,
        texture_scale: Coordinates,
        render_size: Coordinates,
        mip_count: u32,
    ) -> IconMemory {
        IconMemory {
            id: mem.into(),
            source: IconSource::Ready(bytes.as_ref().to_vec()),
            texture_scale,
            render_size,
            mip_count,
        }
    }
    /// Same as [`Icon::memory_sized`], but sources bytes from `key` via the same
    /// `load_asset!`/`AssetLoader` path [`crate::Image`] already uses (baked on native,
    /// fetched async on wasm) instead of requiring them synchronously available up front --
    /// for consumers with large/custom icon sets who don't want every icon baked into the
    /// wasm binary unconditionally.
    pub fn memory_from_asset<ID: Into<IconId>>(
        mem: ID,
        key: AssetKey,
        texture_scale: Coordinates,
        render_size: Coordinates,
        mip_count: u32,
    ) -> IconMemory {
        IconMemory {
            id: mem.into(),
            source: IconSource::Pending(key),
            texture_scale,
            render_size,
            mip_count,
        }
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Visibility::push_remove_packet::<Self>)
            .observe(Remove::push_remove_packet::<Self>)
            .observe(Self::align_render_size);
    }
    /// An icon's public value channel: write `IconValue` to an icon entity and the glyph
    /// follows -- the render marker stays private. Entities that carry `IconValue` as mere
    /// config (a Button root) are skipped by the `With<Icon>` filter.
    fn apply_icon_value(
        trigger: Trigger<Insert, crate::IconValue>,
        values: Query<&crate::IconValue>,
        icons: Query<(), With<Icon>>,
        mut tree: crate::Tree,
    ) {
        let this = trigger.event_target();
        if icons.contains(this) {
            if let Ok(value) = values.get(this) {
                tree.entity(this).insert(Icon::new_marker(value.0));
            }
        }
    }
    /// An icon's on-screen footprint is fixed by its backing texture (ahead-of-time
    /// rasterization -- see `foliage_icons`), never by the layout system. Its `Location`
    /// box is therefore an ALIGNMENT REGION, the same model `Text` uses: the true-size
    /// footprint (registered [`IconMemory::render_size`], [`Icon::SCALE`] fallback) is
    /// placed within the resolved box per `HorizontalAlignment`/`VerticalAlignment` --
    /// Center/Middle by default for icons (inserted by `IconSprout`; `Default` on those
    /// enums is text's Left/Top). A box exactly the footprint's size aligns to itself, so
    /// exact-size spawns are untouched.
    ///
    /// Idempotent across re-fires: once the section IS the footprint, every alignment
    /// produces a zero shift; a genuine relayout resolves a fresh box from `Location`
    /// first, and this re-places within it.
    fn align_render_size(
        trigger: Trigger<Write<Section<Logical>>>,
        mut sections: Query<(
            &mut Section<Logical>,
            &Icon,
            Option<&HorizontalAlignment>,
            Option<&VerticalAlignment>,
        )>,
        render_sizes: Res<IconRenderSizes>,
    ) {
        if let Ok((mut sec, icon, horizontal, vertical)) =
            sections.get_mut(trigger.event_target())
        {
            let target = render_sizes.get(icon.id);
            if sec.area.coordinates != target {
                let slack = (sec.width() - target.a(), sec.height() - target.b());
                // absent components read as an icon's natural alignment (centered), NOT the
                // enums' `Default` (Left/Top -- that's text's prose default)
                let dx = match horizontal.copied().unwrap_or(HorizontalAlignment::Center) {
                    HorizontalAlignment::Left => 0.0,
                    HorizontalAlignment::Center => slack.0 * 0.5,
                    HorizontalAlignment::Right => slack.0,
                };
                let dy = match vertical.copied().unwrap_or(VerticalAlignment::Middle) {
                    VerticalAlignment::Top => 0.0,
                    VerticalAlignment::Middle => slack.1 * 0.5,
                    VerticalAlignment::Bottom => slack.1,
                };
                sec.position += (dx, dy).into();
                sec.area.set_width(target.a());
                sec.area.set_height(target.b());
            }
        }
    }
}
#[derive(Default)]
pub struct IconSprout {
    leaf: LeafSprout,
    id: IconId,
    color: Option<Color>,
}
impl Sprout for IconSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (Icon::new_marker(self.id), self.color.unwrap_or_default())
    }
}
impl IconSprout {
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}
/// `IconId -> render size`, populated as each [`IconMemory`] resolves -- what
/// [`Icon::clamp_render_size`] looks up instead of a single global constant.
///
/// Public because anyone composing icons into a widget needs it: an icon renders at exactly
/// its registered footprint regardless of the `Location` box it's given (the clamp keeps
/// the box's top-left and snaps the size), so a box that isn't exactly [`Self::get`]'s
/// answer puts the icon visibly off-center in it. Composites -- library and custom sprouts
/// alike -- read this to size icon boxes correctly instead of hardcoding a design size.
#[derive(Resource, Default)]
pub struct IconRenderSizes(pub(crate) HashMap<IconId, Coordinates>);
impl IconRenderSizes {
    /// The on-screen footprint `id` will actually render at ([`Icon::SCALE`] until/unless
    /// a memory is registered for it).
    pub fn get<ID: Into<IconId>>(&self, id: ID) -> Coordinates {
        self.0.get(&id.into()).copied().unwrap_or(Icon::SCALE)
    }
}

#[derive(Clone)]
pub(crate) enum IconSource {
    Ready(Vec<u8>),
    Pending(AssetKey),
}
impl Default for IconSource {
    fn default() -> Self {
        IconSource::Ready(Vec::new())
    }
}

#[derive(Component, Clone, Default)]
#[component(on_add = Self::on_add)]
pub struct IconMemory {
    pub id: IconId,
    pub(crate) source: IconSource,
    pub texture_scale: Coordinates,
    pub render_size: Coordinates,
    pub mip_count: u32,
}
impl IconMemory {
    /// Bytes are always resolved (`IconSource::Ready`) by the time an `IconMemory` reaches
    /// the render queue -- `on_add`/`on_retrieved` only ever push a resolved copy.
    pub(crate) fn resolved_bytes(&self) -> &[u8] {
        match &self.source {
            IconSource::Ready(bytes) => bytes.as_slice(),
            IconSource::Pending(_) => {
                unreachable!("IconMemory reaches the render queue only once bytes are resolved")
            }
        }
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let value = world.get::<IconMemory>(this).unwrap().clone();
        let ready = match &value.source {
            IconSource::Ready(bytes) => Some(bytes.clone()),
            IconSource::Pending(key) => world
                .get_resource::<AssetLoader>()
                .unwrap()
                .retrieve(*key)
                .map(|asset| asset.data),
        };
        if let Some(bytes) = ready {
            let id = value.id;
            let render_size = value.render_size;
            let resolved = IconMemory {
                source: IconSource::Ready(bytes),
                ..value
            };
            world
                .get_resource_mut::<IconRenderSizes>()
                .unwrap()
                .0
                .insert(id, render_size);
            world
                .get_resource_mut::<RenderQueue<Icon, IconMemory>>()
                .unwrap()
                .queue
                .insert(this, resolved);
            world.commands().entity(this).despawn();
        } else if let IconSource::Pending(key) = value.source {
            world
                .commands()
                .entity(this)
                .insert(AssetRetrieval::new(key))
                .observe(Self::on_retrieved);
        }
    }
    fn on_retrieved(
        trigger: Trigger<OnRetrieval>,
        mut tree: Tree,
        loader: Res<AssetLoader>,
        memories: Query<&IconMemory>,
        mut queue: ResMut<RenderQueue<Icon, IconMemory>>,
        mut sizes: ResMut<IconRenderSizes>,
    ) {
        let this = trigger.event_target();
        if let Ok(mem) = memories.get(this) {
            if let Some(asset) = loader.retrieve(trigger.event().key) {
                let resolved = IconMemory {
                    source: IconSource::Ready(asset.data),
                    ..mem.clone()
                };
                sizes.0.insert(resolved.id, resolved.render_size);
                queue.queue.insert(this, resolved);
            }
        }
        tree.entity(this).despawn();
    }
}
