//! Elm -- change extraction.
//!
//! # Rowan recomputes. Elm decides what changed.
//!
//! Step 8, and the reason recomputing everything costs nothing downstream. Every renderer keeps what
//! the backend is holding and compares this frame's resolved values against it, so an element that
//! did not change costs one comparison and no upload.
//!
//! The cache is *what the backend holds*, not what the element last was. That is what makes a
//! skipped frame unable to lose a change: there is no flag to miss, only a value that still differs
//! at the next comparison. It is also a contract on the backend, which has to apply every batch it
//! is handed -- a dropped batch leaves the cache claiming something the backend does not have, and
//! nothing afterwards will correct it.
//!
//! # What is compared is not always what is uploaded
//!
//! A [`Panel`](crate::Panel) and a [`Polygon`](crate::Polygon) are described entirely in logical
//! pixels, so one value is both what is compared here and what the vertex buffer holds. The other
//! four are not, and each for the same reason: turning what the element declares into what the GPU
//! draws needs the display's density, and the density stops at the backend
//! ([`Ginkgo`](crate::ginkgo)).
//!
//! | | declares | the backend derives |
//! |---|---|---|
//! | [`Text`](crate::Text) | cells and characters | the cut ink, snapped to device pixels |
//! | [`Line`](crate::Line) | two ends and a weight | four corners, axis-aligned ones snapped |
//! | [`Icon`](crate::Icon) | a box and a field | the sheet rect, and the field's screen-space range |
//! | [`Image`](crate::Image) | a box and a plate | which texture to bind |
//!
//! So extraction is written in logical pixels throughout and compares logical values, and the
//! derivation happens once per written instance rather than once per frame.
//!
//! # What is not looked at
//!
//! An element resolution did not move is not visited: what the backend holds for it is what it
//! should go on holding. So what the backend is told to let go of is said outright, and there are
//! exactly two ways to earn it -- an element that moved and is not painted any more (hidden, culled,
//! or drawing an asset that has not arrived), and an element that withered, which resolution
//! reports when it builds the order again. A frame that moves nothing does nothing here.

use bevy_ecs::component::Component;
use tracing::field::Empty;
use tracing::trace_span;

use crate::aspen::Departed;
use crate::color::Color;
use crate::coordinate::{Area, Position, Section};
use crate::elevation::ResolvedElevation;
use crate::grove::Grove;
use crate::icon::{IconInstance, IconPigment};
use crate::image::{Fit, ImageInstance, ImagePigment};
use crate::leaf::{Leaf, Named};
use crate::line::{LineInstance, LinePigment};
use crate::palette::Fill;
use crate::panel::PanelInstance;
use crate::polygon::{PolygonInstance, PolygonPigment};
use crate::rounding::Corners;
use crate::rowan::Elements;
use crate::text::TextPigment;
use crate::text::font::Font;

/// Which renderer an element carries, and so which instances it is gathered into.
///
/// Decided when the element is described and never afterwards: what an element draws is part of
/// what it is, and one that is to draw something else is a different element. Nothing writes this
/// -- there is no op that can, and the only place it is set is where the element is grown.
///
/// Extraction walks the tree once and routes on this, so a further renderer costs a variant rather
/// than a pass over everything. It is the *only* thing that says what an element is: a set of
/// components that happens to look like a panel is not a panel.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Chlorophyll {
    /// Nothing. Carrying no renderer is the whole of what makes an element a [`Stem`](crate::Stem).
    #[default]
    None,
    /// A filled rectangle.
    Panel,
    /// A run of monospaced glyphs.
    Text,
    /// A filled regular polygon.
    Polygon,
    /// A straight stroke between two points.
    Line,
    /// A vector mark, from a distance field.
    Icon,
    /// A picture, fitted into its box.
    Image,
}

/// What a renderer was told, for whichever renderer the element carries.
///
/// Carried by the [`Bud`](crate::op::Bud) as one value so that growing an element inserts the
/// pigment its own renderer reads and no other. Past that point each is an ordinary component and
/// nothing asks which kind it was.
pub(crate) enum Pigment {
    Panel(PanelPigment),
    Text(TextPigment),
    Polygon(PolygonPigment),
    Line(LinePigment),
    Icon(IconPigment),
    Image(ImagePigment),
}

