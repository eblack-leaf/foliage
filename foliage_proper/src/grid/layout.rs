use crate::EcsExtension;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::Location;
use crate::{CoordinateUnit, Logical, Section, Stem, Tree, Resolve, Resolved};
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Query, ResMut};

/// The current breakpoint, from the viewport's width -- a `Resource`, so there is one
/// answer for the whole app at any moment.
///
/// Every responsive type keys off this: a [`Location`](crate::Location)'s `.xs()`/`.md()`
/// variants, a [`Grid`](crate::Grid)'s per-breakpoint configurations, and
/// [`FontSize`](crate::FontSize)'s per-breakpoint sizes. Each falls back to the nearest
/// smaller breakpoint that was given, so only `xs` is ever required.
///
/// Recomputed on resize; a change re-resolves the layout from the roots down.
#[derive(Resource, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Layout {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}
impl Layout {
    /// Lower bound of `Sm`, in logical pixels. Below this is `Xs`.
    pub const SM: CoordinateUnit = 420.0;
    /// Lower bound of `Md`.
    pub const MD: CoordinateUnit = 600.0;
    /// Lower bound of `Lg`.
    pub const LG: CoordinateUnit = 840.0;
    /// Lower bound of `Xl`.
    pub const XL: CoordinateUnit = 1200.0;
    /// The breakpoint `section`'s width falls in.
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
/// Re-derives [`Layout`] after a resize and re-resolves the layout tree.
///
/// Only root entities are triggered: each resolved `Section` cascades
/// `Resolve<Location>` to its own children, so the whole tree follows from the roots.
pub(crate) fn viewport_changed(
    mut vh: ResMut<ViewportHandle>,
    locations: Query<(Entity, &Stem), With<Location>>,
    mut layout: ResMut<Layout>,
    mut tree: Tree,
) {
    if vh.window_forced_resize() {
        let new = Layout::new(vh.section());
        if new != *layout {
            tree.trigger(Resolved::<Layout>::new());
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
        tree.trigger_targets(Resolve::<Location>::new(), targets);
    }
}
