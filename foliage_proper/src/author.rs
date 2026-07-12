use crate::{EcsExtension, Elevation, Entity, Location, Stem};
use bevy_ecs::bundle::Bundle;

/// The position/hierarchy/elevation state every leaf-spawning `Sprout` type embeds.
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
/// Shared by every `Sprout` type regardless of how it grows: the position/hierarchy/elevation
/// seed data, and the `.at()`/`.stem()`/`.elevate()` chain that sets it.
pub trait Seed: Sized {
    fn seed(&mut self) -> &mut LeafSprout;
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
}
/// Anything `Children::spawn`/`.each()` can grow: knows how to place itself (`Seed`) and how to
/// turn itself into a real, living `Entity` (`photosynthesize`). Most `Sprout` types get this
/// for free from `Sprout`'s blanket impl below; a composite type that builds more than one
/// entity (`ButtonSprout`) implements it directly instead, with its own `photosynthesize()`.
pub trait Photosynthesis: Seed {
    fn photosynthesize<T: EcsExtension>(self, tree: &mut T) -> Entity;
}
/// A type that grows into exactly one entity from exactly one bundle. `bundle()` is the required
/// method (not `photosynthesize()`): every field a `Sprout` carries must fold into the one
/// bundle that gets inserted in a single call, never a follow-up `write_to` after an initial
/// bare insert. Two-phase spawn-then-patch means whatever reactive hook fires on the base insert
/// resolves once against a placeholder default before the real value lands in a second command
/// -- concretely, `ButtonSprout` used to spawn bare and patch `Rounding` in after, which
/// flickered the icon through its default position/visibility before the requested `Rounding`
/// ever took effect. Folding every field into one `bundle()` call is what makes `.with()` safe
/// too: `With::bundle` composes via a tuple instead of a second write.
///
/// Only `Sprout` types get `.with()` and `.bundle()` -- a composite type implements plain
/// `Photosynthesis` instead (see `ButtonSprout`), so those methods simply don't exist on it,
/// rather than existing but silently returning something meaningless if called.
pub trait Sprout: Seed {
    fn bundle(self) -> impl Bundle;
    /// Folds extra components (interaction flags, `Root`, ...) into the same one-shot bundle
    /// that gets inserted -- not a separate `write_to` call after spawn.
    fn with<X: Bundle>(self, extra: X) -> With<Self, X> {
        With { inner: self, extra }
    }
}
impl<S: Sprout> Photosynthesis for S {
    fn photosynthesize<T: EcsExtension>(self, tree: &mut T) -> Entity {
        tree.leaf(self.bundle())
    }
}
/// `LeafSprout` is itself a `Sprout` -- the "no named marker component" case, for a child with
/// no reusable `Sprout` type of its own (a bare interaction hit-area, say). Its marker
/// component, if any, is attached the same way every other extra component is: `.with(...)`.
/// This is what makes `Children::spawn` uniform for *every* child instead of needing a separate
/// raw-bundle path -- `LeafSprout::new()` and `Panel::new()` are both just `Sprout`s.
impl Seed for LeafSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        self
    }
}
impl Sprout for LeafSprout {
    fn bundle(self) -> impl Bundle {
        (
            self.location,
            self.stem,
            self.elevation
                .expect("elevation not set -- call .elevate(...) before spawning"),
        )
    }
}
/// See [`Sprout::with`].
pub struct With<S: Sprout, X: Bundle> {
    inner: S,
    extra: X,
}
impl<S: Sprout, X: Bundle> Seed for With<S, X> {
    fn seed(&mut self) -> &mut LeafSprout {
        self.inner.seed()
    }
}
impl<S: Sprout, X: Bundle> Sprout for With<S, X> {
    fn bundle(self) -> impl Bundle {
        (self.inner.bundle(), self.extra)
    }
}
