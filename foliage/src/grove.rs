use core::time::Duration;

use crate::aspen::{Aspen, Sequence, Tween};
use crate::clock::Clock;
use crate::coordinate::{Area, Axis, Position};
use crate::elm::Elm;
use crate::icon::{Field, Fields};
use crate::image::{Plate, Plates};
use crate::interaction::focus::Focus;
use crate::interaction::input::Incoming;
use crate::interaction::stack::Stack;
use crate::interaction::{Claim, Hold};
use crate::layout::{Layout, Short};
use crate::leaf::{Growth, Leaf, Presence};
use crate::op::Op;
use crate::palette::Scheme;
use crate::pollen::Drift;
use crate::queue::Queue;
use crate::text::font::Fonts;
use crate::text::shape::Shaping;
use crate::tree::Tree;
use crate::vein::{Sap, Vein};
use crate::verbs::Queues;
use crate::view::{Coasting, Momentum, ScrollTo, progress};

/// The surface a frame plants into and reads from.
pub struct Grove {
    pub(crate) tree: Tree,
    pub(crate) elm: Elm,
    pub(crate) queue: Queue,
    pub(crate) clock: Clock,
    /// Every tween that is running, and the names the channels are drawn from.
    pub(crate) aspen: Aspen,
    /// Every registered font. Read by R1 alone, which is the only pass that asks a font anything.
    pub(crate) fonts: Fonts,
    /// Every registered mark's distance field. Read by the backend, which packs them onto its sheet.
    pub(crate) fields: Fields,
    /// Every registered picture's pixels, and the names elements draw them by.
    pub(crate) plates: Plates,
    /// The one thing kept between frames: every run that has been shaped.
    pub(crate) shaping: Shaping,
    pub(crate) drift: Drift,
    pub(crate) viewport: Area,
    pub(crate) pending_resize: Option<Area>,
    pub(crate) layout: Layout,
    pub(crate) short: Short,
    pub(crate) scheme: Scheme,
    /// What arrived from the platform, and the gesture it is making.
    pub(crate) incoming: Incoming,
    /// What the last frame drew, which is what a gesture is resolved against.
    pub(crate) stack: Stack,
    pub(crate) focus: Focus,
    /// Every region still moving from a release, held beside the tree like the running tweens.
    pub(crate) coasting: Coasting,
    /// The destinations written this frame, waiting on the extent R3 measures. Drained at R4.
    pub(crate) sought: Vec<(Leaf, ScrollTo)>,
    /// How far a gesture travels before it is claimed as a drag. Tuned at boot.
    pub(crate) claim: Claim,
    /// How long a press is down before it is a hold. Tuned at boot.
    pub(crate) hold: Hold,
    /// How a released drag coasts. Tuned at boot.
    pub(crate) momentum: Momentum,
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
            aspen: Aspen::default(),
            fonts: Fonts::new(),
            fields: Fields::default(),
            plates: Plates::default(),
            shaping: Shaping::default(),
            drift: Drift::default(),
            viewport,
            pending_resize: None,
            layout: Layout::of(viewport),
            short: Short::No.next(viewport),
            scheme: Scheme::default(),
            incoming: Incoming::default(),
            stack: Stack::default(),
            focus: Focus::default(),
            coasting: Coasting::default(),
            sought: Vec::new(),
            claim: Claim::default(),
            hold: Hold::default(),
            momentum: Momentum::default(),
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
            Vein::Color => Sap::Color(self.tree.fill(leaf)?),
            Vein::Rounding => Sap::Rounding(self.tree.rounding(leaf)?),
            Vein::Ends => {
                let stretched = self.tree.stretched(leaf)?;
                Sap::Ends(stretched.from, stretched.to)
            }
            Vein::Weight => Sap::Weight(self.tree.stroke(leaf)?.weight),
            Vein::Cap => Sap::Cap(self.tree.line_pigment(leaf)?.cap),
            Vein::Shape => Sap::Shape(self.tree.shape(leaf)?),
            Vein::Mark => Sap::Mark(self.tree.icon_pigment(leaf)?.field),
            Vein::Picture => Sap::Picture(self.tree.image_pigment(leaf)?.plate),
            Vein::Fit => Sap::Fit(self.tree.image_pigment(leaf)?.fit),
            // A field says what its run says. Every verb and every read is addressed to the field,
            // so what it is made of is never a name an app has to hold.
            Vein::Text => Sap::Text(match self.tree.parts(leaf) {
                Some(parts) => self.tree.lettering(parts.run)?.to_string(),
                None => self.tree.lettering(leaf)?.to_string(),
            }),
            Vein::Selection => {
                self.tree.parts(leaf)?;
                Sap::Selection(self.tree.editing(leaf).span())
            }
            Vein::Visible => Sap::Visible(self.tree.visible(leaf).0),
            Vein::Opacity => Sap::Opacity(self.tree.opacity(leaf).0),
            Vein::Disabled => Sap::Disabled(self.tree.disabled(leaf).0),
            // The three a region has and nothing else does. `None` where the element does not
            // scroll, which is the reading that keeps "an axis that was not declared has no extent"
            // true of the whole element as well as of one of its axes.
            Vein::Offset => {
                self.tree.scrolls(leaf)?;
                Sap::Position(self.tree.offset(leaf))
            }
            Vein::Extent => {
                self.tree.scrolls(leaf)?;
                Sap::Area(self.tree.extent(leaf))
            }
            Vein::Progress => {
                self.tree.scrolls(leaf)?;
                let (offset, extent) = (self.tree.offset(leaf), self.tree.extent(leaf));
                let own = self.tree.placed(leaf).area;
                Sap::Progress(Position::new(
                    progress(offset, extent, own, Axis::Horizontal),
                    progress(offset, extent, own, Axis::Vertical),
                ))
            }
        })
    }

    /// Registers a mark and hands back the name elements draw it by.
    ///
    /// The same registration [`Foliage::icon`](crate::Foliage::icon) is at boot, available at any
    /// frame -- which is what lets an app that takes root inside the first frame register its marks
    /// there rather than having to thread handles in from outside.
    ///
    /// Not an op, where loading a picture is one. A field is written once and never changes, so
    /// there is nothing for it to be ordered against; a picture's pixels are replaced over the life
    /// of the program, so when they land relative to everything else is a real question and the
    /// queue is what answers it.
    ///
    /// # Panics
    ///
    /// If `field` is smaller than `side` by `side` texels of RGBA.
    pub fn icon(&mut self, field: &[u8], side: u32, range: f32) -> Field {
        self.fields.register(field, side, range)
    }

    /// What holds focus, if anything does.
    ///
    /// Frame-wide rather than per-element, because focus is: one element holds it, and asking every
    /// element in turn whether it is that one is a worse way to ask the same question. Changes are
    /// reported through [`Pollen`](crate::Pollen) like anything else.
    pub fn focused(&self) -> Option<Leaf> {
        self.focus.held()
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

    /// What every [`Palette`](crate::Palette) role currently resolves to.
    ///
    /// Written with [`repaint`](crate::Grow::repaint), which lands at the drain like any other op --
    /// so this is what the frame was drawn in, not what a repaint queued this frame will make it.
    pub fn scheme(&self) -> Scheme {
        self.scheme
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

    fn name(&self) -> Tween {
        self.aspen.name()
    }

    fn group(&self) -> Sequence {
        self.aspen.group()
    }

    fn picture(&mut self) -> Plate {
        self.plates.name()
    }
}
