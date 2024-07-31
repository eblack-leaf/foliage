use foliage::action::{Actionable, ElmHandle};

use crate::image::ImageKeys;

#[derive(Clone)]
pub(crate) struct Home {}
impl Actionable for Home {
    fn apply(self, mut handle: ElmHandle) {
        let leaf = handle.get_resource::<ImageKeys>().leaf;
        // handle.add_element(
        //     "leaf-img",
        //     GridPlacement::new(0.percent().to(100.percent()), 0.percent().to(50.percent())),
        //     10,
        //     None,
        //     |e| {
        //         e.give_attr(OnRetrieve::new(leaf, |b| {
        //             Image::new(ImageHandles::Leaf, b).inherit_aspect_ratio()
        //         }));
        //     },
        // );
    }
}
