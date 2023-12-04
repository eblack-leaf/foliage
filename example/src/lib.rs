use foliage::bevy_ecs;
use foliage::bevy_ecs::change_detection::Res;
use foliage::bevy_ecs::entity::Entity;
use foliage::bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use foliage::bevy_ecs::system::{Local, Resource};
use foliage::circle::{Circle, CircleMipLevel, CircleStyle, Diameter};
use foliage::color::Color;
use foliage::coordinate::CoordinateLeaf;
use foliage::elm::{Elm, Leaf, SystemSets};
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::{Icon, IconId, IconScale};
use foliage::panel::{Panel, PanelStyle};
use foliage::rectangle::Rectangle;
use foliage::scene::Scene;
use foliage::text::{FontSize, MaxCharacters, Text, TextValue};
use foliage::texture::factors::Progress;
use foliage::window::WindowDescriptor;
use foliage::{AndroidInterface, Foliage};

pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((411, 913)),
        )
        .with_renderleaf::<Panel>()
        .with_renderleaf::<Circle>()
        .with_renderleaf::<Rectangle>()
        .with_renderleaf::<Icon>()
        .with_renderleaf::<Text>()
        .with_leaf::<CoordinateLeaf>()
        .with_leaf::<Tester>()
        .with_android_interface(android_interface)
        .run();
}

struct Tester;
#[derive(Resource)]
pub(crate) struct ButtonScene(pub(crate) Scene<i32>);
impl ButtonScene {
    pub(crate) fn new() -> Self {
        let scene = Scene::new()
            .with_node(0, |android_offset: &i32, cmd| -> Entity {
                cmd.spawn(Text::new(
                    (138, 475 + *android_offset).into(),
                    MaxCharacters(45),
                    1.into(),
                    FontSize(14),
                    TextValue::new("Stats - User"),
                    Color::GREY.into(),
                ))
                .id()
            })
            .with_node(1, |android_offset: &i32, cmd| -> Entity {
                cmd.spawn(Icon::new(
                    IconId::new(BundledIcon::CloudSnow),
                    (250, 475 + android_offset).into(),
                    IconScale::Twenty,
                    4.into(),
                    Color::GREY_MEDIUM.into(),
                ))
                .id()
            });
        Self(scene)
    }
}
fn spawn_button_tree(
    mut first_run: Local<bool>,
    button_scene: Res<ButtonScene>,
    mut cmd: Commands,
) {
    if !*first_run {
        let _entities = button_scene.0.spawn_with(&mut cmd);
        *first_run = true;
    }
}
impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        #[cfg(any(not(target_os = "android"), target_arch = "x86_64"))]
        let android_offset = 0;
        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
        let android_offset = 50;
        let mut button_tree = ButtonScene::new();
        button_tree.0.set_args(android_offset);
        elm.job.container.insert_resource(button_tree);
        elm.job
            .main()
            .add_systems((spawn_button_tree.in_set(SystemSets::Spawn),));
        elm.job.container.spawn(Rectangle::new(
            (138, 500 + android_offset).into(),
            (200, 2).into(),
            6.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Rectangle::new(
            (178, 532 + android_offset).into(),
            (200, 10).into(),
            6.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Rectangle::new(
            (178, 532 + android_offset).into(),
            (200, 10).into(),
            5.into(),
            Color::GREEN_DARK.into(),
            Progress::new(0.0, 0.85),
        ));
        elm.job.container.spawn(Rectangle::new(
            (178, 556 + android_offset).into(),
            (200, 10).into(),
            6.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Rectangle::new(
            (178, 556 + android_offset).into(),
            (200, 10).into(),
            5.into(),
            Color::GREEN_MEDIUM.into(),
            Progress::new(0.0, 0.65),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 194 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::NinetySix),
            6.into(),
            Color::from(Color::GREY_DARK).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 194 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::NinetySix),
            5.into(),
            Color::from(Color::GREEN_MEDIUM).with_alpha(1.0),
            Progress::new(0.0, 0.3),
        ));
        elm.job.container.spawn(Rectangle::new(
            (118, 178 + android_offset).into(),
            (4, 128).into(),
            6.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 484 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::NinetySix),
            6.into(),
            Color::from(Color::GREY_DARK).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (10, 484 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::NinetySix),
            5.into(),
            Color::from(Color::LIGHT_GREEN).with_alpha(1.0),
            Progress::new(0.0, 0.85),
        ));
        elm.job.container.spawn(Rectangle::new(
            (118, 468 + android_offset).into(),
            (4, 128).into(),
            6.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::fill(),
            (158, 530 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::Twelve),
            5.into(),
            Color::from(Color::GREEN_MEDIUM).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::fill(),
            (158, 554 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::Twelve),
            5.into(),
            Color::from(Color::GREEN_MEDIUM).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::ring(),
            (138, 210 + android_offset).into(),
            (140, 50).into(),
            4.into(),
            Color::GREEN.into(),
        ));
        elm.job.container.spawn(Text::new(
            (154, 225 + android_offset).into(),
            MaxCharacters(45),
            4.into(),
            FontSize(14),
            TextValue::new("BOOKMARK"),
            Color::GREEN.into(),
        ));
        elm.job.container.spawn(Icon::new(
            IconId::new(BundledIcon::Bookmark),
            (240, 225 + android_offset).into(),
            IconScale::Twenty,
            3.into(),
            Color::GREEN.into(),
        ));
        elm.job.container.spawn(Rectangle::new(
            (148, 280 + android_offset).into(),
            (120, 6).into(),
            7.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Rectangle::new(
            (148, 280 + android_offset).into(),
            (120, 6).into(),
            6.into(),
            Color::GREEN_DARK.into(),
            Progress::new(0.0, 0.25),
        ));
        elm.job.container.spawn(Rectangle::new(
            (178, 277 + android_offset).into(),
            (12, 12).into(),
            5.into(),
            Color::from(Color::GREEN).with_alpha(1.0),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::ring(),
            (10, 10 + android_offset).into(),
            (80, 12).into(),
            3.into(),
            Color::OFF_WHITE.into(),
        ));
        elm.job.container.spawn(Circle::new(
            CircleStyle::ring(),
            (100, 10 + android_offset).into(),
            Diameter::from_mip_level(CircleMipLevel::Twelve),
            4.into(),
            Color::GREY_DARK.into(),
            Progress::new(0.0, 1.0),
        ));
        elm.job.container.spawn(Icon::new(
            IconId::new(BundledIcon::Play),
            (254, 2 + android_offset).into(),
            IconScale::Eighty,
            4.into(),
            Color::GREY_MEDIUM.into(),
        ));
        elm.job.container.spawn(Icon::new(
            IconId::new(BundledIcon::SkipForward),
            (334, 2 + android_offset).into(),
            IconScale::Forty,
            4.into(),
            Color::GREY_MEDIUM.into(),
        ));
        elm.job.container.spawn(Icon::new(
            IconId::new(BundledIcon::Shuffle),
            (384, 2 + android_offset).into(),
            IconScale::Twenty,
            4.into(),
            Color::GREY_MEDIUM.into(),
        ));
    }
}