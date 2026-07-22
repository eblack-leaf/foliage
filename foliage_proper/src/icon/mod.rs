mod pipeline;
use crate::Trigger;
use crate::ash::clip::ClipContext;
use crate::ash::differential::RenderQueue;
use crate::asset::{AssetLoader, AssetRetrieval, OnRetrieval};
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::grid::AspectRatio;
use crate::{
    AssetKey, Attachment, Color, Component, Differential, Foliage, LeafSprout, Logical,
    ResolvedElevation, Section, Sprout, Tree, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, Res, ResMut};
use bevy_ecs::world::DeferredWorld;

pub type IconId = i32;
#[derive(Component, Copy, Clone, PartialEq, Default)]
#[component(on_add = Self::on_add)]
#[require(Color, Differential<Icon, Color>)]
#[require(Differential<Icon, ClipContext>)]
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
        foliage.remove_queue::<Icon>();
        foliage.differential::<Icon, Icon>();
        foliage.differential::<Icon, Section<Logical>>();
        foliage.differential::<Icon, ClipContext>();
        foliage.differential::<Icon, ResolvedElevation>();
        foliage.differential::<Icon, Color>();
        foliage.differential::<Icon, BlendedOpacity>();
    }
}
impl Icon {
    pub fn new<ID: Into<IconId>>(id: ID) -> IconSprout {
        IconSprout {
            id: id.into(),
            ..Default::default()
        }
    }
    pub(crate) fn new_marker<ID: Into<IconId>>(id: ID) -> Self {
        Self { id: id.into() }
    }
    /// Registers a resolution-independent MTSDF field (`bytes` = a `field_size`×`field_size`
    /// RGBA field baked by `foliage_icons`) for `mem`. One field serves every on-screen size;
    /// an icon renders at whatever its `Location` box resolves to, exactly like `Panel`.
    pub fn msdf<ID: Into<IconId>, M: AsRef<[u8]>>(
        mem: ID,
        bytes: M,
        field_size: u32,
        px_range: f32,
    ) -> IconMemory {
        IconMemory {
            id: mem.into(),
            source: IconSource::Ready(bytes.as_ref().to_vec()),
            field_size,
            px_range,
        }
    }
    /// Same as [`Icon::msdf`], but sources the field bytes from `key` via the same
    /// `AssetLoader` path [`crate::Image`] uses (baked on native, fetched async on wasm) --
    /// for consumers who don't want every icon field baked into the wasm binary unconditionally.
    pub fn msdf_from_asset<ID: Into<IconId>>(
        mem: ID,
        key: AssetKey,
        field_size: u32,
        px_range: f32,
    ) -> IconMemory {
        IconMemory {
            id: mem.into(),
            source: IconSource::Pending(key),
            field_size,
            px_range,
        }
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Visibility::push_remove_packet::<Self>)
            .observe(Remove::push_remove_packet::<Self>);
    }
    /// An icon's public value channel: write `IconValue` to an icon entity and the glyph
    /// follows -- the render marker stays private. Entities that carry `IconValue` as mere
    /// config (a Button root) are skipped by the `With<Icon>` filter.
    fn apply_icon_value(
        trigger: Trigger<bevy_ecs::lifecycle::Insert, crate::IconValue>,
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
        // Icons are square (their MTSDF fields are square), so constrain to a 1:1 aspect: the
        // glyph fits centered within whatever `Location` box it's given, never distorting even
        // in a non-square box. This is what lets composites size an icon box loosely (a
        // percentage of their own geometry) without worrying about the box's aspect.
        (
            Icon::new_marker(self.id),
            self.color.unwrap_or_default(),
            AspectRatio::new().xs(1.0),
        )
    }
}
impl IconSprout {
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
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
    /// Square MTSDF field resolution (texels per side).
    pub field_size: u32,
    /// Distance-field spread in texels -- feeds the shader's screen-space edge width.
    pub px_range: f32,
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
            let resolved = IconMemory {
                source: IconSource::Ready(bytes),
                ..value
            };
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
    ) {
        let this = trigger.event_target();
        if let Ok(mem) = memories.get(this) {
            if let Some(asset) = loader.retrieve(trigger.event().key) {
                let resolved = IconMemory {
                    source: IconSource::Ready(asset.data),
                    ..mem.clone()
                };
                queue.queue.insert(this, resolved);
            }
        }
        tree.entity(this).despawn();
    }
}
