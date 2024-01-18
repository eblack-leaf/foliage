pub mod controls;
#[allow(unused)]
mod playlist;
#[allow(unused)]
mod song;
#[allow(unused)]
mod stream;
pub mod track_progress;
#[allow(unused)]
pub mod visualizer;
pub mod volume_control;
use crate::music_player::controls::Controls;
use crate::music_player::track_progress::{TrackEvent, TrackProgress, TrackProgressArgs};
use crate::music_player::volume_control::{VolumeControl, VolumeControlArgs};
use crate::Engen;
use foliage::bevy_ecs;
use foliage::bevy_ecs::change_detection::ResMut;
use foliage::bevy_ecs::component::Component;
use foliage::bevy_ecs::event::EventWriter;
use foliage::bevy_ecs::prelude::{Commands, DetectChanges, IntoSystemConfigs, Res};
use foliage::bevy_ecs::query::Changed;
use foliage::bevy_ecs::system::Query;
use foliage::color::Color;
use foliage::compositor::segment::{ResponsiveSegment, Segment, SegmentUnit};
use foliage::compositor::{Compositor, CurrentView, Segmental, ViewHandle};
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::{Elm, InteractionHandlerTrigger};
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::IconId;
use foliage::interaction::InteractionListener;
use foliage::prebuilt::button::ButtonStyle;
use foliage::prebuilt::circle_button::{CircleButton, CircleButtonArgs};
use foliage::prebuilt::icon_button::{IconButton, IconButtonArgs};
use foliage::prebuilt::icon_text::{IconText, IconTextArgs};
use foliage::prebuilt::text_input::{TextInput, TextInputArgs};
use foliage::scene::{Anchor, SceneCoordinator};
use foliage::text::font::MonospacedFont;
use foliage::text::{MaxCharacters, TextValue};
use foliage::time::TimeDelta;
use foliage::window::ScaleFactor;
use foliage::workflow::WorkflowConnection;

