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
    foliage.set_window_size((400, 600));
    foliage.set_base_url("");
    let view = foliage
        .create_view(GridPlacement::new(1.span(4), 1.span(2)), Grid::new(3, 2))
        .handle();
    let initial = foliage.view(view).create_stage();
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
    let element_creation = foliage.view(view).create_stage();
    let image_selection = foliage.view(view).create_stage();
    foliage.view(view).set_initial_stage(initial);
    foliage.view(view).activate();
    let background = foliage.view(view).add_target().handle();
    let gallery_icon = foliage.view(view).add_target().handle();
    let about_icon = foliage.view(view).add_target().handle();
    println!("gallery-icon: {:?}", gallery_icon.value());
    let gallery_text = foliage.view(view).add_target().handle();
    let image_forward_icon = foliage.view(view).add_target().handle();
    println!("image-forward-icon: {:?}", image_forward_icon.value());
    let image_backward_icon = foliage.view(view).add_target().handle();
    let image = foliage.view(view).add_target().handle();
    println!("image: {:?}", image.value());
    let initial_to_creation = foliage.create_action(Next {
        view,
        next_stage: element_creation,
    });
    let creation_to_selection = foliage.create_action(Next {
        view,
        next_stage: image_selection,
    });
    let back_to_creation = foliage.create_action(Next {
        view,
        next_stage: element_creation,
    });
    let load_gallery_image = foliage.create_action(ChangeImage(image, 0));
    let page_left = foliage.create_action(ChangeImage(image, -1));
    let page_right = foliage.create_action(ChangeImage(image, 1));
    foliage
        .view(view)
        .stage(initial)
        .on_end(initial_to_creation);
    foliage
        .view(view)
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
        .view(view)
        .stage(element_creation)
        .add_signal_targeting(gallery_icon)
        .with_attribute(Icon::new(IconId(0), Color::BLACK))
        .with_attribute(GridPlacement::new(1.span(1), 1.span(1)).except(
            Layout::LANDSCAPE_MOBILE,
            1.span(1),
            1.span(1),
        ))
        .with_attribute(ClickInteractionListener::new())
        .with_attribute(OnClick::new(creation_to_selection));
    foliage
        .view(view)
        .stage(element_creation)
        .add_signal_targeting(about_icon)
        .with_attribute(Icon::new(IconId(0), Color::BLACK))
        .with_attribute(GridPlacement::new(2.span(1), 2.span(1)).except(
            Layout::LANDSCAPE_MOBILE,
            2.span(1),
            2.span(1),
        ))
        .with_attribute(ClickInteractionListener::new())
        .with_attribute(OnClick::new(creation_to_selection));
    foliage
        .view(view)
        .stage(element_creation)
        .add_signal_targeting(image)
        .clean();
    foliage
        .view(view)
        .stage(element_creation)
        .add_signal_targeting(image_forward_icon)
        .clean();
    foliage
        .view(view)
        .stage(element_creation)
        .add_signal_targeting(gallery_text)
        .with_attribute(()) // text placeholder
        .with_attribute(GridPlacement::new(2.span(2), 1.span(1)));
    foliage
        .view(view)
        .stage(image_selection)
        .add_signal_targeting(gallery_icon)
        .clean();
    foliage
        .view(view)
        .stage(image_selection)
        .add_signal_targeting(about_icon)
        .clean();
    foliage
        .view(view)
        .stage(image_selection)
        .add_signal_targeting(image_forward_icon)
        .with_attribute(Icon::new(IconId(1), Color::BLACK))
        .with_attribute(GridPlacement::new(1.span(1), 2.span(1)).except(
            Layout::LANDSCAPE_MOBILE,
            2.span(1),
            1.span(1),
        ))
        .with_attribute(OnClick::new(back_to_creation))
        .with_filtered_attribute(IconId(2), Layout::LANDSCAPE_MOBILE | Layout::LANDSCAPE_EXT)
        .with_attribute(ClickInteractionListener::new())
        .with_transition();
    // foliage
    //     .view(view)
    //     .stage(image_selection)
    //     .signal_action(load_gallery_image);
    foliage
        .view(view)
        .stage(image_selection)
        .add_signal_targeting(image)
        .with_attribute(GridPlacement::new(1.span(1), 1.span(1)).except(
            Layout::LANDSCAPE_MOBILE,
            1.span(1),
            1.span(1),
        ));
    foliage.run();
}
