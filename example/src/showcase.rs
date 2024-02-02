use crate::Engen;
use foliage::color::Color;
use foliage::compositor::layout::Layout;
use foliage::compositor::segment::{ResponsiveSegment, SegmentUnitNumber};
use foliage::compositor::ViewHandle;
use foliage::coordinate::area::Area;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::icon::{FeatherIcon, Icon};
use foliage::image::{Image, ImageId, ImageStorage};
use foliage::prebuilt::aspect_ratio_image::{AspectRatioImage, AspectRatioImageArgs};
use foliage::prebuilt::button::{Button, ButtonArgs, ButtonStyle};
use foliage::prebuilt::circle_progress_bar::CircleProgressBar;
use foliage::prebuilt::progress_bar::{ProgressBar, ProgressBarArgs};
use foliage::text::{MaxCharacters, TextValue};
use foliage::texture::factors::Progress;
use foliage::{icon_fetcher, load_native_asset};

pub(crate) struct Showcase {}
impl Leaf for Showcase {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        Elm::remove_web_element("loading");
        elm.container().spawn(Image::storage(
            ImageId(0),
            ImageStorage::some(Area::from((700, 700))),
        ));
        #[cfg(target_family = "wasm")]
        let img_id = elm.load_remote_asset::<Engen>("/foliage/demo/assets/img.png");
        load_native_asset!(elm, img_id, "../assets/img.png");
        elm.on_fetch(img_id, |data, cmd| {
            cmd.spawn(Image::fill(ImageId(0), data));
        });
        let handle = ViewHandle::new(0, 0);
        elm.add_view_scene_binding::<AspectRatioImage, ()>(
            handle,
            AspectRatioImageArgs::new(ImageId(0), (651, 454)),
            ResponsiveSegment::new(
                0.1.relative(),
                0.3.relative(),
                0.8.relative().max(651.0).min(300.0),
                0.6.relative().max(454.0),
            ),
            (),
        );
        elm.load_remote_icon::<Engen>(
            icon_fetcher!(FeatherIcon::Copy),
            "/foliage/demo/assets/icons/copy.gatl",
        );
        elm.load_remote_icon::<Engen>(
            icon_fetcher!(FeatherIcon::Command),
            "/foliage/demo/assets/icons/command.gatl",
        );
        elm.add_view_scene_binding::<Button, ()>(
            handle,
            ButtonArgs::new(
                ButtonStyle::Ring,
                TextValue::new("ring"),
                MaxCharacters(4),
                FeatherIcon::Copy.id(),
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
                FeatherIcon::Command.id(),
                Color::CYAN_MEDIUM,
                Color::OFF_BLACK,
            ),
            ResponsiveSegment::new(
                0.525.relative(),
                0.05.relative(),
                0.4.relative(),
                40.fixed(),
            )
            .x_exception(Layout::LANDSCAPE_MOBILE, 0.3.relative()),
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