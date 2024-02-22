use crate::button_tree::ButtonTree;
use foliage::bevy_ecs;
use foliage::bevy_ecs::entity::Entity;
use foliage::bevy_ecs::prelude::{Component, Resource};
use foliage::color::monochromatic::{AquaMarine as THEME_COLOR, Monochromatic};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::{ElementStyle, Elm};
use foliage::icon::FeatherIcon;
use foliage::image::{Image, ImageId, ImageStorage};
use foliage::layout::Layout;
use foliage::r_scenes::button::Button;
use foliage::r_scenes::icon_text::IconText;
use foliage::segment::{Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{MaxCharacters, Text, TextValue};

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
    #[icon(path = "icons/tag.icon", opt = FeatherIcon::Tag)]
    _command_id: AssetKey,
}
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
        elm.configure_view_grid(START, MacroGrid::new(8, 6));
        elm.add_view_scene_binding(
            START,
            IconText::new(
                FeatherIcon::Menu,
                Color::GREY,
                MaxCharacters(11),
                TextValue::new("button.rs"),
                Color::GREY,
            ),
            ResponsiveSegment::base(Segment::new(
                3.near().to(6.far()),
                1.near().to(1.far()).maximum(60.0),
            ))
            .justify(Justify::Top),
            (),
        );
        elm.add_view_scene_binding(
            START,
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                ElementStyle::fill(),
                THEME_COLOR::BASE,
                Color::BLACK,
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(3.far()).minimum(115.0).maximum(350.0),
                2.near().to(2.far()).minimum(30.0).maximum(40.0),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    1.near().to(4.far()),
                    2.near().to(2.far()).minimum(30.0).maximum(40.0),
                ),
            )
            .justify(Justify::Top),
            TestHook(),
        );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                TextValue::new("base"),
                THEME_COLOR::MINUS_THREE,
            ),
            ResponsiveSegment::base(Segment::new(
                4.near().to(5.far()),
                2.near().to(2.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
            (),
        );
        elm.add_view_scene_binding(
            START,
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                ElementStyle::fill(),
                THEME_COLOR::MINUS_ONE,
                Color::BLACK,
            ),
            ResponsiveSegment::base(Segment::new(
                5.near().to(6.far()).minimum(115.0).maximum(350.0),
                2.near().to(2.far()).minimum(30.0).maximum(40.0),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    5.near().to(8.far()),
                    2.near().to(2.far()).minimum(30.0).maximum(40.0),
                ),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_binding(
            START,
            Text::new(MaxCharacters(11), TextValue::new("base"), THEME_COLOR::BASE),
            ResponsiveSegment::base(Segment::new(
                7.near().to(8.far()),
                2.near().to(2.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
            (),
        );
        elm.add_view_scene_binding(
            START,
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                ElementStyle::fill(),
                THEME_COLOR::MINUS_TWO,
                Color::BLACK,
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(3.far()).minimum(115.0).maximum(350.0),
                3.near().to(3.far()).minimum(30.0).maximum(40.0),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    1.near().to(4.far()),
                    3.near().to(3.far()).minimum(30.0).maximum(40.0),
                ),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                TextValue::new("text"),
                THEME_COLOR::PLUS_THREE,
            ),
            ResponsiveSegment::base(Segment::new(
                4.near().to(5.far()),
                3.near().to(3.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
            (),
        );
        elm.add_view_scene_binding(
            START,
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                ElementStyle::fill(),
                THEME_COLOR::BASE,
                Color::BLACK,
            ),
            ResponsiveSegment::base(Segment::new(
                5.near().to(6.far()).minimum(115.0).maximum(350.0),
                3.near().to(3.far()).minimum(30.0).maximum(40.0),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    5.near().to(8.far()),
                    3.near().to(3.far()).minimum(30.0).maximum(40.0),
                ),
            )
            .justify(Justify::Top),
            (),
        );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                TextValue::new("text"),
                THEME_COLOR::PLUS_ONE,
            ),
            ResponsiveSegment::base(Segment::new(
                7.near().to(8.far()),
                3.near().to(3.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
            (),
        );
        // elm.add_view_scene_binding(
        //     START,
        //     CircleButton::new(
        //         FeatherIcon::Copy,
        //         ButtonStyle::Fill,
        //         THEME_COLOR::MINUS_ONE,
        //         Color::BLACK,
        //     ),
        //     ResponsiveSegment::base(
        //         2.near().to(3.far()).fixed(40.0),
        //         4.near().to(4.far()).fixed(40.0),
        //     )
        //     .exception(
        //         [Layout::PORTRAIT_MOBILE],
        //         1.near().to(4.far()).fixed(40.0),
        //         4.near().to(4.far()).fixed(40.0),
        //     )
        //     .justify(Justify::Top),
        //     (),
        // );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                TextValue::new("circle"),
                THEME_COLOR::MINUS_ONE,
            ),
            ResponsiveSegment::base(Segment::new(
                4.near().to(5.far()),
                4.near().to(4.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
            (),
        );
        // elm.add_view_scene_binding(
        //     START,
        //     CircleButton::new(
        //         FeatherIcon::Copy,
        //         ButtonStyle::Ring,
        //         THEME_COLOR::PLUS_TWO,
        //         Color::BLACK,
        //     ),
        //     ResponsiveSegment::base(
        //         5.near().to(6.far()).fixed(40.0),
        //         4.near().to(4.far()).fixed(40.0),
        //     )
        //     .exception(
        //         [Layout::PORTRAIT_MOBILE],
        //         5.near().to(8.far()).fixed(40.0),
        //         4.near().to(4.far()).fixed(40.0),
        //     )
        //     .justify(Justify::Top),
        //     (),
        // );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                TextValue::new("circle"),
                THEME_COLOR::PLUS_TWO,
            ),
            ResponsiveSegment::base(Segment::new(
                7.near().to(8.far()),
                4.near().to(4.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
            (),
        );
        // elm.add_view_scene_binding(
        //     START,
        //     IconButton::new(
        //         FeatherIcon::Copy,
        //         ButtonStyle::Fill,
        //         THEME_COLOR::BASE,
        //         Color::BLACK,
        //     ),
        //     ResponsiveSegment::base(
        //         2.near().to(3.far()).fixed(35.0),
        //         5.near().to(5.far()).fixed(35.0),
        //     )
        //     .exception(
        //         [Layout::PORTRAIT_MOBILE],
        //         1.near().to(4.far()).fixed(35.0),
        //         5.near().to(5.far()).fixed(35.0),
        //     )
        //     .justify(Justify::Top),
        //     (),
        // );
        elm.add_view_binding(
            START,
            Text::new(MaxCharacters(11), TextValue::new("icon"), THEME_COLOR::BASE),
            ResponsiveSegment::base(Segment::new(
                4.near().to(5.far()),
                5.near().to(5.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
            (),
        );
        // elm.add_view_scene_binding(
        //     START,
        //     IconButton::new(
        //         FeatherIcon::Copy,
        //         ButtonStyle::Ring,
        //         THEME_COLOR::PLUS_THREE,
        //         Color::BLACK,
        //     ),
        //     ResponsiveSegment::base(
        //         5.near().to(6.far()).fixed(35.0),
        //         5.near().to(5.far()).fixed(35.0),
        //     )
        //     .exception(
        //         [Layout::PORTRAIT_MOBILE],
        //         5.near().to(8.far()).fixed(35.0),
        //         5.near().to(6.far()).fixed(35.0),
        //     )
        //     .justify(Justify::Top),
        //     (),
        // );
        elm.add_view_binding(
            START,
            Text::new(
                MaxCharacters(11),
                TextValue::new("icon"),
                THEME_COLOR::PLUS_THREE,
            ),
            ResponsiveSegment::base(Segment::new(
                7.near().to(8.far()),
                5.near().to(5.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
            (),
        );
        elm.view_trigger::<TestHook, ButtonTree>();
    }
}
#[derive(Component, Copy, Clone)]
struct SampleHook(Entity);
#[derive(Component, Copy, Clone)]
struct TestHook();
