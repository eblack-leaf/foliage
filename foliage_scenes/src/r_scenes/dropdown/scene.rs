use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneHandle};

pub(crate) struct DropdownScene {}
impl DropdownScene {
    pub(crate) fn new(i: usize) -> Self {
        todo!()
    }
}
impl Scene for DropdownScene {
    type Params = ();
    type Filter = ();
    type Components = ();

    fn config(
        entity: Entity,
        coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        todo!()
    }

    fn create(self, binder: Binder) -> SceneHandle {
        todo!()
    }
}