use crate::r_scenes::text_button::TextButton;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::Component;
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::color::Color;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::{BundleExtend, ElementStyle};
use foliage_proper::interaction::InteractionListener;
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    Alignment, AlignmentDesc, AnchorDim, MicroGrid, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, BlankNode, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::MaxCharacters;

#[derive(Component, Copy, Clone)]
pub enum ExpandDirection {
    Up,
    Down,
}
#[derive(Component, Copy, Clone)]
pub enum ExpandState {
    Open,
    Closed,
}
#[derive(Component, Clone)]
pub struct Selection<T: Clone>(pub T);
#[derive(Component, Clone)]
pub struct Values<T: Clone>(pub Vec<T>);
#[derive(Component, Clone, Default)]
pub struct Displays(pub Vec<String>);
pub(crate) struct DropdownScene {
    max_chars: MaxCharacters,
    element_style: ElementStyle,
    display_color: Color,
}
impl DropdownScene {
    pub(crate) fn new(num_displays: usize, expand_direction: ExpandDirection) -> Self {
        todo!()
    }
}
pub struct DropdownSceneComponents {
    pub max_characters: MaxCharacters,
    pub style: ElementStyle,
    pub displays: Displays,
}
impl DropdownSceneComponents {
    pub fn new(mc: MaxCharacters, style: ElementStyle, displays: Displays) -> Self {
        Self {
            max_characters: mc,
            style,
            displays,
        }
    }
}
impl Scene for DropdownScene {
    type Params = ();
    type Filter = ();
    type Components = DropdownSceneComponents;

    fn config(
        entity: Entity,
        coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        todo!()
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.bind(0, Alignment::default(), BlankNode::default());
        binder.bind(
            2,
            Alignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            TextButton::new(),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            DropdownSceneComponents::new(self.max_chars, self.element_style),
        ))
    }
}