//! Aspen -- animation.
//!
//! # A tween resolves both of its endpoints every frame
//!
//! Animating a placement is the hard case, and it is the one the rest of the engine was shaped
//! around. A [`Location`] is not a value: it is a function from context to a box, and the context
//! -- the breakpoint, the trunk's geometry, an anchor's box, a measured extent -- can change while
//! a motion is running. So "animate to this placement" cannot be answered once at the start.
//!
//! ```text
//! box(t) = blend( resolve(from, context now), resolve(to, context now), ease(t) )
//! ```
//!
//! Both endpoints re-resolve every frame, which is why the resolver is a pure function
//! ([`placement::resolve`](crate::placement::resolve)) callable more than once per element per
//! frame. Nothing is cached, so nothing can go stale: a resize, a crossed breakpoint, a moved
//! anchor or a repaint all reach both ends of the motion at once, and it lands exactly on target
//! rather than near it.
//!
//! # Where the target lives
//!
//! Starting a motion writes the target to the element **immediately**, and the motion carries what
//! the element left. Two properties fall out of that and neither has any code:
//!
//! - **Ending changes nothing.** The blend at `t = 1` already equals the plain resolution, so there
//!   is no settling step that could land a pixel off.
//! - **Cancelling is a removal.** A direct write replaces the declaration and drops the motion
//!   (F8); the element is at what was written, and there is no stale state to reconcile.
//!
//! # Progress and application are separate
//!
//! This phase advances every live tween against the one clock (F6) and computes its eased progress.
//! Applying that progress belongs to whichever phase owns the property, and which phase that is
//! follows from one line:
//!
//! > **A blend of the same type as the declaration is written back over it here. A blend of a
//! > different type is left to the phase that reads the declaration.**
//!
//! | Property | Blends to | Applied in |
//! |---|---|---|
//! | [`Motion::Opacity`] | a number, like its declaration | `animate` -- written back |
//! | [`Motion::Location`] | a box, where the declaration is a placement | `resolve` |
//! | [`Motion::Color`], [`Motion::Palette`] | a color, where the declaration is a [`Fill`] | `extract` |
//!
//! The fill case is the one that needs saying. A [`Fill`] is a role *or* a color, and a blend of two
//! of them is always a color -- so even two literals cannot be written back, and the blend belongs
//! where a fill becomes a color. That is extraction, for the same reason it is already where an
//! element's opacity is taken into its fill: nothing then holds a second color for a repaint to
//! have to find.
//!
//! # One writer per property
//!
//! There is at most one motion per property per element, so two motions are never blending the same
//! declaration. A second one on a property already moving **replaces** it, starting from a snapshot
//! of where the element is rather than from where the first motion began -- otherwise the element
//! would jump back to the old start. The drain runs before this phase and cancels the motion on any
//! property it writes, so nothing reaches its applying phase with both a pending write and a
//! running tween.

pub(crate) mod ease;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use core::time::Duration;
use tracing::{debug, trace_span};

use crate::color::Color;
use crate::coordinate::Section;
use crate::grove::Grove;
use crate::leaf::Leaf;
use crate::palette::{Fill, Palette, Scheme};
use crate::placement::location::Location;
use crate::tree::Tree;

pub use ease::{Ease, Timing};

/// A running scalar channel, as the app names it.
///
/// Handed out by [`tween`](crate::Grow::tween) and [`timer`](crate::Grow::timer), and what
/// [`Pollen`](crate::Pollen) is asked about. Opaque: there is nothing to be done with one but read
/// its channel and stop it.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Tween(u64);

/// What can be animated.
///
/// A property belongs here if animating it is a normal thing to want, or if it **cannot be animated
/// from outside** because it needs context only the engine has. [`Location`](Motion::Location) and
/// [`Palette`](Motion::Palette) are the second kind: nothing outside can resolve a placement against
/// a breakpoint and an anchor, or a role against the scheme in force.
///
/// The list is closed because the engine's obligations should be, not because an app's are.
/// Everything else -- a font size, a count of sides, a value foliage has no concept of -- is a
/// [`tween`](crate::Grow::tween), which hands out the clock and the easing and writes nothing.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Motion {
    /// How opaque the element is, multiplied through everything grown under it. Fully transparent
    /// is not there: an element that arrives at zero leaves the box stack.
    Opacity(f32),
    /// The color the element is filled with, stated outright. The ordinary case, and the one that
    /// needs no scheme to mean anything.
    ///
    /// Dropped, like any op naming something it does not apply to, if the element draws nothing.
    Color(Color),
    /// The [`Palette`] role the element is filled with, for a fill that is part of the scheme.
    ///
    /// What a role resolves to is the [`Scheme`](crate::Scheme)'s answer, taken every frame for
    /// whichever ends are roles -- so a repaint mid-motion moves the motion. The two share one
    /// property: a fill can only be going to one place, and a write to it cancels either.
    Palette(Palette),
    /// Where the element sits. Both endpoints re-resolve every frame in the same context, so the
    /// motion stays correct through anything that moves either of them.
    Location(Location),
}