/// What a panel is filled and shaped by: everything the panel renderer was told.
///
/// Grown alongside [`Chlorophyll::Panel`] and by nothing else, so an element carries both or
/// neither. Ordinary declared state, and what [`color`](crate::Grow::color) and
/// [`round`](crate::Grow::round) write -- the decision beside it stays untouched, because an
/// element does not stop being a panel by being repainted.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct PanelPigment {
    pub(crate) fill: Fill,
    pub(crate) rounding: Corners,
}

/// What the backend is holding, one set per renderer.
#[derive(Default)]
pub(crate) struct Elm {
    pub(crate) panels: Instances<PanelInstance>,
    pub(crate) polygons: Instances<PolygonInstance>,
    pub(crate) lines: Instances<LineInstance>,
    pub(crate) icons: Instances<IconInstance>,
    pub(crate) images: Instances<ImageInstance>,
    pub(crate) texts: Runs,
    /// Where one run's glyphs are gathered before they are compared against what is held. Kept
    /// between frames for its capacity, and reused by every run in turn: a frame that changes
    /// nothing must not allocate.
    glyphs: Vec<Glyph>,
    /// Whether the backend was let go of, so the next extraction has nothing to compare against and
    /// states every element rather than only what moved.
    recut: bool,
}

impl Elm {
    /// How much this frame's batch moves, across every renderer. Reported to the trace, and the one
    /// number that says whether an unchanged frame really cost nothing.
    pub(crate) fn moved(&self) -> (usize, usize) {
        let written = self.panels.written.len()
            + self.polygons.written.len()
            + self.lines.written.len()
            + self.icons.written.len()
            + self.images.written.len()
            + self.texts.written.len();
        let withdrawn = self.panels.withdrawn.len()
            + self.polygons.withdrawn.len()
            + self.lines.withdrawn.len()
            + self.icons.withdrawn.len()
            + self.images.withdrawn.len()
            + self.texts.withdrawn.len();
        (written, withdrawn)
    }

    /// Opens an extraction on every renderer, dropping what the last one reported.
    fn open(&mut self) {
        self.panels.open();
        self.polygons.open();
        self.lines.open();
        self.icons.open();
        self.images.open();
        self.texts.open();
    }

    /// Closes an extraction on every renderer.
    fn close(&mut self) {
        self.panels.close();
        self.polygons.close();
        self.lines.close();
        self.icons.close();
        self.images.close();
        self.texts.close();
    }

    /// Lets go of whatever the backend holds for `leaf`, in the renderer it draws with.
    fn withdraw(&mut self, chlorophyll: Chlorophyll, leaf: Leaf) {
        let key = Key::from(leaf);
        match chlorophyll {
            Chlorophyll::None => {}
            Chlorophyll::Panel => self.panels.withdraw(key),
            Chlorophyll::Polygon => self.polygons.withdraw(key),
            Chlorophyll::Line => self.lines.withdraw(key),
            Chlorophyll::Icon => self.icons.withdraw(key),
            Chlorophyll::Image => self.images.withdraw(key),
            Chlorophyll::Text => self.texts.withdraw(key),
        }
    }

    /// Drops everything the backend is held to, so the next extraction writes the tree entire.
    ///
    /// Two things invalidate a comparison against what the backend holds, and both are facts about
    /// the backend rather than about any element:
    ///
    /// - **The density changed.** The backend's copy is in device pixels and this one is not, so a
    ///   display whose density changed leaves every derived instance -- a cut glyph, a snapped
    ///   stroke, a field's screen-space range -- correct against a density that is gone, while the
    ///   logical values they came from are unchanged and compare equal forever.
    /// - **The backend was rebuilt.** A platform that took its surface back and gave another one --
    ///   an Android activity leaving the foreground and returning -- leaves a cache describing
    ///   buffers that no longer exist. Every value would compare equal and nothing would be
    ///   uploaded, against renderers holding nothing.
    ///
    /// Total rather than per renderer, because neither the density nor the device is any
    /// renderer's.
    ///
    /// The next extraction states every element rather than only what moved, because after this
    /// there is nothing held for anything to have moved against.
    pub(crate) fn recut(&mut self) {
        self.recut = true;
        self.panels.forget();
        self.polygons.forget();
        self.lines.forget();
        self.icons.forget();
        self.images.forget();
        self.texts.forget();
    }
}

