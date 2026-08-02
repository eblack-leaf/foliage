use crate::{Elevation, Entity, Location, Parent};
use bevy_ecs::bundle::Bundle;

/// The position/hierarchy/elevation state every `Author` type embeds.
///
/// `elevation` has no default -- an unset `Elevation` silently picking *some* layer (front,
/// back, or "one above whatever the parent is this moment") is exactly the kind of surprise
/// that's cost real debugging time in this codebase: a loop spawning children with no explicit
/// `.elevate()` call would silently stack them all identically, or push them out of the intended
/// z-range, with no signal anything was wrong. Leaving it unset is a hard requirement to specify
/// it, not a soft default to fall back on.
pub struct LeafSprout {
    pub(crate) location: Location,
    pub(crate) stem: Parent,
    pub(crate) elevation: Option<Elevation>,
    /// The optional trimmings, held here rather than as a wrapper type around the spec.
    ///
    /// `.with(bundle)` used to wrap the whole spec in a `WithExtras<S, X>`, which changed its
    /// type -- fine when a spec was consumed by a generic function, impossible now that
    /// specs travel through the boundary as a closed set. Naming each one instead keeps
    /// every spec its own type all the way to the queue.
    pub(crate) grid: Option<crate::Grid>,
    pub(crate) opacity: Option<crate::Opacity>,
    pub(crate) alignment: Option<(crate::HorizontalAlignment, crate::VerticalAlignment)>,
    pub(crate) listener: Option<crate::InteractionListener>,
    pub(crate) propagation: Option<crate::InteractionPropagation>,
    pub(crate) shape: Option<crate::InteractionShape>,
    pub(crate) clip_to_viewport: bool,
    pub(crate) aspect: Option<crate::AspectRatio>,
    pub(crate) content_size: Option<(bool, bool)>,
    pub(crate) font: Option<crate::FontId>,
    pub(crate) overscroll: Option<crate::OverscrollPropagation>,
}
impl Default for LeafSprout {
    fn default() -> Self {
        Self {
            location: Location::new(),
            stem: Parent::none(),
            elevation: None,
            grid: None,
            opacity: None,
            alignment: None,
            listener: None,
            propagation: None,
            shape: None,
            clip_to_viewport: false,
            aspect: None,
            content_size: None,
            font: None,
            overscroll: None,
        }
    }
}
impl LeafSprout {
    /// An unconfigured leaf builder. Authors reach this through
    /// [`Node::sprout`](crate::Node::sprout).
    pub fn new() -> Self {
        Self::default()
    }
}

/// THE authoring trait -- the one tool for primitives, library widgets, and end-user
/// widgets alike. A widget is one entity: components in (via [`Author::root`]), events out.
///
/// - `root` folds every config field into the one bundle inserted at spawn (never a
///   follow-up `write_to` after a bare insert -- two-phase spawn-then-patch lets reactive
///   hooks resolve once against placeholder defaults before the real value lands).
///   The library folds the [`LeafSprout`] seed fields in itself; `root` returns only
///   widget components.
/// - `build` is the private, config-INDEPENDENT skeleton (default empty == a primitive),
///   grown with [`EcsExtension::branch`]. Everything data-dependent -- values and
///   structure -- goes through [`EcsExtension::react`], which runs once at spawn and
///   again on every later write, so initial state and updates share one code path.
///
/// The former `Seed`/`Author`/`Photosynthesis` split existed because single-bundle
/// primitives and multi-entity composites had different spawn paths; `root`+`build`
/// erases the difference, so the three traits collapsed into this one.
///
/// There is no `stem`/`photosynthesize` here on purpose -- spawning is `Sow::grow`'s job
/// (`pub(crate)`), reached only through [`EcsExtension::leaf`]/[`EcsExtension::branch`], so a
/// widget can't be spawned orphaned or skip the mandatory `.elevate(...)` by hand-rolling
/// the chain outside this crate.
pub(crate) trait Author: Sized {
    fn seed(&mut self) -> &mut LeafSprout;
    /// config -> components on the root entity. This IS the widget's public API.
    fn root(self) -> impl Bundle;
    /// private static skeleton + reaction registration. Default empty == a primitive.
    #[allow(unused_variables)]
    fn build(this: Entity, tree: &mut crate::Tree) {}
    /// Folds extra components into the same one-shot bundle that gets inserted -- how the
    /// engine's own specs carry components an app has no business naming.
    fn with<X: Bundle>(self, extra: X) -> WithExtras<Self, X> {
        WithExtras { inner: self, extra }
    }
}

