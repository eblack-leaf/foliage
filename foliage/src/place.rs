//! Stating what an element is when it is described, before it exists.

use crate::coordinate::Axes;
use crate::elevation::Elevation;
use crate::interaction::{Gestures, Shape};
use crate::leaf::Leaf;
use crate::lifecycle::{Opacity, Visible};
use crate::line::Stroke;
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::placement::trace::Trace;
use crate::text::font::{Font, FontSize, Typeface};
use crate::view::{Escape, Scroll, Scrolls};

/// Where the caller was standing. Carried from the call that wrote a placement to the drain that
/// applies it, so a refusal names the write rather than the pass that noticed it.
pub(crate) type Caller = &'static core::panic::Location<'static>;

/// What a seed carries about where it will sit. Anything left unsaid takes its default.
///
/// [`location`](Placement::location) and [`traced`](Placement::traced) are the two ways to say it,
/// and an element states one of them: a box is a rectangle the grammar resolves, and a trace is the
/// same grammar read as vertices. Which of the two a seed offers is a type -- [`Boxed`] against
/// [`Line::trace`](crate::Line::trace) -- so nothing can state both and no pass has to decide
/// which was meant.
#[derive(Clone, Debug, Default)]
pub(crate) struct Placement {
    pub(crate) location: Option<Location>,
    /// The two ends of a stroked element, per breakpoint, in place of a box.
    pub(crate) traced: Option<Trace>,
    /// How thick a stroked element is drawn. Placement input rather than decoration: it is what a
    /// trace's box is inflated by, so a rule with two ends on one line still has a box.
    pub(crate) stroke: Option<Stroke>,
    pub(crate) grid: Option<Grid>,
    pub(crate) anchor: Option<Anchored>,
    pub(crate) elevation: Option<Elevation>,
    /// Absent until the element names a font or a size, because an element with neither has no
    /// character cell and reads zero for everything measured in one.
    pub(crate) typeface: Option<Typeface>,
    pub(crate) manner: Manner,
    /// How it is turned, scaled and slid where it is drawn.
    pub(crate) pose: crate::pose::Pose,
}

impl Placement {
    /// What a mark starts from: nothing stated, and a gesture over it passing through.
    pub(crate) fn mark() -> Self {
        Self {
            manner: Manner {
                gestures: Gestures::mark(),
                ..Manner::default()
            },
            ..Self::default()
        }
    }
}

/// What a seed carries about how it behaves: what it does with a gesture, whether it scrolls, and
/// which of the ways to be off it starts in.
///
/// Everything here is one element's own declaration. None of it predicts what a gesture will turn
/// out to mean, and none of it is derived from anything else -- which is the whole test for
/// belonging here rather than being worked out at the time.
#[derive(Clone, Debug, Default)]
pub(crate) struct Manner {
    pub(crate) gestures: Gestures,
    pub(crate) scrolls: Option<Scrolls>,
    /// Declared [`pinned`](Place::pinned): does not travel with its region's content.
    pub(crate) pinned: bool,
    /// Declared [`floats`](Place::floats): sits over its region rather than in it, and how far out
    /// of the regions above it that reaches.
    pub(crate) floats: Option<Escape>,
    pub(crate) focusing: Focusing,
    pub(crate) visible: Visible,
    pub(crate) opacity: Opacity,
    /// Declared [`together`](Place::together): drawn with what is under it as one picture while
    /// it fades.
    pub(crate) together: bool,
}

/// Where an element sits in focus order, and whether it holds focus inside itself.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Focusing {
    /// What [`focus_order`](Place::focus_order) declared. Equal values keep reading order.
    pub(crate) order: i32,
    /// Declared [`focus_scope`](Place::focus_scope).
    pub(crate) scope: bool,
}

/// The one other element a placement may read, and where it was named.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Anchored {
    pub(crate) to: Leaf,
    pub(crate) at: Caller,
}

/// Reaching a seed's placement.
pub(crate) trait Places {
    fn placement(&mut self) -> &mut Placement;
}

/// Stating where an element will sit, and how it behaves.
///
/// Implemented by every [`Seed`](crate::Seed), so this reads identically whatever is being grown.
/// Sealed: it can be called, never implemented.
#[allow(private_bounds)]
pub trait Place: Places + Sized {
    /// How this element's box is divided for the elements grown under it.
    ///
    /// Undeclared, it is a single column and a single row.
    fn grid(mut self, grid: Grid) -> Self {
        self.placement().grid = Some(grid);
        self
    }

    /// How far in front of its trunk the element sits, from [`Elevation::up`] or
    /// [`Elevation::down`].
    ///
    /// Undeclared, it sits at its trunk's own elevation, which leaves it just in front of it.
    ///
    /// On every element, not only on one that draws: what is grown under an element accumulates
    /// from its elevation, so a wrapper that carried none would flatten the subtree beneath it.
    fn elevate(mut self, elevation: Elevation) -> Self {
        self.placement().elevation = Some(elevation);
        self
    }

