use crate::circle::{Circle, CircleStyle, Diameter};
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::Elm;
use crate::progress_bar::{ProgressBarArgs, ProgressBarBindings, ProgressBarSets};
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use crate::scene_bind_enable;
use crate::texture::factors::Progress;
use bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs, ResMut};
use bevy_ecs::query::{Changed, With, Without};
use bevy_ecs::system::{Query, SystemParamItem};

#[derive(Bundle)]
pub struct CircleProgressBar {
    tag: Tag<Self>,
}
fn resize(
    mut circle_area: Query<&mut Diameter, Without<Tag<CircleProgressBar>>>,
    scene: Query<
        (&SceneHandle, &Area<InterfaceContext>),
        (
            With<Tag<CircleProgressBar>>,
            Changed<Area<InterfaceContext>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
) {
    for (handle, area) in scene.iter() {
        coordinator.update_anchor_area(*handle, *area);
        let back =
            coordinator.binding_entity(&handle.access_chain().target(ProgressBarBindings::Back));
        let front =
            coordinator.binding_entity(&handle.access_chain().target(ProgressBarBindings::Fill));
        *circle_area.get_mut(back).unwrap() = Diameter::new(area.width.min(area.height));
        *circle_area.get_mut(front).unwrap() = Diameter::new(area.width.min(area.height));
    }
}
impl Leaf for CircleProgressBar {
    type SetDescriptor = ProgressBarSets;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((resize
            .in_set(ExternalSet::Configure)
            .in_set(ProgressBarSets::Area)
            .before(<Circle as Leaf>::SetDescriptor::Area),));
        scene_bind_enable!(elm, CircleProgressBar);
    }
}
impl Scene for CircleProgressBar {
    type Bindings = ProgressBarBindings;
    type Args<'a> = ProgressBarArgs;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        let diameter = Diameter::new(anchor.0.section.width());
        binder.bind(
            0,
            (0.near(), 0.near(), 1),
            Circle::new(
                CircleStyle::ring(),
                diameter,
                args.back_color,
                Progress::full(),
            ),
            cmd,
        );
        binder.bind(
            1,
            (0.near(), 0.near(), 0),
            Circle::new(
                CircleStyle::ring(),
                diameter,
                args.fill_color,
                args.progress,
            ),
            cmd,
        );
        Self { tag: Tag::new() }
    }
}