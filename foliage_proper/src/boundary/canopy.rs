use crate::boundary::bloom::Bloom;
use crate::boundary::leaf::{Grown, Leaf, Presence};
use crate::boundary::op::Op;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::interaction::Click;
use crate::opacity::BlendedOpacity;
use crate::{
    AssetKey, AssetLoader, Color, CurrentInteraction, FontSize, Layout, Logical, ResolvedVisibility, Section, Short, Text,
    Time, TimeDelta, View,
};
use bevy_ecs::system::{Query, Res, SystemParam};

/// Everything a frame may observe about the tree, in one read-only bundle.
///
/// Read-only is what makes synchronous sampling possible at all: the whole set can be driven
/// from an immutable borrow of the world while the frame closure runs.
#[derive(SystemParam)]
pub(crate) struct Reads<'w, 's> {
    pub(crate) entities: &'w bevy_ecs::entity::Entities,
    pub(crate) grown: Query<'w, 's, &'static Grown>,
    pub(crate) sections: Query<'w, 's, &'static Section<Logical>>,
    pub(crate) layout_sections: Query<'w, 's, &'static crate::LayoutSection>,
    pub(crate) points: Query<'w, 's, &'static crate::Points<Logical>>,
    pub(crate) views: Query<'w, 's, &'static View>,
    pub(crate) progress: Query<'w, 's, &'static crate::ScrollProgress>,
    pub(crate) visibility: Query<'w, 's, &'static ResolvedVisibility>,
    pub(crate) opacity: Query<'w, 's, &'static BlendedOpacity>,
    pub(crate) elevation: Query<'w, 's, &'static crate::ResolvedElevation>,
    pub(crate) text: Query<'w, 's, &'static Text>,
    pub(crate) values: Query<'w, 's, &'static crate::TextValue>,
    pub(crate) font_sizes: Query<'w, 's, &'static FontSize>,
    pub(crate) colors: Query<'w, 's, &'static Color>,
    pub(crate) enabled: Query<'w, 's, &'static crate::InteractionListener>,
    pub(crate) children: Query<'w, 's, &'static crate::Children>,
    pub(crate) parents: Query<'w, 's, &'static crate::Parent>,
    pub(crate) layout: Res<'w, Layout>,
    pub(crate) short: Res<'w, Short>,
    pub(crate) time: Res<'w, Time>,
    pub(crate) interaction: Res<'w, CurrentInteraction>,
    pub(crate) assets: Res<'w, AssetLoader>,
    pub(crate) viewport: Res<'w, crate::ginkgo::viewport::ViewportHandle>,
    pub(crate) named: Res<'w, crate::Named>,
    pub(crate) keyring: Res<'w, crate::Keyring>,
}

/// Exactly what a frame may observe about an element.
///
/// Exhaustive by construction: if it is not here, user code cannot see it. Deliberately
/// absent are the render internals -- glyph rectangles, caret and selection, resolved clip
/// and depth -- which are the engine's own working state rather than anything an app needs.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Sap {
    /// The on-screen box: position and area together, after any scrolling.
    Section,
    Position,
    Area,
    /// The box before scrolling moved it -- what children resolve against.
    LayoutSection,
    /// The resolved endpoints of a point-mode element, a line or a polyline.
    Points,
    /// Whether it is actually drawn, its ancestors included.
    Visible,
    /// The alpha actually drawn, its ancestors' opacity included.
    Opacity,
    /// Draw order, after the whole tree has been ranked.
    Elevation,
    /// A text element's laid-out contents.
    Text,
    /// The value written to a text-bearing element, which for an input is what has been
    /// typed into it. Differs from [`Text`](Sap::Text) on anything that forwards its value
    /// to a child.
    Value,
    FontSize,
    Color,
    /// Whether this element currently competes for gestures.
    Enabled,
    /// A view's current scroll offset, in logical pixels.
    ScrollOffset,
    /// The full extent a view can scroll over.
    ScrollExtent,
    /// How far through its range a view is scrolled, per axis, 0.0..=1.0.
    ScrollProgress,
    /// The elements grown directly under this one.
    Children,
    /// The element this one sits under, if any.
    Parent,
}

