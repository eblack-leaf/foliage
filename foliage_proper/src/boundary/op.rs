use crate::boundary::leaf::Leaf;
use crate::coordinate::position::Position;
use crate::{
    AssetKey, AssetSource, Color, Ease, Elevation, Entity, FontSize, GlyphColors, Location,
    Logical, Outline, Polygon, Repeat, Rounding, ScrollTo, Tree, World,
};

/// What foliage grows. One variant per core primitive -- the set is closed on purpose: this
/// is the whole vocabulary of things that can come into existence.
pub enum Spec {
    /// An element with no primitive of its own -- a container, a hit area, a group.
    Bare(crate::LeafSprout),
    Panel(crate::PanelSprout),
    Text(crate::TextSprout),
    Icon(crate::IconSprout),
    Image(crate::ImageSprout),
    Line(crate::LineSprout),
    Polygon(crate::PolygonSprout),
    Polyline(crate::PolylineSprout),
    TextInput(crate::TextInputSprout),
}

impl Spec {
    /// Sprout into an already-allocated id, so the `Leaf` handed to the app the moment it
    /// asked names this element and no other.
    fn grow(self, tree: &mut Tree, at: Entity, under: Option<Entity>) {
        match self {
            Spec::Bare(s) => tree.grow_at(at, s, under),
            Spec::Panel(s) => tree.grow_at(at, s, under),
            Spec::Text(s) => tree.grow_at(at, s, under),
            Spec::Icon(s) => tree.grow_at(at, s, under),
            Spec::Image(s) => tree.grow_at(at, s, under),
            Spec::Line(s) => tree.grow_at(at, s, under),
            Spec::Polygon(s) => tree.grow_at(at, s, under),
            Spec::Polyline(s) => tree.grow_at(at, s, under),
            Spec::TextInput(s) => tree.grow_at(at, s, under),
        }
    }
}

/// A value that can be tweened -- the closed set foliage knows how to interpolate on an
/// element. For anything else, tween scalars yourself with `Canopy::tween` and apply the
/// numbers however you like.
#[derive(Copy, Clone)]
pub enum Motion {
    Opacity(f32),
    Color(Color),
    Elevation(Elevation),
    Location(Location),
    Polygon(Polygon),
    Outline(Outline),
}

/// When a tween runs and how it moves.
#[derive(Clone)]
pub struct Timing {
    /// Milliseconds from now until the tween starts.
    pub start: u64,
    /// Milliseconds from now until it finishes.
    pub finish: u64,
    pub ease: Ease,
    pub repeat: Repeat,
    /// Run each repeat backwards from where the last one ended, rather than restarting.
    pub backtrack: bool,
}

impl Timing {
    /// A single pass from now to `finish` milliseconds, linearly.
    pub fn over(finish: u64) -> Self {
        Self {
            start: 0,
            finish,
            ease: Ease::Linear,
            repeat: Repeat::default(),
            backtrack: false,
        }
    }
    /// Delays the start by `start` milliseconds.
    pub fn after(mut self, start: u64) -> Self {
        self.start = start;
        self
    }
    pub fn eased(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }
    pub fn repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }
    pub fn backtrack(mut self) -> Self {
        self.backtrack = true;
        self
    }
}

/// One queued instruction. Never named by user code -- the verbs on
/// [`Canopy`](crate::Canopy)/[`Sprig`](crate::Sprig) push these -- but it is the exhaustive
/// list of what user code can ask the engine to do, and the thing tests exercise.
pub(crate) enum Op {
    Grow {
        leaf: Leaf,
        under: Option<Leaf>,
        spec: Spec,
    },
    Prune(Leaf),
    Enable(Leaf),
    Disable(Leaf),
    Text {
        leaf: Leaf,
        value: String,
    },
    Color {
        leaf: Leaf,
        to: Color,
    },
    Opacity {
        leaf: Leaf,
        to: f32,
    },
    Visible {
        leaf: Leaf,
        yes: bool,
    },
    Location {
        leaf: Leaf,
        to: Location,
    },
    /// Repoints which element this one's `anchor()` values resolve against.
    Anchor {
        leaf: Leaf,
        to: Leaf,
    },
    Elevation {
        leaf: Leaf,
        to: Elevation,
    },
    FontSize {
        leaf: Leaf,
        to: FontSize,
    },
    GlyphColors {
        leaf: Leaf,
        to: GlyphColors,
    },
    Points {
        leaf: Leaf,
        to: Vec<Position<Logical>>,
    },
    DrawProgress {
        leaf: Leaf,
        to: f32,
    },
    Polygon {
        leaf: Leaf,
        to: Polygon,
    },
    Rounding {
        leaf: Leaf,
        to: Rounding,
    },
    Icon {
        leaf: Leaf,
        to: crate::IconId,
    },
    Animate {
        leaf: Leaf,
        to: Motion,
        timing: Timing,
        sequence: Option<Leaf>,
    },
    Sequence(Leaf),
    Timer {
        leaf: Leaf,
        millis: u64,
    },
    Hint {
        leaf: Leaf,
        text: String,
    },
    InputStyle {
        leaf: Leaf,
        style: crate::TextInputStyle,
    },
    Tween {
        tween: crate::Tween,
        channels: Vec<crate::boundary::tween::Channel>,
        timing: Timing,
    },
    Scroll {
        leaf: Leaf,
        to: ScrollTo,
    },
    Name {
        leaf: Leaf,
        name: String,
    },
    LoadAsset {
        key: AssetKey,
        source: AssetSource,
    },
}

