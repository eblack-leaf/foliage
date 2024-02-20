use foliage::bevy_ecs::prelude::Commands;
use foliage::compositor::segment::MacroGrid;
use foliage::tree::Branch;

pub struct ShowcaseTree {}
impl Branch for ShowcaseTree {
    fn grow(&self, grid: MacroGrid, cmd: &mut Commands) {}
}