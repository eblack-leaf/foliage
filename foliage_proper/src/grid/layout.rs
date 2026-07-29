use crate::EcsExtension;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::Location;
use crate::{CoordinateUnit, Logical, Resolve, Resolved, Section, Stem, Tree};
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

/// Whether the viewport is vertically cramped -- a phone held landscape, a short desktop
/// window, a mobile browser whose address bar is eating the difference. A `Resource`, like
/// [`Layout`], so there is one answer for the whole app.
///
/// Deliberately **orthogonal** to `Layout` rather than a sixth breakpoint: width and height
/// are independent, and `Layout` is a total order whose fallback chain ("nearest smaller
/// breakpoint that was given") only means anything along one axis. A landscape phone is
/// genuinely `Md`-wide *and* cramped-tall, and squashing that into one enum has to discard
/// one of the two.
///
/// Consumed by [`Location::short`](crate::Location::short), which wins over the
/// width-derived pick when set and this is [`Short::Yes`]. Everything without a `short`
/// configuration is unaffected, so turning this on changes nothing until an author opts a
/// specific value in.
#[derive(Resource, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Short {
    Yes,
    No,
}
impl Short {
    /// Below this height (logical px) the viewport becomes [`Short::Yes`]. Sized to catch
    /// phones in landscape (~330-390 after browser chrome) and little else -- at 480 it also
    /// claimed ordinary short desktop windows, which have plenty of room and only want the
    /// normal layout.
    pub const ENTER: CoordinateUnit = 400.0;
    /// At or above this height it returns to [`Short::No`]. The gap between the two is a
    /// deliberate deadband, and it is asymmetric on purpose -- *sticky toward short*.
    ///
    /// A single threshold thrashes: on mobile web the address bar hides and shows as the
    /// user scrolls, so a viewport resting near the boundary would cross it on every
    /// scroll and re-fire `Resolve<Location>` across the entire tree each time. The
    /// stickiness resolves that, and it leans the safe way -- a false `Yes` is a layout
    /// that is merely more conservative than it needed to be, while a false `No` is
    /// content running off the bottom of the screen, which is the failure this exists to
    /// prevent.
    pub const EXIT: CoordinateUnit = 440.0;
    /// The next state, given the current one -- the hysteresis. Between [`ENTER`](Self::ENTER)
    /// and [`EXIT`](Self::EXIT) the answer is whatever it already was.
    pub fn next(self, section: Section<Logical>) -> Self {
        let height = section.height();
        match self {
            Self::No if height < Self::ENTER => Self::Yes,
            Self::Yes if height >= Self::EXIT => Self::No,
            current => current,
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
    mut short: ResMut<Short>,
    mut tree: Tree,
) {
    if vh.window_forced_resize() {
        let new = Layout::new(vh.section());
        // `Short` rides the same `Resolved::<Layout>` notification rather than carrying its
        // own: both are inputs to the same resolve, and an author reacting to one wants to
        // re-read both anyway.
        let new_short = short.next(vh.section());
        if new != *layout || new_short != *short {
            tree.trigger(Resolved::<Layout>::new());
            *layout = new;
            *short = new_short;
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