/// Which declared property a motion is moving.
///
/// The key one is stored under, so starting a second motion on a property replaces the first and a
/// write to that property cancels it -- both by naming the same slot. It is the *property* and not
/// the kind of value stated about it: two ways to say where a fill is going are two `Motion`
/// variants sharing this one slot, and a write to the fill cancels either.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Property {
    Location,
    Opacity,
    Fill,
}

/// Where a motion started from.
///
/// A **declared** endpoint is a value the app wrote, so it re-resolves every frame in the current
/// context and the motion stays correct through anything that changes what it means.
///
/// A **snapshot** is what a retarget takes, and that is right rather than a compromise: a mid-motion
/// blend is a synthesized intermediate that never corresponded to any declared state, so there is
/// nothing for it to be stale relative to. It is a starting pixel, and a starting pixel is all it
/// needs to be. The target stays fully responsive, so the landing is exact regardless.
#[derive(Clone, Debug)]
pub(crate) enum Departed<D, S> {
    Declared(D),
    Snapshot(S),
}

/// The endpoints a motion carries, which are the ones the element is not already holding.
#[derive(Clone, Debug)]
enum Moving {
    /// The placement the element left. Its target is the element's own [`Location`], written when
    /// the motion started.
    Location(Departed<Location, Section>),
    /// The fill the element left, role or literal alike. Its target is the element's own fill,
    /// written when the motion started.
    Fill(Departed<Fill, Color>),
    /// Both ends, because neither is left on the element: the blend is written back over the
    /// declaration every frame, so the element holds where the motion has reached and not where it
    /// began or where it is going.
    Opacity { from: f32, to: f32 },
}

/// How far a tween has come, against the one clock.
#[derive(Copy, Clone, Debug)]
struct Progress {
    elapsed: Duration,
    timing: Timing,
    /// The eased progress, taken once when the frame's delta is charged.
    ///
    /// Held rather than recomputed on each read, because the phases that apply a blend read it
    /// again after this one has run -- `resolve` twice, once per axis -- and a shaped ease is a
    /// curve parameter solved for rather than a multiply. One solve per tween per frame.
    at: f32,
    /// Whether a frame's delta has been charged to it yet.
    ///
    /// A tween created at the drain begins at this frame's instant, and the delta is how long the
    /// interval *ending* at that instant took -- time that elapsed before the tween existed.
    /// Charging it would move the element on the frame it was told to start, away from where it
    /// currently is.
    charged: bool,
}

impl Progress {
    fn new(timing: Timing) -> Self {
        Self {
            elapsed: Duration::ZERO,
            timing,
            at: timing.at(Duration::ZERO),
            charged: false,
        }
    }

    /// Takes this frame's delta, and reports the progress it leaves.
    fn advance(&mut self, delta: Duration) -> f32 {
        match self.charged {
            true => self.elapsed += delta,
            false => self.charged = true,
        }
        self.at = self.timing.at(self.elapsed);
        self.at
    }

    /// Progress as this frame left it.
    fn at(&self) -> f32 {
        self.at
    }

    fn done(&self) -> bool {
        self.timing.done(self.elapsed)
    }
}

/// One running motion.
#[derive(Clone, Debug)]
struct Motioning {
    moving: Moving,
    progress: Progress,
}

impl Motioning {
    /// The color this motion is currently blended to, given the fill it is moving toward.
    ///
    /// What a retarget snapshots. A blend of two fills is a color and not a fill, so there is
    /// nothing else it could be.
    fn tint(&self, scheme: &Scheme, to: Fill) -> Color {
        let Moving::Fill(departed) = &self.moving else {
            return to.color(scheme);
        };
        let from = match departed {
            Departed::Declared(fill) => fill.color(scheme),
            Departed::Snapshot(color) => *color,
        };
        from.blend(to.color(scheme), self.progress.at())
    }
}

