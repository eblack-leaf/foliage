use foliage::aspect_ratio_image::AspectRatioImage;
use foliage::asset::AssetKey;
use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::{Component, Resource};
use foliage::button::{Button, ButtonStyle};
use foliage::circle_progress_bar::CircleProgressBar;
use foliage::color::Color;
use foliage::compositor::segment::{Grid, ResponsiveSegment, SegmentUnitDesc};
use foliage::compositor::ViewHandle;
use foliage::coordinate::area::Area;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::{Elm, InteractionHandlerTrigger};
use foliage::icon::FeatherIcon;
use foliage::image::{Image, ImageId, ImageStorage};
use foliage::progress_bar::ProgressBar;
use foliage::text::{MaxCharacters, TextValue};
use foliage::texture::factors::Progress;

#[foliage::assets(crate::Engen, "../assets/", "/foliage/demo/assets/")]
#[derive(Resource, Clone)]
pub(crate) struct Assets {
    #[bytes(path = "something.dat", group = f_asset)]
    _something: AssetKey,
    #[bytes(path = "img.png", group = f_asset)]
    image_id: AssetKey,
    #[icon(path = "icons/copy.gatl", opt = FeatherIcon::Copy)]
    _copy_id: AssetKey,
    #[icon(path = "icons/command.gatl", opt = FeatherIcon::Command)]
    _command_id: AssetKey,
}
pub(crate) struct Showcase {}
impl Leaf for Showcase {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        let handle = ViewHandle::new(0, 0);
        elm.configure_view_grid(handle, Grid::new(4, 8));
        Elm::remove_web_element("loading");
        elm.container().spawn(Image::storage(
            ImageId(0),
            ImageStorage::some(Area::from((700, 700))),
        ));
        let assets = Assets::proc_gen_load(elm);
        elm.on_fetch(*assets.f_asset.get(1).unwrap(), |data, cmd| {
            cmd.spawn(Image::fill(ImageId(0), data));
        });
        elm.container().insert_resource(assets);
        elm.add_view_scene_binding(
            handle,
            AspectRatioImage::new(ImageId(0), (651, 454)),
            ResponsiveSegment::base(1.near().to(4.far()), 4.near().to(8.far())),
            (),
        );
        elm.add_view_scene_binding(
            handle,
            Button::new(
                ButtonStyle::Ring,
                TextValue::new("ring"),
                MaxCharacters(4),
                FeatherIcon::Copy.id(),
                Color::CYAN_MEDIUM,
                Color::OFF_BLACK,
            ),
            ResponsiveSegment::base(
                1.near().to(2.far()).minimum(120.0).maximum(300.0),
                1.near().to(1.far()).minimum(35.0).maximum(45.0),
            ),
            (),
        );
        elm.add_view_scene_binding(
            handle,
            Button::new(
                ButtonStyle::Fill,
                TextValue::new("fill"),
                MaxCharacters(4),
                FeatherIcon::Command.id(),
                Color::CYAN_MEDIUM,
                Color::OFF_BLACK,
            ),
            ResponsiveSegment::base(
                3.near().to(4.far()).minimum(120.0).maximum(300.0),
                1.near().to(1.far()).minimum(35.0).maximum(45.0),
            ),
            (),
        );
        elm.add_view_scene_binding(
            handle,
            ProgressBar::new(
                Progress::new(0.0, 0.67),
                Color::CYAN_MEDIUM,
                Color::GREY_DARK,
            ),
            ResponsiveSegment::base(1.near().to(2.far()), 3.far().to(4.fixed())),
            (),
        );
        elm.add_view_scene_binding(
            handle,
            CircleProgressBar::new(
                Progress::new(0.0, 0.67),
                Color::CYAN_MEDIUM,
                Color::GREY_DARK,
            ),
            ResponsiveSegment::base(1.near().to(44.fixed()), 3.near().to(44.fixed())),
            (),
        );
        elm.view_trigger::<TestHook>(InteractionHandlerTrigger::Active, |_, cv| {
            cv.change_view(ViewHandle::new(0, 1));
        });
    }
}

#[derive(Component, Copy, Clone)]
struct TestHook();