/// One glyph of a run: the cell it occupies, which character occupies it, and what it is filled
/// with.
///
/// The cell is in logical pixels and already offset by the run's own box, because where a character
/// lands is what wrapping decided and wrapping is the engine's. Where the *ink* sits inside that
/// cell is not here: that is the rasteriser's, which is the only thing that knows what shape it made
/// and at what density it made it.
///
/// The colour is per glyph because a [`tint`](crate::Grow::tint) is a fill over a range of the run's
/// own index space. A run with no tints resolves the same colour for every one of them, which costs
/// the comparison it would have cost anyway.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Glyph {
    pub(crate) cell: Section,
    pub(crate) character: char,
    pub(crate) color: Color,
}

/// The runs, and every glyph of each.
///
/// Deliberately not an [`Instances`]: a run is **one** entry in the one stack whose renderer holds
/// many things under it, and that is the whole difference. What is diffed here is the run entire --
/// its glyphs, its face, its rank and its clip -- because a run's glyphs move, refill and restack
/// together, and finding which of them changed would cost more than rewriting the run.
#[derive(Default)]
pub(crate) struct Runs {
    held: Named<Key, Run>,
    /// Runs the backend holds at something other than what is now wanted, or does not hold at all.
    /// Keys rather than values: what the backend is to apply is what is held for them, which it
    /// reads back at [`run`](Runs::run).
    pub(crate) written: Vec<Key>,
    /// Runs the backend holds and should not, in a stable order.
    pub(crate) withdrawn: Vec<Key>,
}

/// One run, as the backend is to hold it.
pub(crate) struct Run {
    /// Every glyph that leaves ink, in reading order. A space advances the wrap and is not one.
    pub(crate) glyphs: Vec<Glyph>,
    /// Which face the glyphs are cut from, and at what size. The backend rasterises against these,
    /// so they are part of what the run is rather than of how it was measured.
    pub(crate) font: Font,
    pub(crate) size: u32,
    pub(crate) rank: ResolvedElevation,
    pub(crate) clip: Section,
}

impl Runs {
    /// Opens an extraction, dropping what the last one reported.
    fn open(&mut self) {
        self.written.clear();
        self.withdrawn.clear();
    }

    /// Takes one run as it now stands, and reports it written if the backend is holding it
    /// otherwise.
    fn want(
        &mut self,
        key: Key,
        rank: ResolvedElevation,
        clip: Section,
        font: Font,
        size: u32,
        glyphs: &[Glyph],
    ) {
        match self.held.get_mut(&key) {
            Some(held) => {
                if held.font == font
                    && held.size == size
                    && held.rank == rank
                    && held.clip == clip
                    && held.glyphs == glyphs
                {
                    return;
                }
                held.font = font;
                held.size = size;
                held.rank = rank;
                held.clip = clip;
                // Rewritten in place, so a run that changed costs its own glyphs and no allocation.
                held.glyphs.clear();
                held.glyphs.extend_from_slice(glyphs);
            }
            None => {
                self.held.insert(
                    key,
                    Run {
                        glyphs: glyphs.to_vec(),
                        font,
                        size,
                        rank,
                        clip,
                    },
                );
            }
        }
        self.written.push(key);
    }

    /// Lets go of a run, if the backend is holding one.
    fn withdraw(&mut self, key: Key) {
        if self.held.remove(&key).is_some() {
            self.withdrawn.push(key);
        }
    }

    /// Closes the extraction.
    fn close(&mut self) {
        // Two identical runs have to extract identically, whatever order what went was found in.
        self.withdrawn.sort();
    }

    /// Drops what the backend is held to. See [`Elm::recut`].
    fn forget(&mut self) {
        self.held.clear();
    }

