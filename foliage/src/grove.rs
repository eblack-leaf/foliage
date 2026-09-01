use core::time::Duration;

use crate::clock::Clock;
use crate::coordinate::Area;
use crate::leaf::{Leaf, Presence};
use crate::op::Op;
use crate::pollen::Drift;
use crate::queue::Queue;
use crate::tree::Tree;
use crate::vein::{Sap, Vein};
use crate::verbs::Queues;

/// The surface a frame plants into and reads from.
pub struct Grove {
    pub(crate) tree: Tree,
    pub(crate) queue: Queue,
    pub(crate) clock: Clock,
    pub(crate) drift: Drift,
    pub(crate) viewport: Area,
    pub(crate) pending_resize: Option<Area>,
    pub(crate) again: bool,
    pub(crate) frames: u64,
}

impl Grove {
    pub(crate) fn new(viewport: Area) -> Self {
        Self {
            tree: Tree::new(),
            queue: Queue::default(),
            clock: Clock::new(),
            drift: Drift::default(),
            viewport,
            pending_resize: None,
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
        })
    }

    /// The visible area.
    pub fn viewport(&self) -> Area {
        self.viewport
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

    fn allocate(&self) -> Leaf {
        self.tree.allocate()
    }
}
