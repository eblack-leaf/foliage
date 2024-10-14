use crate::icon::IconHandles;
use crate::leaf_model::LeafModel;
use foliage::bevy_ecs::entity::Entity;
use foliage::bevy_ecs::event::Event;
use foliage::bevy_ecs::prelude::{IntoSystemSetConfigs, Resource, Trigger};
use foliage::color::{Grey, Monochromatic};
use foliage::elm::{Elm, ExternalStage};
use foliage::grid::aspect::screen;
use foliage::grid::responsive::evaluate::EvaluateLocation;
use foliage::grid::responsive::ResponsiveLocation;
use foliage::grid::unit::TokenUnit;
use foliage::interaction::OnClick;
use foliage::leaf::{Leaf, ResolveElevation, ResolveVisibility};
use foliage::opacity::ResolveOpacity;
use foliage::panel::Rounding;
use foliage::style::Coloring;
use foliage::text::{FontSize, Text};
use foliage::tree::{EcsExtension, Tree};
use foliage::twig::button::Button;
use foliage::bevy_ecs;
use foliage::{schedule_stage, Root};
use foliage::twig::{Branch, Twig};

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
    pub(crate) concepts_button: Entity,
    pub(crate) usage_button: Entity,
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
        let concepts_button = tree
            .spawn(Leaf::new().elevation(4))
            .insert(
                Button::new(
                    IconHandles::Concepts,
                    Coloring::new(Grey::base(), Grey::minus_one()),
                )
                .rounded(Rounding::all(0.2))
                .with_text("CONCEPTS", FontSize::new(14))
                .outline(1),
            )
            .insert(
                ResponsiveLocation::new()
                    .top(10.px())
                    .left(10.px())
                    .width(250.px())
                    .height(250.px()),
            )
            .observe(observant)
            .insert(EvaluateLocation::full())
            .insert(ResolveElevation::default())
            .insert(ResolveOpacity::default())
            .insert(ResolveVisibility::default())
            .id();
        tree.visibility(concepts_button, false);
        let usage_button = tree
            .spawn(Leaf::new().elevation(4))
            .insert(
                Button::new(
                    IconHandles::Usage,
                    Coloring::new(Grey::base(), Grey::minus_one()),
                )
                .with_text("USAGE", FontSize::new(14)),
            )
            .id();
        let name = tree
            .spawn(Leaf::new().elevation(1))
            .insert(Text::new("FOLIAGE", FontSize::new(60), Grey::plus_three()))
            .insert(
                ResponsiveLocation::new()
                    .center_x(screen().center_x())
                    .center_y(25.percent().height().from(screen()))
                    .width(75.percent().width().of(screen()))
                    .height(64.px()),
            )
            .id();
        let leaf_model = tree.branch(Twig::new(LeafModel::args()));
        HomeHandle {
            concepts_button,
            usage_button,
            name,
            leaf_model,
        }
    }
}
