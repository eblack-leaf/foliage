//! Stating what an element is when it is described, before it exists.

use bevy_ecs::component::Component;

use crate::coordinate::Axes;
use crate::elevation::Elevation;
use crate::interaction::{Gestures, Shape};
use crate::leaf::Leaf;
use crate::lifecycle::{Opacity, Visible};
use crate::line::{Stroke, Traced};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
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
/// [`Line::between`](crate::Line::between) -- so nothing can state both and no pass has to decide
/// which was meant.
#[derive(Clone, Debug, Default)]
pub(crate) struct Placement {
    pub(crate) location: Option<Location>,
    /// The two ends of a stroked element, in place of a box.
    pub(crate) traced: Option<Traced>,
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
}

/// Where an element sits in focus order, and whether it holds focus inside itself.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Focusing {
    /// What [`focus_order`](Place::focus_order) declared. Equal values keep reading order.
    pub(crate) order: i32,
    /// Declared [`focus_scope`](Place::focus_scope).
    pub(crate) scope: bool,
}

/// The one other element a placement may read, and where it was named.
#[derive(Component, Copy, Clone, Debug)]
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
    fn interactive(mut self) -> Self {
        self.placement().manner.gestures.receives = true;
        self
    }

    /// A gesture over the element goes to whatever is beneath it.
    ///
    /// What a composite marks its own decoration with. A label drawn over a button, a highlight, a
    /// gradient across a card -- each of them is above its target at those pixels, and each would
    /// otherwise be what a press lands on.
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
    /// Fully transparent is **not there**: it is out of the box stack and receives nothing, which
    /// closes the case of an element faded out that went on taking presses. Anything above zero is
    /// there and takes them normally.
    fn opacity(mut self, opacity: f32) -> Self {
        self.placement().manner.opacity = Opacity::new(opacity);
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
/// [`between`](crate::Line::between). An `at` it could be handed and would ignore is the kind of
/// surface that has to be remembered rather than read, so it is not there to hand it.
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
