use crate::icon::IconHandles;
use foliage::bevy_ecs::event::Event;
use foliage::color::{Grey, Monochromatic};
use foliage::elm::Elm;
use foliage::grid::{GridLocation, TokenUnit};
use foliage::interaction::OnClick;
use foliage::leaf::{TriggerEventSignal, TriggeredEvent};
use foliage::style::Coloring;
use foliage::tree::{EcsExtension, Tree};
use foliage::twig::button::Button;
use foliage::twig::{Branch, Twig};
use foliage::{bevy_ecs, Root};
impl Root for LeafModel {
    fn define(elm: &mut Elm) {}
}
pub(crate) struct LeafModel {
    pub(crate) buttons: Vec<Button>,
}
pub(crate) struct LeafModelHandle {
    // entity structure
}
#[derive(Event, Clone)]
pub(crate) struct EventTest {}
impl Branch for LeafModel {
    type Handle = ();

    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let triggered = tree
            .spawn(TriggerEventSignal::default())
            .insert(TriggeredEvent::new(EventTest {}))
            .id();
        tree.branch(
            Twig::new(Button::args(
                IconHandles::Concepts,
                Coloring::new(Grey::base(), Grey::minus_one()),
                OnClick::new(triggered),
            ))
            .elevation(4)
            .located(
                GridLocation::new()
                    .width(150.px())
                    .height(50.px())
                    .left(10.px())
                    .top(10.px()),
            ),
        );
    }
}
