use crate::icon::IconHandles;
use crate::leaf_model::LeafModel;
use foliage::bevy_ecs;
use foliage::bevy_ecs::entity::Entity;
use foliage::bevy_ecs::event::Event;
use foliage::bevy_ecs::prelude::{IntoSystemSetConfigs, Resource, Trigger};
use foliage::color::{Grey, Monochromatic};
use foliage::elm::{Elm, ExternalStage};
use foliage::grid::aspect::screen;
use foliage::grid::location::GridLocation;
use foliage::grid::unit::TokenUnit;
use foliage::interaction::OnClick;
use foliage::panel::Rounding;
use foliage::style::Coloring;
use foliage::text::{FontSize, Text};
use foliage::tree::{EcsExtension, Tree};
use foliage::twig::button::Button;
use foliage::twig::{Branch, Twig};
use foliage::{schedule_stage, Root};

#[schedule_stage]
pub(crate) enum ModelStage {
    First,
    Second,
}

impl Root for Home {
    fn attach(elm: &mut Elm) {
        elm.scheduler.main.configure_sets(
            (ModelStage::First, ModelStage::Second)
                .chain()
                .in_set(ExternalStage::Action),
        );
    }
}

pub(crate) struct Home {}

#[derive(Resource)]
pub(crate) struct HomeHandle {
    pub(crate) concepts_button: Button,
    pub(crate) usage_button: Button,
    pub(crate) name: Entity,
    pub(crate) leaf_model: LeafModel,
}

#[derive(Event, Clone)]
pub(crate) struct EventTest {}

pub(crate) fn observant(trigger: Trigger<OnClick>) {
    // do something
}
impl Branch for Home {
    type Handle = HomeHandle;
    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let concepts_button = tree.branch(
            Twig::new(
                Button::args(
                    IconHandles::Concepts,
                    Coloring::new(Grey::base(), Grey::minus_one()),
                )
                .rounded(Rounding::all(0.2))
                .with_text("CONCEPTS", FontSize::new(14))
                .outline(1),
            )
            .elevation(4)
            .location(
                GridLocation::new()
                    .width(250.px())
                    .height(50.px())
                    .left(10.px())
                    .top(10.px()),
            ),
        );
        tree.entity(concepts_button.panel).observe(observant);
        tree.visibility(concepts_button.panel, false);
        let usage_button = tree.branch(Twig::new(
            Button::args(
                IconHandles::Usage,
                Coloring::new(Grey::base(), Grey::minus_one()),
            )
            .with_text("USAGE", FontSize::new(14)),
        ));
        let name = tree.add_leaf();
        tree.elevation(name, 1);
        tree.location(
            name,
            GridLocation::new()
                .center_x(screen().center_x())
                .center_y(25.percent().height().from(screen()))
                .width(75.percent().width().of(screen()))
                .height(64.px()),
        );
        tree.entity(name)
            .insert(Text::new("FOLIAGE", FontSize::new(60), Grey::plus_three()));
        tree.flush(name);
        let leaf_model = tree.branch(
            Twig::new(LeafModel::args()).elevation(10).location(
                GridLocation::new()
                    .left(screen().left())
                    .top(screen().top())
                    .right(screen().right())
                    .bottom(screen().bottom()),
            ),
        );
        HomeHandle {
            concepts_button,
            usage_button,
            name,
            leaf_model,
        }
    }
}