/// One running scalar channel: the engine's clock and easing, made available to a value it has no
/// concept of. It writes nothing.
#[derive(Copy, Clone, Debug)]
struct Channel {
    from: f32,
    to: f32,
    progress: Progress,
}

/// Every running tween.
///
/// Held beside the tree rather than on the elements, because what is moving is a small set and
/// almost never the tree: this phase walks what is running, and the loop can ask whether anything
/// is (F9) without a pass over anything.
#[derive(Default)]
pub(crate) struct Aspen {
    motions: HashMap<(Leaf, Property), Motioning>,
    channels: HashMap<Tween, Channel>,
    /// Where a channel's name comes from. Atomic because naming one takes `&self` on either side of
    /// the boundary, like every other name the engine hands out.
    names: AtomicU64,
}

impl Aspen {
    /// A name for one channel. Never reused, so a stale one is inert.
    pub(crate) fn name(&self) -> Tween {
        Tween(self.names.fetch_add(1, Ordering::Relaxed))
    }

    /// Whether nothing is running, which is half of whether the loop owes a frame.
    pub(crate) fn idle(&self) -> bool {
        self.motions.is_empty() && self.channels.is_empty()
    }

    /// How many tweens are running.
    fn live(&self) -> usize {
        self.motions.len() + self.channels.len()
    }

    /// The motion moving `leaf`'s placement: where it came from, and how far it has come.
    ///
    /// Read by R2, once per axis. The `None` case is the whole tree on almost every frame, so it is
    /// answered without hashing when nothing is moving at all.
    pub(crate) fn location(&self, leaf: Leaf) -> Option<(&Departed<Location, Section>, f32)> {
        let motioning = self.moving(leaf, Property::Location)?;
        match &motioning.moving {
            Moving::Location(departed) => Some((departed, motioning.progress.at())),
            _ => None,
        }
    }

    /// The motion moving `leaf`'s fill, on the same terms. Read by extraction.
    pub(crate) fn fill(&self, leaf: Leaf) -> Option<(&Departed<Fill, Color>, f32)> {
        let motioning = self.moving(leaf, Property::Fill)?;
        match &motioning.moving {
            Moving::Fill(departed) => Some((departed, motioning.progress.at())),
            _ => None,
        }
    }

    fn moving(&self, leaf: Leaf, property: Property) -> Option<&Motioning> {
        if self.motions.is_empty() {
            return None;
        }
        self.motions.get(&(leaf, property))
    }

    /// Drops the motion on one property, reporting whether one was running.
    ///
    /// F8: this is what a direct write does before it lands, so no property ever reaches its
    /// applying phase with both a pending write and a running tween.
    pub(crate) fn cancel(&mut self, leaf: Leaf, property: Property) -> bool {
        self.motions.remove(&(leaf, property)).is_some()
    }

    /// Drops everything running on elements that have gone.
    pub(crate) fn wither(&mut self, gone: &[Leaf]) {
        if self.motions.is_empty() {
            return;
        }
        for leaf in gone {
            for property in [Property::Location, Property::Opacity, Property::Fill] {
                self.motions.remove(&(*leaf, property));
            }
        }
    }

    /// Ends a channel before it has run out, reporting whether one was running.
    ///
    /// A channel has no declaration to write, so there is no direct write for it to be cancelled by
    /// the way a motion is. This is that.
    pub(crate) fn stop(&mut self, tween: Tween) -> bool {
        self.channels.remove(&tween).is_some()
    }

    pub(crate) fn channel(&mut self, tween: Tween, from: f32, to: f32, timing: Timing) {
        self.channels.insert(
            tween,
            Channel {
                from,
                to,
                progress: Progress::new(timing),
            },
        );
    }
}

