use core::time::Duration;

use crate::asset::{Bytes, Destination, Origin, Supply, retrieve};
use crate::aspen::{Aspen, Sequence, Tween};
use crate::clipboard::Clipboard;
use crate::clock::Clock;
use crate::coordinate::{Area, Axis, Position};
use crate::elm::Elm;
use crate::icon::{Field, Fields};
use crate::image::{Plate, Plates};
use crate::interaction::focus::Focus;
use crate::interaction::input::Incoming;
use crate::interaction::stack::Stack;
use crate::interaction::{Claim, Hold};
use crate::keyboard::Keyboard;
use crate::layout::{Layout, Short};
use crate::leaf::{Growth, Leaf, Presence};
use crate::link::Links;
use crate::naming::Naming;
use crate::op::Op;
use crate::palette::Scheme;
use crate::pollen::Drift;
use crate::queue::{Queue, Wake};
use crate::sprig::{Sprig, Watches};
use crate::text::Font;
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
    /// How the platform's loop is roused when work outside a frame queues an op. Installed by
    /// `photosynthesize`; absent under the headless suite, which runs its own frames.
    pub(crate) wake: Wake,
    /// The one source of names, drawn from here and from every [`Sprig`].
    pub(crate) naming: Naming,
    /// The handle the frame is reached by from off it, and what it publishes for one.
    pub(crate) sprig: Sprig,
    /// Every standing read a [`Sprig`] asked for.
    pub(crate) watched: Watches,
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
    /// The system clipboard, and what this program last put on it.
    pub(crate) clipboard: Clipboard,
    /// The soft keyboard, and which one is up.
    pub(crate) keyboard: Keyboard,
    /// Whether a URL reaches the host.
    pub(crate) links: Links,
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
        let tree = Tree::new();
        let naming = Naming::new(tree.remote());
        let queue = Queue::default();
        let wake = Wake::default();
        Self {
            sprig: Sprig::new(queue.clone(), wake.clone(), naming.clone()),
            watched: Watches::default(),
            tree,
            elm: Elm::default(),
            queue,
            wake,
            naming,
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
            clipboard: Clipboard::default(),
            keyboard: Keyboard::default(),
            links: Links::default(),
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

    /// Registers a font and hands back the name elements compose in.
    ///
    /// Takes the bytes, or an [`Origin`] to read them from -- and hands back the name either way, at
    /// once. A face that has yet to arrive is composed in the bundled one until it does, so a page
    /// laid out in [`letters`](crate::Source::letters) is laid out sensibly from the first frame and
    /// reflows when the real face lands. [`loaded`](crate::Pollen::loaded) is how an app waits for
    /// that instead.
    ///
    /// ```no_run
    /// # use foliage::{Grove, Origin};
    /// # fn f(grove: &mut Grove, bundled: &[u8]) {
    /// // `bundled` is what `include_bytes!("assets/mono.ttf")` produced.
    /// let here = grove.font(bundled);
    /// # #[cfg(not(target_family = "wasm"))]
    /// let read = grove.font(Origin::path("assets/mono.ttf"));
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// If bytes that were given outright are not a monospaced font. Every measurement foliage makes
    /// is a count of character cells -- [`letters`](crate::Source::letters), a letter-pitched track,
    /// max-content width, wrapping -- so a proportional font does not degrade, it silently puts
    /// every column address somewhere it does not belong.
    ///
    /// Bytes that were *read* are refused rather than panicked on, and reported as
    /// [`missing`](crate::Pollen::missing): what a path or a URL turned out to hold is not something
    /// the program stated.
    pub fn font(&mut self, bytes: impl Into<Bytes>) -> Font {
        let font = self.naming.face();
        match bytes.into().0 {
            Supply::Held(bytes) => self.fonts.register(font, &bytes),
            Supply::At(origin) => self.retrieve(Destination::Face(font), origin),
        }
        font
    }

    /// Registers a mark and hands back the name elements draw it by.
    ///
    /// The same registration [`Foliage::icon`](crate::Foliage::icon) is at boot, available at any
    /// frame -- which is what lets an app that takes root inside the first frame register its marks
    /// there rather than having to thread handles in from outside.
    ///
    /// `field` is a multi-channel signed distance field: `side` by `side` texels of RGBA, row-major.
    /// `range` is how many texels the baked distance spread covers. Both are stated by the app
    /// because they are facts about how the field was baked, and a fetched one cannot be asked.
    ///
    /// An element drawing a mark that has yet to arrive occupies its box and draws nothing, and
    /// appears in the frame it lands.
    ///
    /// # Panics
    ///
    /// If bytes that were given outright are smaller than `side` by `side` texels of RGBA. Bytes
    /// that were read are refused rather than panicked on, and reported as
    /// [`missing`](crate::Pollen::missing).
    pub fn icon(&mut self, field: impl Into<Bytes>, side: u32, range: f32) -> Field {
        let name = self.naming.mark();
        match field.into().0 {
            Supply::Held(bytes) => self.fields.register(name, &bytes, side, range),
            Supply::At(origin) => self.retrieve(Destination::Mark(name, side, range), origin),
        }
        name
    }

    /// Registers a picture and hands back the name elements draw it by.
    ///
    /// PNG or JPEG, decoded here. The format is read from the bytes and the size is what the decode
    /// says, so neither is stated: a name and a path can both be wrong about what a file holds, and
    /// the file cannot be.
    ///
    /// Pixels an app made itself are [`pixels`](crate::Grow::pixels), which states a size because
    /// there is nothing to read one from. An element drawing a picture that has yet to arrive
    /// occupies its box and draws nothing, and appears in the frame it lands.
    pub fn image(&mut self, bytes: impl Into<Bytes>) -> Plate {
        let plate = self.naming.plate();
        match bytes.into().0 {
            Supply::Held(bytes) => {
                if let Err(refused) = self.plates.decoded(plate, &bytes) {
                    tracing::warn!(plate = plate.0, reason = refused, "asset missing");
                    self.drift.missing.insert(plate.into());
                }
            }
            Supply::At(origin) => self.retrieve(Destination::Picture(plate), origin),
        }
        plate
    }

    /// Starts a read, against the queue and the wake it will come back through.
    fn retrieve(&mut self, destination: Destination, origin: Origin) {
        retrieve(&self.queue, &self.wake, destination, origin);
    }

    /// Opens the platform edges, once, at boot.
    ///
    /// Called by `photosynthesize` after the [`Wake`] is installed and by nothing else -- so the
    /// headless suite, which runs frames by hand, reaches none of them. That is what keeps a test
    /// off the clipboard, the keyboard and the browser of whoever is running it, and it is the same
    /// seam the wake itself sits on: what is here on both sides of it is engine state, and what is
    /// past it is the host.
    pub(crate) fn attach(&mut self) {
        self.clipboard.attach();
        self.keyboard.attach(&self.wake);
        self.links.attach();
    }

    /// A handle on the engine that can be carried off the frame.
    ///
    /// Everything an app writes here it can write from a thread, a promise or a callback through
    /// one of these, and it reads identically: a [`Sprig`] carries [`Grow`](crate::Grow) entire and
    /// pushes onto the one queue this does.
    ///
    /// Every call hands back the same handle rather than a new one, so the reports and the watches
    /// are one stream however many workers are holding it.
    pub fn sprig(&self) -> Sprig {
        self.sprig.clone()
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
        self.naming.leaf()
    }

    fn name(&self) -> Tween {
        self.naming.tween()
    }

    fn group(&self) -> Sequence {
        self.naming.sequence()
    }

    fn picture(&mut self) -> Plate {
        self.naming.plate()
    }
}
