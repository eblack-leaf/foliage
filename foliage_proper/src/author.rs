use crate::tree::Sow;
use crate::{EcsExtension, Elevation, Entity, Location, Stem};
use bevy_ecs::bundle::Bundle;

/// The position/hierarchy/elevation state every `Sprout` type embeds.
///
/// `elevation` has no default -- an unset `Elevation` silently picking *some* layer (front,
/// back, or "one above whatever the parent is this moment") is exactly the kind of surprise
/// that's cost real debugging time in this codebase: a loop spawning children with no explicit
/// `.elevate()` call would silently stack them all identically, or push them out of the intended
/// z-range, with no signal anything was wrong. Leaving it unset is a hard requirement to specify
/// it, not a soft default to fall back on.
pub struct LeafSprout {
    pub(crate) location: Location,
    pub(crate) stem: Stem,
    pub(crate) elevation: Option<Elevation>,
}
impl Default for LeafSprout {
    fn default() -> Self {
        Self {
            location: Location::new(),
            stem: Stem::none(),
            elevation: None,
        }
    }
}
impl LeafSprout {
    pub fn new() -> Self {
        Self::default()
    }
}

/// THE authoring trait -- the one tool for primitives, library widgets, and end-user
/// widgets alike. A widget is one entity: components in (via [`Sprout::root`]), events out.
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
/// The former `Seed`/`Sprout`/`Photosynthesis` split existed because single-bundle
/// primitives and multi-entity composites had different spawn paths; `root`+`build`
/// erases the difference, so the three traits collapsed into this one.
pub trait Sprout: Sized {
    fn seed(&mut self) -> &mut LeafSprout;
    /// config -> components on the root entity. This IS the widget's public API.
    fn root(self) -> impl Bundle;
    /// private static skeleton + reaction registration. Default empty == a primitive.
    #[allow(unused_variables)]
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {}
    fn at(mut self, location: Location) -> Self {
        self.seed().location = location;
        self
    }
    fn stem(mut self, parent: Entity) -> Self {
        self.seed().stem = Stem::some(parent);
        self
    }
    fn elevate(mut self, e: Elevation) -> Self {
        self.seed().elevation = Some(e);
        self
    }
    /// Folds extra components (interaction flags, end-user data, ...) into the same
    /// one-shot bundle that gets inserted -- not a separate `write_to` after spawn.
    fn with<X: Bundle>(self, extra: X) -> With<Self, X> {
        With { inner: self, extra }
    }
    fn photosynthesize<T: EcsExtension>(mut self, tree: &mut T) -> Entity {
        let leaf = core::mem::take(self.seed());
        let this = tree.leaf((
            leaf.location,
            leaf.stem,
            leaf.elevation
                .expect("elevation not set -- call .elevate(...) before spawning"),
            self.root(),
        ));
        Self::build(this, tree);
        this
    }
}

/// `LeafSprout` is itself a `Sprout` -- the "no named marker component" case, for a child with
/// no reusable type of its own (a bare interaction hit-area, say). Its marker component, if
/// any, is attached the same way every other extra component is: `.with(...)`. This is what
/// makes [`EcsExtension::branch`] uniform for *every* child instead of needing a raw path.
impl Sprout for LeafSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        self
    }
    fn root(self) -> impl Bundle {}
}

/// See [`Sprout::with`].
pub struct With<S: Sprout, X: Bundle> {
    inner: S,
    extra: X,
}
impl<S: Sprout, X: Bundle> Sprout for With<S, X> {
    fn seed(&mut self) -> &mut LeafSprout {
        self.inner.seed()
    }
    fn root(self) -> impl Bundle {
        (self.inner.root(), self.extra)
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        S::build(this, tree);
    }
}