impl Op {
    /// The `Leaf` this op acts on, if any. `LoadAsset` names no element.
    fn subject(&self) -> Option<Leaf> {
        match self {
            Op::Grow { leaf, .. }
            | Op::Text { leaf, .. }
            | Op::Color { leaf, .. }
            | Op::Opacity { leaf, .. }
            | Op::Visible { leaf, .. }
            | Op::Location { leaf, .. }
            | Op::Anchor { leaf, .. }
            | Op::Elevation { leaf, .. }
            | Op::FontSize { leaf, .. }
            | Op::GlyphColors { leaf, .. }
            | Op::Points { leaf, .. }
            | Op::DrawProgress { leaf, .. }
            | Op::Polygon { leaf, .. }
            | Op::Rounding { leaf, .. }
            | Op::Icon { leaf, .. }
            | Op::Animate { leaf, .. }
            | Op::Scroll { leaf, .. }
            | Op::Name { leaf, .. } => Some(*leaf),
            Op::Timer { leaf, .. } | Op::Hint { leaf, .. } | Op::InputStyle { leaf, .. } => {
                Some(*leaf)
            }
            Op::Prune(leaf) | Op::Enable(leaf) | Op::Disable(leaf) => Some(*leaf),
            // Its own leaf names something that does not exist yet, like a grow.
            Op::Sequence(_) => None,
            // Neither names an element: a tween is a stream of numbers, an asset is bytes.
            Op::Tween { .. } | Op::LoadAsset { .. } => None,
        }
    }
}

