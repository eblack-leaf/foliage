//! The greenhouse: an app grown with no window, for its own tests.
//!
//! An app's [`Root`] is the same value under a window and in here. What differs is everything the
//! platform does for it: input arrives because a test wrote it, time passes because a test moved
//! the clock, and a frame runs because a test asked for one. Past that point it is the one path --
//! a press written here enters the pipeline exactly where a translated mouse press does, and the
//! frame that answers it is the frame a window would have run.
//!
//! ```
//! use foliage::{
//!     Area, Boxed, Greenhouse, Grove, Grow, Leaf, Location, Palette, Panel, Place, Pollen,
//!     Position, Root, Source, left, top,
//! };
//!
//! struct Counter {
//!     button: Leaf,
//!     count: u32,
//! }
//!
//! impl Root for Counter {
//!     fn take_root(grove: &mut Grove) -> Self {
//!         let button = grove.plant(
//!             Panel::new()
//!                 .color(Palette::Accent)
//!                 .interactive()
//!                 .at(Location::new().xs(left(20.px()).width(80.px()), top(20.px()).height(40.px()))),
//!         );
//!         Counter { button, count: 0 }
//!     }
//!
//!     fn frame(&mut self, _grove: &mut Grove, pollen: Pollen) {
//!         if pollen.clicked(self.button) {
//!             self.count += 1;
//!         }
//!     }
//! }
//!
//! let mut house = Greenhouse::<Counter>::new(Area::new(200.0, 100.0));
//! house.frame();
//! house.tap(Position::new(60.0, 40.0));
//! house.frame();
//! // Reported in the frame after the one that saw it, as every report is.
//! house.frame();
//! assert_eq!(house.root().count, 1);
//! ```
//!
//! # A picture of a frame
//!
//! [`render`](Greenhouse::render) draws what the last frame settled into a texture no window
//! presents and reads it back: the same renderers, the same pass, and the same tree, so a picture
//! taken here is a picture of what the app shows. A device is acquired the first time one is asked
//! for and kept, and where the machine has none at all the answer is `None` rather than a failure.

use core::time::Duration;

use crate::coordinate::{Area, Position};
use crate::fern;
use crate::foliage::Tuning;
use crate::grove::Grove;
use crate::interaction::input::{Device, Input, Key, Modifiers, Phase};
use crate::pollen::Pollen;
use crate::root::{Registered, Root, Rooted};
#[cfg(not(target_family = "wasm"))]
use crate::{ash::Ash, color::Color, elm, ginkgo::Ginkgo, palette::Palette};

/// How far apart frames are when the greenhouse runs time on its own: a 60Hz display's.
const FRAME: Duration = Duration::from_millis(16);

/// How long [`settle`](Greenhouse::settle) runs frames for before deciding nothing will settle.
const PATIENCE: Duration = Duration::from_secs(30);

/// An app grown with no window: input written in, time moved by hand, and frames run on request.
///
/// Nothing runs on its own. Every input method queues what a window would have queued and returns,
/// and [`frame`](Greenhouse::frame) is what takes it in -- so a test reads the way the loop runs,
/// one frame at a time, and nothing it did not write can happen between two of its lines.
pub struct Greenhouse<R: Root> {
    grove: Grove,
    root: Registered<R>,
    /// What the last frame handed the app.
    pollen: Pollen,
    /// Where the pointer is, which is where a press lands.
    cursor: Position,
    /// Whether the pointer is down, which decides whether a move is a gesture.
    held: bool,
    #[cfg(not(target_family = "wasm"))]
    lamp: Option<Lamp>,
}

/// A device with nothing to present to, and the renderers built against it.
#[cfg(not(target_family = "wasm"))]
struct Lamp {
    ginkgo: Ginkgo,
    ash: Ash,
    /// What the target is sized to, so a resize of the grove can be followed.
    area: Area,
}

/// The app's turn in a frame, keeping what it was handed.
struct Kept<'a, R: Root> {
    root: &'a mut Registered<R>,
    pollen: Option<Pollen>,
}

impl<R: Root> Rooted for Kept<'_, R> {
    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        self.pollen = Some(pollen.clone());
        self.root.frame(grove, pollen);
    }
}

impl<R: Root> Greenhouse<R> {
    /// The app, registered and not yet grown, in a surface `viewport` logical pixels across.
    ///
    /// [`Root::take_root`] runs inside the first [`frame`](Greenhouse::frame), as it does under a
    /// window -- so anything tuned before then is in place when it does.
    pub fn new(viewport: Area) -> Self {
        Self {
            grove: Grove::new(viewport),
            root: Registered::new(),
            pollen: Pollen::default(),
            cursor: Position::default(),
            held: false,
            #[cfg(not(target_family = "wasm"))]
            lamp: None,
        }
    }

    /// Sets one of the engine's tuning values, as [`Foliage::tune`](crate::Foliage::tune) does.
    #[allow(private_bounds)]
    pub fn tune(&mut self, tuning: impl Tuning) -> &mut Self {
        tuning.tune(&mut self.grove);
        self
    }

