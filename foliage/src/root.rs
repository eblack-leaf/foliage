use crate::grove::Grove;
use crate::pollen::Pollen;

/// An app, from the engine's side: what it is once grown, and what it does each frame.
///
/// Nothing here is handed to the engine and the engine has no way to reach it. The value stays on
/// this side, is borrowed for the length of a frame, and touches the tree only through the
/// [`Grove`] it is lent.
pub trait Root: Sized + 'static {
    /// Grows the tree and returns the app.
    ///
    /// Called once, inside the first frame, with a `Grove` no different from any other frame's:
    /// ops issued here land in the same frame, and a [`Leaf`](crate::Leaf) it hands back is usable
    /// immediately.
    fn take_root(grove: &mut Grove) -> Self;

    /// This frame: what happened, and what to do about it.
    ///
    /// Called once per frame, after the engine has settled and before anything is drawn. Ops
    /// issued here are drained in the order written as soon as it returns, so nothing an app
    /// queues can land while it is still running.
    fn frame(&mut self, grove: &mut Grove, pollen: Pollen);
}

/// The app as the grove holds it: one thing to call per frame, with the concrete [`Root`] type
/// erased.
pub(crate) trait Rooted {
    fn frame(&mut self, grove: &mut Grove, pollen: Pollen);
}

/// A registered root, before and after it has taken.
pub(crate) struct Registered<R: Root>(pub(crate) Option<R>);

impl<R: Root> Registered<R> {
    pub(crate) fn new() -> Self {
        Self(None)
    }
}

impl<R: Root> Rooted for Registered<R> {
    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        let root = match &mut self.0 {
            Some(root) => root,
            slot => slot.insert(R::take_root(grove)),
        };
        root.frame(grove, pollen);
    }
}
