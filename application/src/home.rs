use foliage::branch::{Branch, Tree};

#[derive(Clone)]
pub(crate) struct Home {}
impl Branch for Home {
    fn grow(self, mut tree: Tree) {}
}
