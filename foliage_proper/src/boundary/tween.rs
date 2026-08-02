use crate::boundary::bloom::{Bloom, Emissions};
use crate::boundary::op::Timing;
use crate::{Ease, Easement, Repeat, Time, Tree};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::{Query, Res, ResMut};
/// A running scalar tween, named so its values can be told apart when they arrive.
///
/// The name is the runner's own id, for the same reason a [`Leaf`](crate::Leaf) is: it is
/// unique by construction and never reused within a generation, where a counter of our own
/// would eventually wrap and start handing back names that are already in use.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Tween(pub(crate) Entity);

/// A start and an end for one number.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Channel {
    pub from: f32,
    pub to: f32,
}

impl Channel {
    pub fn new(from: f32, to: f32) -> Self {
        Self { from, to }
    }
    /// The value this channel holds `through` of the way along, 0.0..=1.0.
    fn at(&self, through: f32) -> f32 {
        self.from + (self.to - self.from) * through
    }
    fn reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
        }
    }
}

impl From<(f32, f32)> for Channel {
    fn from((from, to): (f32, f32)) -> Self {
        Self::new(from, to)
    }
}

/// A tween foliage runs on your behalf, reporting each frame's numbers rather than writing
/// them anywhere.
///
/// The engine already reduces every animation to scalar channels, so this exposes that
/// directly: hand over start/end pairs and a [`Timing`], receive
/// [`Bloom::Tween`](crate::Bloom::Tween) each frame with the current value of each channel,
/// and apply them to whatever you like -- including things foliage has no concept of. This is
/// what lets a library build its own animatable properties without any engine cooperation.
#[derive(Component)]
pub(crate) struct Tweening {
    pub(crate) tween: Tween,
    /// Held as [`Channel`]s rather than the engine's own `Interpolations`: that type exists to
    /// be *read back into* a component by index, with the channel order as an unwritten
    /// contract between two halves of an `Animate` impl. Here the numbers themselves are the
    /// product and their order is the app's own, so there is nothing to reassemble.
    channels: Vec<Channel>,
    easement: Easement,
    elapsed: u64,
    start: u64,
    finish: u64,
    ease: Ease,
    repeat: Repeat,
    backtrack: bool,
    passes: u32,
}

impl Tweening {
    pub(crate) fn new(tween: Tween, channels: Vec<Channel>, timing: Timing) -> Self {
        Self {
            tween,
            channels,
            easement: Easement::new(timing.ease.clone()),
            elapsed: 0,
            start: timing.start,
            // A zero-length span would divide by zero; one millisecond is the shortest a
            // tween can meaningfully be anyway.
            finish: timing.finish.max(timing.start + 1),
            ease: timing.ease,
            repeat: timing.repeat,
            backtrack: timing.backtrack,
            passes: 0,
        }
    }
}

/// Advances every running tween and reports this frame's numbers.
///
/// Runs in the same set as the component animations, so a tween's values and an animated
/// element's values belong to the same instant.
pub(crate) fn drive_tweens(
    time: Res<Time>,
    mut tweens: Query<(Entity, &mut Tweening)>,
    mut emissions: ResMut<Emissions>,
    mut tree: Tree,
) {
    let delta = time.frame_diff().as_millis() as u64;
    for (entity, mut tweening) in tweens.iter_mut() {
        tweening.elapsed += delta;
        if tweening.elapsed < tweening.start {
            continue;
        }
        let span = tweening.finish - tweening.start;
        let through = ((tweening.elapsed - tweening.start) as f32 / span as f32).min(1.0);
        let eased = tweening.easement.percent_changed(through);
        let values = tweening
            .channels
            .iter()
            .map(|channel| channel.at(eased))
            .collect::<Vec<f32>>();
        emissions.push(Bloom::Tween {
            tween: tweening.tween,
            values,
        });
        if through < 1.0 {
            continue;
        }
        tweening.passes += 1;
        let again = match tweening.repeat {
            Repeat::Once => false,
            Repeat::Times(n) => tweening.passes < n,
            Repeat::Forever => true,
        };
        if again {
            if tweening.backtrack {
                for channel in tweening.channels.iter_mut() {
                    *channel = channel.reversed();
                }
            }
            tweening.elapsed = 0;
            let ease = tweening.ease.clone();
            tweening.easement = Easement::new(ease);
        } else {
            emissions.push(Bloom::TweenDone(tweening.tween));
            tree.despawn(entity);
        }
    }
}
