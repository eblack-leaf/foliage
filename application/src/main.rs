use foliage::asset::{AssetKey, OnRetrieve};
use foliage::bevy_ecs::prelude::{Resource, World};
use foliage::bevy_ecs::system::Command;
use foliage::color::Color;
use foliage::grid::{Grid, GridCoordinate, GridPlacement, Layout};
use foliage::icon::{Icon, IconId, IconRequest};
use foliage::image::Image;
use foliage::interaction::{ClickInteractionListener, OnClick};
use foliage::panel::{Panel, Rounding};
use foliage::signal::TriggerTarget;
use foliage::view::{CurrentViewStage, Stage, ViewHandle};
use foliage::{bevy_ecs, load_asset};
use foliage::{stage_binding, target_binding, Foliage};

#[derive(Clone)]
struct Next {
    view: ViewHandle,
    next_stage: Stage,
}
impl Command for Next {
    fn apply(self, world: &mut World) {
        world
            .get_mut::<CurrentViewStage>(self.view.repr())
            .expect("no-current")
            .set(self.next_stage);
    }
}
#[derive(Resource)]
struct GalleryImages {
    images: Vec<AssetKey>,
    current: usize,
}
impl GalleryImages {
    fn load(foliage: &mut Foliage) -> Self {
        load_asset!(foliage, "assets/test_image.png", one);
        Self {
            images: vec![one],
            current: 0,
        }
    }
    fn current_image(&self) -> AssetKey {
        *self.images.get(self.current).expect("unloaded-asset")
    }
    fn advance(&mut self, amount: i32) {
        self.current = (self.current as i32 + amount)
            .max(0)
            .min(self.images.len().checked_sub(1).unwrap_or_default() as i32)
            as usize;
    }
}
#[derive(Clone)]
struct ChangeImage(TriggerTarget, i32);
impl Command for ChangeImage {
    fn apply(self, world: &mut World) {
        world
            .get_resource_mut::<GalleryImages>()
            .expect("gallery-images")
            .advance(self.1);
        let key = world
            .get_resource_mut::<GalleryImages>()
            .expect("gallery-images")
            .current_image();
        world
            .entity_mut(self.0.value())
            .insert(OnRetrieve::new(key, |asset| Image::new(0, asset)));
    }
}
#[derive(Clone)]
struct ClipboardMessage(&'static str);
impl Command for ClipboardMessage {
    fn apply(self, world: &mut World) {
        // get clipboard + write message.0 to it
    }
}
#[target_binding]
enum ContentTargets {
    FirstName,
    LastName,
    Title,
    Image,
}
#[stage_binding]
enum ContentStages {
    Initial,
    Gallery,
    About,
}
#[target_binding]
enum ControlTargets {
    PageLeft,
    PageRight,
    CopyTwitter,
    CopyEmail,
    Background,
    Home,
    GalleryIcon,
    AboutIcon,
}
#[stage_binding]
enum ControlStages {
    Initial,
    Creation,
    Gallery,
    About,
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_window_size((800, 360));
    foliage.set_base_url("");
    foliage.spawn(IconRequest::new(
        0,
        include_bytes!("assets/activity.icon").to_vec(),
    ));
    foliage.spawn(IconRequest::new(
        1,
        include_bytes!("assets/airplay.icon").to_vec(),
    ));
    foliage.spawn(IconRequest::new(
        2,
        include_bytes!("assets/alert-circle.icon").to_vec(),
    ));
    let images = GalleryImages::load(&mut foliage);
    foliage.insert_resource(images);
    let mem = Image::memory(0, (1200, 1200));
    foliage.spawn(mem);
    let mut content = foliage
        .create_view(
            GridPlacement::new(1.span(2), 1.span(2))
                .except(Layout::LANDSCAPE_MOBILE, 1.span(5), 1.span(4))
                .except(Layout::LANDSCAPE_EXT, 3.span(6), 1.span(4)),
            Grid::new(3, 3),
        )
        .with_target(ContentTargets::Image)
        .with_stage(ContentStages::Initial)
        .with_stage(ContentStages::Gallery)
        .with_stage(ContentStages::About)
        .set_initial_stage(ContentStages::Initial)
        .finish();
    let load_gallery_image =
        foliage.create_action(ChangeImage(content.target(ContentTargets::Image), 0));
    let page_left = foliage.create_action(ChangeImage(content.target(ContentTargets::Image), -1));
    let page_right = foliage.create_action(ChangeImage(content.target(ContentTargets::Image), 1));
    let to_content_gallery = foliage.create_action(Next {
        view: content.handle(),
        next_stage: content.stage(ContentStages::Gallery),
    });
    let to_content_about = foliage.create_action(Next {
        view: content.handle(),
        next_stage: content.stage(ContentStages::About),
    });
    let to_content_blank = foliage.create_action(Next {
        view: content.handle(),
        next_stage: content.stage(ContentStages::Initial),
    });
    let mut control_panel = foliage
        .create_view(
            GridPlacement::new(1.span(2), 2.span(2))
                .except(Layout::LANDSCAPE_MOBILE, 6.span(3), 1.span(4))
                .except(Layout::LANDSCAPE_EXT, 10.span(3), 1.span(4))
                .except(Layout::PORTRAIT_MOBILE, 1.span(4), 6.span(3))
                .except(Layout::PORTRAIT_EXT, 1.span(4), 10.span(3))
                .except(Layout::SQUARE_EXT, 4.span(5), 6.span(3))
                .except(Layout::SQUARE_MAX, 4.span(5), 10.span(3))
                .except(Layout::WIDE_DESKTOP, 3.span(6), 6.span(3))
                .except(Layout::TALL_DESKTOP, 3.span(6), 10.span(3)),
            Grid::new(3, 4),
        )
        .with_target(ControlTargets::Background)
        .with_target(ControlTargets::GalleryIcon)
        .with_target(ControlTargets::AboutIcon)
        .with_target(ControlTargets::PageLeft)
        .with_target(ControlTargets::PageRight)
        .with_target(ControlTargets::Home)
        .with_target(ControlTargets::CopyTwitter)
        .with_target(ControlTargets::CopyEmail)
        .with_stage(ControlStages::Initial)
        .with_stage(ControlStages::Creation)
        .with_stage(ControlStages::Gallery)
        .with_stage(ControlStages::About)
        .set_initial_stage(ControlStages::Initial)
        .finish();
    let copy_twitter_address = foliage.create_action(ClipboardMessage("jblack@twitter"));
    let copy_email_address = foliage.create_action(ClipboardMessage("jblack@gmail.com"));
    let to_creation = foliage.create_action(Next {
        view: control_panel.handle(),
        next_stage: control_panel.stage(ControlStages::Creation),
    });
    let to_image_controls = foliage.create_action(Next {
        view: control_panel.handle(),
        next_stage: control_panel.stage(ControlStages::Gallery),
    });
    let to_about_controls = foliage.create_action(Next {
        view: control_panel.handle(),
        next_stage: control_panel.stage(ControlStages::About),
    });
    content.define_stage(
        ContentStages::Initial,
        |stage| {
            stage.add_signal_targeting(stage.target(ContentTargets::Image), |s| s.clean());
        },
        &mut foliage,
    );
    content.define_stage(
        ContentStages::Gallery,
        |stage| {
            stage.signal_action(load_gallery_image);
            stage.add_signal_targeting(stage.target(ContentTargets::Image), |sr| {
                sr.with_attribute(GridPlacement::new(1.span(3), 1.span(3)))
            });
        },
        &mut foliage,
    );
    control_panel.define_stage(
        ControlStages::Initial,
        |stage| {
            stage.add_signal_targeting(stage.target(ControlTargets::Background), |sr| {
                sr.with_attribute(Panel::new(Rounding::all(0.05), Color::WHITE))
                    .with_attribute(
                        GridPlacement::new(1.span(3), 1.span(4))
                            .ignore_gap()
                            .offset_layer(5),
                    )
            });
            stage.on_end(to_creation);
        },
        &mut foliage,
    );
    control_panel.define_stage(
        ControlStages::Creation,
        |stage| {
            stage.add_signal_targeting(stage.target(ControlTargets::GalleryIcon), |sr| {
                sr.with_attribute(Icon::new(IconId(0), Color::BLACK))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(2)))
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(OnClick::new(to_image_controls).with(to_content_gallery))
            });
            stage.add_signal_targeting(stage.target(ControlTargets::AboutIcon), |sr| {
                sr.with_attribute(Icon::new(IconId(0), Color::BLACK))
                    .with_attribute(GridPlacement::new(1.span(1), 3.span(2)))
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(OnClick::new(to_about_controls))
            });
            stage.add_signal_targeting(stage.target(ControlTargets::PageRight), |sr| sr.clean());
            stage.add_signal_targeting(stage.target(ControlTargets::PageLeft), |sr| sr.clean());
            stage.add_signal_targeting(stage.target(ControlTargets::Home), |sr| sr.clean());
            stage.add_signal_targeting(stage.target(ControlTargets::CopyTwitter), |sr| sr.clean());
            stage.add_signal_targeting(stage.target(ControlTargets::CopyEmail), |sr| sr.clean());
        },
        &mut foliage,
    );
    control_panel.define_stage(
        ControlStages::Gallery,
        |stage| {
            stage.add_signal_targeting(stage.target(ControlTargets::GalleryIcon), |sr| sr.clean());
            stage.add_signal_targeting(stage.target(ControlTargets::AboutIcon), |sr| sr.clean());
            stage.add_signal_targeting(stage.target(ControlTargets::PageRight), |s| {
                s.with_attribute(Icon::new(IconId(1), Color::BLACK))
                    .with_filtered_attribute(
                        IconId(2),
                        Layout::LANDSCAPE_MOBILE | Layout::LANDSCAPE_EXT,
                    )
                    .with_attribute(GridPlacement::new(2.span(1), 1.span(1)))
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(OnClick::new(page_right))
            });
            stage.add_signal_targeting(stage.target(ControlTargets::PageLeft), |s| {
                s.with_attribute(Icon::new(IconId(1), Color::BLACK))
                    .with_filtered_attribute(
                        IconId(2),
                        Layout::LANDSCAPE_MOBILE | Layout::LANDSCAPE_EXT,
                    )
                    .with_attribute(GridPlacement::new(2.span(1), 4.span(1)))
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(OnClick::new(page_left))
            });
            stage.add_signal_targeting(stage.target(ControlTargets::Home), |s| {
                s.with_attribute(Icon::new(IconId(1), Color::BLACK))
                    .with_attribute(OnClick::new(to_creation).with(to_content_blank))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
                    .with_attribute(ClickInteractionListener::new())
            });
        },
        &mut foliage,
    );
    control_panel.define_stage(
        ControlStages::About,
        |stage| {
            stage.add_signal_targeting(stage.target(ControlTargets::Home), |sr| {
                sr.with_attribute(Icon::new(IconId(1), Color::BLACK))
                    .with_attribute(OnClick::new(to_creation).with(to_content_blank))
                    .with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
                    .with_attribute(ClickInteractionListener::new())
            });
            stage.add_signal_targeting(stage.target(ControlTargets::CopyTwitter), |sr| {
                sr.with_attribute(Icon::new(IconId(2), Color::BLACK))
                    .with_attribute(GridPlacement::new(1.span(1), 2.span(1)))
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(OnClick::new(copy_twitter_address))
            });
            stage.add_signal_targeting(stage.target(ControlTargets::CopyEmail), |sr| {
                sr.with_attribute(Icon::new(IconId(2), Color::BLACK))
                    .with_attribute(GridPlacement::new(1.span(1), 3.span(1)))
                    .with_attribute(ClickInteractionListener::new())
                    .with_attribute(OnClick::new(copy_email_address))
            });
            stage.add_signal_targeting(stage.target(ControlTargets::GalleryIcon), |sr| sr.clean());
            stage.add_signal_targeting(stage.target(ControlTargets::AboutIcon), |sr| sr.clean());
        },
        &mut foliage,
    );
    foliage.run();
}
