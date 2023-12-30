pub mod controls;
mod playlist;
mod song;
mod stream;
pub mod track_progress;
pub mod visualizer;
mod volume_control;

use crate::music_player::controls::Controls;
use crate::music_player::track_progress::{TrackEvent, TrackProgress, TrackProgressArgs};
use foliage::bevy_ecs::change_detection::ResMut;
use foliage::bevy_ecs::event::EventWriter;
use foliage::bevy_ecs::prelude::Commands;
use foliage::color::Color;
use foliage::compositor::segment::{ResponsiveSegment, Segment, SegmentDesc};
use foliage::compositor::workflow::{
    TransitionBindValidity, TransitionDescriptor, WorkflowDescriptor, WorkflowHandle,
    WorkflowStage, WorkflowTransition,
};
use foliage::compositor::Compositor;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use std::time::Duration;

pub struct MusicPlayer {}
fn setup(
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
    mut events: EventWriter<WorkflowTransition>,
    mut track_events: EventWriter<TrackEvent>,
) {
    // segments
    let playlist_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (0.05.relative(), 0.05.relative()),
        (0.9.relative(), 0.15.relative()),
        0,
    )));
    let volume_control_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (0.05.relative(), 0.2.relative()),
        (0.9.relative(), 0.1.relative()),
        0,
    )));
    let stream_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (0.05.relative(), 0.3.relative()),
        (0.9.relative(), 0.15.relative()),
        0,
    )));
    let visualizer_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (0.05.relative(), 0.45.relative()),
        (324.fixed(), 48.fixed()),
        0,
    )));
    let song_info_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (0.05.relative(), 0.55.relative()),
        (0.9.relative(), 0.15.relative()),
        0,
    )));
    let progress_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (0.1.relative(), 0.75.relative()),
        (0.8.relative(), 0.1.relative()),
        0,
    )));
    let control_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (0.15.relative(), 0.85.relative()),
        (0.7.relative(), 0.15.relative()),
        0,
    )));
    // transition
    let transition = TransitionDescriptor::new(&mut cmd)
        .bind_scene::<Controls>(vec![(control_segment, TransitionBindValidity::all(), ())])
        .bind_scene::<TrackProgress>(vec![(
            progress_segment,
            TransitionBindValidity::all(),
            TrackProgressArgs::new(Color::GREEN_MEDIUM, Color::GREY_DARK),
        )])
        .build();
    // add-to-workflow
    compositor.add_workflow(
        WorkflowDescriptor::new(WorkflowHandle(0))
            .with_transition(WorkflowStage(0), transition)
            .workflow(),
    );
    // trigger starting transition
    events.send(WorkflowTransition(WorkflowHandle(0), WorkflowStage(0)));
    track_events.send(TrackEvent {
        length: Duration::from_secs(18),
    });
}
impl Leaf for MusicPlayer {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.startup().add_systems((setup,));
    }
}
