use crate::circle::{Circle, CircleStyle, Diameter};
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::{BundleExtend, Elm};
use crate::interaction::InteractionListener;
use crate::prebuilt::progress_bar::{ProgressBar, ProgressBarArgs};
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use crate::set_descriptor;
use crate::texture::factors::Progress;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use bevy_ecs::query::{Changed, With, Without};
use bevy_ecs::system::{Query, ResMut, SystemParamItem};
#[derive(Component)]
pub struct ProgressPercent(pub f32);
#[derive(Bundle)]
pub struct InteractiveProgressBar {
    tag: Tag<Self>,
    percent: ProgressPercent,
}
pub enum InteractiveProgressBarBindings {
    Marker,
    Progress,
}
impl From<InteractiveProgressBarBindings> for SceneBinding {
    fn from(value: InteractiveProgressBarBindings) -> Self {
        SceneBinding(value as i32)
    }
}
pub struct InteractiveProgressBarArgs {
    pub percent: f32,
    pub color: Color,
    pub back_color: Color,
}
impl InteractiveProgressBarArgs {
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
            With<Tag<InteractiveProgressBar>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut rectangles: Query<&mut Area<InterfaceContext>, Without<Tag<InteractiveProgressBar>>>,
) {
    tracing::trace!("updating-interactive-progress-bars");
    for (handle, area, percent) in scenes.iter() {
        coordinator.update_anchor_area(*handle, *area);
        let m_ac = handle
            .access_chain()
            .target(InteractiveProgressBarBindings::Marker);
        coordinator.get_alignment_mut(&m_ac).pos.horizontal = metrics(*area, percent.0).near();
        let p_ac = handle
            .access_chain()
            .target(InteractiveProgressBarBindings::Progress);
        let prog = coordinator.binding_entity(&p_ac);
        rectangles.get_mut(prog).unwrap().width = area.width;
    }
}
fn metrics(area: Area<InterfaceContext>, percent: f32) -> CoordinateUnit {
    area.width * percent - 6f32
}
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct InteractiveProgressBarHook(pub(crate) Option<Position<InterfaceContext>>);
fn interact(
    mut scenes: Query<(&SceneHandle, &Area<InterfaceContext>, &mut ProgressPercent)>,
    mut coordinator: ResMut<SceneCoordinator>,
    mut interaction_listeners: Query<(&InteractionListener, &mut InteractiveProgressBarHook)>,
    mut progresses: Query<&mut Progress>,
) {
    for (handle, area, mut percent) in scenes.iter_mut() {
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
            coordinator.get_alignment_mut(&m_ac).pos.horizontal = metrics(*area, percent.0).near();
            progresses.get_mut(prog).unwrap().1 = percent.0;
            hook.0.replace(listener.interaction.current);
        }
    }
}
impl Scene for InteractiveProgressBar {
    type Bindings = InteractiveProgressBarBindings;
    type Args<'a> = InteractiveProgressBarArgs;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        binder.bind(
            InteractiveProgressBarBindings::Marker,
            (
                metrics(anchor.0.section.area, args.percent).near(),
                0.center(),
                0,
            ),
            Circle::new(
                CircleStyle::fill(),
                Diameter::new(12f32),
                args.color,
                Progress::full(),
            )
            .extend(InteractionListener::default())
            .extend(InteractiveProgressBarHook(None)),
            cmd,
        );
        let (_h, entity) = binder.bind_scene::<ProgressBar>(
            InteractiveProgressBarBindings::Progress.into(),
            (0.near(), 0.center(), 1).into(),
            (anchor.0.section.area.width, 4f32).into(),
            &ProgressBarArgs::new(
                Progress::new(0.0, args.percent),
                args.color,
                args.back_color,
            ),
            &(),
            cmd,
        );
        cmd.entity(entity)
            .insert(InteractionListener::default())
            .insert(InteractiveProgressBarHook::default());
        Self {
            tag: Tag::new(),
            percent: ProgressPercent(args.percent),
        }
    }
}
