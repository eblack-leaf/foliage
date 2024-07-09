use foliage::asset::{AssetKey, OnRetrieve};
use foliage::bevy_ecs::prelude::{Resource, World};
use foliage::bevy_ecs::system::Command;
use foliage::clipboard::ClipboardWrite;
use foliage::color::{Color, Grey, Monochromatic};
use foliage::grid::{Grid, GridCoordinate, GridPlacement};
use foliage::icon::Icon;
use foliage::image::Image;
use foliage::interaction::{ClickInteractionListener, OnClick};
use foliage::panel::{Panel, Rounding};
use foliage::signal::TriggerTarget;
use foliage::style::InteractiveColor;
use foliage::text::{FontSize, Text};
use foliage::view::{CurrentViewStage, Stage, ViewHandle};
use foliage::{bevy_ecs, load_asset};
use foliage::{stage_binding, target_binding, Foliage};

#[derive(Resource)]
struct Media {
    paintings: Vec<(AssetKey, &'static str, &'static str)>,
    current_painting: usize,
    personal: Vec<AssetKey>,
}
impl Media {
    fn advance(&mut self, amount: i32) -> AssetKey {
        self.current_painting = (self.current_painting as i32 + amount)
            .max(0)
            .min(self.paintings.len().checked_sub(1).unwrap_or_default() as i32)
            as usize;
        self.paintings.get(self.current_painting).unwrap().0
    }
    fn current_string(&self) -> String {
        format!("{}|{}", self.current_painting + 1, self.paintings.len())
    }
    fn desc_string(&self) -> String {
        self.paintings
            .get(self.current_painting)
            .unwrap()
            .1
            .to_string()
    }
    fn info_string(&self) -> String {
        self.paintings
            .get(self.current_painting)
            .unwrap()
            .2
            .to_string()
    }
}
const IMAGE_SLOT: i32 = 0;
#[derive(Clone)]
struct ChangePainting {
    amount: i32,
    image: TriggerTarget,
    current: TriggerTarget,
    desc: TriggerTarget,
    info: TriggerTarget,
}
impl ChangePainting {
    fn new(
        amount: i32,
        image: TriggerTarget,
        current: TriggerTarget,
        desc: TriggerTarget,
        info: TriggerTarget,
    ) -> Self {
        Self {
            amount,
            image,
            current,
            desc,
            info,
        }
    }
}
impl Command for ChangePainting {
    fn apply(self, world: &mut World) {
        let key = world
            .get_resource_mut::<Media>()
            .unwrap()
            .advance(self.amount);
        world
            .entity_mut(self.image.value())
            .insert(OnRetrieve::new(key, |asset| {
                Image::new(IMAGE_SLOT, asset).inherit_aspect_ratio()
            }));
        let current_string = world.get_resource::<Media>().unwrap().current_string();
        world.entity_mut(self.current.value()).insert(Text::new(
            current_string,
            FontSize::new(20),
            Grey::LIGHT,
        ));
        let desc_string = world.get_resource::<Media>().unwrap().desc_string();
        world.entity_mut(self.desc.value()).insert(Text::new(
            desc_string,
            FontSize::new(20),
            Grey::LIGHT,
        ));
        let info_string = world.get_resource::<Media>().unwrap().info_string();
        world.entity_mut(self.info.value()).insert(Text::new(
            info_string,
            FontSize::new(20),
            Grey::LIGHT,
        ));
    }
}
#[target_binding]
enum IntroContentTargets {
    FirstName,
    LastName,
    Artist,
}
#[target_binding]
enum IntroControlTargets {
    GalleryIcon,
    GalleryIconBackdrop,
    GalleryText,
    AboutIcon,
    AboutIconBackdrop,
    AboutText,
}
#[target_binding]
enum GalleryContentTargets {
    Image,
}
#[target_binding]
enum GalleryControlTargets {
    Forward,
    ForwardBackdrop,
    Backward,
    BackwardBackdrop,
    Home,
    HomeBackdrop,
    Current,
    Description,
    Info,
}
#[target_binding]
enum AboutContentTargets {
    Name,
    Bio,
    Picture,
}
#[target_binding]
enum AboutControlTargets {
    Home,
    HomeBackdrop,
    InstagramIcon,
    InstagramIconBackdrop,
    EmailText,
}
#[stage_binding]
enum IntroContentStages {
    On,
    Off,
}
#[stage_binding]
enum IntroControlStages {
    On,
    Off,
}
#[stage_binding]
enum GalleryContentStages {
    On,
    Off,
}
#[stage_binding]
enum AboutContentStages {
    On,
    Off,
}
#[stage_binding]
enum GalleryControlStages {
    On,
    Off,
}
#[stage_binding]
enum AboutControlStages {
    On,
    Off,
}
#[derive(Clone)]
struct SwitchView {
    on: ViewHandle,
    on_stage: Stage,
    off: ViewHandle,
    off_stage: Stage,
}
impl SwitchView {
    fn new(on: ViewHandle, on_stage: Stage, off: ViewHandle, off_stage: Stage) -> Self {
        Self {
            on,
            on_stage,
            off,
            off_stage,
        }
    }
}
impl Command for SwitchView {
    fn apply(self, world: &mut World) {
        world
            .get_mut::<CurrentViewStage>(self.on.repr())
            .unwrap()
            .set(self.on_stage);
        world
            .get_mut::<CurrentViewStage>(self.off.repr())
            .unwrap()
            .set(self.off_stage);
    }
}
#[derive(Copy, Clone)]
enum IconHandles {
    Home,
    Forward,
    Backward,
    Instagram,
    Email,
    Gallery,
    About,
}
impl IconHandles {
    fn value(self) -> i32 {
        self as i32
    }
}
const INSTAGRAM_HANDLE: &str = "@jimblackrva";
const EMAIL_HANDLE: &str = "jimblack@gmail.com";
const PROCESS_DESCRIPTION: &str = "My paintings are developed by applying numerous layers of oil \
paint and cold wax with frequent scraping of the surface throughout the process. \
Observation of both natural environments and man made structures inform the work which can be \
seen in the organic drawings, the linear content and use of text. \
I think of my work as experimental rather than narrative. I am not out to tell a story or make a \
statement. I want the viewer to spend time with the paintings in person, and see where the \
experience takes them. The themes of ambiguity and tension frequent the work. I strive for each \
painting to present a ragged eloquence, and hopefully have a presence about them that \
brings viewers closer. ";
const BIO_TEXT: &str =
    "Abstract painter Jim Black has been active in the vibrant Richmond Virginia \
     art scene for over 20 years. A native of North Carolina, Jim completed his BFA \
      in Painting and Printmaking from Virginia Commonwealth University in 2003. \
      He has exhibited his work in numerous galleries in Virginia. \
      His paintings are part of private and corporate collections in Virginia, \
      California, Chicago, Florida, Colorado, Philadelphia and Washington DC.";
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((360, 800));
    foliage.enable_tracing(tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE));
    foliage.set_base_url("");
    foliage.load_icon(
        IconHandles::Home.value(),
        include_bytes!("assets/icons/home.icon"),
    );
    foliage.load_icon(
        IconHandles::Forward.value(),
        include_bytes!("assets/icons/chevrons-right.icon"),
    );
    foliage.load_icon(
        IconHandles::Backward.value(),
        include_bytes!("assets/icons/chevrons-left.icon"),
    );
    foliage.load_icon(
        IconHandles::Instagram.value(),
        include_bytes!("assets/icons/instagram.icon"),
    );
    foliage.load_icon(
        IconHandles::Email.value(),
        include_bytes!("assets/icons/inbox.icon"),
    );
    foliage.load_icon(
        IconHandles::Gallery.value(),
        include_bytes!("assets/icons/grid.icon"),
    );
    foliage.load_icon(
        IconHandles::About.value(),
        include_bytes!("assets/icons/at-sign.icon"),
    );
    foliage.spawn(Image::memory(IMAGE_SLOT, (1400, 1400)));
    let bio_pic = load_asset!(foliage, "assets/media/main-scaled.jpg");
    let media = Media {
        paintings: vec![
            (
                load_asset!(foliage, "assets/gallery/painting_0.jpg"),
                "untitled #1 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_1.jpg"),
                "untitled #2 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_2.jpg"),
                "untitled #3 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_3.jpg"),
                "untitled #4 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_4.jpg"),
                "untitled #5 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_5.jpg"),
                "untitled #6 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_6.jpg"),
                "untitled #7 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_7.jpg"),
                "untitled #8 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_8.jpg"),
                "untitled #9 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_9.jpg"),
                "untitled #10 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_10.jpg"),
                "untitled #11 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_11.jpg"),
                "untitled #12 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_12.jpg"),
                "untitled #13 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_13.jpg"),
                "untitled #14 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_14.jpg"),
                "untitled #15 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_15.jpg"),
                "untitled #16 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_16.jpg"),
                "untitled #17 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_17.jpg"),
                "untitled #18 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_18.jpg"),
                "untitled #19 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_19.jpg"),
                "untitled #20 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_20.jpg"),
                "untitled #21 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_21.jpg"),
                "untitled #22 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_22.jpg"),
                "untitled #23 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_23.jpg"),
                "untitled #24 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_24.jpg"),
                "untitled #25 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_25.jpg"),
                "untitled #26 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_26.jpg"),
                "untitled #27 - 32x48",
                "2019",
            ),
            // (load_asset!(foliage, "assets/gallery/painting_27.jpg"),"", ""),
            (
                load_asset!(foliage, "assets/gallery/painting_28.jpg"),
                "untitled #28 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_29.jpg"),
                "untitled #29 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_30.jpg"),
                "untitled #30 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_31.jpg"),
                "untitled #31 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_32.jpg"),
                "untitled #32 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_33.jpg"),
                "untitled #33 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_34.jpg"),
                "untitled #34 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_35.jpg"),
                "untitled #35 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_36.jpg"),
                "untitled #36 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_37.jpg"),
                "untitled #37 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_38.jpg"),
                "untitled #38 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_39.jpg"),
                "untitled #39 - 32x48",
                "2019",
            ),
            (
                load_asset!(foliage, "assets/gallery/painting_40.jpg"),
                "untitled #40 - 32x48",
                "2019",
            ),
        ],
        current_painting: 0,
        personal: vec![],
    };
    foliage.insert_resource(media);
    let mut intro_content = foliage
        .create_view(GridPlacement::new(1.span(4), 1.span(5)), Grid::new(4, 3))
        .with_stage(IntroContentStages::Off)
        .with_stage(IntroContentStages::On)
        .with_target(IntroContentTargets::FirstName)
        .with_target(IntroContentTargets::LastName)
        .with_target(IntroContentTargets::Artist)
        .set_initial_stage(IntroContentStages::On)
        .finish();
    intro_content.define_stage(
        IntroContentStages::Off,
        |stage| stage.clean_view(),
        &mut foliage,
    );
    intro_content.define_stage(
        IntroContentStages::On,
        |stage| {
            stage.add_signal_targeting(stage.target(IntroContentTargets::FirstName), |s| {
                s.with_attribute(Text::new("JIM", FontSize::new(84), Color::WHITE))
                    .with_attribute(GridPlacement::new(1.span(4), 1.span(1)))
            });
            stage.add_signal_targeting(stage.target(IntroContentTargets::LastName), |s| {
                s.with_attribute(Text::new("BLACK", FontSize::new(48), Grey::LIGHT))
                    .with_attribute(GridPlacement::new(2.span(2), 2.span(1)))
            });
            stage.add_signal_targeting(stage.target(IntroContentTargets::Artist), |s| {
                s.with_attribute(Text::new("RVA | ARTIST", FontSize::new(20), Grey::BASE))
                    .with_attribute(GridPlacement::new(2.span(2), 3.span(1)))
            });
        },
        &mut foliage,
    );
    let mut intro_controls = foliage
        .create_view(GridPlacement::new(1.span(4), 7.span(2)), Grid::new(3, 4))
        .with_stage(IntroControlStages::Off)
        .with_stage(IntroControlStages::On)
        .with_target(IntroControlTargets::GalleryIcon)
        .with_target(IntroControlTargets::GalleryIconBackdrop)
        .with_target(IntroControlTargets::AboutIcon)
        .with_target(IntroControlTargets::AboutIconBackdrop)
        .with_target(IntroControlTargets::GalleryText)
        .with_target(IntroControlTargets::AboutText)
        .set_initial_stage(IntroControlStages::On)
        .finish();
    intro_controls.define_stage(
        IntroControlStages::Off,
        |stage| stage.clean_view(),
        &mut foliage,
    );
    let mut gallery_controls = foliage
        .create_view(GridPlacement::new(1.span(4), 7.span(2)), Grid::new(4, 3))
        .with_stage(GalleryControlStages::Off)
        .with_stage(GalleryControlStages::On)
        .with_target(GalleryControlTargets::Forward)
        .with_target(GalleryControlTargets::ForwardBackdrop)
        .with_target(GalleryControlTargets::Backward)
        .with_target(GalleryControlTargets::BackwardBackdrop)
        .with_target(GalleryControlTargets::Current)
        .with_target(GalleryControlTargets::Home)
        .with_target(GalleryControlTargets::HomeBackdrop)
        .with_target(GalleryControlTargets::Info)
        .with_target(GalleryControlTargets::Description)
        .finish();
    gallery_controls.define_stage(
        GalleryControlStages::Off,
        |stage| stage.clean_view(),
        &mut foliage,
    );
    let to_intro_controls_from_gallery = foliage.create_action(SwitchView::new(
        intro_controls.handle(),
        intro_controls.stage(IntroControlStages::On),
        gallery_controls.handle(),
        gallery_controls.stage(GalleryControlStages::Off),
    ));
    let mut gallery_content = foliage
        .create_view(GridPlacement::new(1.span(4), 1.span(6)), Grid::new(1, 1))
        .with_stage(GalleryContentStages::Off)
        .with_stage(GalleryContentStages::On)
        .with_target(GalleryContentTargets::Image)
        .finish();
    let to_intro_content_from_gallery = foliage.create_action(SwitchView::new(
        intro_content.handle(),
        intro_content.stage(IntroContentStages::On),
        gallery_content.handle(),
        gallery_content.stage(GalleryContentStages::Off),
    ));
    let forward = foliage.create_action(ChangePainting::new(
        1,
        gallery_content.target(GalleryContentTargets::Image),
        gallery_controls.target(GalleryControlTargets::Current),
        gallery_controls.target(GalleryControlTargets::Description),
        gallery_controls.target(GalleryControlTargets::Info),
    ));
    let backward = foliage.create_action(ChangePainting::new(
        -1,
        gallery_content.target(GalleryContentTargets::Image),
        gallery_controls.target(GalleryControlTargets::Current),
        gallery_controls.target(GalleryControlTargets::Description),
        gallery_controls.target(GalleryControlTargets::Info),
    ));
    gallery_controls.define_stage(
        GalleryControlStages::On,
        |stage| {
            stage.add_signal_targeting(stage.target(GalleryControlTargets::Forward), |s| {
                s.with_attribute(Icon::new(IconHandles::Forward.value(), Color::BLACK))
                    .with_attribute(GridPlacement::new(4.span(1), 3.span(1)))
            });
            let linked = vec![stage.target(GalleryControlTargets::Forward)];
            stage.add_signal_targeting(stage.target(GalleryControlTargets::ForwardBackdrop), |s| {
                s.with_attribute(Panel::new(Rounding::all(1.0), Color::WHITE))
                    .with_attribute(GridPlacement::new(4.span(1), 3.span(1)).fixed_area((48, 48)))
                    .with_attribute(ClickInteractionListener::new().as_circle())
                    .with_attribute(
                        InteractiveColor::new(Color::WHITE, Color::BLACK).with_linked(linked),
                    )
                    .with_attribute(OnClick::new(forward))
            });
            stage.add_signal_targeting(stage.target(GalleryControlTargets::Backward), |s| {
                s.with_attribute(Icon::new(IconHandles::Backward.value(), Color::BLACK))
                    .with_attribute(GridPlacement::new(2.span(1), 3.span(1)))
            });
            let linked = vec![stage.target(GalleryControlTargets::Backward)];
            stage.add_signal_targeting(
                stage.target(GalleryControlTargets::BackwardBackdrop),
                |s| {
                    s.with_attribute(Panel::new(Rounding::all(1.0), Color::WHITE))
                        .with_attribute(ClickInteractionListener::new().as_circle())
                        .with_attribute(
                            GridPlacement::new(2.span(1), 3.span(1)).fixed_area((48, 48)),
                        )
                        .with_attribute(
                            InteractiveColor::new(Color::WHITE, Color::BLACK).with_linked(linked),
                        )
                        .with_attribute(OnClick::new(backward))
                },
            );
            stage.add_signal_targeting(stage.target(GalleryControlTargets::Home), |s| {
                s.with_attribute(Icon::new(IconHandles::Home.value(), Color::WHITE))
                    .with_attribute(GridPlacement::new(1.span(1), 3.span(1)))
            });
            let linked = vec![stage.target(GalleryControlTargets::Home)];
            stage.add_signal_targeting(stage.target(GalleryControlTargets::HomeBackdrop), |s| {
                s.with_attribute(Panel::new(Rounding::all(1.0), Grey::DARK))
                    .with_attribute(GridPlacement::new(1.span(1), 3.span(1)).fixed_area((48, 48)))
                    .with_attribute(
                        InteractiveColor::new(Grey::DARK, Color::WHITE).with_linked(linked),
                    )
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(
                        OnClick::new(to_intro_controls_from_gallery)
                            .with(to_intro_content_from_gallery),
                    )
            });
            stage.add_signal_targeting(stage.target(GalleryControlTargets::Current), |s| {
                s.with_attribute(GridPlacement::new(3.span(1), 3.span(1)))
            });
            stage.add_signal_targeting(stage.target(GalleryControlTargets::Description), |s| {
                s.with_attribute(GridPlacement::new(1.span(4), 1.span(1)))
            });
            stage.add_signal_targeting(stage.target(GalleryControlTargets::Info), |s| {
                s.with_attribute(GridPlacement::new(1.span(4), 2.span(1)))
            });
        },
        &mut foliage,
    );
    let show_gallery_controls = foliage.create_action(SwitchView::new(
        gallery_controls.handle(),
        gallery_controls.stage(GalleryControlStages::On),
        intro_controls.handle(),
        intro_controls.stage(GalleryControlStages::Off),
    ));
    gallery_content.define_stage(
        GalleryContentStages::Off,
        |stage| stage.clean_view(),
        &mut foliage,
    );
    let load_image = foliage.create_action(ChangePainting::new(
        0,
        gallery_content.target(GalleryContentTargets::Image),
        gallery_controls.target(GalleryControlTargets::Current),
        gallery_controls.target(GalleryControlTargets::Description),
        gallery_controls.target(GalleryControlTargets::Info),
    ));
    gallery_content.define_stage(
        GalleryContentStages::On,
        |stage| {
            stage.add_signal_targeting(stage.target(GalleryContentTargets::Image), |s| {
                s.with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
            });
            stage.signal_action(load_image);
        },
        &mut foliage,
    );
    let show_gallery_content = foliage.create_action(SwitchView::new(
        gallery_content.handle(),
        gallery_content.stage(GalleryContentStages::On),
        intro_content.handle(),
        intro_content.stage(IntroContentStages::Off),
    ));
    let mut about_controls = foliage
        .create_view(GridPlacement::new(1.span(4), 8.span(1)), Grid::new(5, 1))
        .with_stage(AboutControlStages::Off)
        .with_stage(AboutControlStages::On)
        .with_target(AboutControlTargets::Home)
        .with_target(AboutControlTargets::HomeBackdrop)
        .with_target(AboutControlTargets::InstagramIcon)
        .with_target(AboutControlTargets::InstagramIconBackdrop)
        .with_target(AboutControlTargets::EmailText)
        .finish();
    about_controls.define_stage(
        AboutControlStages::Off,
        |stage| stage.clean_view(),
        &mut foliage,
    );
    let to_intro_controls_from_about = foliage.create_action(SwitchView::new(
        intro_controls.handle(),
        intro_controls.stage(IntroControlStages::On),
        about_controls.handle(),
        about_controls.stage(AboutControlStages::Off),
    ));
    let mut about_content = foliage
        .create_view(GridPlacement::new(1.span(4), 1.span(7)), Grid::new(4, 8))
        .with_stage(AboutContentStages::Off)
        .with_stage(AboutContentStages::On)
        .with_target(AboutContentTargets::Name)
        .with_target(AboutContentTargets::Bio)
        .with_target(AboutContentTargets::Picture)
        .finish();
    let to_intro_content_from_about = foliage.create_action(SwitchView::new(
        intro_content.handle(),
        intro_content.stage(IntroContentStages::On),
        about_content.handle(),
        about_content.stage(AboutContentStages::Off),
    ));
    let copy_instagram = foliage.create_action(ClipboardWrite::new(INSTAGRAM_HANDLE));
    let copy_email = foliage.create_action(ClipboardWrite::new(EMAIL_HANDLE));
    about_controls.define_stage(
        AboutControlStages::On,
        |stage| {
            stage.add_signal_targeting(stage.target(AboutControlTargets::Home), |s| {
                s.with_attribute(Icon::new(IconHandles::Home.value(), Color::WHITE))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
            });
            let linked = vec![stage.target(AboutControlTargets::Home)];
            stage.add_signal_targeting(stage.target(AboutControlTargets::HomeBackdrop), |s| {
                s.with_attribute(Panel::new(Rounding::all(1.0), Grey::DARK))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(1)).fixed_area((48, 48)))
                    .with_attribute(
                        InteractiveColor::new(Grey::DARK, Color::WHITE).with_linked(linked),
                    )
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(
                        OnClick::new(to_intro_controls_from_about)
                            .with(to_intro_content_from_about),
                    )
            });
            stage.add_signal_targeting(stage.target(AboutControlTargets::EmailText), |s| {
                s.with_attribute(Text::new(EMAIL_HANDLE, FontSize::new(16), Grey::LIGHT))
                    .with_attribute(GridPlacement::new(3.span(3), 1.span(1)))
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(OnClick::new(copy_email))
            });
            stage.add_signal_targeting(stage.target(AboutControlTargets::InstagramIcon), |s| {
                s.with_attribute(Icon::new(IconHandles::Instagram.value(), Color::BLACK))
                    .with_attribute(GridPlacement::new(2.span(1), 1.span(1)))
            });
            let linked = vec![stage.target(AboutControlTargets::InstagramIcon)];
            stage.add_signal_targeting(
                stage.target(AboutControlTargets::InstagramIconBackdrop),
                |s| {
                    s.with_attribute(Panel::new(Rounding::all(1.0), Color::WHITE))
                        .with_attribute(
                            GridPlacement::new(2.span(1), 1.span(1)).fixed_area((48, 48)),
                        )
                        .with_attribute(ClickInteractionListener::new().as_circle())
                        .with_attribute(
                            InteractiveColor::new(Color::WHITE, Color::BLACK).with_linked(linked),
                        )
                        .with_attribute(OnClick::new(copy_instagram))
                },
            );
        },
        &mut foliage,
    );
    about_content.define_stage(
        AboutContentStages::Off,
        |stage| stage.clean_view(),
        &mut foliage,
    );
    about_content.define_stage(
        AboutContentStages::On,
        |stage| {
            stage.add_signal_targeting(stage.target(AboutContentTargets::Name), |s| {
                s.with_attribute(Text::new("Jim Black", FontSize::new(48), Color::WHITE))
                    .with_attribute(GridPlacement::new(1.span(4), 1.span(1)))
            });
            stage.add_signal_targeting(stage.target(AboutContentTargets::Bio), |s| {
                s.with_attribute(Text::new(BIO_TEXT, FontSize::new(13), Color::WHITE))
                    .with_attribute(GridPlacement::new(1.span(4), 2.span(3)))
            });
            stage.add_signal_targeting(stage.target(AboutContentTargets::Picture), |s| {
                s.with_attribute(OnRetrieve::new(bio_pic, |asset| {
                    Image::new(IMAGE_SLOT, asset).inherit_aspect_ratio()
                }))
                .with_attribute(GridPlacement::new(1.span(4), 5.span(4)))
            });
        },
        &mut foliage,
    );
    let show_about_controls = foliage.create_action(SwitchView::new(
        about_controls.handle(),
        about_controls.stage(AboutControlStages::On),
        intro_controls.handle(),
        intro_controls.stage(IntroControlStages::Off),
    ));
    let show_about_content = foliage.create_action(SwitchView::new(
        about_content.handle(),
        about_content.stage(AboutControlStages::On),
        intro_content.handle(),
        intro_content.stage(IntroControlStages::Off),
    ));
    intro_controls.define_stage(
        IntroControlStages::On,
        |stage| {
            stage.add_signal_targeting(stage.target(IntroControlTargets::GalleryIcon), |s| {
                s.with_attribute(Icon::new(IconHandles::Gallery.value(), Color::BLACK))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(2)))
            });
            let linked = vec![stage.target(IntroControlTargets::GalleryIcon)];
            stage.add_signal_targeting(
                stage.target(IntroControlTargets::GalleryIconBackdrop),
                |s| {
                    s.with_attribute(Panel::new(Rounding::all(1.0), Color::WHITE))
                        .with_attribute(
                            GridPlacement::new(1.span(1), 1.span(2)).fixed_area((48, 48)),
                        )
                        .with_attribute(
                            InteractiveColor::new(Color::WHITE, Color::BLACK).with_linked(linked),
                        )
                        .with_attribute(ClickInteractionListener::new().as_circle())
                        .with_attribute(
                            OnClick::new(show_gallery_controls).with(show_gallery_content),
                        )
                },
            );
            stage.add_signal_targeting(stage.target(IntroControlTargets::AboutIcon), |s| {
                s.with_attribute(Icon::new(IconHandles::About.value(), Color::BLACK))
                    .with_attribute(GridPlacement::new(1.span(1), 3.span(2)))
            });
            let linked = vec![stage.target(IntroControlTargets::AboutIcon)];
            stage.add_signal_targeting(stage.target(IntroControlTargets::AboutIconBackdrop), |s| {
                s.with_attribute(Panel::new(Rounding::all(1.0), Color::WHITE))
                    .with_attribute(GridPlacement::new(1.span(1), 3.span(2)).fixed_area((48, 48)))
                    .with_attribute(
                        InteractiveColor::new(Color::WHITE, Color::BLACK).with_linked(linked),
                    )
                    .with_attribute(ClickInteractionListener::new().as_circle())
                    .with_attribute(OnClick::new(show_about_controls).with(show_about_content))
            });
            stage.add_signal_targeting(stage.target(IntroControlTargets::GalleryText), |s| {
                s.with_attribute(Text::new("GALLERY", FontSize::new(24), Color::WHITE))
                    .with_attribute(GridPlacement::new(2.span(2), 1.span(2)))
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(OnClick::new(show_gallery_controls).with(show_gallery_content))
            });
            stage.add_signal_targeting(stage.target(IntroControlTargets::AboutText), |s| {
                s.with_attribute(Text::new("ABOUT", FontSize::new(24), Color::WHITE))
                    .with_attribute(GridPlacement::new(2.span(2), 3.span(2)))
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(OnClick::new(show_about_controls).with(show_about_content))
            });
        },
        &mut foliage,
    );

    foliage.run();
}
