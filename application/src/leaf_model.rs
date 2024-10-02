use foliage::tree::Tree;
use foliage::twig::{Branch, Twig};

pub(crate) struct LeafModel {}
pub(crate) struct LeafModelHandle {
    // entity structure
}
impl Branch for LeafModel {
    type Handle = ();

    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        todo!()
    }
}