    /// What is held for one run, which is what the backend is to be holding for it.
    pub(crate) fn run(&self, key: Key) -> Option<&Run> {
        self.held.get(&key)
    }
}

/// What one instance is held under.
///
/// A plain number, and deliberately not a [`Leaf`]. An element is one of these -- its name's own
/// bits -- and so is a glyph inside a run, so a renderer whose element draws *many* things keeps its
/// own [`Instances`] under its own numbering and hands the one stack a single entry. Nothing about
/// holding, diffing or uploading has to know which of the two it is looking at.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(crate) struct Key(pub(crate) u64);

impl From<Leaf> for Key {
    fn from(leaf: Leaf) -> Self {
        Self(leaf.id())
    }
}

/// One renderer's instances: what the backend holds, and the difference the last extraction found.
///
/// Generic over the instance, because what a renderer sends is the renderer's own business. This
/// owns the comparison and knows nothing about what is being compared.
///
/// The rank is the one thing every renderer has in common, so it is carried here rather than
/// inside each renderer's instance: where an element sits in the one stack is a fact about the
/// element, not about what it happens to draw, and the backend needs it in a different form from
/// the one the resolver produced. Keeping it out is also what leaves an instance free to be
/// exactly the bytes a vertex buffer takes, for the renderers whose instance is those bytes.
pub(crate) struct Instances<I> {
    held: Named<Key, Held<I>>,
    /// Instances the backend does not hold, or holds at a different value or rank.
    pub(crate) written: Vec<Stacked<I>>,
    /// Instances the backend holds and should not, in a stable order.
    pub(crate) withdrawn: Vec<Key>,
}

/// One instance, where in the one stack it is to be drawn, and what it is clipped to.
///
/// The clip is beside the instance rather than inside it for the same reason the rank is: it is not
/// the renderer's data. It says where the backend is allowed to paint, which is a property of the
/// region the element sits in, and the backend applies it to the pass rather than to the panel.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Stacked<I> {
    pub(crate) key: Key,
    pub(crate) rank: ResolvedElevation,
    pub(crate) clip: Section,
    pub(crate) instance: I,
}

/// One instance the backend holds.
struct Held<I> {
    instance: I,
    rank: ResolvedElevation,
    clip: Section,
}

impl<I> Default for Instances<I> {
    fn default() -> Self {
        Self {
            held: Named::default(),
            written: Vec::new(),
            withdrawn: Vec::new(),
        }
    }
}

impl<I: Copy + PartialEq> Instances<I> {
    /// Opens an extraction, dropping what the last one reported.
    fn open(&mut self) {
        self.written.clear();
        self.withdrawn.clear();
    }

    /// States one instance as it now stands -- at the rank it resolved to and inside the clip it
    /// resolved under -- and reports it written if the backend is holding it otherwise.
    ///
    /// The holding is updated in place, so a frame that rewrites an instance allocates nothing, and
    /// one that states it at what is already held writes nothing.
    fn want(&mut self, key: impl Into<Key>, rank: ResolvedElevation, clip: Section, instance: I) {
        let key = key.into();
        match self.held.get_mut(&key) {
            Some(held) => {
                if held.instance == instance && held.rank == rank && held.clip == clip {
                    return;
                }
                held.instance = instance;
                held.rank = rank;
                held.clip = clip;
            }
            None => {
                self.held.insert(
                    key,
                    Held {
                        instance,
                        rank,
                        clip,
                    },
                );
            }
        }
        self.written.push(Stacked {
            key,
            rank,
            clip,
            instance,
        });
    }

    /// Lets go of an instance, if the backend is holding one.
    fn withdraw(&mut self, key: Key) {
        if self.held.remove(&key).is_some() {
            self.withdrawn.push(key);
        }
    }

    /// Closes the extraction.
    fn close(&mut self) {
        // Two identical runs have to extract identically, whatever order what went was found in.
        self.withdrawn.sort();
    }

    /// Drops what the backend is held to. See [`Elm::recut`].
    fn forget(&mut self) {
        self.held.clear();
    }

