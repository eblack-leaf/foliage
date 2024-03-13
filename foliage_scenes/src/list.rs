use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::procedure::Procedure;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, BlankNode, Scene, SceneComponents, SceneHandle};
use foliage_proper::segment::ResponsiveSegment;
use foliage_proper::view::ViewBuilder;

pub struct List<T> {
    pub items: Vec<T>,
    pub rs: ResponsiveSegment,
}
impl<T: Scene + Clone> List<T> {
    pub fn new(items: Vec<T>, rs: ResponsiveSegment) -> Self {
        Self { items, rs }
    }
}
impl<T: Scene + Clone> Procedure for List<T> {
    fn steps(self, view_builder: &mut ViewBuilder) {
        let scene = view_builder.add_scene(ListBase::new(self.items.len() as u32), self.rs);
        for i in 0..self.items.len() {
            view_builder.place_scene_on(
                scene.bindings().get(i as i32),
                self.items.get(i).cloned().unwrap(),
            );
        }
    }
}
pub struct ListBase {
    pub num_items: u32,
}
impl ListBase {
    pub fn new(num_items: u32) -> Self {
        Self { num_items }
    }
}
impl Scene for ListBase {
    type Params = ();
    type Filter = ();
    type Components = ();

    fn config(_entity: Entity, _ext: &mut SystemParamItem<Self::Params>, _bindings: &Bindings) {
        todo!()
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let interval = 1f32 / self.num_items as f32 * 0.8;
        for i in 0..self.num_items {
            binder.bind(
                i as i32,
                MicroGridAlignment::new(
                    0.percent_from(RelativeMarker::Center),
                    (i as f32 * interval).percent_from(RelativeMarker::Top),
                    1.percent_of(AnchorDim::Width),
                    interval.percent_of(AnchorDim::Height),
                ),
                BlankNode::default(),
            );
        }
        binder.finish::<Self>(SceneComponents::new(MicroGrid::new(), ()))
    }
}