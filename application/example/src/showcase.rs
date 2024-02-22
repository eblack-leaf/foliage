use crate::actual::Showcase;
use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::{Component, Resource};
use foliage::coordinate::area::Area;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::image::{Image, ImageId, ImageStorage};

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
pub(crate) struct ShowcaseMain {}
impl Leaf for ShowcaseMain {
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
        elm.enable_seed::<Showcase>();
        elm.navigate::<Showcase>();

        // elm.add_view_binding(
        //     START,
        //     Text::new(
        //         MaxCharacters(11),
        //         TextValue::new("circle"),
        //         THEME_COLOR::PLUS_TWO,
        //     ),
        //     ResponsiveSegment::base(Segment::new(
        //         7.near().to(8.far()),
        //         4.near().to(4.far()).minimum(30.0).maximum(40.0),
        //     ))
        //     .justify(Justify::Top)
        //     .without_portrait_mobile()
        //     .without_portrait_tablet(),
        //     (),
        // );
        // // elm.add_view_scene_binding(
        // //     START,
        // //     IconButton::new(
        // //         FeatherIcon::Copy,
        // //         ButtonStyle::Fill,
        // //         THEME_COLOR::BASE,
        // //         Color::BLACK,
        // //     ),
        // //     ResponsiveSegment::base(
        // //         2.near().to(3.far()).fixed(35.0),
        // //         5.near().to(5.far()).fixed(35.0),
        // //     )
        // //     .exception(
        // //         [Layout::PORTRAIT_MOBILE],
        // //         1.near().to(4.far()).fixed(35.0),
        // //         5.near().to(5.far()).fixed(35.0),
        // //     )
        // //     .justify(Justify::Top),
        // //     (),
        // // );
        // elm.add_view_binding(
        //     START,
        //     Text::new(MaxCharacters(11), TextValue::new("icon"), THEME_COLOR::BASE),
        //     ResponsiveSegment::base(Segment::new(
        //         4.near().to(5.far()),
        //         5.near().to(5.far()).minimum(30.0).maximum(40.0),
        //     ))
        //     .justify(Justify::Top)
        //     .without_portrait_mobile()
        //     .without_portrait_tablet(),
        //     (),
        // );
        // // elm.add_view_scene_binding(
        // //     START,
        // //     IconButton::new(
        // //         FeatherIcon::Copy,
        // //         ButtonStyle::Ring,
        // //         THEME_COLOR::PLUS_THREE,
        // //         Color::BLACK,
        // //     ),
        // //     ResponsiveSegment::base(
        // //         5.near().to(6.far()).fixed(35.0),
        // //         5.near().to(5.far()).fixed(35.0),
        // //     )
        // //     .exception(
        // //         [Layout::PORTRAIT_MOBILE],
        // //         5.near().to(8.far()).fixed(35.0),
        // //         5.near().to(6.far()).fixed(35.0),
        // //     )
        // //     .justify(Justify::Top),
        // //     (),
        // // );
        // elm.add_view_binding(
        //     START,
        //     Text::new(
        //         MaxCharacters(11),
        //         TextValue::new("icon"),
        //         THEME_COLOR::PLUS_THREE,
        //     ),
        //     ResponsiveSegment::base(Segment::new(
        //         7.near().to(8.far()),
        //         5.near().to(5.far()).minimum(30.0).maximum(40.0),
        //     ))
        //     .justify(Justify::Top)
        //     .without_portrait_mobile()
        //     .without_portrait_tablet(),
        //     (),
        // );
    }
}
#[derive(Component, Copy, Clone)]
struct TestHook();