    /// The one other element this one's placement may read through [`anchor()`](crate::anchor).
    ///
    /// It carries every reading a trunk does, so an element grown away from what it describes --
    /// to clear a stack, or a clip -- goes on addressing that element's grid, box, font and measure
    /// in the same words.
    ///
    /// # Panics
    ///
    /// If the anchor would close a cycle. A ↔ B is a contradiction rather than an ordering problem:
    /// an element whose placement cannot resolve has no box, so there is no state for it to fall
    /// back to. It is refused when the op is applied, naming both elements and the write that did
    /// it, which leaves the tree acyclic by construction.
    #[track_caller]
    fn anchored(mut self, to: Leaf) -> Self {
        self.placement().anchor = Some(Anchored {
            to,
            at: core::panic::Location::caller(),
        });
        self
    }

    /// Which registered font the element composes in.
    ///
    /// On every element, not only on one that draws glyphs: a font and a size are what give a
    /// character cell its size, and a cell is what [`letters`](crate::Source::letters) and a
    /// letter-pitched track are measured in. An element that names neither has no cell and reads
    /// zero for both.
    ///
    /// Undeclared, it is the bundled font.
    fn font(mut self, font: Font) -> Self {
        self.placement().typeface.get_or_insert_default().font = font;
        self
    }

    /// How large the element's characters are, per breakpoint.
    ///
    /// Undeclared, it is [`FontSize::DEFAULT`].
    fn font_size(mut self, size: FontSize) -> Self {
        self.placement().typeface.get_or_insert_default().size = size;
        self
    }

    /// The element receives gestures.
    ///
    /// A gesture goes to the top of the box stack whatever is there; this decides whether the
    /// element at the top does anything with it. An element at the top that did not say this
    /// **eats** the gesture, which is what a backdrop, a sheet backing and a menu's padding are,
    /// and none of them declares anything to be it.
    ///
    /// It is also what makes an element reachable by focus, because the set that asked to receive
    /// input is the set a keyboard should be able to reach.
    ///
    /// On a mark it overrides the passing through a mark starts with. What takes one out of reach
    /// is [`visible`](Place::visible), [`disable`](crate::Grow::disable),
    /// [`intangible`](Place::intangible), or a fade to nothing -- on it or on anything it is grown
    /// under. A press target that shows nothing is a [`Stem`](crate::Stem), or a panel in a clear
    /// color, at full opacity.
    fn interactive(mut self) -> Self {
        self.placement().manner.gestures.receives = true;
        self
    }

    /// A gesture over the element goes to whatever is beneath it.
    ///
    /// What a composite marks its own decoration with. A highlight, a gradient across a card, the
    /// ground of a row -- each of them is above its target at those pixels, and each would
    /// otherwise be what a press lands on. A mark -- [`Text`](crate::Text),
    /// [`Icon`](crate::Icon), [`Polygon`](crate::Polygon), [`Line`](crate::Line) -- already
    /// passes gestures through unless it says [`tangible`](Place::tangible), so it needs nothing.
    ///
    /// It does not take the element out of the box stack, only out of what may be the top of it:
    /// the drag that follows a press still finds the region containing it, so scrolling works over
    /// decoration exactly as it does over anything else.
    ///
    /// Not what a backdrop, a sheet backing or a menu's padding is. Those are solid and eat the
    /// gesture, which is what an element at the top that did not declare
    /// [`interactive`](Place::interactive) already does.
    fn intangible(mut self) -> Self {
        self.placement().manner.gestures.intangible = true;
        self
    }

    /// A gesture over the element stops on it: the opposite of
    /// [`intangible`](Place::intangible).
    ///
    /// What every surface -- a panel, an image, a stem -- already is, and what a mark is not until
    /// it says so. For a mark that should block what is beneath it without receiving anything
    /// itself; one that receives says [`interactive`](Place::interactive), which is tangible
    /// already.
    fn tangible(mut self) -> Self {
        let gestures = &mut self.placement().manner.gestures;
        gestures.intangible = false;
        gestures.passes = false;
        self
    }

    /// Which drags this element takes.
    ///
    /// The one thing about a gesture the engine cannot work out for itself: a slider taking drags
    /// along its own axis is information only the app has.
    ///
    /// Undeclared, an element **takes no drags** -- so it holds a gesture only until that gesture
    /// becomes a drag and then yields, which is what makes a button inside a scrolling list behave
    /// on touch. Press it and it holds; drag and the list scrolls; release without moving and it
    /// gets a tap.
    fn drags(mut self, axes: Axes) -> Self {
        self.placement().manner.gestures.drags = Some(axes);
        self
    }

