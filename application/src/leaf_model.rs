use foliage::branch::{Branch, Tree};
#[derive(Clone)]
pub(crate) struct LeafModel {}
impl Branch for LeafModel {
    fn grow(self, tree: Tree) {
        todo!()
    }
}
