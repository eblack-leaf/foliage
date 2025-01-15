use crate::{Branch, Component, Logical, Section, Tree, Update, Write};
use crate::{Differential, Stem};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::Query;
use bevy_ecs::world::{DeferredWorld, OnInsert};
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
#[require(InheritedClip, ResolvedClip, Differential<(), ResolvedClip>)]
// #[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
pub(crate) struct ClipSection(pub(crate) Section<Logical>);
impl ClipSection {
    pub(crate) fn write_section(
        trigger: Trigger<Write<Section<Logical>>>,
        sections: Query<&Section<Logical>>,
        mut tree: Tree,
    ) {
        // trigger on-insert w/ current section
        let value = *sections.get(trigger.entity()).unwrap();
        tree.entity(trigger.entity()).insert(ClipSection(value));
    }
    pub(crate) fn stem_insert(trigger: Trigger<OnInsert, Stem>, mut tree: Tree) {
        tree.trigger_targets(Update::<InheritedClip>::new(), trigger.entity());
    }
    pub(crate) fn update_inherited(
        trigger: Trigger<Update<InheritedClip>>,
        mut tree: Tree,
        stems: Query<&Stem>,
        sections: Query<&Section<Logical>>,
    ) {
        // calculate all upward in tree
        let this = trigger.entity();
        let mut stem = *stems.get(this).unwrap();
        let mut section = *sections.get(this).unwrap();
        let mut inherited = InheritedClip(None);
        while stem.id.is_some() {
            let id = stem.id.unwrap();
            let next = *sections.get(id).unwrap();
            let blended = next.intersection(section).unwrap_or_default();
            if inherited.0.is_none() {
                inherited.0.replace(next);
            } else {
                inherited.0.replace(
                    blended
                        .intersection(inherited.0.unwrap())
                        .unwrap_or_default(),
                );
            }
            stem = *stems.get(id).unwrap();
            section = next;
        }
        tree.entity(this).insert(inherited);
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        // set deps inherited from resolved
        let inherited = *world.get::<InheritedClip>(this).unwrap();
        let current = *world.get::<Section<Logical>>(this).unwrap();
        let resolved = if let Some(i) = inherited.0 {
            ResolvedClip(i.intersection(current).unwrap_or_default())
        } else {
            ResolvedClip(current)
        };
        world.commands().entity(this).insert(resolved);
        let deps = world.get::<Branch>(this).unwrap().ids.clone();
        for d in deps {
            world
                .commands()
                .entity(d)
                .insert(InheritedClip(Some(resolved.0)));
        }
    }
}
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
#[component(on_insert = ClipSection::on_insert)]
pub(crate) struct InheritedClip(pub(crate) Option<Section<Logical>>);
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ResolvedClip(pub(crate) Section<Logical>);
