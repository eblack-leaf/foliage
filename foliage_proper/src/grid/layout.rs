use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::Location;
use crate::{CoordinateUnit, Logical, Section, Stem, Tree, Update, Write};
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, ResMut, Resource};

#[derive(Resource, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Layout {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}
impl Layout {
    pub const SM: CoordinateUnit = 420.0;
    pub const MD: CoordinateUnit = 600.0;
    pub const LG: CoordinateUnit = 840.0;
    pub const XL: CoordinateUnit = 1200.0;
    pub fn new(section: Section<Logical>) -> Self {
        if section.width() >= Self::XL {
            Self::Xl
        } else if section.width() >= Self::LG {
            Self::Lg
        } else if section.width() >= Self::MD {
            Self::Md
        } else if section.width() >= Self::SM {
            Self::Sm
        } else {
            Self::Xs
        }
    }
}
pub(crate) fn viewport_changed(
    mut vh: ResMut<ViewportHandle>,
    locations: Query<(Entity, &Stem), With<Location>>,
    mut layout: ResMut<Layout>,
    mut tree: Tree,
) {
    if vh.window_forced_resize() {
        let new = Layout::new(vh.section());
        if new != *layout {
            // Write<Layout> => responsive font-size configure + user stuff
            println!("layout changed: {:?}", new);
            tree.trigger(Write::<Layout>::new());
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
