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
use foliage::Foliage;
use foliage::{bevy_ecs, load_asset};

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
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_window_size((800, 360));
    foliage.set_base_url("");
    let content = foliage
        .create_view(
            GridPlacement::new(1.span(2), 1.span(2))
                .except(Layout::LANDSCAPE_MOBILE, 1.span(5), 1.span(4))
                .except(Layout::LANDSCAPE_EXT, 3.span(6), 1.span(4)),
            Grid::new(3, 3),
        )
        .handle();
    let image = foliage.view(content).add_target().handle();
    let content_gallery = foliage.view(content).create_stage();
    let load_gallery_image = foliage.create_action(ChangeImage(image, 0));
    let page_left = foliage.create_action(ChangeImage(image, -1));
    let page_right = foliage.create_action(ChangeImage(image, 1));
    foliage
        .view(content)
        .stage(content_gallery)
        .signal_action(load_gallery_image);
    foliage
        .view(content)
        .stage(content_gallery)
        .add_signal_targeting(image)
        .with_attribute(
            GridPlacement::new(1.span(1), 1.span(1))
                .except(Layout::LANDSCAPE_MOBILE, 1.span(1), 1.span(1))
                .except(Layout::LANDSCAPE_EXT, 1.span(1), 1.span(1)),
        );
    let content_about = foliage.view(content).create_stage();
    let content_blank = foliage.view(content).create_stage();
    foliage
        .view(content)
        .stage(content_blank)
        .add_signal_targeting(image)
        .clean();
    let to_content_gallery = foliage.create_action(Next {
        view: content,
        next_stage: content_gallery,
    });
    let to_content_about = foliage.create_action(Next {
        view: content,
        next_stage: content_about,
    });
    let back_to_blank = foliage.create_action(Next {
        view: content,
        next_stage: content_blank,
    });
    foliage.view(content).set_initial_stage(content_blank);
    let control_panel = foliage
        .create_view(
            GridPlacement::new(1.span(2), 2.span(2))
                .except(Layout::LANDSCAPE_MOBILE, 5.span(3), 1.span(4))
                .except(Layout::LANDSCAPE_EXT, 9.span(3), 1.span(4))
                .except(Layout::PORTRAIT_MOBILE, 1.span(4), 5.span(3))
                .except(Layout::PORTRAIT_EXT, 1.span(4), 9.span(3))
                .except(Layout::SQUARE_EXT, 2.span(6), 5.span(3))
                .except(Layout::SQUARE_MAX, 3.span(6), 9.span(3))
                .except(Layout::WIDE_DESKTOP, 3.span(6), 5.span(3))
                .except(Layout::TALL_DESKTOP, 3.span(6), 9.span(3)),
            Grid::new(3, 2),
        )
        .handle();
    let initial = foliage.view(control_panel).create_stage();
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
    let gallery_or_about = foliage.view(control_panel).create_stage();
    let image_controls = foliage.view(control_panel).create_stage();
    let about_controls = foliage.view(control_panel).create_stage();
    foliage.view(control_panel).set_initial_stage(initial);
    let background = foliage.view(control_panel).add_target().handle();
    let gallery_icon = foliage.view(control_panel).add_target().handle();
    let about_icon = foliage.view(control_panel).add_target().handle();
    let gallery_choice_text = foliage.view(control_panel).add_target().handle();
    let about_choice_text = foliage.view(control_panel).add_target().handle();
    let image_forward_icon = foliage.view(control_panel).add_target().handle();
    let image_backward_icon = foliage.view(control_panel).add_target().handle();
    let initial_to_creation = foliage.create_action(Next {
        view: control_panel,
        next_stage: gallery_or_about,
    });
    let creation_to_selection = foliage.create_action(Next {
        view: control_panel,
        next_stage: image_controls,
    });
    let creation_to_about = foliage.create_action(Next {
        view: control_panel,
        next_stage: about_controls,
    });
    let back_to_creation = foliage.create_action(Next {
        view: control_panel,
        next_stage: gallery_or_about,
    });
    foliage
        .view(control_panel)
        .stage(initial)
        .on_end(initial_to_creation);
    foliage
        .view(control_panel)
        .stage(initial)
        .add_signal_targeting(background)
        .with_attribute(Panel::new(Rounding::all(0.05), Color::WHITE))
        .with_attribute(
            GridPlacement::new(1.span(3), 1.span(2))
                .ignore_gap()
                .offset_layer(5),
        )
        .with_transition(); // the PositionAdjust transition to move
    foliage
        .view(control_panel)
        .stage(gallery_or_about)
        .add_signal_targeting(gallery_icon)
        .with_attribute(Icon::new(IconId(0), Color::BLACK))
        .with_attribute(GridPlacement::new(1.span(1), 1.span(1)))
        .with_attribute(ClickInteractionListener::new())
        .with_attribute(OnClick::new(creation_to_selection));
    foliage
        .view(control_panel)
        .stage(gallery_or_about)
        .add_signal_targeting(about_icon)
        .with_attribute(Icon::new(IconId(0), Color::BLACK))
        .with_attribute(GridPlacement::new(1.span(1), 2.span(1)))
        .with_attribute(ClickInteractionListener::new())
        .with_attribute(OnClick::new(creation_to_about));
    foliage
        .view(control_panel)
        .stage(gallery_or_about)
        .add_signal_targeting(gallery_choice_text)
        .with_attribute(()) // text placeholder
        .with_attribute(GridPlacement::new(2.span(2), 1.span(1)));
    foliage
        .view(control_panel)
        .stage(gallery_or_about)
        .add_signal_targeting(image_forward_icon)
        .clean();
    foliage
        .view(control_panel)
        .stage(gallery_or_about)
        .add_signal_targeting(image_backward_icon)
        .clean();
    foliage
        .view(control_panel)
        .stage(gallery_or_about)
        .add_signal_targeting(gallery_choice_text)
        .clean();
    foliage
        .view(control_panel)
        .stage(gallery_or_about)
        .add_signal_targeting(about_choice_text)
        .clean();
    foliage
        .view(control_panel)
        .stage(image_controls)
        .add_signal_targeting(gallery_icon)
        .clean();
    foliage
        .view(control_panel)
        .stage(image_controls)
        .add_signal_targeting(about_icon)
        .clean();
    foliage.run();
}
