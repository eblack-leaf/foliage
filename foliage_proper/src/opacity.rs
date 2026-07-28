use crate::Trigger;
use crate::anim::interpolation::Interpolations;
use crate::{Animate, Attachment, Branch, Component, Foliage, Stem, Tree};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
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
/// Multiplies down the `Stem` chain: a child at `0.5` inside a parent at `0.5` renders at
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
        trigger: Trigger<Insert, Stem>,
        mut tree: Tree,
        stems: Query<&Stem>,
        blended: Query<&BlendedOpacity>,
    ) {
        let this = trigger.event_target();
        let stem = stems.get(this).unwrap();
        if let Some(entity) = stem.id {
            let resolved = *blended.get(entity).unwrap();
            tree.entity(this).insert(InheritedOpacity {
                value: resolved.value,
            });
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let inherited = world.get::<InheritedOpacity>(this).unwrap();
        let current = world.get::<Opacity>(this).unwrap();
        let blended = BlendedOpacity::new(inherited.value * current.value);
        world.commands().entity(this).insert(blended);
        let deps = world.get::<Branch>(this).unwrap().ids.clone();
        for d in deps.iter() {
            world
                .commands()
                .entity(*d)
                .insert(InheritedOpacity::new(blended.value));
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
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}
impl Default for BlendedOpacity {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EcsExtension, Elevation, Entity, Foliage, Leaf, Location, Sprout};

    fn blended_of(foliage: &mut Foliage, entity: Entity) -> f32 {
        foliage.world.get::<BlendedOpacity>(entity).unwrap().value
    }

    #[test]
    fn a_bare_leaf_defaults_to_fully_opaque() {
        let mut foliage = Foliage::new();
        let leaf = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
        foliage.world.flush();
        assert_eq!(blended_of(&mut foliage, leaf), 1.0);
    }

    #[test]
    fn a_childs_blended_opacity_multiplies_through_its_parent() {
        let mut foliage = Foliage::new();
        let parent = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .with(Opacity::new(0.5)),
        );
        let child = foliage.world.branch(
            parent,
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .with(Opacity::new(0.5)),
        );
        foliage.world.flush();
        assert_eq!(blended_of(&mut foliage, parent), 0.5);
        assert_eq!(
            blended_of(&mut foliage, child),
            0.25,
            "0.5 (parent) * 0.5 (own) -- not 0.5 and not 1.0"
        );
    }

    #[test]
    fn a_parents_opacity_change_propagates_down_to_an_already_spawned_child() {
        let mut foliage = Foliage::new();
        let parent = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
        let child = foliage.world.branch(
            parent,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        foliage.world.flush();
        assert_eq!(
            blended_of(&mut foliage, child),
            1.0,
            "sanity: fully opaque before any change"
        );

        foliage.write_to(parent, Opacity::new(0.4));
        foliage.world.flush();
        assert_eq!(
            blended_of(&mut foliage, child),
            0.4,
            "the child never had its own Opacity written -- this is purely the parent's \
             on_insert hook pushing a new blended value down to an existing child, not \
             just a freshly-spawned one inheriting it"
        );
    }
}
