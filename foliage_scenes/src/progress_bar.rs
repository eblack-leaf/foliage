use foliage_macros::InnerSceneBinding;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::{Changed, Or, With, Without};
use foliage_proper::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage_proper::color::Color;
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::InterfaceContext;
use foliage_proper::differential::Despawn;
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::Elm;
use foliage_proper::rectangle::Rectangle;
use foliage_proper::scene::align::SceneAligner;
use foliage_proper::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use foliage_proper::set_descriptor;
use foliage_proper::texture::factors::Progress;
#[derive(Bundle)]
pub struct ProgressBarComponents {
    tag: Tag<Self>,
    progress: Progress,
}
#[derive(InnerSceneBinding)]
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
            With<Tag<ProgressBarComponents>>,
            Or<(Changed<Area<InterfaceContext>>, Changed<Progress>)>,
        ),
    >,
    mut rectangles: Query<
        (&mut Area<InterfaceContext>, &mut Progress),
        Without<Tag<ProgressBarComponents>>,
    >,
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
        *rectangles.get_mut(front).unwrap().1 = Progress::new(0.0, progress.end());
    }
}
#[derive(Clone)]
pub struct ProgressBar {
    pub back_color: Color,
    pub fill_color: Color,
    pub progress: Progress,
}
impl ProgressBar {
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
    type Components = ProgressBarComponents;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: Self,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self::Components {
        let entity = binder.bind(
            ProgressBarBindings::Back,
            (0.from_left(), 0.from_left(), 1),
            Rectangle::new(anchor.0.section.area, args.back_color, Progress::full()),
            cmd,
        );
        tracing::trace!("binding-progress-back: {:?}", entity);
        let entity = binder.bind(
            ProgressBarBindings::Fill,
            (0.from_left(), 0.from_left(), 0),
            Rectangle::new(anchor.0.section.area, args.fill_color, args.progress),
            cmd,
        );
        tracing::trace!("binding-progress-fill: {:?}", entity);
        Self::Components {
            tag: Tag::new(),
            progress: args.progress,
        }
    }
}