    /// How many instances the backend is holding.
    #[allow(unused)]
    pub(crate) fn len(&self) -> usize {
        self.held.len()
    }

    /// What the backend is holding for `key`.
    #[allow(unused)]
    pub(crate) fn holding(&self, key: impl Into<Key>) -> Option<I> {
        self.held.get(&key.into()).map(|held| held.instance)
    }
}

/// Step 8. Resolved state becomes instances, and only where it differs from what is already drawn.
pub(crate) fn run(grove: &mut Grove) {
    let step = trace_span!(
        "extract",
        written = Empty,
        withdrawn = Empty,
        glyphs = Empty,
        kept = Empty
    );
    let _entered = step.enter();
    grove.elm.open();
    // Detached for the walk so that gathering a run's glyphs -- which reads the shaping cache -- and
    // handing them over -- which writes what is held -- are not the same borrow. It goes back below,
    // with whatever capacity the widest run this frame gave it.
    let mut glyphs = core::mem::take(&mut grove.elm.glyphs);
    // Taken off the grove for the walk, and put back below. Resolution left it holding the order it
    // used, what each element settled at, and what moved, which is what decides how much of this
    // there is to do.
    let elements = core::mem::take(&mut grove.elements);
    let mut total = 0;
    let mut kept = 0;
    // Nothing is held to compare against, so nothing can be said to be unchanged.
    let recut = core::mem::take(&mut grove.elm.recut);
    // Whatever the backend held for an element that withered, it holds for nothing now.
    for &(leaf, chlorophyll) in &elements.gone {
        grove.elm.withdraw(chlorophyll, leaf);
    }
    let surface = Section::new(Position::default(), grove.viewport);
    for at in 0..elements.len() {
        let chlorophyll = elements.standing[at].chlorophyll;
        // Carrying no renderer is the whole of what makes an element a stem.
        if chlorophyll == Chlorophyll::None {
            continue;
        }
        // Nothing extraction reads about it moved, so what the backend holds for it is what it
        // should go on holding, and there is nothing to say.
        if !(elements.moved[at] || recut) {
            kept += 1;
            continue;
        }
        let stated = match painted(&elements, at, surface) {
            Some(painted) => {
                let drawing = Drawing {
                    elements: &elements,
                    at,
                    painted,
                };
                state(grove, &mut glyphs, drawing, chlorophyll, &mut total)
            }
            None => false,
        };
        // Moved, and not drawn: hidden, culled, or drawing something that has not arrived. What
        // the backend was holding for it has to go.
        if !stated {
            grove.elm.withdraw(chlorophyll, elements.order[at]);
        }
    }
    grove.elm.glyphs = glyphs;
    grove.elements = elements;
    step.record("kept", kept);
    grove.elm.close();
    let (written, withdrawn) = grove.elm.moved();
    step.record("written", written);
    step.record("withdrawn", withdrawn);
    step.record("glyphs", total);
}

/// One element being drawn: where it sits in the order resolution left, and what of it is painted.
struct Drawing<'a> {
    elements: &'a Elements,
    at: usize,
    painted: Painted,
}

