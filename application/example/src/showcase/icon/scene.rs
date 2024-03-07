use foliage::bevy_ecs::entity::Entity;
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::color::monochromatic::{Asparagus, Monochromatic};
use foliage::icon::{FeatherIcon, Icon};
use foliage::scene::micro_grid::{AlignmentDesc, MicroGrid, MicroGridAlignment, RelativeMarker};
use foliage::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
#[derive(Clone)]
pub struct IconDisplay {
    pub icons: Vec<FeatherIcon>,
}
impl IconDisplay {
    pub fn new(icons: Vec<FeatherIcon>) -> Self {
        Self { icons }
    }
}
impl Scene for IconDisplay {
    type Params = ();
    type Filter = ();
    type Components = ();

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        todo!()
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let interval = 1f32 / 9f32;
        let mut index = 0;
        for x in 0..3 {
            for y in 0..3 {
                let alignment = MicroGridAlignment::new(
                    (x as f32 * interval).percent_from(RelativeMarker::Left),
                    (y as f32 * interval).percent_from(RelativeMarker::Top),
                    24.fixed(),
                    24.fixed(),
                );
                let color = if y == 0 {
                    Asparagus::MINUS_ONE
                } else if y == 1 {
                    Asparagus::BASE
                } else {
                    Asparagus::PLUS_ONE
                };
                binder.bind(
                    index,
                    alignment,
                    Icon::new(*self.icons.get(index as usize).unwrap(), color),
                );
                index += 1;
            }
        }
        binder.finish::<Self>(SceneComponents::new(MicroGrid::new(), ()))
    }
}