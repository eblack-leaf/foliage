use crate::r_scenes::text_button::TextButton;
use crate::r_scenes::UIColor;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::Component;
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::ElementStyle;
use foliage_proper::scene::micro_grid::{
    Alignment, AlignmentDesc, AnchorDim, MicroGrid, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::{MaxCharacters, TextValue};
use foliage_proper::bevy_ecs;
#[derive(Component, Copy, Clone)]
pub enum ExpandDirection {
    Up,
    Down,
}
#[derive(Component, Copy, Clone)]
pub enum ExpandState {
    Expanded,
    Collapsed,
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
    displays: Displays,
    ui_color: UIColor,
}
impl DropdownScene {
    pub(crate) fn new(
        displays: Displays,
        expand_direction: ExpandDirection,
        element_style: ElementStyle,
        ui_color: UIColor,
    ) -> Self {
        todo!()
    }
}
#[derive(Component, Copy, Clone)]
pub struct CurrentSelection(pub u32);
pub struct DropdownSceneComponents {
    pub max_characters: MaxCharacters,
    pub style: ElementStyle,
    pub displays: Displays,
    pub ui_color: UIColor,

}
impl DropdownSceneComponents {
    pub fn new(
        mc: MaxCharacters,
        style: ElementStyle,
        displays: Displays,
        ui_color: UIColor,
    ) -> Self {
        Self {
            max_characters: mc,
            style,
            displays,
            ui_color,
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
        // cfg style and display button colors
        todo!()
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        // to have Selection<T> inserted
        // + when change Selection<T> derive base-text value with the .to_string() of T (or From)
        // base node
        binder.bind_scene(
            0,
            Alignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            TextButton::new(
                TextValue::new(self.displays.0.get(0).expect("need at least one display")),
                self.max_chars,
                self.element_style,
                self.ui_color,
            ),
        );
        for i in 1..self.displays.0.len() {
            let binding = i;
            // bind conditional text-button with display value (parallel) + offset 1 (for base)
            //
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            DropdownSceneComponents::new(
                self.max_chars,
                self.element_style,
                self.displays,
                self.ui_color,
            ),
        ))
    }
}
