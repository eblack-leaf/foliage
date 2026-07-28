use crate::{IconId, Tree};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use std::sync::Arc;

pub(crate) mod button;
pub(crate) mod card;
#[cfg(feature = "composite-extras")]
pub(crate) mod carousel;
#[cfg(feature = "composite-extras")]
pub(crate) mod checkbox;
#[cfg(feature = "composite-extras")]
pub(crate) mod dropdown;
#[cfg(feature = "composite-extras")]
pub(crate) mod list;
#[cfg(feature = "composite-extras")]
pub(crate) mod pagination;
#[cfg(feature = "composite-extras")]
pub(crate) mod polyline;
#[cfg(feature = "composite-extras")]
pub(crate) mod popover;
#[cfg(feature = "composite-extras")]
pub(crate) mod radio_group;
pub(crate) mod router;
#[cfg(feature = "composite-extras")]
pub(crate) mod segmented_control;
pub(crate) mod slider;
#[cfg(feature = "composite-extras")]
pub(crate) mod tabs;
pub(crate) mod text_input;
#[cfg(feature = "composite-extras")]
pub(crate) mod toggle;

/// THE slot convention: how a library composite hosts arbitrary author content (a Card's
/// main region) without baking in what that content is. The composite calls the closure with a
/// transient **slot** entity -- a private, positioned, `Grid::default()` child it just
/// spawned -- and the author branches whatever they want under it with full
/// `Location`/`Grid`/`Anchor` freedom, returning their content root. The closure is `Fn`
/// (not `FnOnce`) and lives inside the widget's config component, so rewriting that config
/// via `write_to` re-runs it against a fresh slot: config writes ARE the re-render API, the
/// same `react` door as every value. Slots carry no interaction opinions -- author content
/// may be interactive.
pub type SlotFn = Arc<dyn Fn(&mut Tree, Entity) -> Entity + Send + Sync>;
/// [`SlotFn`]'s indexed variant, for collection composites (List rows, Carousel pages):
/// called once per index with that index's own slot. No return -- unlike a Card's main
/// region (removed on close, hence `SlotFn`'s returned root), collection slots are torn
/// down wholesale by the composite, so there's nothing for the author to hand back.
pub type IndexedSlotFn = Arc<dyn Fn(&mut Tree, Entity, usize) + Send + Sync>;

/// A text-bearing entity's public value channel: write it to a `Text` entity (or a widget
/// root that forwards it, like Button/TextInput) and the content follows.
#[derive(Component, Clone, Default)]
pub struct TextValue(pub String);
/// An icon-bearing entity's public value channel, same contract as [`TextValue`].
#[derive(Component, Copy, Clone, Default)]
pub struct IconValue(pub IconId);
/// A progress-bearing widget's public value channel (0.0..=1.0), same contract as
/// [`TextValue`]: write it to a widget root that carries one (Slider) and the visuals
/// follow. Values outside the range are clamped where they're consumed.
#[derive(Component, Copy, Clone, Default)]
pub struct Progress(pub f32);
/// A paged widget's current-page value channel (Pagination, Carousel), same contract as
/// [`TextValue`]. Writes outside `0..PageCount` are clamped where they're consumed.
#[derive(Component, Copy, Clone, Default)]
pub struct PageIndex(pub usize);
/// A paged widget's page-count channel, the other half of [`PageIndex`].
#[derive(Component, Copy, Clone, Default)]
pub struct PageCount(pub usize);
/// Emitted at a paged widget's root whenever its [`PageIndex`] changes -- an indicator
/// click, a prev/next step, or a programmatic write -- including once at spawn with the
/// initial value, since the reaction that fires it re-fires initial state like every
/// `react`.
///
/// Lives here rather than with `Pagination` because it is the shared page-change channel:
/// `Router`, `Tabs` and `Carousel` all listen for it without necessarily hosting a
/// pagination strip.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct PageChanged {
    pub index: usize,
}
