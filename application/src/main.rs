use foliage::coordinate::section::Section;
use foliage::image::Image;
use foliage::{CoreLeaves, Foliage};

fn main() {
    let mut foliage = Foliage::new();
    foliage.set_window_size((400, 600));
    foliage.attach_leaves::<CoreLeaves>();
    let slot = Image::slot(0, (400, 400));
    let fill = Image::new(
        0,
        Section::new((10, 10), (200, 200)),
        0,
        include_bytes!("test_image.png").to_vec(),
    );
    foliage.spawn(slot);
    foliage.spawn(fill);
    foliage.run();
}
