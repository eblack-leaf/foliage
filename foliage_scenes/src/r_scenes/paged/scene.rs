use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::animate::trigger::Trigger;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::component::Component;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{IntoSystemConfigs, World};
use foliage_proper::bevy_ecs::query::{With, Without};
use foliage_proper::bevy_ecs::system::{Command, Query, SystemParamItem};
use foliage_proper::circle::Circle;
use foliage_proper::conditional::ConditionalCommand;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{Elm, Style};
use foliage_proper::icon::{FeatherIcon, Icon};
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, BlankNode, Scene, SceneComponents, SceneHandle};

use crate::r_scenes::circle_button::CircleButton;
use crate::r_scenes::ellipsis::{Ellipsis, Selected};
use crate::r_scenes::{BackgroundColor, Colors, Direction, ForegroundColor};

pub struct PageStructure {
    pub decrement_icon: FeatherIcon,
    pub increment_icon: FeatherIcon,
    pub colors: Colors,
    pub direction: Direction,
    pub num_pages: u32,
}
impl PageStructure {
    pub fn new(
        d: FeatherIcon,
        i: FeatherIcon,
        c: Colors,
        direction: Direction,
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
    Display,
}
#[derive(Component, Copy, Clone)]
pub struct Page(pub i32);
#[derive(Component, Copy, Clone)]
pub struct PageMax(pub u32);
#[derive(Bundle)]
pub struct PageStructureComponents {
    pub page: Page,
    pub page_max: PageMax,
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
            (&'static mut ForegroundColor, &'static mut BackgroundColor),
            Without<Tag<PageStructure>>,
        >,
    );
    type Filter = ();
    type Components = PageStructureComponents;

    fn config(
        entity: Entity,
        _coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        let decrement = bindings.get(PageStructureBindings::PageDecrement);
        let increment = bindings.get(PageStructureBindings::PageIncrement);
        if let Ok((fc, bc)) = ext.0.get(entity) {
            *ext.1.get_mut(decrement).unwrap().0 = *fc;
            *ext.1.get_mut(decrement).unwrap().1 = *bc;
            *ext.1.get_mut(increment).unwrap().0 = *fc;
            *ext.1.get_mut(increment).unwrap().1 = *bc;
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let decrement_alignment = match self.direction {
            Direction::Horizontal => MicroGridAlignment::new(
                4.fixed_from(RelativeMarker::Left),
                0.percent_from(RelativeMarker::Center),
                44.fixed(),
                44.fixed(),
            ),
            Direction::Vertical => MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Top),
                44.fixed(),
                44.fixed(),
            ),
        };
        let decrement = binder.bind_scene(
            PageStructureBindings::PageDecrement,
            decrement_alignment,
            CircleButton::new(self.decrement_icon, Style::fill(), self.colors),
        );
        let increment_alignment = match self.direction {
            Direction::Horizontal => MicroGridAlignment::new(
                (-4).fixed_from(RelativeMarker::Right),
                0.percent_from(RelativeMarker::Center),
                44.fixed(),
                44.fixed(),
            ),
            Direction::Vertical => MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                (-4).percent_from(RelativeMarker::Bottom),
                44.fixed(),
                44.fixed(),
            ),
        };
        let increment = binder.bind_scene(
            PageStructureBindings::PageIncrement,
            increment_alignment,
            CircleButton::new(self.increment_icon, Style::fill(), self.colors),
        );
        let element_alignment = match self.direction {
            Direction::Horizontal => MicroGridAlignment::new(
                52.fixed_from(RelativeMarker::Left),
                0.percent_from(RelativeMarker::Top),
                1.0.percent_of(AnchorDim::Width).adjust(-52.0 * 2.0),
                1.0.percent_of(AnchorDim::Height).adjust(-24.0),
            ),
            Direction::Vertical => MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Left),
                52.fixed_from(RelativeMarker::Top),
                1.0.percent_of(AnchorDim::Width).adjust(-24.0),
                1.0.percent_of(AnchorDim::Height).adjust(-52.0 * 2.0),
            ),
        };
        // bind display
        let display_alignment = match self.direction {
            Direction::Horizontal => MicroGridAlignment::new(
                0.0.percent_from(RelativeMarker::Center),
                0.0.percent_from(RelativeMarker::Bottom).adjust(-4.0),
                0.6.percent_of(AnchorDim::Width),
                20.fixed(),
            ),
            Direction::Vertical => MicroGridAlignment::new(
                0.0.percent_from(RelativeMarker::Center),
                0.0.percent_from(RelativeMarker::Bottom),
                0.6.percent_of(AnchorDim::Width),
                20.fixed(),
            ),
        };
        let display = binder.bind_scene(
            PageStructureBindings::Display,
            display_alignment,
            Ellipsis::new(
                self.num_pages,
                self.direction,
                self.colors.foreground.0,
                Some(0),
            ),
        );
        let mut to_be_bound = vec![];
        for i in 3..self.num_pages + 3 {
            to_be_bound.push(
                binder
                    .bind_conditional(i as i32, element_alignment, BlankNode::default())
                    .this(),
            );
        }
        binder.extend(
            decrement.root(),
            ConditionalCommand(SelectionChange {
                bindings: to_be_bound.clone(),
                root: binder.root(),
                page_change: -1,
                display: display.root(),
            }),
        );
        binder.extend(
            increment.root(),
            ConditionalCommand(SelectionChange {
                bindings: to_be_bound.clone(),
                root: binder.root(),
                page_change: 1,
                display: display.root(),
            }),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new().min_height(200.0),
            PageStructureComponents {
                page: Page(0),
                page_max: PageMax(self.num_pages),
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
        elm.enable_conditional_command::<SelectionChange>();
        elm.main().add_systems(
            foliage_proper::scene::config::<PageStructure>
                .in_set(SetDescriptor::Update)
                .before(<CircleButton as Leaf>::SetDescriptor::Update)
                .before(<Icon as Leaf>::SetDescriptor::Update)
                .before(<Circle as Leaf>::SetDescriptor::Update),
        );
    }
}
#[derive(Clone)]
struct SelectionChange {
    bindings: Vec<Entity>,
    root: Entity,
    page_change: i32,
    display: Entity,
}
impl Command for SelectionChange {
    fn apply(self, world: &mut World) {
        let max = world.get::<PageMax>(self.root).unwrap().0 as i32;
        let initial = world.get::<Page>(self.root).unwrap().0;
        let new = initial.checked_add(self.page_change).unwrap_or_default();
        if new == -1 {
            return;
        }
        if new >= max {
            return;
        }
        world.get_mut::<Page>(self.root).unwrap().0 = new;
        let selected = world.get::<Page>(self.root).unwrap().0;
        world
            .get_mut::<Selected>(self.display)
            .unwrap()
            .0
            .replace(selected as u32);
        for (i, branch) in self.bindings.iter().enumerate() {
            *world.get_mut::<Trigger>(*branch).unwrap() = if i == selected as usize {
                Trigger::active()
            } else {
                Trigger::inverse()
            };
        }
    }
}