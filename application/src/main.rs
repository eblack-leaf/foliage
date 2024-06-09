use foliage::coordinate::section::Section;
use foliage::image::Image;
use foliage::{CoreLeaves, Foliage};

fn main() {
    let mut foliage = Foliage::new();
    foliage.set_window_size((400, 600));
    foliage.attach_leaves::<CoreLeaves>();
    let view = foliage.create_view().template().padding().handle();
    let initial = foliage.view(view).create_stage();
    let element_creation = foliage.view(view).create_stage();
    let background = foliage.view(view).add_target().handle();
    let gallery_icon = foliage.view(view).add_target().handle();
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
