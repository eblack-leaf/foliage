use crate::icon::IconHandles;
use foliage::bevy_ecs::event::{Event, EventReader};
use foliage::bevy_ecs::prelude::IntoSystemConfigs;
use foliage::bevy_ecs::system::{Res, Resource};
use foliage::color::{Grey, Monochromatic};
use foliage::elm::{Elm, ExternalStage};
use foliage::grid::{GridLocation, TokenUnit};
use foliage::interaction::OnClick;
use foliage::leaf::{TriggerEventSignal, TriggeredEvent};
use foliage::style::Coloring;
use foliage::text::{FontSize, TextValue};
use foliage::tree::{EcsExtension, Tree};
use foliage::twig::button::Button;
use foliage::twig::{Branch, Twig};
use foliage::{bevy_ecs, bevy_ecs::schedule::IntoSystemSetConfigs, schedule_stage, Root};
#[schedule_stage]
pub(crate) enum ModelStage {
    First,
    Second,
}
impl Root for LeafModel {
    fn attach(elm: &mut Elm) {
        elm.scheduler
            .main
            .add_systems(event_test.in_set(ExternalStage::Action));
        elm.scheduler.main.configure_sets(
            (ModelStage::First, ModelStage::Second)
                .chain()
                .in_set(ExternalStage::Action),
        );
    }
}

pub(crate) struct LeafModel {}
#[derive(Resource)]
pub(crate) struct LeafModelHandle {
    pub(crate) buttons: Vec<Button>,
}
#[derive(Event, Clone)]
pub(crate) struct EventTest {}
pub(crate) fn event_test(
    mut reader: EventReader<EventTest>,
    mut tree: Tree,
    model: Res<LeafModelHandle>,
) {
    for _event in reader.read() {
        tree.entity(model.buttons.get(0).unwrap().text)
            .insert(TextValue::new("Hello, world!"));
    }
}
impl Branch for LeafModel {
    type Handle = LeafModelHandle;
    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let triggered = tree
            .spawn(TriggerEventSignal::default())
            .insert(TriggeredEvent::new(EventTest {}))
            .id();
        let button = tree.branch(
            Twig::new(
                Button::args(
                    IconHandles::Concepts,
                    Coloring::new(Grey::base(), Grey::minus_one()),
                    OnClick::new(triggered),
                )
                .with_text("hello", FontSize::new(14)),
            )
            .elevation(4)
            .located(
                GridLocation::new()
                    .width(250.px())
                    .height(50.px())
                    .left(10.px())
                    .top(10.px()),
            ),
        );
        LeafModelHandle {
            buttons: vec![button],
        }
    }
}
