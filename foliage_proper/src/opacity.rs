use crate::AsTree;
use crate::Trigger;
use crate::anim::interpolation::Interpolations;
use crate::{Animate, Attachment, Children, Component, Foliage, Parent, Tree};
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_insert = Opacity::on_insert)]
#[require(InheritedOpacity, BlendedOpacity)]
/// How transparent an entity and its subtree are, `0.0` fully transparent through `1.0`
/// fully opaque.
///
/// Multiplies down the `Parent` chain: a child at `0.5` inside a parent at `0.5` renders at
/// `0.25`, so fading a container fades everything in it as one. The product the renderer
/// reads is [`BlendedOpacity`].
///
/// Animatable, and the usual way to fade UI in and out. Distinct from a color's own alpha
/// ([`Color::with_opacity`](crate::Color::with_opacity)), which is a fixed property of
/// one color and does not inherit. Fully transparent entities are still drawn and still
/// hit-test -- use [`Visibility`](crate::Visibility) to actually take one out.
pub struct Opacity {
    pub value: f32,
}
impl Attachment for Opacity {
    fn attach(foliage: &mut Foliage) {
        foliage.enable_animation::<Self>();
        foliage.define(Opacity::stem_insert);
    }
}
impl Opacity {
    /// `0.0` fully transparent, `1.0` fully opaque. Values are not clamped here.
    pub fn new(value: f32) -> Opacity {
        Opacity { value }
    }
    fn stem_insert(
        trigger: Trigger<Insert, Parent>,
        mut tree: Tree,
        stems: Query<&Parent>,
        blended: Query<&BlendedOpacity>,
    ) {
        let this = trigger.event_target();
        let stem = stems.get(this).unwrap();
        if let Some(entity) = stem.id {
            let resolved = *blended.get(entity).unwrap();
            tree.write_to(
                this,
                InheritedOpacity {
                    value: resolved.value,
                },
            );
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let inherited = world.get::<InheritedOpacity>(this).unwrap();
        let current = world.get::<Opacity>(this).unwrap();
        let blended = BlendedOpacity::new(inherited.value * current.value);
        world.tree().write_to(this, blended);
        let deps = world.get::<Children>(this).unwrap().ids.clone();
        for d in deps.iter() {
            world
                .tree()
                .write_to(*d, InheritedOpacity::new(blended.value));
        }
    }
}
impl Animate for Opacity {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new().with(start.value, end.value)
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(o) = interpolations.read(0) {
            self.value = o;
        }
    }
}
impl Default for Opacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}
/// The product of every ancestor's [`Opacity`], excluding this entity's own. Maintained
/// by the engine as the tree changes.
#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_insert = Opacity::on_insert)]
pub struct InheritedOpacity {
    pub value: f32,
}
impl InheritedOpacity {
    /// Set by the engine as the tree changes; not something to write directly.
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}
impl Default for InheritedOpacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}
/// This entity's own [`Opacity`] times [`InheritedOpacity`] -- the single alpha the
/// renderer multiplies into every fragment. Read-only; write [`Opacity`] to change it.
#[repr(C)]
#[derive(Component, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlendedOpacity {
    pub value: f32,
}
impl BlendedOpacity {
    /// Set by the engine from [`Opacity`] and [`InheritedOpacity`]; not something to
    /// write directly.
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}
impl Default for BlendedOpacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}
