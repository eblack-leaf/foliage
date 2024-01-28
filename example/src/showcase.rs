use foliage::color::Color;
use foliage::compositor::segment::{ResponsiveSegment, Segment};
use foliage::compositor::ViewHandle;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::IconId;
use foliage::prebuilt::button::{Button, ButtonArgs, ButtonStyle};
use foliage::text::{MaxCharacters, TextValue};

pub(crate) struct Showcase {}
impl Leaf for Showcase {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        let handle = ViewHandle::new(0, 0);
        elm.add_view_scene_binding::<Button, ()>(
            handle,
            ButtonArgs::new(
                ButtonStyle::Ring,
                TextValue::new("ring"),
                MaxCharacters(4),
                IconId::new(BundledIcon::Copy),
                Color::CYAN_MEDIUM,
                Color::OFF_BLACK,
            ),
            ResponsiveSegment::at(0).all(Segment::from_coords(
                "x:0.1|offset:20|min:25|max:50|l.mb:0.05|l.tb:0.05|max(l.tb):100",
                "y:0.1",
                "w:0.4",
                "h:0.2",
            )),
            (),
        );
    }
}