/// Starts a motion, replacing whatever was moving that property, and reports whether it applied.
///
/// The target is written to the element here rather than when the motion ends, so what the element
/// declares is already where it is going. What the motion carries is what the element no longer
/// holds: the placement or the role it left, or -- for a value written back over its own
/// declaration -- both of its ends.
pub(crate) fn animate(grove: &mut Grove, leaf: Leaf, motion: Motion, timing: Timing) -> bool {
    let scheme = grove.scheme;
    let Grove { tree, aspen, .. } = grove;
    let (property, moving) = match motion {
        Motion::Location(to) => {
            let departed = match aspen.motions.contains_key(&(leaf, Property::Location)) {
                // Where the element is now, which is a box between two placements rather than a
                // placement of its own.
                true => Departed::Snapshot(tree.placed(leaf)),
                false => Departed::Declared(tree.location(leaf).cloned().unwrap_or_default()),
            };
            tree.set_location(leaf, to);
            (Property::Location, Moving::Location(departed))
        }
        Motion::Opacity(to) => {
            // The blend is written back over the declaration every frame, so what the element
            // declares *is* where it currently is -- a retarget needs nothing else.
            let from = tree.opacity(leaf).0;
            (
                Property::Opacity,
                Moving::Opacity {
                    from,
                    to: to.clamp(0.0, 1.0),
                },
            )
        }
        // Two ways to say where a fill is going, one property and one applier. The value is
        // normalised the moment it arrives, so nothing past this point knows which was written.
        Motion::Color(to) => match filling(tree, aspen, &scheme, leaf, Fill::Literal(to)) {
            Some(moving) => (Property::Fill, moving),
            None => return false,
        },
        Motion::Palette(to) => match filling(tree, aspen, &scheme, leaf, Fill::Role(to)) {
            Some(moving) => (Property::Fill, moving),
            None => return false,
        },
    };
    aspen.motions.insert(
        (leaf, property),
        Motioning {
            moving,
            progress: Progress::new(timing),
        },
    );
    true
}

/// Writes a fill target to the element and hands back what it left, or `None` if there is no fill
/// on it to move.
///
/// A retarget snapshots the color, because a blend of two fills is a color and not a fill. Otherwise
/// the endpoint is the fill the element declared, which resolves against the scheme every frame
/// exactly as the target does -- so a role at either end follows a repaint and a literal at either
/// end does not, which is the whole of what the two mean.
fn filling(
    tree: &mut Tree,
    aspen: &Aspen,
    scheme: &Scheme,
    leaf: Leaf,
    to: Fill,
) -> Option<Moving> {
    let pigment = tree.pigment(leaf)?;
    let departed = match aspen.motions.get(&(leaf, Property::Fill)) {
        Some(motioning) => Departed::Snapshot(motioning.tint(scheme, pigment.fill)),
        None => Departed::Declared(pigment.fill),
    };
    tree.set_fill(leaf, to);
    Some(Moving::Fill(departed))
}

/// Step 5. Every live tween advances against the one clock.
///
/// The clock is the frame's, sampled once at step 1, so two tweens started together stay together
/// and a test that moves it by hand is exact.
pub(crate) fn run(grove: &mut Grove) {
    let _step = trace_span!("animate", tweens = grove.aspen.live()).entered();
    let delta = grove.clock.delta();
    channels(grove, delta);
    motions(grove, delta);
}

/// The channels: a value each, reported outward and written nowhere.
fn channels(grove: &mut Grove, delta: Duration) {
    let Grove { aspen, drift, .. } = grove;
    aspen.channels.retain(|tween, channel| {
        let at = channel.progress.advance(delta);
        drift
            .tweens
            .insert(*tween, blend(channel.from, channel.to, at));
        if !channel.progress.done() {
            return true;
        }
        // The frame it ends reports its end value and its finish together, so an app reading one
        // never has to infer the other from an absence.
        drift.finished.insert(*tween);
        debug!(tween = tween.0, "tween finished");
        false
    });
}

/// The motions: advanced here, and applied by whichever phase owns what they are moving.
fn motions(grove: &mut Grove, delta: Duration) {
    let Grove {
        aspen, tree, drift, ..
    } = grove;
    aspen.motions.retain(|(leaf, _), motioning| {
        let at = motioning.progress.advance(delta);
        if let Moving::Opacity { from, to } = motioning.moving {
            tree.set_opacity(*leaf, blend(from, to, at));
        }
        if !motioning.progress.done() {
            return true;
        }
        // What the element declares is already the target, so ending is a removal: the blend at the
        // end equals the plain reading of the declaration, and the frame after is identical.
        drift.landed.insert(*leaf);
        debug!(leaf = leaf.id(), "tween landed");
        false
    });
}

/// One number a fraction of the way to another.
///
/// Exact at both ends rather than merely close. A motion lands *on* its target, and an arithmetic
/// form that arrived a rounding error short of one would put every landing a fraction out and leave
/// every arrival needing a correction after it. The two comparisons are against the values
/// [`Ease`] returns exactly at the ends of a motion, and against nothing else -- a shape that
/// overshoots is left to overshoot.
pub(crate) fn blend(from: f32, to: f32, at: f32) -> f32 {
    if at == 0.0 {
        return from;
    }
    if at == 1.0 {
        return to;
    }
    from + (to - from) * at
}