pub struct MusicPlayer {}
#[allow(unused)]
fn setup(
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
    mut current: ResMut<CurrentView>,
    mut track_events: EventWriter<TrackEvent>,
) {
    let begin = ViewHandle::new(0, 0);
    compositor.add_view(begin);
    compositor.add_view(ViewHandle::new(0, 1));
    current.0 = begin;
    track_events.send(TrackEvent {
        length: TimeDelta::from_secs(24),
    });
    Elm::remove_web_element("loading");
}
fn transitions(
    current: ResMut<CurrentView>,
    mut cmd: Commands,
    mut coordinator: ResMut<SceneCoordinator>,
    mut compositor: ResMut<Compositor>,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
) {
    if current.is_changed() {
        let begin = ViewHandle::new(0, 0);
        let next = ViewHandle::new(0, 1);
        if current.0 == begin {
            // spawn stuff with responsive seg
            let (_handle, entity) = coordinator.spawn_scene::<VolumeControl>(
                Anchor::default().with_layer(0),
                &VolumeControlArgs::new(0.3, Color::GREEN_MEDIUM, Color::GREY_DARK),
                &(),
                &mut cmd,
            );
            tracing::trace!("volume-control is:{:?}", entity);
            cmd.entity(entity).insert(Segmental::new(
                ResponsiveSegment::new_with_view(begin, 0).all(
                    Segment::new()
                        .with_x(SegmentUnit::new(0.5).relative())
                        .with_y(SegmentUnit::new(25.0).fixed())
                        .with_w(SegmentUnit::new(0.425).relative())
                        .with_h(SegmentUnit::new(12.0).fixed()),
                ),
            ));
            compositor.add_to_view(begin, entity);
            let (_handle, entity) = coordinator.spawn_scene::<TrackProgress>(
                Anchor::default().with_layer(0),
                &TrackProgressArgs::new(Color::GREEN_MEDIUM, Color::GREY_DARK),
                &(Res::clone(&font), Res::clone(&scale_factor)),
                &mut cmd,
            );
            cmd.entity(entity).insert(Segmental::new(
                ResponsiveSegment::new_with_view(begin, 0).all(
                    Segment::new()
                        .with_x(SegmentUnit::new(0.1).relative())
                        .with_y(SegmentUnit::new(0.75).relative())
                        .with_w(SegmentUnit::new(0.8).relative())
                        .with_h(SegmentUnit::new(60.0).fixed()),
                ),
            ));
            compositor.add_to_view(begin, entity);
            let (_handle, entity) = coordinator.spawn_scene::<Controls>(
                Anchor::default().with_layer(0),
                &(),
                &(),
                &mut cmd,
            );
            cmd.entity(entity).insert(Segmental::new(
                ResponsiveSegment::new_with_view(begin, 0).all(
                    Segment::new()
                        .with_x(SegmentUnit::new(0.15).relative())
                        .with_y(SegmentUnit::new(0.85).relative())
                        .with_w(SegmentUnit::new(0.7).relative())
                        .with_h(SegmentUnit::new(0.15).relative()),
                ),
            ));
            compositor.add_to_view(begin, entity);
            let (_handle, entity) = coordinator.spawn_scene::<IconButton>(
                Anchor::default(),
                &IconButtonArgs::new(
                    IconId::new(BundledIcon::ChevronsLeft),
                    ButtonStyle::Ring,
                    Color::GREEN_MEDIUM,
                    Color::GREY_DARK,
                ),
                &(),
                &mut cmd,
            );
            tracing::trace!("icon-button-to-next is:{:?}", entity);
            cmd.entity(entity)
                .insert(Segmental::new(
                    ResponsiveSegment::new_with_view(begin, 0).all(
                        Segment::new()
                            .with_x(SegmentUnit::new(10.0).fixed())
                            .with_y(SegmentUnit::new(19.0).fixed())
                            .with_w(SegmentUnit::new(28.0).fixed())
                            .with_h(SegmentUnit::new(28.0).fixed()),
                    ),
                ))
                .insert(GoBack());
            compositor.add_to_view(begin, entity);
        } else if current.0 == next {
            let (_handle, entity) = coordinator.spawn_scene::<IconText>(
                Anchor::default(),
                &IconTextArgs::new(
                    IconId::new(BundledIcon::Tag),
                    MaxCharacters(10),
                    TextValue::new("hello-them"),
                    Color::OFF_WHITE,
                    Color::OFF_WHITE,
                ),
                &(Res::clone(&font), Res::clone(&scale_factor)),
                &mut cmd,
            );
            cmd.entity(entity).insert(Segmental::new(
                ResponsiveSegment::new_with_view(next, 0).all(
                    Segment::new()
                        .with_x(SegmentUnit::new(0.15).relative())
                        .with_y(SegmentUnit::new(0.5).relative())
                        .with_w(SegmentUnit::new(0.5).relative())
                        .with_h(SegmentUnit::new(0.15).relative()),
                ),
            ));
            compositor.add_to_view(begin, entity);
            let (_handle, entity) = coordinator.spawn_scene::<IconButton>(
                Anchor::default(),
                &IconButtonArgs::new(
                    IconId::new(BundledIcon::ChevronsRight),
                    ButtonStyle::Ring,
                    Color::GREEN_MEDIUM,
                    Color::GREY_DARK,
                ),
                &(),
                &mut cmd,
            );
            cmd.entity(entity)
                .insert(Segmental::new(
                    ResponsiveSegment::new_with_view(next, 0).all(
                        Segment::new()
                            .with_x(SegmentUnit::new(100.0).fixed())
                            .with_y(SegmentUnit::new(19.0).fixed())
                            .with_w(SegmentUnit::new(28.0).fixed())
                            .with_h(SegmentUnit::new(28.0).fixed()),
                    ),
                ))
                .insert(GoForward());
            compositor.add_to_view(next, entity);
        }
    }
}
#[derive(Component)]
pub(crate) struct GoBack();
#[derive(Component)]
pub(crate) struct GoForward();
fn page_back(
    go_backs: Query<(&GoBack, &InteractionListener), Changed<InteractionListener>>,
    go_forward: Query<(&GoForward, &InteractionListener), Changed<InteractionListener>>,
    mut current: ResMut<CurrentView>,
) {
    for (_h, listener) in go_backs.iter() {
        if listener.active() {
            current.0 = ViewHandle::new(0, 1);
        }
    }
    for (_h, listener) in go_forward.iter() {
        if listener.active() {
            current.0 = ViewHandle::new(0, 0);
        }
    }
}
#[derive(Component, Copy, Clone)]
pub(crate) struct WorkflowTest(pub(crate) u32);
impl Leaf for MusicPlayer {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.startup().add_systems((setup,));
        elm.main().add_systems((
            transitions.in_set(ExternalSet::ViewBindings),
            page_back.in_set(ExternalSet::Process),
        ));
        elm.add_view_scene_binding::<CircleButton, WorkflowTest>(
            ViewHandle::new(0, 0),
            CircleButtonArgs::new(
                IconId::new(BundledIcon::AlignLeft),
                ButtonStyle::Ring,
                Color::RED_MEDIUM,
                Color::OFF_BLACK,
            ),
            ResponsiveSegment::new(0).all(
                Segment::new()
                    .with_x(SegmentUnit::new(100.0).fixed())
                    .with_y(SegmentUnit::new(100.0).fixed())
                    .with_w(SegmentUnit::new(36.0).fixed())
                    .with_h(SegmentUnit::new(36.0).fixed()),
            ),
            WorkflowTest(34),
        );
        elm.add_interaction_handler::<WorkflowTest, WorkflowConnection<Engen>>(
            InteractionHandlerTrigger::Active,
            |wc, ih| {
                wc.send(ih.0);
            },
        );
        elm.add_view_scene_binding::<TextInput, ()>(
            ViewHandle::new(0, 0),
            TextInputArgs::new(
                MaxCharacters(10),
                TextValue::new("hello"),
                None,
                Color::GREEN_MEDIUM,
                Color::GREY_DARK,
            ),
            ResponsiveSegment::new(0).all(
                Segment::new()
                    .with_x(SegmentUnit::new(10.0).fixed())
                    .with_y(SegmentUnit::new(300.0).fixed())
                    .with_w(SegmentUnit::new(300.0).fixed())
                    .with_h(SegmentUnit::new(35.0).fixed()),
            ),
            (),
        );
    }
}
