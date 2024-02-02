use foliage::elm::leaf::Leaf;
use foliage::elm::Elm;
use foliage::prebuilt::icon_text::IconText;

pub(crate) struct Home {}
impl Leaf for Home {
    type SetDescriptor = ();

    fn attach(elm: &mut Elm) {
        elm.add_view_scene_binding::<IconText, ()>();
    }
}
