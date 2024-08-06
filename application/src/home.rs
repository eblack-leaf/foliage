use foliage::action::{Actionable, ElmHandle};

#[derive(Clone)]
pub(crate) struct Home {}
impl Actionable for Home {
    fn apply(self, mut handle: ElmHandle) {}
}
