use crate::progress_bar::ProgressBar;
use foliage_macros::InnerSceneBinding;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::component::Component;
use foliage_proper::bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::{Changed, With, Without};
use foliage_proper::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage_proper::circle::{Circle, CircleStyle, Diameter};
use foliage_proper::color::Color;
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::position::Position;
use foliage_proper::coordinate::{CoordinateUnit, InterfaceContext};
use foliage_proper::differential::Despawn;
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{BundleExtend, Elm};
use foliage_proper::interaction::{InteractionListener, InteractionShape};
use foliage_proper::scene::align::SceneAligner;
use foliage_proper::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use foliage_proper::set_descriptor;
use foliage_proper::texture::factors::Progress;

#[derive(Component)]
pub struct ProgressPercent(pub f32);
#[derive(Bundle)]
pub struct InteractiveProgressBarComponents {
    tag: Tag<Self>,
    percent: ProgressPercent,
}
#[derive(InnerSceneBinding)]
pub enum InteractiveProgressBarBindings {
    Marker,
    Progress,
}
#[derive(Clone)]
pub struct InteractiveProgressBar {
    pub percent: f32,
    pub color: Color,
    pub back_color: Color,
}
impl InteractiveProgressBar {
    pub fn new<C: Into<Color>>(percent: f32, c: C, bc: C) -> Self {
        Self {
            color: c.into(),
            percent,
            back_color: bc.into(),
        }
    }
}
set_descriptor!(
    pub enum InteractiveProgressBarSets {
        Area,
    }
);
impl Leaf for InteractiveProgressBar {
    type SetDescriptor = InteractiveProgressBarSets;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, Self::SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(((resize, interact)
            .chain()
            .in_set(Self::SetDescriptor::Area)
            .before(<ProgressBar as Leaf>::SetDescriptor::Area),));
    }
}
fn resize(
    scenes: Query<
        (&SceneHandle, &Area<InterfaceContext>, &ProgressPercent),
        (
            Changed<Area<InterfaceContext>>,
            With<Tag<InteractiveProgressBarComponents>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut rectangles: Query<
        &mut Area<InterfaceContext>,
        Without<Tag<InteractiveProgressBarComponents>>,
    >,
    mut progress: Query<&mut Progress>,
) {
    // tracing::trace!("updating-interactive-progress-bars");
    for (handle, area, percent) in scenes.iter() {
        coordinator.update_anchor_area(*handle, *area);
        let m_ac = handle
            .access_chain()
            .target(InteractiveProgressBarBindings::Marker);
        coordinator.get_alignment_mut(&m_ac).pos.horizontal = metrics(*area, percent.0).close();
        let p_ac = handle
            .access_chain()
            .target(InteractiveProgressBarBindings::Progress);
        let prog = coordinator.binding_entity(&p_ac);
        rectangles.get_mut(prog).unwrap().width = area.width;
        *progress.get_mut(prog).unwrap().end_mut() = percent.0;
    }
}
fn metrics(area: Area<InterfaceContext>, percent: f32) -> CoordinateUnit {
    area.width * percent - 6f32
}
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct InteractiveProgressBarHook(pub(crate) Option<Position<InterfaceContext>>);
fn interact(
    mut scenes: Query<(
        &SceneHandle,
        &Area<InterfaceContext>,
        &mut ProgressPercent,
        &Despawn,
    )>,
    mut coordinator: ResMut<SceneCoordinator>,
    mut interaction_listeners: Query<(&InteractionListener, &mut InteractiveProgressBarHook)>,
    mut progresses: Query<&mut Progress>,
) {
    for (handle, area, mut percent, despawn) in scenes.iter_mut() {
        if !despawn.should_despawn() {
            let m_ac = handle
                .access_chain()
                .target(InteractiveProgressBarBindings::Marker);
            let p_ac = handle
                .access_chain()
                .target(InteractiveProgressBarBindings::Progress);
            let marker = coordinator.binding_entity(&m_ac);
            let prog = coordinator.binding_entity(&p_ac);
            let entity = if interaction_listeners.get(marker).unwrap().0.engaged() {
                Some(marker)
            } else if interaction_listeners.get(prog).unwrap().0.engaged() {
                Some(prog)
            } else {
                None
            };
            if let Some(e) = entity {
                let (listener, mut hook) = interaction_listeners.get_mut(e).unwrap();
                if listener.engaged_start() {
                    hook.0.replace(listener.interaction.current);
                }
                let diff = listener.interaction.current - hook.0.unwrap();
                let p = diff.x / area.width;
                percent.0 += p;
                percent.0 = percent.0.min(1.0).max(0.0);
                coordinator.get_alignment_mut(&m_ac).pos.horizontal =
                    metrics(*area, percent.0).close();
                *progresses.get_mut(prog).unwrap().end_mut() = percent.0;
                hook.0.replace(listener.interaction.current);
            }
        }
    }
}
impl Scene for InteractiveProgressBar {
    type Bindings = InteractiveProgressBarBindings;
    type Components = InteractiveProgressBarComponents;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: Self,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self::Components {
        let entity = binder.bind(
            InteractiveProgressBarBindings::Marker,
            (
                metrics(anchor.0.section.area, args.percent).close(),
                0.center(),
                0,
            ),
            Circle::new(
                CircleStyle::fill(),
                Diameter::new(12f32),
                args.color,
                Progress::full(),
            )
            .extend(InteractionListener::default().with_shape(InteractionShape::Circle))
            .extend(InteractiveProgressBarHook(None)),
            cmd,
        );
        tracing::trace!("binding-interactive-progress-marker: {:?}", entity);
        let (handle, entity) = binder.bind_scene(
            InteractiveProgressBarBindings::Progress.into(),
            (0.close(), 0.center(), 1).into(),
            (anchor.0.section.area.width, 4f32).into(),
            ProgressBar::new(
                Progress::new(0.0, args.percent),
                args.color,
                args.back_color,
            ),
            &(),
            cmd,
        );
        tracing::trace!(
            "binding-interactive-progress-progress: {:?}:{:?}",
            handle,
            entity
        );
        cmd.entity(entity)
            .insert(InteractionListener::default())
            .insert(InteractiveProgressBarHook::default());
        Self::Components {
            tag: Tag::new(),
            percent: ProgressPercent(args.percent),
        }
    }
}
