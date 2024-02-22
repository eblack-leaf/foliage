use crate::button_tree::{OtherHook, SampleHook, ShowcaseSeed};
use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::{Commands, Component, Resource};
use foliage::coordinate::area::Area;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::image::{Image, ImageId, ImageStorage};
use foliage::r_scenes::button::Button;
use foliage::segment::ResponsiveSegment;
use foliage::text::Text;
use foliage::tree::{BranchHandle, BranchSet};

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
        elm.enable_seed::<ShowcaseSeed>();
        elm.navigate::<ShowcaseSeed>();
        elm.add_interaction_handler::<SampleHook, Commands>(|sh, cmd| {
            cmd.spawn(BranchSet(BranchHandle(0), sh.0));
            cmd.spawn(BranchSet(BranchHandle(1), sh.0));
            cmd.spawn(BranchSet(BranchHandle(2), sh.0));
            if !sh.0 {
                cmd.spawn(BranchSet(BranchHandle(3), false));
                cmd.spawn(BranchSet(BranchHandle(4), false));
                cmd.spawn(BranchSet(BranchHandle(5), false));
            }
            sh.0 = !sh.0;
        });
        elm.add_interaction_handler::<OtherHook, Commands>(|sh, cmd| {
            cmd.spawn(BranchSet(BranchHandle(3), sh.0));
            cmd.spawn(BranchSet(BranchHandle(4), sh.0));
            cmd.spawn(BranchSet(BranchHandle(5), sh.0));
            sh.0 = !sh.0;
        });
        elm.enable_conditional::<Text>();
        elm.enable_conditional::<ResponsiveSegment>();
        elm.enable_conditional_scene::<Button>();
        elm.enable_conditional::<OtherHook>();
        // style starts below

        // elm.add_view_binding(
        //     START,
        //     Text::new(
        //         MaxCharacters(11),
        //         TextValue::new("text"),
        //         THEME_COLOR::PLUS_ONE,
        //     ),
        //     ResponsiveSegment::base(Segment::new(
        //         7.near().to(8.far()),
        //         3.near().to(3.far()).minimum(30.0).maximum(40.0),
        //     ))
        //     .justify(Justify::Top)
        //     .without_portrait_mobile()
        //     .without_portrait_tablet(),
        //     (),
        // );
        // // elm.add_view_scene_binding(
        // //     START,
        // //     CircleButton::new(
        // //         FeatherIcon::Copy,
        // //         ButtonStyle::Fill,
        // //         THEME_COLOR::MINUS_ONE,
        // //         Color::BLACK,
        // //     ),
        // //     ResponsiveSegment::base(
        // //         2.near().to(3.far()).fixed(40.0),
        // //         4.near().to(4.far()).fixed(40.0),
        // //     )
        // //     .exception(
        // //         [Layout::PORTRAIT_MOBILE],
        // //         1.near().to(4.far()).fixed(40.0),
        // //         4.near().to(4.far()).fixed(40.0),
        // //     )
        // //     .justify(Justify::Top),
        // //     (),
        // // );
        // elm.add_view_binding(
        //     START,
        //     Text::new(
        //         MaxCharacters(11),
        //         TextValue::new("circle"),
        //         THEME_COLOR::MINUS_ONE,
        //     ),
        //     ResponsiveSegment::base(Segment::new(
        //         4.near().to(5.far()),
        //         4.near().to(4.far()).minimum(30.0).maximum(40.0),
        //     ))
        //     .justify(Justify::Top)
        //     .without_portrait_mobile()
        //     .without_portrait_tablet(),
        //     (),
        // );
        // // elm.add_view_scene_binding(
        // //     START,
        // //     CircleButton::new(
        // //         FeatherIcon::Copy,
        // //         ButtonStyle::Ring,
        // //         THEME_COLOR::PLUS_TWO,
        // //         Color::BLACK,
        // //     ),
        // //     ResponsiveSegment::base(
        // //         5.near().to(6.far()).fixed(40.0),
        // //         4.near().to(4.far()).fixed(40.0),
        // //     )
        // //     .exception(
        // //         [Layout::PORTRAIT_MOBILE],
        // //         5.near().to(8.far()).fixed(40.0),
        // //         4.near().to(4.far()).fixed(40.0),
        // //     )
        // //     .justify(Justify::Top),
        // //     (),
        // // );
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
        elm.view_trigger::<TestHook, ShowcaseSeed>();
    }
}
#[derive(Component, Copy, Clone)]
struct TestHook();