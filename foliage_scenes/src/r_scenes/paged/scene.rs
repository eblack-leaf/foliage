use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::component::Component;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::IntoSystemConfigs;
use foliage_proper::bevy_ecs::query::{With, Without};
use foliage_proper::bevy_ecs::system::{Query, SystemParamItem};
use foliage_proper::circle::Circle;
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{Elm, Style};
use foliage_proper::icon::{FeatherIcon, Icon};
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, BlankNode, Scene, SceneComponents, SceneHandle};

use crate::r_scenes::circle_button::CircleButton;
use crate::r_scenes::{BackgroundColor, Colors, ForegroundColor};

pub struct PageStructure {
    pub decrement_icon: FeatherIcon,
    pub increment_icon: FeatherIcon,
    pub colors: Colors,
    pub direction: PageDirection,
    pub num_pages: u32,
}
#[derive(Component, Copy, Clone)]
pub enum PageDirection {
    Horizontal,
    Vertical,
}
impl PageStructure {
    pub fn new(
        d: FeatherIcon,
        i: FeatherIcon,
        c: Colors,
        direction: PageDirection,
        num_pages: u32,
    ) -> Self {
        Self {
            decrement_icon: d,
            increment_icon: i,
            colors: c,
            direction,
            num_pages,
        }
    }
}
#[derive(InnerSceneBinding)]
pub enum PageStructureBindings {
    PageDecrement,
    PageIncrement,
}
#[derive(Component, Copy, Clone)]
pub struct Page(pub i32);
pub struct PageStructureComponents {
    pub page: Page,
    pub colors: Colors,
}
impl Scene for PageStructure {
    type Params = (
        Query<
            'static,
            'static,
            (&'static ForegroundColor, &'static BackgroundColor),
            With<Tag<PageStructure>>,
        >,
        Query<
            'static,
            'static,
            (&'static ForegroundColor, &'static BackgroundColor),
            Without<Tag<PageStructure>>,
        >,
    );
    type Filter = ();
    type Components = PageStructureComponents;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let decrement = bindings.get(PageStructureBindings::PageDecrement);
        let increment = bindings.get(PageStructureBindings::PageIncrement);
        if let Ok((fc, bc)) = ext.0.get(entity) {
            *ext.1.get_mut(decrement).unwrap() = (fc, bc);
            *ext.1.get_mut(increment).unwrap() = (fc, bc);
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let decrement_alignment = match self.direction {
            PageDirection::Horizontal => MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Left),
                0.percent_from(RelativeMarker::Center),
                0.10.percent_of(AnchorDim::Width),
                0.10.percent_of(AnchorDim::Width),
            ),
            PageDirection::Vertical => MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Left),
                0.percent_from(RelativeMarker::Center),
                0.10.percent_of(AnchorDim::Width),
                0.10.percent_of(AnchorDim::Width),
            ),
        };
        binder.bind_scene(
            PageStructureBindings::PageDecrement,
            decrement_alignment,
            CircleButton::new(self.decrement_icon, Style::fill(), self.colors),
        );
        let increment_alignment = match self.direction {
            PageDirection::Horizontal => MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Right),
                0.percent_from(RelativeMarker::Center),
                0.10.percent_of(AnchorDim::Width),
                0.10.percent_of(AnchorDim::Width),
            ),
            PageDirection::Vertical => MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Right),
                0.percent_from(RelativeMarker::Center),
                0.10.percent_of(AnchorDim::Width),
                0.10.percent_of(AnchorDim::Width),
            ),
        };
        binder.bind_scene(
            PageStructureBindings::PageDecrement,
            increment_alignment,
            CircleButton::new(self.increment_icon, Style::fill(), self.colors),
        );
        let element_alignment = match self.direction {
            PageDirection::Horizontal => MicroGridAlignment::new(
                0.125.percent_from(RelativeMarker::Left),
                0.percent_from(RelativeMarker::Center),
                0.75.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            PageDirection::Vertical => MicroGridAlignment::new(
                0.125.percent_from(RelativeMarker::Left),
                0.percent_from(RelativeMarker::Center),
                0.75.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
        };
        for i in 2..self.num_pages + 2 {
            binder.bind(i as i32, element_alignment, BlankNode::default());
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            PageStructureComponents {
                page: Page(0),
                colors: self.colors,
            },
        ))
    }
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Leaf for PageStructure {
    type SetDescriptor = SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
        _elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            foliage_proper::scene::config::<PageStructure>
                .in_set(SetDescriptor::Update)
                .before(<CircleButton as Leaf>::SetDescriptor::Update)
                .before(<Icon as Leaf>::SetDescriptor::Update)
                .before(<Circle as Leaf>::SetDescriptor::Update),
        );
    }
}