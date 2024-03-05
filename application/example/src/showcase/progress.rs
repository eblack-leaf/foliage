use foliage::segment::MacroGrid;
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

pub struct ProgressShowcase;
impl Viewable for ProgressShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.finish()
    }
}