/// How an element is set up before it grows: where it sits, how it stacks, how it behaves.
///
/// Every spec carries the same set, so `Panel::new()`, `Text::new()` and [`Bare::new`] are
/// configured identically. Sealed -- [`Author`] is `pub(crate)`, so this can be called but
/// never implemented, and the set stays exactly what the engine knows how to apply.
#[allow(private_bounds)]
pub trait Sprout: Author {
    /// Where this element sits and how big it is.
    fn at(mut self, location: Location) -> Self {
        self.seed().location = location;
        self
    }
    /// Draw order relative to its siblings. Required -- there is no default, because a
    /// silently-chosen layer is the kind of thing found much later.
    fn elevate(mut self, e: Elevation) -> Self {
        self.seed().elevation = Some(e);
        self
    }
    /// Lays children out on a grid. Required on anything with children: a child's
    /// `Location` resolves against its parent's grid, and a parent without one is a panic
    /// at spawn rather than a silent misplacement.
    fn grid(mut self, grid: crate::Grid) -> Self {
        self.seed().grid = Some(grid);
        self
    }
    /// Starting opacity, inherited by everything beneath.
    fn opacity(mut self, value: f32) -> Self {
        self.seed().opacity = Some(crate::Opacity::new(value));
        self
    }
    /// How content sits within this element's own box.
    fn align(mut self, h: crate::HorizontalAlignment, v: crate::VerticalAlignment) -> Self {
        self.seed().alignment = Some((h, v));
        self
    }
    /// Competes for gestures: the topmost interactive element under the pointer wins.
    fn interactive(mut self) -> Self {
        self.seed().listener = Some(crate::InteractionListener::new());
        self.seed().propagation = Some(crate::InteractionPropagation::grab());
        self
    }
    /// Notified when a gesture crosses it, but never wins one -- for overlays and
    /// decoration that must not eat input.
    fn pass_through(mut self) -> Self {
        self.seed().propagation = Some(crate::InteractionPropagation::pass_through());
        self
    }
    /// Refuses drag-panning through this element, for a knob or slider owning its own drag.
    fn holds_drag(mut self) -> Self {
        let current = self
            .seed()
            .propagation
            .unwrap_or(crate::InteractionPropagation::grab());
        self.seed().propagation = Some(current.disable_drag());
        self
    }
    /// Hit-tests as a circle rather than a rectangle.
    fn round_hit_area(mut self) -> Self {
        self.seed().shape = Some(crate::InteractionShape::Circle);
        self
    }
    /// Whether scroll this element cannot consume passes outward to an ancestor.
    ///
    /// On by default -- which is what lets a drag inside a region still move the page behind
    /// it once the region hits its own end. Turn it off to trap scrolling inside.
    fn overscroll(mut self, passes: bool) -> Self {
        self.seed().overscroll = Some(crate::OverscrollPropagation(passes));
        self
    }
    /// Clips this element to the viewport rather than to its nearest scrolling ancestor.
    fn clip_to_viewport(mut self) -> Self {
        self.seed().clip_to_viewport = true;
        self
    }
    /// Constrains the resolved box to a width:height ratio.
    fn aspect(mut self, ratio: crate::AspectRatio) -> Self {
        self.seed().aspect = Some(ratio);
        self
    }
    /// Sizes the box from the text's own measured extent rather than bounding it.
    fn sized_by_content(mut self, width: bool, height: bool) -> Self {
        self.seed().content_size = Some((width, height));
        self
    }
    /// Draws this element's text in a face registered with
    /// [`Foliage::font`](crate::Foliage::font), rather than the built-in one.
    fn font(mut self, font: crate::FontId) -> Self {
        self.seed().font = Some(font);
        self
    }
}

impl<S: Author> Sprout for S {}

/// `LeafSprout` is itself a `Author` -- the "no named marker component" case, for a child with
/// no reusable type of its own (a bare interaction hit-area, say). Its marker component, if
/// any, is attached the same way every other extra component is: `.with(...)`. This is what
/// makes [`EcsExtension::branch`] uniform for *every* child instead of needing a raw path.
impl Author for LeafSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        self
    }
    fn root(self) -> impl Bundle {}
}

/// See [`Author::with`].
///
/// Named `WithExtras` rather than `With` because `foliage::*` re-exports bevy's prelude,
/// and a `With` here shadows the query filter every consumer writes
/// (`Query<Entity, With<Marker>>`). Nothing ever names this type -- it is only what `.with()`
/// returns -- so the collision was all cost.
pub(crate) struct WithExtras<S: Author, X: Bundle> {
    inner: S,
    extra: X,
}
impl<S: Author, X: Bundle> Author for WithExtras<S, X> {
    fn seed(&mut self) -> &mut LeafSprout {
        self.inner.seed()
    }
    fn root(self) -> impl Bundle {
        (self.inner.root(), self.extra)
    }
    fn build(this: Entity, tree: &mut crate::Tree) {
        S::build(this, tree);
    }
}
