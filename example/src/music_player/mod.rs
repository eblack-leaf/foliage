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
use foliage::bevy_ecs::change_detection::ResMut;
use foliage::bevy_ecs::event::{EventReader, EventWriter};
use foliage::bevy_ecs::prelude::{Commands, IntoSystemConfigs, Res};
use foliage::color::Color;
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::r_compositor::segment::{ResponsiveSegment, Segment, SegmentUnit};
use foliage::r_compositor::{Compositor, Segmental, ViewHandle, ViewTransition};
use foliage::scene::{Anchor, SceneCoordinator};
use foliage::text::font::MonospacedFont;
use foliage::time::TimeDelta;
use foliage::window::ScaleFactor;

pub struct MusicPlayer {}
#[allow(unused)]
fn setup(
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
    mut events: EventWriter<ViewTransition>,
    mut track_events: EventWriter<TrackEvent>,
) {
    let begin = ViewHandle::new(0, 0);
    compositor.add_view(begin);
    // trigger starting transition
    events.send(ViewTransition(begin));
    track_events.send(TrackEvent {
        length: TimeDelta::from_secs(24),
    });
}
fn transitions(
    mut events: EventReader<ViewTransition>,
    mut cmd: Commands,
    mut coordinator: ResMut<SceneCoordinator>,
    mut compositor: ResMut<Compositor>,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
) {
    for e in events.read() {
        let begin = ViewHandle::new(0, 0);
        if e.0 == begin {
            // spawn stuff with responsive seg
            let (_handle, entity) = coordinator.spawn_scene::<VolumeControl>(
                Anchor::default().with_layer(0),
                &VolumeControlArgs::new(0.3, Color::GREEN_MEDIUM, Color::GREY_DARK),
                &(),
                &mut cmd,
            );
            cmd.entity(entity).insert(Segmental::new(
                ResponsiveSegment::new(begin, 0).all(
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
                ResponsiveSegment::new(begin, 0).all(
                    Segment::new()
                        .with_x(SegmentUnit::new(0.1).relative())
                        .with_y(SegmentUnit::new(0.75).relative())
                        .with_w(SegmentUnit::new(0.7).relative())
                        .with_h(SegmentUnit::new(0.15).relative()),
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
                ResponsiveSegment::new(begin, 0).all(
                    Segment::new()
                        .with_x(SegmentUnit::new(0.15).relative())
                        .with_y(SegmentUnit::new(0.85).fixed())
                        .with_w(SegmentUnit::new(0.425).relative())
                        .with_h(SegmentUnit::new(12.0).fixed()),
                ),
            ));
        }
    }
}
impl Leaf for MusicPlayer {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.startup().add_systems((setup,));
        elm.main()
            .add_systems((transitions.in_set(ExternalSet::Spawn),));
    }
}
