use foliage::color::Color;
use foliage::compositor::segment::{ResponsiveSegment, SegmentUnitNumber};
use foliage::compositor::ViewHandle;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::IconId;
use foliage::prebuilt::button::{Button, ButtonArgs, ButtonStyle};
use foliage::prebuilt::circle_progress_bar::CircleProgressBar;
use foliage::prebuilt::progress_bar::{ProgressBar, ProgressBarArgs};
use foliage::text::{MaxCharacters, TextValue};
use foliage::texture::factors::Progress;

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
            ResponsiveSegment::new(
                0.075.relative(),
                0.05.relative(),
                0.4.relative(),
                40.fixed(),
            ),
            (),
        );
        elm.add_view_scene_binding::<Button, ()>(
            handle,
            ButtonArgs::new(
                ButtonStyle::Fill,
                TextValue::new("fill"),
                MaxCharacters(4),
                IconId::new(BundledIcon::Copy),
                Color::CYAN_MEDIUM,
                Color::OFF_BLACK,
            ),
            ResponsiveSegment::new(
                0.525.relative(),
                0.05.relative(),
                0.4.relative(),
                40.fixed(),
            ),
            (),
        );
        elm.add_view_scene_binding::<ProgressBar, ()>(
            handle,
            ProgressBarArgs::new(
                Progress::new(0.0, 0.67),
                Color::CYAN_MEDIUM,
                Color::GREY_DARK,
            ),
            ResponsiveSegment::new(0.1.relative(), 0.2.relative(), 0.4.relative(), 4.fixed()),
            (),
        );
        elm.add_view_scene_binding::<CircleProgressBar, ()>(
            handle,
            ProgressBarArgs::new(
                Progress::new(0.0, 0.67),
                Color::CYAN_MEDIUM,
                Color::GREY_DARK,
            ),
            ResponsiveSegment::new(
                0.3.relative().offset(-24.0),
                0.2.relative().offset(24.0),
                48.fixed(),
                48.fixed(),
            ),
            (),
        );
    }
}
