use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::differential::Despawn;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::Elm;
use crate::rectangle::Rectangle;
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use crate::set_descriptor;
use crate::texture::factors::Progress;
use bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Query, ResMut, SystemParamItem};
use derive_macros::SceneBinding;

#[derive(Bundle)]
pub struct ProgressBar {
    tag: Tag<Self>,
    progress: Progress,
}
#[derive(SceneBinding)]
pub enum ProgressBarBindings {
    Back,
    Fill,
}
set_descriptor!(
    pub enum ProgressBarSets {
        Area,
    }
);
impl Leaf for ProgressBar {
    type SetDescriptor = ProgressBarSets;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, ProgressBarSets::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main()
            .add_systems((resize.in_set(ProgressBarSets::Area),));
    }
}
fn resize(
    scenes: Query<
        (&SceneHandle, &Area<InterfaceContext>, &Progress, &Despawn),
        (
            With<Tag<ProgressBar>>,
            Or<(Changed<Area<InterfaceContext>>, Changed<Progress>)>,
        ),
    >,
    mut rectangles: Query<(&mut Area<InterfaceContext>, &mut Progress), Without<Tag<ProgressBar>>>,
    mut coordinator: ResMut<SceneCoordinator>,
) {
    for (handle, area, progress, despawn) in scenes.iter() {
        if despawn.should_despawn() {
            continue;
        }
        coordinator.update_anchor_area(*handle, *area);
        let back =
            coordinator.binding_entity(&handle.access_chain().target(ProgressBarBindings::Back));
        let front =
            coordinator.binding_entity(&handle.access_chain().target(ProgressBarBindings::Fill));
        *rectangles.get_mut(back).unwrap().0 = *area;
        *rectangles.get_mut(front).unwrap().0 = *area;
        *rectangles.get_mut(front).unwrap().1 = Progress::new(0.0, progress.1);
    }
}
pub struct ProgressBarArgs {
    pub back_color: Color,
    pub fill_color: Color,
    pub progress: Progress,
}
impl ProgressBarArgs {
    pub fn new<C: Into<Color>>(progress: Progress, color: C, back_color: C) -> Self {
        Self {
            back_color: back_color.into(),
            fill_color: color.into(),
            progress,
        }
    }
}
impl Scene for ProgressBar {
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
        let entity = binder.bind(
            ProgressBarBindings::Back,
            (0.near(), 0.near(), 1),
            Rectangle::new(anchor.0.section.area, args.back_color, Progress::full()),
            cmd,
        );
        tracing::trace!("binding-progress-back: {:?}", entity);
        let entity = binder.bind(
            ProgressBarBindings::Fill,
            (0.near(), 0.near(), 0),
            Rectangle::new(anchor.0.section.area, args.fill_color, args.progress),
            cmd,
        );
        tracing::trace!("binding-progress-fill: {:?}", entity);
        Self {
            tag: Tag::new(),
            progress: args.progress,
        }
    }
}