    /// The app.
    ///
    /// # Panics
    ///
    /// Before the first [`frame`](Greenhouse::frame), which is where it is grown.
    pub fn root(&self) -> &R {
        self.root
            .0
            .as_ref()
            .expect("no frame has run: the app is grown in the first")
    }

    /// The app, to change between frames.
    ///
    /// # Panics
    ///
    /// Before the first [`frame`](Greenhouse::frame), which is where it is grown.
    pub fn root_mut(&mut self) -> &mut R {
        self.root
            .0
            .as_mut()
            .expect("no frame has run: the app is grown in the first")
    }

    /// The grove, to read what the last frame settled: [`tap`](Grove::tap),
    /// [`focused`](Grove::focused), [`hovered`](Grove::hovered).
    pub fn grove(&self) -> &Grove {
        &self.grove
    }

    /// The grove, to write to between frames. What is queued here lands in the next frame's
    /// drain, after the app's own turn -- as an op from outside a frame does under a window.
    pub fn grove_mut(&mut self) -> &mut Grove {
        &mut self.grove
    }

    /// What the last frame handed the app.
    pub fn pollen(&self) -> &Pollen {
        &self.pollen
    }

    /// Runs one frame, with the app taking its turn in it, and hands back what the app was handed.
    ///
    /// No time passes in it beyond what [`advance`](Greenhouse::advance) moved since the last.
    pub fn frame(&mut self) -> Pollen {
        let mut kept = Kept {
            root: &mut self.root,
            pollen: None,
        };
        fern::run(&mut self.grove, Some(&mut kept));
        self.pollen = kept.pollen.unwrap_or_default();
        // A device that exists is held to every batch, or what it holds stops being what extraction
        // thinks it holds -- and to the frame's size before the batch, as a window is resized
        // before the frame after it, so what is prepared at a size (the pictures a fade draws
        // apart) is prepared at the one it is drawn at.
        #[cfg(not(target_family = "wasm"))]
        if let Some(lamp) = &mut self.lamp {
            let area = self.grove.viewport;
            if lamp.area != area {
                lamp.ginkgo.resize(area, 1.0);
                lamp.area = area;
            }
            lamp.ash.absorb(
                &self.grove.elm,
                &self.grove.fonts,
                &self.grove.fields,
                &self.grove.plates,
                &lamp.ginkgo,
            );
        }
        self.pollen.clone()
    }

    /// Moves the clock, to be taken up by the next frame. Exact: a motion told five seconds have
    /// passed is five seconds further on, with none of the cap a window puts on a long gap.
    pub fn advance(&mut self, millis: u64) {
        self.grove.clock.advance(Duration::from_millis(millis));
    }

    /// Moves the clock and runs the frame that takes it up.
    pub fn step(&mut self, millis: u64) -> Pollen {
        self.advance(millis);
        self.frame()
    }

    /// Runs frames a display's interval apart until nothing more is owed: every motion landed,
    /// every report delivered, every gesture past becoming a hold. Hands back how many it ran.
    ///
    /// What an app asked for with [`again_in`](Grove::again_in) is not waited on: an ambient frame
    /// is asked for again every time, and would never settle.
    ///
    /// # Panics
    ///
    /// If thirty seconds of frames go by and something is still owed -- an app asking for
    /// [`again`](Grove::again) every frame, or a motion that does not end.
    pub fn settle(&mut self) -> usize {
        let mut frames = 0;
        let mut waited = Duration::ZERO;
        while self.grove.owed() {
            assert!(
                waited < PATIENCE,
                "the greenhouse ran {frames} frames and something is still owed"
            );
            self.advance(FRAME.as_millis() as u64);
            self.frame();
            frames += 1;
            waited += FRAME;
        }
        frames
    }

    /// Resizes the surface, taking effect at the next frame.
    pub fn resize(&mut self, viewport: Area) {
        self.grove.pending_resize = Some(viewport);
    }

    /// Moves the pointer. A move with the pointer down is a gesture; with it up, it is where the
    /// cursor rests -- what is hovered, and where the next press lands.
    pub fn point(&mut self, at: Position) {
        self.cursor = at;
        self.grove.rest(Some(at));
        if self.held {
            self.grove
                .incoming
                .contact(Phase::Moved, at, 0, Device::Mouse);
            self.grove.incoming.take(Input::Moved(at));
        }
    }

    /// Puts the pointer down at `at`, moving it there first if it is not.
    pub fn press(&mut self, at: Position) {
        if at != self.cursor || self.grove.cursor.is_none() {
            self.point(at);
        }
        self.held = true;
        self.grove
            .incoming
            .contact(Phase::Down, at, 0, Device::Mouse);
        self.grove.incoming.take(Input::Pressed(at));
    }

