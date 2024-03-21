use foliage::coordinate::area::Area;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::image::{Image, ImageId, ImageStorage};
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

pub struct ImageShowcase {}
impl Viewable for ImageShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.add(
            Image::new(ImageId(0)),
            ResponsiveSegment::base(Segment::new(2.near().to(6.far()), 2.near().to(5.far())))
                .at_layer(5),
        );
        view_builder.finish()
    }
}
impl Leaf for ImageShowcase {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        // elm.container().spawn(Image::storage(
        //     ImageId(0),
        //     ImageStorage::some(Area::new(651.0, 454.0)),
        // ));
    }
}