/// States what one element draws, reporting whether it draws anything at all.
///
/// `false` for an element whose renderer has nothing to draw yet -- a mark whose field has not
/// arrived, a picture whose pixels have not -- which is absent from the batch rather than held as
/// blank.
fn state(
    grove: &mut Grove,
    glyphs: &mut Vec<Glyph>,
    drawing: Drawing,
    chlorophyll: Chlorophyll,
    total: &mut usize,
) -> bool {
    let Drawing {
        elements,
        at,
        painted,
    } = drawing;
    let leaf = elements.order[at];
    let rank = elements.rank[at];
    match chlorophyll {
        // Answered by the caller: carrying no renderer is the whole of what makes an element a stem.
        Chlorophyll::None => false,
        Chlorophyll::Panel => {
            // Grown together and by nothing else, so a panel always has one.
            let Some(pigment) = grove.tree.panel_pigment(leaf) else {
                return false;
            };
            let instance = PanelInstance::new(
                painted.section,
                tint(grove, leaf, pigment.fill).faded(painted.opacity),
                pigment.rounding,
            );
            grove.elm.panels.want(leaf, rank, painted.clip, instance);
            true
        }
        Chlorophyll::Polygon => {
            let Some(pigment) = grove.tree.polygon_pigment(leaf) else {
                return false;
            };
            // The shape is read plainly, with no blend applied here: a shape blends to a shape,
            // so a motion moving one writes it back over the declaration every frame and what
            // the element holds is already where the motion has reached.
            let instance = PolygonInstance::new(
                painted.section,
                tint(grove, leaf, pigment.fill).faded(painted.opacity),
                pigment.shape,
            );
            grove.elm.polygons.want(leaf, rank, painted.clip, instance);
            true
        }
        // A stroke's ends are resolved geometry in their own right, settled beside the box the way
        // the drawn box is: the box is the rectangle around them grown by half the weight, and
        // which of its two diagonals the stroke runs along is not something a rectangle can say.
        Chlorophyll::Line => {
            let (Some(pigment), Some(stretched), Some(stroke)) = (
                grove.tree.line_pigment(leaf),
                elements.stretched[at],
                grove.tree.stroke(leaf),
            ) else {
                return false;
            };
            let instance = LineInstance {
                from: stretched.from,
                to: stretched.to,
                color: tint(grove, leaf, pigment.fill).faded(painted.opacity),
                weight: stroke.weight,
                cap: pigment.cap,
            };
            grove.elm.lines.want(leaf, rank, painted.clip, instance);
            true
        }
        Chlorophyll::Icon => {
            let Some(pigment) = grove.tree.icon_pigment(leaf) else {
                return false;
            };
            // A field that has not arrived draws nothing and occupies its box, exactly as a
            // picture with no pixels does. Absent from the batch rather than held as blank: the
            // sheet cuts a mark when the batch names one, so an instance written before the
            // field landed would be a mark nothing ever asked the sheet for again.
            if grove.fields.mark(pigment.field).is_none() {
                return false;
            }
            let instance = IconInstance {
                // Square, because a distance field is: the mark sits in the largest square its
                // box holds rather than stretching to the box's own ratio.
                section: squared(painted.section),
                color: tint(grove, leaf, pigment.fill).faded(painted.opacity),
                field: pigment.field,
            };
            grove.elm.icons.want(leaf, rank, painted.clip, instance);
            true
        }
        Chlorophyll::Image => {
            let Some(pigment) = grove.tree.image_pigment(leaf) else {
                return false;
            };
            // A plate whose pixels have not arrived draws nothing and occupies its box. It is
            // absent from the batch rather than held as blank, so the frame the pixels land is
            // the frame it appears, with nothing to undo.
            let Some(picture) = grove.plates.size(pigment.plate) else {
                return false;
            };
            let (section, crop) = fitted(painted.section, picture, pigment.fit);
            let instance = ImageInstance {
                section,
                crop,
                radii: pigment.rounding.radii(section),
                opacity: painted.opacity,
                plate: pigment.plate,
            };
            grove.elm.images.want(leaf, rank, painted.clip, instance);
            true
        }
        // A run's box, fill, rank and clip resolve exactly as a panel's do. What is different is
        // that it draws *many* things at one rank: it is one entry in the one stack, and its
        // renderer holds its glyphs under its own numbering -- which is what a [`Key`] is a
        // number rather than a [`Leaf`] for.
        Chlorophyll::Text => {
            // Grown together and by nothing else, so a run always has both.
            let (Some(pigment), Some(typeface)) =
                (grove.tree.text_pigment(leaf), grove.tree.typeface(leaf))
            else {
                return false;
            };
            let color = tint(grove, leaf, pigment.fill).faded(painted.opacity);
            let size = typeface.size.at(grove.layout, grove.short);
            // The run R1 left the element holding, which is what it was measured and wrapped as.
            let Some(shaped) = elements.composed[at].as_deref() else {
                return false;
            };
            let tints = grove.tree.tints(leaf);
            let origin = painted.section.position;
            let cell = shaped.cell();
            glyphs.clear();
            // Wrapped at the width the run resolved to, which is the width it was measured at.
            shaped.place(painted.section.width(), |character, index, at| {
                glyphs.push(Glyph {
                    cell: Section::new(origin.moved(at), cell),
                    character,
                    // The run's own fill, unless a tint claims this character. Resolved here
                    // because this is where a fill becomes a colour, which is the same reason
                    // the run's own is.
                    color: match tints.and_then(|tints| tints.over(index)) {
                        Some(fill) => fill.color(&grove.scheme).faded(painted.opacity),
                        None => color,
                    },
                });
            });
            *total += glyphs.len();
            grove
                .elm
                .texts
                .want(leaf.into(), rank, painted.clip, typeface.font, size, glyphs);
            true
        }
    }
}

