use foliage::bevy_ecs::prelude::World;
use foliage::bevy_ecs::system::Command;
use foliage::coordinate::section::Section;
use foliage::image::Image;
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
    let view = foliage.create_view().template().padding().handle();
    let initial = foliage.view(view).create_stage();
    let element_creation = foliage.view(view).create_stage();
    foliage.view(view).set_initial_stage(initial);
    foliage.view(view).activate();
    let background = foliage.view(view).add_target().handle();
    let gallery_icon = foliage.view(view).add_target().handle();
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
        .add_signal(background)
        .with_attribute(()) // 0 - 1 grid-placement w/ exceptions + relative (0% - 100%)
        .with_transition(); // the PositionAdjust transition to move
    foliage
        .view(view)
        .stage(element_creation)
        .add_signal(gallery_icon)
        .with_attribute(())
        .with_transition();
    // initial element
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