/// Applies a frame's queued ops in the order they were issued.
///
/// FIFO is the whole contract: ops land in the order the app wrote them, so a grow followed
/// by a write to the same `Leaf` behaves the way it reads.
///
/// An op naming a `Leaf` that has withered is dropped silently. That is what makes a stale
/// handle inert instead of a panic: the internal queries downstream still `.unwrap()`, and a
/// stale `Leaf` never reaches them.
pub(crate) fn apply(world: &mut World, queue: &mut Vec<Op>) {
    if queue.is_empty() {
        return;
    }
    for op in queue.drain(..) {
        // The sequence an `Animate` joins, if it still exists. Resolved with the subject
        // because both are liveness questions and both have to be answered before the tree
        // below takes the world.
        let mut joined = None;
        // A grow's own leaf is *supposed* to name nothing yet -- that is what the op is for.
        // Everything else, including a grow's parent, has to still be alive.
        let subject = match &op {
            Op::Grow { under, .. } => match under {
                // parent gone: the whole subtree the app meant to build under it is moot
                Some(parent) if !alive(world, *parent) => continue,
                Some(parent) => Some(parent.0),
                None => None,
            },
            Op::Sequence(_) | Op::Tween { .. } | Op::LoadAsset { .. } => None,
            // Resolved here, alongside every other liveness check, because the tree below
            // borrows the world mutably and this reads it.
            Op::Animate { leaf, sequence, .. } => {
                if !alive(world, *leaf) {
                    continue;
                }
                joined = sequence.filter(|seq| alive(world, *seq)).map(|seq| seq.0);
                Some(leaf.0)
            }
            // The one op whose *target* has to be alive as well as its subject. An `Anchor`
            // naming a withered element cannot resolve, and a box carrying `anchor()` values
            // that cannot resolve does not merely stay put -- it has nowhere to be at all.
            // Dropped rather than written, like every other op that names something gone.
            Op::Anchor { leaf, to } => {
                if !alive(world, *leaf) || !alive(world, *to) {
                    continue;
                }
                Some(leaf.0)
            }
            _ => match op.subject() {
                Some(leaf) if alive(world, leaf) => Some(leaf.0),
                _ => continue,
            },
        };
        let mut tree = crate::AsTree::tree(world);
        match op {
            Op::Grow { leaf, spec, .. } => spec.grow(&mut tree, leaf.0, subject),
            Op::Prune(_) => tree.remove(subject.unwrap()),
            Op::Enable(_) => tree.enable(subject.unwrap()),
            Op::Disable(_) => tree.disable(subject.unwrap()),
            Op::Text { value, .. } => tree.write_to(subject.unwrap(), crate::TextValue(value)),
            Op::Color { to, .. } => tree.write_to(subject.unwrap(), to),
            Op::Opacity { to, .. } => tree.write_to(subject.unwrap(), crate::Opacity::new(to)),
            Op::Visible { yes, .. } => tree.write_to(subject.unwrap(), crate::Visibility::new(yes)),
            Op::Location { to, .. } => tree.write_to(subject.unwrap(), to),
            Op::Anchor { to, .. } => tree.write_to(subject.unwrap(), crate::Anchor::new(to.0)),
            Op::Elevation { to, .. } => tree.write_to(subject.unwrap(), to),
            Op::FontSize { to, .. } => tree.write_to(subject.unwrap(), to),
            Op::GlyphColors { to, .. } => tree.write_to(subject.unwrap(), to),
            Op::Points { to, .. } => tree.write_to(subject.unwrap(), crate::PolylinePoints(to)),
            Op::DrawProgress { to, .. } => {
                tree.write_to(subject.unwrap(), crate::PolylineDrawProgress(to))
            }
            Op::Polygon { to, .. } => tree.write_to(subject.unwrap(), to),
            Op::Rounding { to, .. } => tree.write_to(subject.unwrap(), to),
            Op::Icon { to, .. } => tree.write_to(subject.unwrap(), crate::IconValue(to)),
            // A sequence whose leaf has withered is simply no sequence -- the animation still
            // runs, it just reports to nothing.
            Op::Animate { to, timing, .. } => {
                animate(&mut tree, subject.unwrap(), to, timing, joined)
            }
            Op::Sequence(leaf) => tree.sequence_at(leaf.0),
            Op::Timer { millis, leaf } => tree.timer_at(leaf.0, millis),
            Op::Hint { text, .. } => tree.write_to(subject.unwrap(), crate::HintText::new(text)),
            Op::InputStyle { style, .. } => tree.write_to(subject.unwrap(), style),
            Op::Tween {
                tween,
                channels,
                timing,
            } => {
                tree.spawn_tween(crate::boundary::tween::Tweening::new(
                    tween, channels, timing,
                ));
            }
            Op::Scroll { to, .. } => tree.write_to(subject.unwrap(), to),
            Op::Name { name, .. } => tree.name(subject.unwrap(), name),
            Op::LoadAsset { key, source } => {
                tree.send(crate::asset::LoadAsset { key, source });
            }
        }
        // Flushed per op rather than once at the end: an element's own structure is built by
        // reactions that only run on flush, and a later op in this same queue may name what
        // they produced -- or depend on this one existing before it can be parented to.
        world.flush();
    }
}

/// Whether `leaf` still names something. An allocated-but-not-yet-spawned id reports `false`,
/// which is correct for every op except a grow: nothing can be written to an element that
/// does not exist yet, and within one queue the grow always precedes the write.
fn alive(world: &World, leaf: Leaf) -> bool {
    world.entities().contains(leaf.0)
}

fn animate(tree: &mut Tree, entity: Entity, to: Motion, timing: Timing, sequence: Option<Entity>) {
    // Resolved before the macro so the two arms cannot spawn one each. An animation the app
    // did not join to a sequence -- and one whose sequence withered before it was applied --
    // still needs something to be counted against, so it gets an anonymous one that reports
    // to nobody. See `Tree::spawn_sequence`.
    let sequence = match sequence {
        Some(seq) => seq,
        None => tree.spawn_sequence(),
    };
    macro_rules! run {
        ($value:expr) => {{
            let anim = crate::Animation::new($value)
                .targeting(entity)
                .start(timing.start)
                .finish(timing.finish)
                .eased(timing.ease)
                .during(sequence);
            let anim = if timing.backtrack {
                anim.backtrack()
            } else {
                anim
            };
            let anim = match timing.repeat {
                Repeat::Once => anim,
                Repeat::Times(n) => anim.loops(n),
                Repeat::Forever => anim.forever(),
            };
            tree.animate(anim);
        }};
    }
    match to {
        Motion::Opacity(v) => run!(crate::Opacity::new(v)),
        Motion::Color(v) => run!(v),
        Motion::Elevation(v) => run!(v),
        Motion::Location(v) => run!(v),
        Motion::Polygon(v) => run!(v),
        Motion::Outline(v) => run!(v),
    }
}
