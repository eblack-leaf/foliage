use foliage::r_scenes::dropdown::Dropdown;
use foliage::segment::MacroGrid;
use foliage::text::{Text, TextValue};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

pub struct Overlay;
impl Viewable for Overlay {
    const GRID: MacroGrid = MacroGrid::new(8, 8);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.apply_aesthetic(Dropdown::<Text, TextValue>::new());
        view_builder.finish()
    }
}