use crate::{EcsExtension, Entity, Photosynthesis};

/// Spawns a Composite's (or a one-off structure's) children under a common `parent`, auto-filling
/// `.stem(parent)` on any [`Photosynthesis`] -- a named `Sprout` (`Panel::new()`, `Text::new()`), or a
/// bare `LeafSprout::new()` for a child with no marker component of its own. Every child uses the
/// same `.elevate()/.at()/.with()` chain regardless of which; there's exactly one way to spawn
/// a child here, not a separate path for Spec-backed vs. raw ones.
///
/// A child's `Location` is whatever `.at(...)` sets, or omitted entirely if never called: some
/// children (e.g. an icon whose position depends on a reactive property set later) must spawn
/// with no `Location` component at all, since giving one an empty default up front makes it fail
/// layout resolution immediately and get auto-hidden before the real value arrives.
pub struct Children<'t, T: EcsExtension> {
    parent: Entity,
    tree: &'t mut T,
}
impl<'t, T: EcsExtension> Children<'t, T> {
    pub fn new(parent: Entity, tree: &'t mut T) -> Self {
        Self { parent, tree }
    }
    pub fn parent(&self) -> Entity {
        self.parent
    }
    pub fn tree(&mut self) -> &mut T {
        self.tree
    }
    pub fn spawn<S: Photosynthesis>(&mut self, spec: S) -> Entity {
        spec.stem(self.parent).photosynthesize(self.tree)
    }
    /// Nests a second `composite()` under this one's `parent`, auto-filling `.stem(parent)` on
    /// `root` the same way [`Children::spawn`] does -- so a sub-group's own root never needs the
    /// enclosing parent threaded through by hand (`children.parent()`/`.stem(...)`).
    pub fn composite<S: Photosynthesis, R>(
        &mut self,
        root: S,
        build: impl FnOnce(&mut Children<T>) -> R,
    ) -> (Entity, R) {
        self.tree.composite(root.stem(self.parent), build)
    }
    /// Spawns per item. `build` receives the item's index, the item itself, and this `Children`
    /// (so it can call `spawn`/`each` again), and decides everything about how that child is
    /// placed and ordered — row, column, bottom-up, chained-stack, whatever. This only removes
    /// the loop/bookkeeping ceremony, never the placement decision.
    ///
    /// `build` returns whatever the caller needs to keep per item -- a single `Entity` when
    /// there's just one, or a tuple/struct of several when one iteration spawns more than one
    /// related entity (e.g. a button plus its caption) and the caller needs to wire behavior
    /// onto a specific one of them afterward, without falling back to a hand-rolled `Vec` of
    /// "things to wire later" alongside this one.
    pub fn each<I, F, R>(&mut self, items: I, mut build: F) -> Vec<R>
    where
        I: IntoIterator,
        F: FnMut(usize, I::Item, &mut Self) -> R,
    {
        items
            .into_iter()
            .enumerate()
            .map(|(i, item)| build(i, item, self))
            .collect()
    }
}
