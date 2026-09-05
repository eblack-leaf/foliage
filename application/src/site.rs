//! The app: what it holds between frames, and what it does in one.

use foliage::{Grove, Pollen, Root};

/// The app.
///
/// Whatever it keeps is its own. Nothing here is handed to the engine, and the engine has no way to
/// reach it: the value stays on this side and touches the tree only through the [`Grove`] it is
/// lent for the length of a frame.
pub(crate) struct Site;

impl Root for Site {
    fn take_root(_grove: &mut Grove) -> Self {
        todo!("grow the tree")
    }

    fn frame(&mut self, _grove: &mut Grove, _pollen: Pollen) {
        todo!("read the frame, and write what follows from it")
    }
}
