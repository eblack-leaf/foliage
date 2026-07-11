use crate::{EcsExtension, Elevation, Entity, Location, Stem};
use bevy_ecs::bundle::Bundle;

/// The position/hierarchy/elevation state every leaf-spawning `Spec` type embeds.
///
/// `elevation` has no default -- an unset `Elevation` silently picking *some* layer (front,
/// back, or "one above whatever the parent is this moment") is exactly the kind of surprise
/// that's cost real debugging time in this codebase: a loop spawning children with no explicit
/// `.elevate()` call would silently stack them all identically, or push them out of the intended
/// z-range, with no signal anything was wrong. Leaving it unset is a hard requirement to specify
/// it, not a soft default to fall back on.
pub struct LeafSpec {
    pub(crate) location: Location,
    pub(crate) stem: Stem,
    pub(crate) elevation: Option<Elevation>,
}
impl Default for LeafSpec {
    fn default() -> Self {
        Self {
            location: Location::new(),
            stem: Stem::none(),
            elevation: None,
        }
    }
}
impl LeafSpec {
    pub fn new() -> Self {
        Self::default()
    }
}
/// `LeafSpec` is itself a `LeafBuilder` -- the "no named marker component" case, for a child
/// with no reusable `Spec` type of its own (a bare interaction hit-area, say). Its marker
/// component, if any, is attached the same way every other extra component is: `.with(...)`.
/// This is what makes `Children::spawn` uniform for *every* child instead of needing a separate
/// raw-bundle path -- `LeafSpec::new()` and `Panel::new()` are both just `LeafBuilder`s.
impl LeafBuilder for LeafSpec {
    fn leaf_spec(&mut self) -> &mut LeafSpec {
        self
    }
    fn bundle(self) -> impl Bundle {
        (
            self.location,
            self.stem,
            self.elevation
                .expect("elevation not set -- call .elevate(...) before spawning"),
        )
    }
}

/// Implement `leaf_spec` and `bundle` and any `Spec` type gets `.at()`/`.stem()`/`.elevate()`/
/// `.with()`/`.spawn()` for free. `bundle` (not `spawn`) is the required method: every field a
/// `Spec` carries must fold into the one bundle `spawn` inserts in a single call, never a
/// follow-up `write_to` after an initial bare insert. Two-phase spawn-then-patch means whatever
/// reactive hook fires on the base insert resolves once against a placeholder default before the
/// real value lands in a second command -- concretely, `ButtonSpec` used to spawn bare and patch
/// `Rounding` in after, which flickered the icon through its default position/visibility before
/// the requested `Rounding` ever took effect. Folding every field into one `bundle()` call is
/// what makes `.with()` safe too: `With::bundle` composes via a tuple instead of a second write,
/// so attaching e.g. `Icon` through `.with(...)` is exactly as one-shot as a field on the `Spec`
/// itself.
pub trait LeafBuilder: Sized {
    fn leaf_spec(&mut self) -> &mut LeafSpec;
    fn at(mut self, location: Location) -> Self {
        self.leaf_spec().location = location;
        self
    }
    fn stem(mut self, parent: Entity) -> Self {
        self.leaf_spec().stem = Stem::some(parent);
        self
    }
    fn elevate(mut self, e: Elevation) -> Self {
        self.leaf_spec().elevation = Some(e);
        self
    }
    fn bundle(self) -> impl Bundle;
    fn spawn<T: EcsExtension>(self, tree: &mut T) -> Entity {
        tree.leaf(self.bundle())
    }
    /// Folds extra components (interaction flags, `Root`, ...) into the same one-shot bundle
    /// `spawn` inserts -- not a separate `write_to` call after spawn.
    fn with<X: Bundle>(self, extra: X) -> With<Self, X> {
        With { inner: self, extra }
    }
}
/// See [`LeafBuilder::with`].
pub struct With<S: LeafBuilder, X: Bundle> {
    inner: S,
    extra: X,
}
impl<S: LeafBuilder, X: Bundle> LeafBuilder for With<S, X> {
    fn leaf_spec(&mut self) -> &mut LeafSpec {
        self.inner.leaf_spec()
    }
    fn bundle(self) -> impl Bundle {
        (self.inner.bundle(), self.extra)
    }
}