    /// The element takes the wheel.
    ///
    /// A notch over it is reported to it as [`wheeled`](crate::Pollen::wheeled) and scrolls
    /// nothing, as a drag it [`drags`](Place::drags) on scrolls nothing. What a notch means is the
    /// app's: a map zooms on it, a dial turns.
    ///
    /// Undeclared, a notch scrolls the region under it. Only an element that is
    /// [`interactive`](Place::interactive) takes it, as only one of those takes a drag.
    fn wheels(mut self) -> Self {
        self.placement().manner.gestures.wheels = true;
        self
    }

    /// Hits are tested against the ellipse inscribed in the element's box rather than the box.
    ///
    /// For a round control, so it does not take presses in the square corners it does not draw.
    fn round_hit_area(mut self) -> Self {
        self.placement().manner.gestures.shape = Shape::Round;
        self
    }

    /// The element scrolls, on the axes named.
    ///
    /// Dividing an element's box with a [`grid`](Place::grid) says nothing about scrolling: an
    /// element scrolls because it said so, and for no other reason. An axis that was not named does
    /// not scroll and has no extent -- it is not a scrolling axis with a range of zero.
    ///
    /// A drag anywhere inside it scrolls it, whether or not what the drag landed on is a target,
    /// and a region that reaches its end hands the drag outward to the next one containing it --
    /// unless it said it [`contain`](Scroll::contain)s that axis, which is the one knob.
    ///
    /// ```no_run
    /// # use foliage::{Axes, Place, Scroll, Stem};
    /// Stem::new().scrolls(Axes::Vertical);
    /// Stem::new().scrolls(Scroll::new(Axes::Both).contain(Axes::Vertical));
    /// ```
    fn scrolls(mut self, scroll: impl Into<Scroll>) -> Self {
        self.placement().manner.scrolls = Some(Scrolls(scroll.into()));
        self
    }

    /// The element does not travel with its region's content.
    ///
    /// A header that stays at the top while content slides under it, a button pinned to a corner.
    /// *Moving with the content* and *counting toward extent* are the same question, so this is one
    /// declaration rather than two that could drift out of agreement: a pinned element receives no
    /// offset from its nearest scrolling ancestor, and contributes nothing to that region's extent.
    ///
    /// It keeps its place in the tree, and with it the clipping, the opacity product and the
    /// disable cascade that parenting it outside the view and anchoring back would have cost.
    fn pinned(mut self) -> Self {
        self.placement().manner.pinned = true;
        self
    }

    /// The element sits over the region it is grown in rather than in it.
    ///
    /// A menu that opens past the edge of the list it belongs to, a tooltip beside a row. What an
    /// [`anchor`](Place::anchored) buys is a position taken from one element while living under
    /// another; without this, an element positioned *outside* its trunk is still cut off at the
    /// trunk's edge, which is the whole point of having put it out there.
    ///
    /// Two consequences, from one declaration because they are one question: it is **not clipped**
    /// by that region, and it **contributes nothing** to that region's extent. An overlay is not
    /// content, so it neither hides behind the edge nor invents room to scroll to.
    ///
    /// It still travels with the region's content, which is what keeps a menu against the row that
    /// opened it. That is the whole difference from [`pinned`](Place::pinned): this one escapes the
    /// clip and keeps the movement, and that one escapes the movement and keeps the clip.
    ///
    /// How far the clip escape reaches is [`Escape`]'s, and is stated rather than defaulted because
    /// no one answer is right everywhere. The extent half takes no such argument and needs none: a
    /// region contributes its own box and never its content to whatever contains it, so there is no
    /// second region for an overlay to be excluded from.
    ///
    /// ```no_run
    /// # use foliage::{Escape, Leaf, Panel, Place};
    /// # fn f(sheet: Leaf) {
    /// Panel::new().floats(Escape::Region);        // out of the list it is in
    /// Panel::new().floats(Escape::Surface);       // out of everything
    /// Panel::new().floats(Escape::Within(sheet)); // out of everything up to the sheet
    /// # }
    /// ```
    fn floats(mut self, escape: Escape) -> Self {
        self.placement().manner.floats = Some(escape);
        self
    }

    /// Where the element sits in focus order, relative to the elements around it.
    ///
    /// Focus order is reading order, derived. This pulls one element earlier (negative) or later
    /// (positive) where a layout's meaning differs from its geometry; everything sharing a value
    /// keeps reading order among themselves, so stating one moves one element and renumbers
    /// nothing.
    fn focus_order(mut self, order: i32) -> Self {
        self.placement().manner.focusing.order = order;
        self
    }

    /// Focus cycles inside this element while it is in there.
    ///
    /// What a drawer or a dialog declares. Without it, stepping through an overlay walks off into
    /// the page behind it.
    fn focus_scope(mut self) -> Self {
        self.placement().manner.focusing.scope = true;
        self
    }

