use std::sync::{Arc, Mutex};

use ::tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::registry::{LookupSpan, Registry};

use super::{grove, tick};
use crate::stem::Stem;
use crate::text_input::TextInput;
use crate::vein::Vein;
use crate::verbs::Grow;

/// Collects the level of every event emitted while it is installed.
#[derive(Default, Clone)]
struct Levels(Arc<Mutex<Vec<Level>>>);

impl Levels {
    fn above_trace(&self) -> usize {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter(|level| **level < Level::TRACE)
            .count()
    }

    fn clear(&self) {
        self.0.lock().unwrap().clear();
    }
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Levels {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        self.0.lock().unwrap().push(*event.metadata().level());
    }
}

/// Spans are per frame and per step and events are per thing that happened, so neither scales
/// with the size of the tree. Total recomputation is only affordable while that holds.
#[test]
fn a_frame_that_changes_nothing_emits_nothing_above_trace() {
    let levels = Levels::default();
    let subscriber = Registry::default().with(levels.clone());

    ::tracing::subscriber::with_default(subscriber, || {
        let mut grove = grove();
        let trunk = grove.plant(Stem::new());
        for _ in 0..512 {
            grove.branch(trunk, Stem::new());
        }
        tick(&mut grove);

        levels.clear();
        for _ in 0..8 {
            tick(&mut grove);
        }

        assert_eq!(levels.above_trace(), 0);
    });
}

/// A watch is a read the frame takes rather than a change it makes, so a frame in which the watched
/// property did not move has nothing to say about it. The reading itself is a trace event, which is
/// where everything that happens per element belongs.
#[test]
fn a_watch_at_rest_emits_nothing_above_trace() {
    let levels = Levels::default();
    let subscriber = Registry::default().with(levels.clone());

    ::tracing::subscriber::with_default(subscriber, || {
        let mut grove = grove();
        let mut sprig = grove.sprig();
        let trunk = grove.plant(Stem::new());
        for _ in 0..64 {
            let leaf = grove.branch(trunk, Stem::new());
            sprig.watch(leaf, Vein::Drawn);
        }
        tick(&mut grove);

        levels.clear();
        for _ in 0..8 {
            tick(&mut grove);
        }

        assert_eq!(levels.above_trace(), 0);
    });
}

/// A field costs nothing while nothing is happening to it. It is four elements, a caret that is
/// re-placed on every edit and a region that is asked to keep showing one -- none of which is a
/// reason for a frame that took no keystroke to say anything.
#[test]
fn a_field_at_rest_emits_nothing_above_trace() {
    let levels = Levels::default();
    let subscriber = Registry::default().with(levels.clone());

    ::tracing::subscriber::with_default(subscriber, || {
        let mut grove = grove();
        let field = grove.plant(TextInput::new());
        grove.focus(field);
        tick(&mut grove);
        tick(&mut grove);

        levels.clear();
        for _ in 0..8 {
            tick(&mut grove);
        }

        assert_eq!(levels.above_trace(), 0);
    });
}
