use crate::{Elevation, Entity, Location, Stem};

/// The position/hierarchy/elevation state every leaf-spawning `Spec` type embeds.
pub struct LeafSpec {
    pub(crate) location: Location,
    pub(crate) stem: Stem,
    pub(crate) elevation: Elevation,
}
impl Default for LeafSpec {
    fn default() -> Self {
        Self {
            location: Location::new(),
            stem: Stem::none(),
            elevation: Elevation::up(1),
        }
    }
}

/// Implement `leaf_spec` and any `Spec` type gets `.at()`/`.stem()`/`.elevate()` for free.
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
        self.leaf_spec().elevation = e;
        self
    }
}
