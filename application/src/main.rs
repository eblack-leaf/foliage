use foliage::asset::{AssetKey, OnRetrieve};
use foliage::bevy_ecs::prelude::{Resource, World};
use foliage::bevy_ecs::system::Command;
use foliage::color::Color;
use foliage::grid::{Grid, GridCoordinate, GridPlacement};
use foliage::icon::Icon;
use foliage::image::Image;
use foliage::interaction::{ClickInteractionListener, OnClick};
use foliage::panel::{Panel, Rounding};
use foliage::signal::TriggerTarget;
use foliage::style::InteractiveColor;
use foliage::text::Text;
use foliage::view::{CurrentViewStage, Stage, ViewHandle};
use foliage::{bevy_ecs, load_asset};
use foliage::{stage_binding, target_binding, Foliage};

#[derive(Resource)]
struct Media {
    paintings: Vec<AssetKey>,
    current_painting: usize,
    personal: Vec<AssetKey>,
}
impl Media {
    fn advance(&mut self, amount: i32) -> AssetKey {
        self.current_painting = (self.current_painting as i32 + amount)
            .max(0)
            .min(self.paintings.len().checked_sub(1).unwrap_or_default() as i32)
            as usize;
        *self.paintings.get(self.current_painting).unwrap()
    }
}
#[derive(Clone)]
struct ChangePainting(i32, TriggerTarget);
impl Command for ChangePainting {
    fn apply(self, world: &mut World) {
        let key = world.get_resource_mut::<Media>().unwrap().advance(self.0);
        world
            .entity_mut(self.1.value())
            .insert(OnRetrieve::new(key, |asset| {
                Image::new(0, asset).inherit_aspect_ratio()
            }));
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
    Backward,
    Home,
    Current,     // 1 | 60
    Description, // title + desc
    Info,        // year + materials
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
    TwitterIcon,
    TwitterIconBackdrop,
    TwitterText,
    EmailIcon,
    EmailIconBackdrop,
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
    Twitter,
    Email,
    Gallery,
    About,
}
impl IconHandles {
    fn value(self) -> i32 {
        self as i32
    }
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_window_size((360, 800));
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
        IconHandles::Twitter.value(),
        include_bytes!("assets/icons/twitter.icon"),
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
    let media = Media {
        paintings: vec![
            load_asset!(foliage, "assets/gallery/painting_0.jpg"),
            load_asset!(foliage, "assets/gallery/painting_1.jpg"),
            load_asset!(foliage, "assets/gallery/painting_2.jpg"),
            load_asset!(foliage, "assets/gallery/painting_3.jpg"),
            load_asset!(foliage, "assets/gallery/painting_4.jpg"),
            load_asset!(foliage, "assets/gallery/painting_5.jpg"),
            load_asset!(foliage, "assets/gallery/painting_6.jpg"),
            load_asset!(foliage, "assets/gallery/painting_7.jpg"),
            load_asset!(foliage, "assets/gallery/painting_8.jpg"),
            load_asset!(foliage, "assets/gallery/painting_9.jpg"),
            load_asset!(foliage, "assets/gallery/painting_10.jpg"),
            load_asset!(foliage, "assets/gallery/painting_11.jpg"),
            load_asset!(foliage, "assets/gallery/painting_12.jpg"),
            load_asset!(foliage, "assets/gallery/painting_13.jpg"),
            load_asset!(foliage, "assets/gallery/painting_14.jpg"),
            load_asset!(foliage, "assets/gallery/painting_15.jpg"),
            load_asset!(foliage, "assets/gallery/painting_16.jpg"),
            load_asset!(foliage, "assets/gallery/painting_17.jpg"),
            load_asset!(foliage, "assets/gallery/painting_18.jpg"),
            load_asset!(foliage, "assets/gallery/painting_19.jpg"),
            load_asset!(foliage, "assets/gallery/painting_20.jpg"),
            load_asset!(foliage, "assets/gallery/painting_21.jpg"),
            load_asset!(foliage, "assets/gallery/painting_22.jpg"),
            load_asset!(foliage, "assets/gallery/painting_23.jpg"),
            load_asset!(foliage, "assets/gallery/painting_24.jpg"),
            load_asset!(foliage, "assets/gallery/painting_25.jpg"),
            load_asset!(foliage, "assets/gallery/painting_26.jpg"),
            // load_asset!(foliage, "assets/gallery/painting_27.jpg"),
            load_asset!(foliage, "assets/gallery/painting_28.jpg"),
            load_asset!(foliage, "assets/gallery/painting_29.jpg"),
            load_asset!(foliage, "assets/gallery/painting_30.jpg"),
            load_asset!(foliage, "assets/gallery/painting_31.jpg"),
            load_asset!(foliage, "assets/gallery/painting_32.jpg"),
            load_asset!(foliage, "assets/gallery/painting_33.jpg"),
            load_asset!(foliage, "assets/gallery/painting_34.jpg"),
            load_asset!(foliage, "assets/gallery/painting_35.jpg"),
            load_asset!(foliage, "assets/gallery/painting_36.jpg"),
            load_asset!(foliage, "assets/gallery/painting_37.jpg"),
            load_asset!(foliage, "assets/gallery/painting_38.jpg"),
            load_asset!(foliage, "assets/gallery/painting_39.jpg"),
            load_asset!(foliage, "assets/gallery/painting_40.jpg"),
        ],
        current_painting: 0,
        personal: vec![],
    };
    foliage.insert_resource(media);
    let mut intro_content = foliage
        .create_view(GridPlacement::new(1.span(1), 1.span(1)), Grid::new(3, 4))
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
                s.with_attribute(Text::new("JIM", Color::WHITE))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
            });
            stage.add_signal_targeting(stage.target(IntroContentTargets::LastName), |s| {
                s.with_attribute(Text::new("BLACK", Color::WHITE))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
            });
            stage.add_signal_targeting(stage.target(IntroContentTargets::Artist), |s| {
                s.with_attribute(Text::new("RVA | ARTIST", Color::WHITE))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
            });
        },
        &mut foliage,
    );
    let mut intro_controls = foliage
        .create_view(GridPlacement::new(1.span(1), 1.span(1)), Grid::new(1, 1))
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
        .create_view(GridPlacement::new(1.span(1), 1.span(1)), Grid::new(1, 1))
        .with_stage(GalleryControlStages::Off)
        .with_stage(GalleryControlStages::On)
        .with_target(GalleryControlTargets::Forward)
        .with_target(GalleryControlTargets::Backward)
        .with_target(GalleryControlTargets::Current)
        .with_target(GalleryControlTargets::Home)
        .with_target(GalleryControlTargets::Info)
        .with_target(GalleryControlTargets::Description)
        .finish();
    gallery_controls.define_stage(
        GalleryControlStages::Off,
        |stage| stage.clean_view(),
        &mut foliage,
    );
    gallery_controls.define_stage(
        GalleryControlStages::On,
        |stage| {
            // TODO
        },
        &mut foliage,
    );
    let show_gallery_controls = foliage.create_action(SwitchView::new(
        gallery_controls.handle(),
        gallery_controls.stage(GalleryControlStages::On),
        intro_controls.handle(),
        intro_controls.stage(GalleryControlStages::Off),
    ));
    let mut gallery_content = foliage
        .create_view(GridPlacement::new(1.span(1), 1.span(1)), Grid::new(1, 1))
        .with_stage(GalleryContentStages::Off)
        .with_stage(GalleryContentStages::On)
        .with_target(GalleryContentTargets::Image)
        .finish();
    gallery_content.define_stage(
        GalleryContentStages::Off,
        |stage| stage.clean_view(),
        &mut foliage,
    );
    gallery_content.define_stage(
        GalleryContentStages::On,
        |stage| {
            // TODO
        },
        &mut foliage,
    );
    let show_gallery_content = foliage.create_action(SwitchView::new(
        gallery_content.handle(),
        gallery_content.stage(GalleryContentStages::On),
        intro_content.handle(),
        intro_content.stage(IntroContentStages::Off),
    ));
    intro_controls.define_stage(
        IntroControlStages::On,
        |stage| {
            stage.add_signal_targeting(stage.target(IntroControlTargets::GalleryIcon), |s| {
                s.with_attribute(Icon::new(IconHandles::Gallery.value(), Color::BLACK))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
            });
            let linked = vec![stage.target(IntroControlTargets::GalleryIcon)];
            stage.add_signal_targeting(
                stage.target(IntroControlTargets::GalleryIconBackdrop),
                |s| {
                    s.with_attribute(Panel::new(Rounding::all(1.0), Color::WHITE))
                        .with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
                        .with_attribute(
                            InteractiveColor::new(Color::WHITE, Color::BLACK).with_linked(linked),
                        )
                        .with_attribute(ClickInteractionListener::new().as_circle())
                        .with_attribute(
                            OnClick::new(show_gallery_controls).with(show_gallery_content),
                        )
                },
            );
        },
        &mut foliage,
    );

    foliage.run();
}
