use core::time::Duration;

use crate::clock::Clock;
use crate::coordinate::Area;
use crate::elm::Elm;
use crate::layout::{Layout, Short};
use crate::leaf::{Growth, Leaf, Presence};
use crate::op::Op;
use crate::pollen::Drift;
use crate::queue::Queue;
use crate::tree::Tree;
use crate::vein::{Sap, Vein};
use crate::verbs::Queues;

/// The surface a frame plants into and reads from.
pub struct Grove {
    pub(crate) tree: Tree,
    pub(crate) elm: Elm,
    pub(crate) queue: Queue,
    pub(crate) clock: Clock,
    pub(crate) drift: Drift,
    pub(crate) viewport: Area,
    pub(crate) pending_resize: Option<Area>,
    pub(crate) layout: Layout,
    pub(crate) short: Short,
    pub(crate) again: bool,
    pub(crate) frames: u64,
}

impl Grove {
    pub(crate) fn new(viewport: Area) -> Self {
        Self {
            tree: Tree::new(),
            elm: Elm::default(),
            queue: Queue::default(),
            clock: Clock::new(),
            drift: Drift::default(),
            viewport,
            pending_resize: None,
            layout: Layout::of(viewport),
            short: Short::No.next(viewport),
            again: false,
            frames: 0,
        }
    }

    /// What `leaf` names right now.
    pub fn presence(&self, leaf: Leaf) -> Presence {
        self.tree.presence(leaf)
    }

    /// Reads one property of an element, or `None` if it has withered, has not been grown yet, or
    /// does not carry that property.
    pub fn tap(&self, leaf: Leaf, vein: Vein) -> Option<Sap> {
        if !self.tree.is_live(leaf) {
            return None;
        }
        Some(match vein {
            Vein::Branches => Sap::Leaves(self.tree.branches(leaf)),
            Vein::Trunk => Sap::Leaf(self.tree.trunk(leaf)),
            Vein::Placed => Sap::Section(self.tree.placed(leaf)),
            Vein::Drawn => Sap::Section(self.tree.drawn(leaf)),
            Vein::Anchor => Sap::Leaf(self.tree.anchor(leaf)),
            Vein::Elevation => Sap::Elevation(self.tree.elevation(leaf)),
            Vein::Color => Sap::Color(self.tree.pigment(leaf)?.color),
            Vein::Rounding => Sap::Rounding(self.tree.pigment(leaf)?.rounding),
        })
    }

    /// The visible area.
    pub fn viewport(&self) -> Area {
        self.viewport
    }

    /// The width breakpoint in force, which every placement is read against.
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Whether the viewport is vertically cramped.
    pub fn short(&self) -> Short {
        self.short
    }

    /// How long the last frame took.
    pub fn frame_time(&self) -> Duration {
        self.clock.delta()
    }

    /// Time since the engine was built.
    pub fn elapsed(&self) -> Duration {
        self.clock.elapsed()
    }

    /// Asks for another frame after this one.
    ///
    /// The engine idles when nothing is owed. An app driving its own motion from
    /// [`frame_time`](Grove::frame_time) -- coasting a scroll, running a hand-rolled transition --
    /// has nothing the engine can detect, and calls this for as long as it is doing something.
    pub fn again(&mut self) {
        self.again = true;
    }
}

impl Queues for Grove {
    fn queue(&mut self, op: Op) {
        self.queue.push(op);
    }

    fn allocate(&self) -> (Leaf, Growth) {
        self.tree.allocate()
    }
}