/// Where an element is painted, and inside what.
struct Painted {
    section: Section,
    clip: Section,
    opacity: f32,
}

/// What is painted of the element at `at`, or `None` if nothing is.
///
/// Hidden is the app's intent and culled is this pass's decision, taken here from the clip rect and
/// recorded nowhere: an element scrolled out of its region is absent from the batch and unchanged in
/// every other respect, so scrolling back to it needs nothing to be undone.
fn painted(elements: &Elements, at: usize, surface: Section) -> Option<Painted> {
    let inherited = elements.inherited[at];
    let section = elements.drawn[at];
    // What a scrolling ancestor leaves visible, never wider than the surface: a clip is what the
    // backend scissors the pass to, and nothing outside the surface is painted whatever the rect
    // says.
    let clip = elements.clip[at].intersect(surface);
    if !inherited.visible || section.intersect(clip).is_empty() {
        return None;
    }
    Some(Painted {
        section,
        clip,
        opacity: inherited.opacity,
    })
}

/// The largest square `section` holds, centred in it.
fn squared(section: Section) -> Section {
    let side = section.width().min(section.height());
    Section::new(
        Position::new(
            section.left() + (section.width() - side) / 2.0,
            section.top() + (section.height() - side) / 2.0,
        ),
        Area::new(side, side),
    )
}

/// The box a picture is drawn into, and what part of it is shown.
///
/// One of the two moves, never both: fitting inside the box changes the box and shows the whole
/// picture, and filling the box keeps the box and shows part of the picture. Stretching does
/// neither and is the only one that distorts.
fn fitted(section: Section, picture: Area, fit: Fit) -> (Section, [f32; 4]) {
    const WHOLE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
    if picture.width <= 0.0 || picture.height <= 0.0 || section.is_empty() {
        return (section, WHOLE);
    }
    let picture_ratio = picture.width / picture.height;
    let box_ratio = section.width() / section.height();
    match fit {
        Fit::Stretch => (section, WHOLE),
        Fit::Aspect => {
            let area = match box_ratio > picture_ratio {
                true => Area::new(section.height() * picture_ratio, section.height()),
                false => Area::new(section.width(), section.width() / picture_ratio),
            };
            let position = Position::new(
                section.left() + (section.width() - area.width) / 2.0,
                section.top() + (section.height() - area.height) / 2.0,
            );
            (Section::new(position, area), WHOLE)
        }
        Fit::Crop => {
            let (width, height) = match box_ratio > picture_ratio {
                true => (1.0, picture_ratio / box_ratio),
                false => (box_ratio / picture_ratio, 1.0),
            };
            (
                section,
                [(1.0 - width) / 2.0, (1.0 - height) / 2.0, width, height],
            )
        }
    }
}

/// What a fill currently paints as.
///
/// A blend of two fills is a color rather than a fill, so a motion on one is applied here -- where a
/// fill becomes a color -- and not written onto the element. Both ends are read against the same
/// scheme at the same instant, so a repaint mid-motion moves whichever of them is a role.
fn tint(grove: &Grove, leaf: Leaf, fill: Fill) -> Color {
    let target = fill.color(&grove.scheme);
    match grove.aspen.fill(leaf) {
        Some((Departed::Declared(fill), at)) => fill.color(&grove.scheme).blend(target, at),
        Some((Departed::Snapshot(color), at)) => color.blend(target, at),
        None => target,
    }
}
