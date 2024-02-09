use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::{Component, Resource};
use foliage::button::{Button, ButtonStyle};
use foliage::circle_button::CircleButton;
use foliage::color::{Color, Monochromatic, Orange};
use foliage::compositor::segment::{Grid, Justify, ResponsiveSegment, SegmentUnitDesc};
use foliage::compositor::ViewHandle;
use foliage::coordinate::area::Area;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::{Elm, InteractionHandlerTrigger};
use foliage::icon::FeatherIcon;
use foliage::icon_button::IconButton;
use foliage::icon_text::IconText;
use foliage::image::{Image, ImageId, ImageStorage};
use foliage::rectangle::Rectangle;
use foliage::text::{FontSize, MaxCharacters, Text, TextValue};
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
    #[icon(path = "icons/menu.icon", opt = FeatherIcon::Menu)]
    _command_id: AssetKey,
}
pub(crate) const START: ViewHandle = ViewHandle::new(0, 0);
pub(crate) const TWO: ViewHandle = ViewHandle::new(0, 0);
pub(crate) struct Showcase {}
impl Leaf for Showcase {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        // general setup
        Elm::remove_web_element("loading");
        let assets = Assets::proc_gen_load(elm);
        elm.on_fetch(*assets.f_asset.get(1).unwrap(), |data, cmd| {
            cmd.spawn(Image::fill(ImageId(0), data));
        });
        elm.container().spawn(Image::storage(
            ImageId(0),
            ImageStorage::some(Area::from((700, 700))),
        ));
        elm.container().insert_resource(assets);
        // style starts below
        elm.configure_view_grid(START, Grid::new(8, 6));
        elm.add_view_scene_binding(
            START,
            IconText::new(
                FeatherIcon::Menu,
                MaxCharacters(6),
                TextValue::new("Button"),
                Color::WHITE,
                Color::GREY,
            ),
            ResponsiveSegment::base(
                1.near().to(4.far()),
                1.near().to(1.far()).minimum(35.0).maximum(45.0),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_binding(
            START,
            Rectangle::new(Area::default(), Color::WHITE, Progress::full()),
            ResponsiveSegment::base(1.near().to(4.near()), 1.near().offset(50.0).to(4.fixed()))
                .without_landscape_mobile(),
            (),
        );
        elm.add_view_scene_binding(
            START,
            Button::new(
                ButtonStyle::Fill,
                TextValue::new("copy"),
                MaxCharacters(4),
                FeatherIcon::Copy,
                Orange::MINUS_TWO,
                Color::BLACK,
            ),
            ResponsiveSegment::base(
                1.near().to(4.near()).minimum(135.0).maximum(217.0),
                2.near().to(2.far()).minimum(35.0).maximum(40.0),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                FontSize(24),
                TextValue::new("Fill"),
                Color::WHITE,
            ),
            ResponsiveSegment::base(
                4.far().to(8.near()),
                2.near().to(2.far()).minimum(35.0).maximum(40.0),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_scene_binding(
            START,
            Button::new(
                ButtonStyle::Ring,
                TextValue::new("copy"),
                MaxCharacters(4),
                FeatherIcon::Copy,
                Orange::MINUS_TWO,
                Color::BLACK,
            ),
            ResponsiveSegment::base(
                1.near().to(4.near()).minimum(135.0).maximum(217.0),
                3.near().to(3.far()).minimum(35.0).maximum(40.0),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                FontSize(24),
                TextValue::new("Ring"),
                Color::WHITE,
            ),
            ResponsiveSegment::base(
                4.far().to(8.near()),
                3.near().to(3.far()).minimum(35.0).maximum(40.0),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_scene_binding(
            START,
            CircleButton::new(
                FeatherIcon::Copy,
                ButtonStyle::Fill,
                Orange::PLUS_ONE,
                Color::BLACK,
            ),
            ResponsiveSegment::base(
                1.near().to(4.near()).minimum(40.0).maximum(40.0),
                4.near().to(4.far()).minimum(40.0).maximum(40.0),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                FontSize(24),
                TextValue::new("Circle"),
                Color::WHITE,
            ),
            ResponsiveSegment::base(
                4.far().to(8.near()),
                4.near().to(4.far()).minimum(35.0).maximum(40.0),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_scene_binding(
            START,
            IconButton::new(
                FeatherIcon::Copy,
                ButtonStyle::Ring,
                Orange::BASE,
                Color::BLACK,
            ),
            ResponsiveSegment::base(
                1.near().to(4.near()).minimum(35.0).maximum(35.0),
                5.near().to(5.far()).minimum(35.0).maximum(35.0),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                FontSize(24),
                TextValue::new("Icon-Only"),
                Color::WHITE,
            ),
            ResponsiveSegment::base(
                4.far().to(8.near()),
                5.near().to(5.far()).minimum(35.0).maximum(40.0),
            )
            .justify(Justify::Top),
            (),
        );
        elm.view_trigger::<TestHook>(InteractionHandlerTrigger::Active, |_, cv| {
            cv.change_view(TWO);
        });
    }
}

#[derive(Component, Copy, Clone)]
struct TestHook();