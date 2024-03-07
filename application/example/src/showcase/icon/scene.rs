use foliage::bevy_ecs::entity::Entity;
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::color::monochromatic::{Asparagus, Monochromatic};
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
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

    fn config(_entity: Entity, _ext: &mut SystemParamItem<Self::Params>, _bindings: &Bindings) {
        todo!()
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let interval = 1f32 / 4f32;
        let mut index = 0;
        for x in 1..=3 {
            for y in 1..=3 {
                let x_amount = x as f32 * interval;
                let y_amount = y as f32 * interval;
                let alignment = MicroGridAlignment::new(
                    x_amount.percent_from(RelativeMarker::Left),
                    y_amount.percent_from(RelativeMarker::Top),
                    24.fixed(),
                    24.fixed(),
                );
                let color = if y == 1 {
                    Asparagus::MINUS_ONE
                } else if y == 2 {
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
impl Leaf for IconDisplay {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.enable_conditional_scene::<IconDisplay>();
    }
}