    /// Lifts the pointer at `at`, dragging it there first if it is not.
    pub fn release(&mut self, at: Position) {
        if at != self.cursor {
            self.point(at);
        }
        self.held = false;
        self.grove.incoming.contact(Phase::Up, at, 0, Device::Mouse);
        self.grove.incoming.take(Input::Released(at));
    }

    /// A press and a release at one point, taken in by the same frame.
    pub fn tap(&mut self, at: Position) {
        self.press(at);
        self.release(at);
    }

    /// Takes the gesture away rather than finishing it, as a window losing focus does.
    pub fn cancel(&mut self) {
        if self.held {
            self.held = false;
            self.grove
                .incoming
                .contact(Phase::Cancelled, self.cursor, 0, Device::Mouse);
            self.grove.incoming.take(Input::Cancelled);
        }
    }

    /// The pointer leaves the surface: nothing is under it, and a gesture it was making is taken
    /// away.
    pub fn leave(&mut self) {
        if self.grove.cursor.is_some() {
            self.grove.rest(None);
        }
        self.cancel();
    }

    /// A wheel or trackpad scroll at `at`, stated as the distance a drag the same way would cover.
    pub fn wheel(&mut self, at: Position, delta: Position) {
        if at != self.cursor {
            self.point(at);
        }
        self.grove.incoming.take(Input::Wheeled { at, delta });
    }

    /// One key, pressed on its own.
    pub fn key(&mut self, key: Key) {
        self.grove.incoming.take(Input::Keyed(key));
    }

    /// One key pressed with `modifiers` held, which is three events in order: what goes down, the
    /// key, and everything coming back up.
    pub fn chord(&mut self, modifiers: Modifiers, key: Key) {
        self.grove.incoming.take(Input::Modifiers(modifiers));
        self.grove.incoming.take(Input::Keyed(key));
        self.grove
            .incoming
            .take(Input::Modifiers(Modifiers::default()));
    }

    /// A run of characters, one keystroke each, as a keyboard would type them.
    pub fn type_text(&mut self, text: &str) {
        for character in text.chars() {
            self.key(Key::Typed(character));
        }
    }

    /// Draws what the last frame settled and reads it back.
    ///
    /// At one device pixel per logical pixel, so a point in the picture is the point the tree was
    /// laid out in. `None` where the machine has no adapter to draw with.
    ///
    /// The first call acquires the device, and what is already grown is stated to it whole; every
    /// frame after that is handed to it as it runs, as a window's renderers are.
    #[cfg(not(target_family = "wasm"))]
    pub fn render(&mut self) -> Option<Picture> {
        if self.lamp.is_none() {
            let area = self.grove.viewport;
            let ginkgo = pollster::block_on(Ginkgo::offscreen(area, 1.0))?;
            let mut ash = Ash::new(&ginkgo);
            // A backend built fresh holds nothing, so the extraction that fills it states every
            // element rather than what moved.
            self.grove.elm.recut();
            elm::run(&mut self.grove);
            ash.absorb(
                &self.grove.elm,
                &self.grove.fonts,
                &self.grove.fields,
                &self.grove.plates,
                &ginkgo,
            );
            self.lamp = Some(Lamp { ginkgo, ash, area });
        }
        let lamp = self.lamp.as_ref()?;
        let clear = self.grove.scheme().color(Palette::Surface);
        lamp.ash.draw(&lamp.ginkgo, clear);
        let (width, height, pixels) = lamp.ginkgo.read()?;
        Some(Picture {
            width,
            height,
            pixels,
        })
    }
}

/// A frame as it was drawn: RGBA, one byte a channel, row-major, sRGB.
#[cfg(not(target_family = "wasm"))]
#[derive(Clone, PartialEq, Eq)]
pub struct Picture {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

#[cfg(not(target_family = "wasm"))]
impl Picture {
    /// How many texels across, which at one device pixel per logical pixel is the viewport's
    /// width rounded.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// How many texels down.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Every texel, RGBA, row-major, with nothing between rows.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// The colour drawn at a point, or `None` off the picture.
    pub fn at(&self, at: Position) -> Option<Color> {
        if at.x < 0.0 || at.y < 0.0 {
            return None;
        }
        let (x, y) = (at.x as u32, at.y as u32);
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = ((y * self.width + x) * 4) as usize;
        let texel = &self.pixels[index..index + 4];
        let channel = |byte: u8| byte as f32 / 255.0;
        Some(Color::rgba(
            channel(texel[0]),
            channel(texel[1]),
            channel(texel[2]),
            channel(texel[3]),
        ))
    }

    /// The picture as a PNG file, to keep beside a test or look at when one fails.
    pub fn png(&self) -> Vec<u8> {
        use image::ImageEncoder;

        let mut file = Vec::new();
        image::codecs::png::PngEncoder::new(&mut file)
            .write_image(
                &self.pixels,
                self.width,
                self.height,
                image::ExtendedColorType::Rgba8,
            )
            .expect("an RGBA picture encodes");
        file
    }
}

#[cfg(not(target_family = "wasm"))]
impl core::fmt::Debug for Picture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Picture")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}
