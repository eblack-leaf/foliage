use foliage::branch::{Branch, Twig};

#[derive(Clone)]
pub(crate) struct Home {}
impl Twig for Home {
    fn grow(self, mut branch: Branch) {}
}