/// What a [`Sap`] reads back.
#[derive(Clone, Debug)]
pub enum Sample<'a> {
    Section(Section<Logical>),
    Position(Position<Logical>),
    Area(Area<Logical>),
    Points(&'a [Position<Logical>]),
    Flag(bool),
    Scalar(f32),
    Pair(f32, f32),
    Size(u32),
    Color(Color),
    Text(&'a str),
    Leaves(Vec<Leaf>),
    Leaf(Option<Leaf>),
}

/// The frame surface: what happened, what things are, and what to do about it.
///
/// Handed to the closure given to [`photosynthesize`](crate::Foliage::photosynthesize), once
/// per frame, at the point where engine state has settled and before the frame is drawn.
/// Everything sampled here is current as of this frame.
///
/// Commands issued here are applied in the order they are written, immediately after the
/// closure returns -- so they take effect in the same frame that produced them.
pub struct Canopy<'w, 's> {
    pub(crate) reads: Reads<'w, 's>,
    pub(crate) queue: &'w mut Vec<Op>,
    pub(crate) blooms: Vec<Bloom>,
    /// Shared with every [`Sprig`](crate::Sprig), so a name allocated here and one allocated
    /// on another thread can never be the same name.
    pub(crate) allocator: bevy_ecs::entity::RemoteAllocator,
}

impl crate::boundary::verbs::Queues for Canopy<'_, '_> {
    fn push(&mut self, op: Op) {
        self.queue.push(op);
    }
    fn allocate(&self) -> Leaf {
        Leaf(self.allocator.alloc())
    }
}

impl<'w, 's> Canopy<'w, 's> {
    /// Takes this frame's emissions. Moved rather than lent, so the usual
    /// `for bloom in canopy.take() { canopy.text(..) }` needs no dance to satisfy the
    /// borrow checker.
    pub fn take(&mut self) -> Vec<Bloom> {
        core::mem::take(&mut self.blooms)
    }

    /// What `leaf` names right now.
    ///
    /// Told apart by the id itself: a name that was never spawned and one whose element has
    /// been taken down are different generations, so this needs no bookkeeping of its own.
    pub fn presence(&self, leaf: Leaf) -> Presence {
        if self.reads.entities.contains(leaf.0) {
            Presence::Live
        } else if self.reads.entities.contains_spawned(leaf.0) {
            // The id exists but this generation of it does not: something was here and is
            // gone. An id that was allocated and never spawned reports neither.
            Presence::Withered
        } else {
            Presence::Planted
        }
    }

    /// Reads one property of an element, or `None` if it has withered, has not been grown
    /// yet, or simply does not carry that property.
    pub fn sample(&self, leaf: Leaf, what: Sap) -> Option<Sample<'_>> {
        let entity = leaf.0;
        let reads = &self.reads;
        Some(match what {
            Sap::Section => Sample::Section(*reads.sections.get(entity).ok()?),
            Sap::Position => Sample::Position(reads.sections.get(entity).ok()?.position),
            Sap::Area => Sample::Area(reads.sections.get(entity).ok()?.area),
            Sap::LayoutSection => Sample::Section(reads.layout_sections.get(entity).ok()?.0),
            Sap::Points => Sample::Points(reads.points.get(entity).ok()?.data.as_slice()),
            Sap::Visible => Sample::Flag(reads.visibility.get(entity).ok()?.visible()),
            Sap::Opacity => Sample::Scalar(reads.opacity.get(entity).ok()?.value),
            Sap::Elevation => Sample::Scalar(reads.elevation.get(entity).ok()?.value()),
            Sap::Text => Sample::Text(reads.text.get(entity).ok()?.value.as_str()),
            Sap::Value => Sample::Text(reads.values.get(entity).ok()?.0.as_str()),
            Sap::FontSize => Sample::Size(reads.font_sizes.get(entity).ok()?.xs),
            Sap::Color => Sample::Color(*reads.colors.get(entity).ok()?),
            Sap::Enabled => Sample::Flag(!reads.enabled.get(entity).ok()?.disabled()),
            Sap::ScrollOffset => Sample::Position(reads.views.get(entity).ok()?.offset()),
            Sap::ScrollExtent => Sample::Section(reads.views.get(entity).ok()?.extent()),
            Sap::ScrollProgress => {
                let progress = reads.progress.get(entity).ok()?;
                Sample::Pair(progress.x(), progress.y())
            }
            // Only elements the app grew itself: the caret inside a text input is foliage's
            // business, and reporting it would hand back a `Leaf` naming something the app
            // never asked for and cannot meaningfully act on.
            Sap::Children => Sample::Leaves(
                reads
                    .children
                    .get(entity)
                    .ok()?
                    .ids
                    .iter()
                    .filter(|child| reads.grown.contains(**child))
                    .map(|child| Leaf(*child))
                    .collect(),
            ),
            Sap::Parent => Sample::Leaf(reads.parents.get(entity).ok()?.id.map(Leaf)),
        })
    }

    /// The on-screen box `leaf` resolved to.
    pub fn section(&self, leaf: Leaf) -> Option<Section<Logical>> {
        match self.sample(leaf, Sap::Section)? {
            Sample::Section(section) => Some(section),
            _ => None,
        }
    }
    /// A text element's contents.
    pub fn text_of(&self, leaf: Leaf) -> Option<&str> {
        match self.sample(leaf, Sap::Text)? {
            Sample::Text(value) => Some(value),
            _ => None,
        }
    }
    /// A view's current scroll offset.
    pub fn scroll_offset(&self, leaf: Leaf) -> Option<Position<Logical>> {
        match self.sample(leaf, Sap::ScrollOffset)? {
            Sample::Position(position) => Some(position),
            _ => None,
        }
    }
    /// Whether an element is actually drawn.
    pub fn is_visible(&self, leaf: Leaf) -> Option<bool> {
        match self.sample(leaf, Sap::Visible)? {
            Sample::Flag(flag) => Some(flag),
            _ => None,
        }
    }

    /// The current breakpoint.
    pub fn layout(&self) -> Layout {
        *self.reads.layout
    }
    /// Whether the viewport is vertically cramped.
    pub fn short(&self) -> bool {
        *self.reads.short == Short::Yes
    }
    /// How long the last frame took. Scale per-frame motion by this and it runs at the same
    /// speed at any frame rate.
    pub fn frame_time(&self) -> TimeDelta {
        self.reads.time.frame_diff()
    }
    /// The visible area.
    pub fn viewport(&self) -> Section<Logical> {
        self.reads.viewport.section()
    }
    /// Where the current gesture started, is now, and ended.
    pub fn pointer(&self) -> Click {
        self.reads.interaction.click()
    }
    /// The current drag's smoothed speed, in logical pixels per millisecond.
    pub fn pointer_velocity(&self) -> Position<Logical> {
        self.reads.interaction.velocity()
    }
    /// The element recorded under `name` by [`Grows::name`](crate::Grows::name), if one is.
    pub fn named(&self, name: impl AsRef<str>) -> Option<Leaf> {
        self.reads.named.lookup(name.as_ref()).map(Leaf)
    }
    /// The asset key recorded under `name`, if one is.
    pub fn asset_named(&self, name: impl AsRef<str>) -> Option<AssetKey> {
        self.reads.keyring.lookup(name.as_ref())
    }
    /// An asset's bytes, or `None` while it is still loading.
    pub fn asset(&self, key: AssetKey) -> Option<Vec<u8>> {
        self.reads.assets.retrieve(key).map(|asset| asset.data)
    }
}
