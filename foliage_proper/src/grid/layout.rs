use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::Location;
use crate::{LogicalContext, Section, Stem, Tree, Update};
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, ResMut, Resource};

#[derive(Resource, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Layout {
    Sm,
    Md,
    Lg,
    Xl,
}
impl Layout {
    pub fn new(section: Section<LogicalContext>) -> Self {
        todo!()
    }
}
pub(crate) fn viewport_changed(
    mut vh: ResMut<ViewportHandle>,
    locations: Query<(Entity, &Stem), With<Location>>,
    mut layout: ResMut<Layout>,
    mut tree: Tree,
) {
    if vh.updated() {
        let new = Layout::new(vh.section());
        if new != *layout {
            // Write<Layout> => responsive font-size configure + user stuff
            *layout = new;
        }
        let mut targets = vec![];
        for (e, stem) in locations.iter() {
            if stem.id.is_none() {
                targets.push(e);
            }
        }
        if targets.is_empty() {
            return;
        }
        tree.trigger_targets(Update::<Location>::new(), targets);
    }
}
