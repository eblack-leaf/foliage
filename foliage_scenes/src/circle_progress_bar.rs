use crate::progress_bar::{ProgressBarBindings, ProgressBarSets};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs, ResMut};
use foliage_proper::bevy_ecs::query::{Changed, With, Without};
use foliage_proper::bevy_ecs::system::{Query, SystemParamItem};
use foliage_proper::circle::{Circle, CircleStyle, Diameter};
use foliage_proper::color::Color;
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::InterfaceContext;
use foliage_proper::differential::Despawn;
use foliage_proper::elm::config::ElmConfiguration;
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::Elm;
use foliage_proper::scene::align::SceneAligner;
use foliage_proper::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use foliage_proper::texture::factors::Progress;

#[derive(Bundle)]
pub struct CircleProgressBarComponents {
    tag: Tag<Self>,
}
fn resize(
    mut circle_area: Query<&mut Area<InterfaceContext>, Without<Tag<CircleProgressBarComponents>>>,
    scene: Query<
        (&SceneHandle, &Area<InterfaceContext>, &Despawn),
        (
            With<Tag<CircleProgressBarComponents>>,
            Changed<Area<InterfaceContext>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
) {
    for (handle, area, despawn) in scene.iter() {
        if despawn.should_despawn() {
            continue;
        }
        coordinator.update_anchor_area(*handle, *area);
        let back =
            coordinator.binding_entity(&handle.access_chain().target(ProgressBarBindings::Back));
        let front =
            coordinator.binding_entity(&handle.access_chain().target(ProgressBarBindings::Fill));
        circle_area.get_mut(back).unwrap().width = area.width.min(area.height);
        circle_area.get_mut(front).unwrap().width = area.width.min(area.height);
        tracing::trace!("updating-circle-progress-bars");
    }
}
impl Leaf for CircleProgressBar {
    type SetDescriptor = ProgressBarSets;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((resize
            .in_set(ProgressBarSets::Area)
            .before(<Circle as Leaf>::SetDescriptor::Area),));
    }
}
#[derive(Clone)]
pub struct CircleProgressBar {
    pub back_color: Color,
    pub fill_color: Color,
    pub progress: Progress,
}
impl CircleProgressBar {
    pub fn new<C: Into<Color>>(progress: Progress, color: C, back_color: C) -> Self {
        Self {
            back_color: back_color.into(),
            fill_color: color.into(),
            progress,
        }
    }
}
impl Scene for CircleProgressBar {
    type Bindings = ProgressBarBindings;
    type Components = CircleProgressBarComponents;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: Self,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self::Components {
        let diameter = Diameter::new(anchor.0.section.width());
        binder.bind(
            ProgressBarBindings::Back,
            (0.close(), 0.close(), 1),
            Circle::new(
                CircleStyle::ring(),
                diameter,
                args.back_color,
                Progress::full(),
            ),
            cmd,
        );
        binder.bind(
            ProgressBarBindings::Fill,
            (0.close(), 0.close(), 0),
            Circle::new(
                CircleStyle::ring(),
                diameter,
                args.fill_color,
                args.progress,
            ),
            cmd,
        );
        Self::Components { tag: Tag::new() }
    }
}
