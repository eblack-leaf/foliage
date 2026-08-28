use crate::boundary::moss::Moss;
use crate::boundary::forest::Forest;

/// An app, from the engine's side: what it is once grown, and what it does each frame.
///
/// Registered by type with [`Foliage::root`](crate::Foliage::root), which is the only way an
/// app is handed control. [`take_root`](Root::take_root) runs inside the first frame, before
/// that frame's emissions are delivered, and returns the app itself -- so "not grown yet" is
/// never a state an app has to hold or check for.
///
/// Nothing here is handed to the engine and the engine has no way to reach it: the value
/// stays on this side, is only ever borrowed for the length of a frame, and touches the tree
/// exclusively through the [`Forest`] it is lent.
pub trait Root: Sized + 'static {
    /// Grows the tree and returns the app.
    ///
    /// Called once, at the first frame, with a `Forest` no different from any other frame's:
    /// commands issued here land in the same frame, and a [`Leaf`](crate::Leaf) it hands back
    /// is usable immediately.
    fn take_root(forest: &mut Forest) -> Self;
    /// This frame: what happened, and what to do about it.
    ///
    /// Called once per frame after the engine has settled and before anything is drawn.
    /// Emissions arrive in the order the frame collected them, and commands issued here are
    /// applied in the order written as soon as it returns.
    fn frame(&mut self, forest: &mut Forest, mosses: Vec<Moss>);
}

/// The app as `Foliage` holds it: one boxed thing to call per frame, with the concrete `Root`
/// type erased. `Foliage` cannot name that type -- it is the app's -- and does not need to.
pub(crate) trait Rooted {
    fn frame(&mut self, forest: &mut Forest<'_, '_>, mosses: Vec<Moss>);
}

/// A registered root before and after it has taken. The engine cannot build one at
/// registration time -- growing a tree needs a live `Forest`, which only exists inside a
/// frame -- so the first-frame check lives here, written once, instead of in every app.
pub(crate) struct Planted<R: Root>(Option<R>);

impl<R: Root> Planted<R> {
    pub(crate) fn new() -> Self {
        Self(None)
    }
}

impl<R: Root> Rooted for Planted<R> {
    fn frame(&mut self, forest: &mut Forest<'_, '_>, mosses: Vec<Moss>) {
        let root = match &mut self.0 {
            Some(root) => root,
            slot => slot.insert(R::take_root(forest)),
        };
        root.frame(forest, mosses);
    }
}
