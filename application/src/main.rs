use foliage::bevy_ecs::prelude::World;
use foliage::bevy_ecs::system::Command;
use foliage::color::Color;
use foliage::coordinate::placement::Placement;
use foliage::coordinate::position::Position;
use foliage::coordinate::section::Section;
use foliage::grid::{Grid, GridCoordinate, GridPlacement, GridTemplate, LayoutConfiguration};
use foliage::icon::{Icon, IconId};
use foliage::image::Image;
use foliage::panel::{Panel, PanelCornerRounding};
use foliage::view::{CurrentViewStage, Stage, ViewHandle};
use foliage::{CoreLeaves, Foliage};

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
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_window_size((400, 600));
    foliage.attach_leaves::<CoreLeaves>();
    let view = foliage
        .create_view(Grid::new(3, 2))
        .handle();
    let initial = foliage.view(view).create_stage();
    let element_creation = foliage.view(view).create_stage();
    let image_selection = foliage.view(view).create_stage();
    foliage.view(view).set_initial_stage(initial);
    foliage.view(view).activate();
    let background = foliage.view(view).add_target().handle();
    let gallery_text = foliage.view(view).add_target().handle();
    let image_forward_icon = foliage.view(view).add_target().handle();
    let image_backward_icon = foliage.view(view).add_target().handle();
    let initial_to_creation = foliage.create_action(Next {
        view,
        next_stage: element_creation,
    });
    foliage
        .view(view)
        .stage(initial)
        .on_end(initial_to_creation);
    foliage
        .view(view)
        .stage(initial)
        .add_signal_targeting(background)
        .with_attribute(Panel::new(
            Placement::default(),
            PanelCornerRounding::all(4f32),
            Color::WHITE,
        ))
        .with_attribute(GridPlacement::new(1.span(3), 1.span(2)).ignore_gap())
        .with_transition(); // the PositionAdjust transition to move
    foliage
        .view(view)
        .stage(element_creation)
        .add_signal_targeting(gallery_text)
        .with_attribute(()) // text placeholder
        .with_attribute(GridPlacement::new(2.span(2), 1.span(1)));
    foliage
        .view(view)
        .stage(image_selection)
        .add_signal_targeting(image_forward_icon)
        .with_attribute(Icon::new(IconId(0), Color::BLACK, Position::default(), 1))
        .with_attribute(GridPlacement::new(1.span(1), 2.span(1)).except(
            LayoutConfiguration::EIGHT_FOUR | LayoutConfiguration::TWELVE_FOUR,
            2.span(1),
            1.span(1),
        ))
        .with_attribute(()) // on-click (normal aka left-right)
        .with_filtered_attribute(
            (IconId(1), (/* on-click (up-down) */)),
            LayoutConfiguration::EIGHT_FOUR | LayoutConfiguration::TWELVE_FOUR,
        ) // up @ landscape-mobile | up-transition (on-click)
        .with_transition();
    let slot = Image::slot(0, (400, 400));
    // stage-2 when image created signal this attribute based on the current photo selection
    let fill = Image::new(
        0,
        Section::new((10, 10), (200, 200)),
        0,
        include_bytes!("test_image.png").to_vec(),
    );
    foliage.run();
}