    /// Whether the element is drawn at all.
    ///
    /// The real hide: skipped by drawing, out of the box stack, and contributing nothing to a
    /// containing region's extent, while keeping its state and its [`Leaf`]. App intent only --
    /// content scrolled out of sight is not hidden, and still counts.
    fn visible(mut self, visible: bool) -> Self {
        self.placement().manner.visible = Visible(visible);
        self
    }

    /// How opaque the element is, in `0.0..=1.0`, multiplied through everything grown under it.
    ///
    /// Each element under it is faded on its own, so where two of them overlap, the one behind
    /// shows through the one in front. Declare [`together`](Place::together) on the element to fade
    /// it and everything under it as one picture instead.
    ///
    /// Fully transparent draws nothing and takes nothing: the element and everything under it
    /// are out of the box stack, wherever the fade was declared, so a fade to nothing is enough to
    /// be rid of something. A press target that shows nothing -- the ground of a row that only
    /// fades in while it is pointed at -- is a [`Stem`](crate::Stem), or a panel in a clear color,
    /// at full opacity, with what fades in grown under it.
    fn opacity(mut self, opacity: f32) -> Self {
        self.placement().manner.opacity = Opacity::new(opacity);
        self
    }

    /// Draws the element and everything grown under it as one picture whenever they are drawn at
    /// less than full opacity, so a fade fades the whole of it at once.
    ///
    /// Without it, opacity is applied to each element on its own. A card fading out shows its
    /// ground through its text, and a button's face through its label. With it, the card is drawn
    /// first at full strength, and the picture is what fades.
    ///
    /// While it fades, it is drawn at its own place in the stack, and nothing grown elsewhere sits
    /// between its parts. That is how a CSS stacking context behaves. At full opacity nothing is
    /// different, and it costs nothing. While it fades it costs one picture the size of the window,
    /// so keep this to what fades as a whole: a card, a sheet, a page coming in. A `together`
    /// inside another one that is fading is drawn as part of the outer picture.
    fn together(mut self) -> Self {
        self.placement().manner.together = true;
        self
    }

    /// Turns the element where it is drawn, clockwise by `radians` about its middle -- and
    /// everything grown under it with it.
    ///
    /// Draw-time, as a [`shift`](crate::Grow::shift) is: where the layout put it does not change,
    /// so nothing placed against it moves and nothing is laid out again. What is drawn and what is
    /// pressed turn; [`Drawn`](crate::Vein::Drawn) still reads the upright box. Written later with
    /// [`turn`](crate::Grow::turn), and moved with [`Motion::Turn`](crate::Motion::Turn).
    fn turn(mut self, radians: f32) -> Self {
        self.placement().pose.turn = radians;
        self
    }

    /// Draws the element `scale` times its size about its middle, and everything grown under it
    /// with it: a card pressed down to `0.97`, a mark that swells while it is lit.
    ///
    /// Draw-time, as [`turn`](Place::turn) is. Written later with [`scale`](crate::Grow::scale), and
    /// moved with [`Motion::Scale`](crate::Motion::Scale).
    fn scale(mut self, scale: f32) -> Self {
        self.placement().pose.scale = scale;
        self
    }

    /// Draws the element `by` its own size from where the layout put it, and everything grown under
    /// it with it: `(1.0, 0.0)` is one of its own widths to the right.
    ///
    /// A [`shift`](crate::Grow::shift) measured in the element rather than in pixels, so a sheet
    /// slides out by exactly its width without asking how wide it came out. The two add together.
    /// Written later with [`slide`](crate::Grow::slide), and moved with
    /// [`Motion::Slide`](crate::Motion::Slide).
    fn slide(mut self, by: crate::Position) -> Self {
        self.placement().pose.slide = by;
        self
    }
}

impl<T: Places> Place for T {}

/// A seed placed by a box.
///
/// Everything the engine draws as a rectangle -- a [`Stem`](crate::Stem), a
/// [`Panel`](crate::Panel), a [`Text`](crate::Text), an [`Icon`](crate::Icon), an
/// [`Image`](crate::Image), a [`Polygon`](crate::Polygon) -- and nothing else.
///
/// It is a separate trait from [`Place`] rather than a method on it because a
/// [`Line`](crate::Line) has no box to state: it is two ends, said with
/// [`between`](crate::Line::between) or, per breakpoint, with [`trace`](crate::Line::trace). An
/// `at` it could be handed and would ignore is the kind of surface that has to be remembered rather
/// than read, so it is not there to hand it.
///
/// Sealed: it can be called, never implemented.
#[allow(private_bounds)]
pub trait Boxed: Places + Sized {
    /// Where the element sits.
    ///
    /// An element that says nothing fills its parent.
    fn at(mut self, location: Location) -> Self {
        self.placement().location = Some(location);
        self
    }
}
