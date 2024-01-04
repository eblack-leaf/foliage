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

use crate::music_player::track_progress::TrackEvent;
use foliage::bevy_ecs::change_detection::ResMut;
use foliage::bevy_ecs::event::{EventReader, EventWriter};
use foliage::bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::r_compositor::{Compositor, ViewHandle, ViewTransition};
use foliage::time::TimeDelta;

pub struct MusicPlayer {}
#[allow(unused)]
fn setup(
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
    mut events: EventWriter<ViewTransition>,
    mut track_events: EventWriter<TrackEvent>,
) {
    // segments
    let begin = ViewHandle::new(0, 0);
    compositor.add_view(begin);
    // let playlist_segment = compositor.add_segment(ResponsiveSegment::new(begin).all(Segment::new(
    //     (0.025.relative(), 10.fixed()),
    //     (0.45.relative(), 0.15.relative()),
    //     0,
    // )));
    // let volume_control_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
    //     (0.5.relative(), 25.fixed()),
    //     (0.425.relative(), 12.fixed()),
    //     0,
    // )));
    // let stream_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
    //     (0.05.relative(), 0.3.relative()),
    //     (0.9.relative(), 0.15.relative()),
    //     0,
    // )));
    // let visualizer_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
    //     (0.05.relative(), 0.45.relative()),
    //     (324.fixed(), 48.fixed()),
    //     0,
    // )));
    // let song_info_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
    //     (0.05.relative(), 0.55.relative()),
    //     (0.9.relative(), 0.15.relative()),
    //     0,
    // )));
    // let progress_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
    //     (0.1.relative(), 0.75.relative()),
    //     (0.8.relative(), 60.fixed()),
    //     0,
    // )));
    // let control_segment = compositor.add_segment(ResponsiveSegment::all(Segment::new(
    //     (0.15.relative(), 0.85.relative()),
    //     (0.7.relative(), 0.15.relative()),
    //     0,
    // )));
    // trigger starting transition
    events.send(ViewTransition(begin));
    track_events.send(TrackEvent {
        length: TimeDelta::from_secs(24),
    });
}
fn transitions(mut events: EventReader<ViewTransition>) {
    for e in events.read() {
        if e.0 == ViewHandle::new(0, 0) {
            // spawn stuff with responsive seg
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