//! Every word the site puts on screen, in one place.
//!
//! Split out so prose can be edited without hunting through four builders for the string, and
//! without having to re-derive whether the box it lands in will hold it. Nothing here decides
//! layout; it is text and the budget each piece of text has.
//!
//! # Reading the budgets
//!
//! Every item says one of three things:
//!
//! - **free** -- the element wraps and everything below it moves down. Write what you like.
//!   These are the column's `lead`, `prose` and card-free paragraphs.
//! - **one line, N chars** -- a fixed box. Past N the text is scissored, not wrapped.
//! - **N chars over M lines** -- a fixed box that wraps. Past M lines it runs out of the
//!   bottom, and the surrounding layout does not move to make room.
//!
//! Budgets are characters, because the whole site is set in one monospaced family: a glyph
//! advances 0.6 of the font size, always. So a box `w` pixels wide holds `w / (0.6 * size)`
//! characters, and the sizes are `type_scale`'s -- LABEL 13 (7.8px a character), BODY 14 (8.4),
//! TITLE 16 (9.6), HEADLINE 22 (13.2), DISPLAY 32 (19.2).
//!
//! Every budget below is quoted at its *worst* breakpoint, which is not always the narrowest
//! window. The content column is `viewport - 32` up to `md`, and `min(viewport - 228, 720)`
//! from `md` on -- the rail arrives at 600px and takes 188 of the width the window just gained,
//! so a 600px window has a **narrower** column than a 599px one. Where that step is the binding
//! case it is called out. The floor assumed throughout is a 320px viewport.

use crate::icons::IconHandles;
use crate::site::cards::CardSpec;
use crate::site::figure::{Label, Node, PlateSpec, Side};

// ---- hero ------------------------------------------------------------------------------------

pub(crate) mod hero {
    use std::ops::Range;

    /// The project's namesake. **Fixed** -- not really copy at all.
    ///
    /// Three things are pinned to it: `EXTENSION_AT` below indexes the `.rs` that takes the
    /// accent, and `WORDMARK_XS`/`WORDMARK_MD`/`WORDMARK_SHORT` in `hero.rs` are each sized so
    /// ten characters at 0.6em fit the viewport that step covers. A different number of
    /// characters means re-deriving all three.
    pub(crate) const WORDMARK: &str = "foliage.rs";
    /// Where `.rs` starts, so the extension carries the accent and the name stays plain.
    pub(crate) const EXTENSION_AT: usize = 7;

    /// Under the wordmark. **~35 chars a line, 2 lines.**
    ///
    /// Free-ish: it wraps, and the button row is anchored to its bottom so extra lines push the
    /// row down rather than landing on it. The cap is landscape, where the tagline gets 52% of
    /// the width -- about 295px on a 568px-wide phone, so 35 characters a line. Two lines is
    /// comfortable there; three starts crowding the fold hint.
    pub(crate) const TAGLINE: &str = "cross-platform UI in Rust";

    /// The fold control. **One line, ~12 chars.** The hit band is derived from this label's own
    /// box, so it grows with the word rather than being a separate number to keep in step.
    pub(crate) const HINT: &str = "learn more";

    /// The live readings above the wordmark. **Fixed format -- 39 characters, exactly.**
    ///
    /// Every field is padded to a fixed width because the ranges below index into the rendered
    /// string to colour it. Changing the layout of this line means changing all six ranges with
    /// it, and the two clamps are what keep the indices true at the edges: five digits of width
    /// or three of milliseconds would shift everything after them.
    pub(crate) fn hud_line(width: f32, height: f32, ms: f32) -> String {
        format!(
            "{:>4}x{:<4}  {:>4.1}ms  short xs sm md lg xl",
            (width.round() as i32).min(9999),
            (height.round() as i32).min(9999),
            ms.min(99.9),
        )
    }

    /// Character spans in the line built by [`hud_line`].
    pub(crate) const AT_W: Range<usize> = 0..4;
    pub(crate) const AT_H: Range<usize> = 5..9;
    pub(crate) const AT_MS: Range<usize> = 11..15;
    pub(crate) const AT_SHORT: Range<usize> = 19..24;
    pub(crate) const AT_BREAKPOINT: [Range<usize>; 5] = [25..27, 28..30, 31..33, 34..36, 37..39];
}

// ---- rail ------------------------------------------------------------------------------------

pub(crate) mod rail {
    /// The section list, in order. **One line, 13 chars.**
    ///
    /// The rail is 148px wide less its insets, which is about thirteen characters at TITLE.
    /// `interaction` wrapped and `primitives` was one letter off it.
    ///
    /// These are labels only -- routing is by index, and the index is the position in this
    /// array. Renaming one is free; reordering renames every route with it. Each also appears
    /// as its page's own `display` title, which is a separate string further down.
    pub(crate) const SECTIONS: [&str; 8] = [
        "overview",
        "leaf",
        "layout",
        "renderers",
        "motion",
        "input",
        "text",
        "assets",
    ];

    /// The brand mark, which is also the way back to the hero. **Fixed** -- `BRAND_EXTENSION_AT`
    /// indexes the `.rs`, same as the hero's wordmark, and it is meant to read as that mark
    /// smaller rather than as a second piece of wording.
    pub(crate) const BRAND: &str = "foliage.rs";
    pub(crate) const BRAND_EXTENSION_AT: usize = 7;
}

// ---- page furniture ---------------------------------------------------------------------------

/// Titles and headings, which every page draws through the same two calls.
///
/// Both wrap rather than clip, so nothing breaks if they run long -- but a wrapped heading reads
/// as a mistake, so treat these as one-line budgets:
///
/// - **`display` (page title): 14 chars.** Set in caps at DISPLAY over a 288px column.
/// - **`heading` (section): 20 chars.** Set in caps at HEADLINE over the same column.
pub(crate) mod headings {
    pub(crate) const OVERVIEW: &str = "overview";
    pub(crate) const OVERVIEW_LIBRARY: &str = "structure";
    pub(crate) const OVERVIEW_WHERE: &str = "where to go";

    pub(crate) const LEAF: &str = "leaf";
    pub(crate) const LEAF_RESOLVING: &str = "resolving";
    pub(crate) const LEAF_CLIPPING: &str = "clipping";
    pub(crate) const LEAF_INHERITING: &str = "inheriting";
    pub(crate) const LEAF_LIFETIME: &str = "lifetime";

    pub(crate) const LAYOUT: &str = "layout";
    pub(crate) const LAYOUT_GRID: &str = "grid";
    pub(crate) const LAYOUT_ANCHOR: &str = "anchor";
    pub(crate) const LAYOUT_MEASURE: &str = "measure";

    pub(crate) const MOTION: &str = "motion";
    pub(crate) const MOTION_EASE: &str = "easing";
    pub(crate) const MOTION_TIMING: &str = "timing";
    pub(crate) const MOTION_TWEENS: &str = "what tweens";
    pub(crate) const MOTION_REPEAT: &str = "repeat";
    pub(crate) const MOTION_SEQUENCE: &str = "sequence";

    pub(crate) const RENDERERS: &str = "renderers";
    pub(crate) const RENDERERS_ROUNDING: &str = "rounding";
    pub(crate) const RENDERERS_SIDES: &str = "sides";
    pub(crate) const RENDERERS_GLYPH: &str = "glyph";
    pub(crate) const RENDERERS_DRAW: &str = "draw";
    pub(crate) const RENDERERS_ICON: &str = "icon scale";
    pub(crate) const RENDERERS_IMAGE: &str = "image fit";

    pub(crate) const INPUT: &str = "input";
    pub(crate) const INPUT_HIT: &str = "hit shape";
    pub(crate) const INPUT_GRAB: &str = "who answers";
    pub(crate) const INPUT_GESTURE: &str = "the gesture";
    pub(crate) const INPUT_SCROLL: &str = "scroll";
    pub(crate) const INPUT_FIELD: &str = "text input";

    pub(crate) const TEXT: &str = "text";
    pub(crate) const TEXT_SIZE: &str = "size per step";
    pub(crate) const TEXT_LETTERS: &str = "the character grid";
    pub(crate) const TEXT_CONTENT: &str = "content size";
    pub(crate) const TEXT_COLOR: &str = "per-glyph color";
    pub(crate) const TEXT_FONT: &str = "registered fonts";

    pub(crate) const ASSETS: &str = "assets";
    pub(crate) const ASSETS_KEY: &str = "key before bytes";
    pub(crate) const ASSETS_ARTWORK: &str = "swapping artwork";
    pub(crate) const ASSETS_WHERE: &str = "bundled or fetched";
    pub(crate) const ASSETS_FONTS: &str = "fonts are not assets";
}

// ---- overview ---------------------------------------------------------------------------------

pub(crate) mod overview {
    /// The opener, with the accent rule down its left edge. **Free.**
    pub(crate) const LEAD: &str = "Everything on screen is a leaf. Any leaf can branch from another becoming its dependent; \
         thus forming a tree. Applications then become a collection of trees allowing expressive \
         coordination of visual and non-visual elements. This mental model serves as the basis \
         for the name foliage.";

    /// Under `the library`. **Free.**
    pub(crate) const LIBRARY: &str = "foliage is a UI framework for Rust. It renders through wgpu and runs the same source on \
         desktop, on the web and on Android. State lives in an ECS world and commanded through actions\
         sent to the core engine. Messages can be read back to respond to core internal events. All \
         user logic lives in a per-frame closure handed to the photosynthesis process (event loop), \
         which configures and runs the renderers based on state that tick to bring it all your code \
         to life.";

    /// Under `where to go`, above the destination buttons. **Free.**
    pub(crate) const WHERE: &str = "Check out the docs for thorough code reference. If your looking for more explanation, the book\
        is a guide and in depth walkthrough of the library and its core mechanisms. If you prefer just\
        delving through code; the entire library is available on github.";

    /// The three destination buttons. **One line, ~10 chars** -- the label box is 90px at TITLE,
    /// and three of them share the row.
    pub(crate) const DOCS: &str = "docs";
    pub(crate) const BOOK: &str = "book";
    pub(crate) const GITHUB: &str = "github";
}

/// The overview's seven cards, in the order they appear.
///
/// - **`title`: 45 chars over 2 lines**, caps at TITLE. One-line titles leave the second line
///   empty, which is what keeps every card's body starting on the same baseline as its
///   neighbour's -- so a two-line title is fine and a three-line one runs into the body.
/// - **`body`: 105 chars over 4 lines**, at BODY.
///
/// Both are quoted at a 320px viewport one-up, which is the binding case: the card is a *fixed*
/// 208px tall, so text that does not fit is not wrapped onto a taller card, it runs out of the
/// bottom of this one. The two-up step at `lg` is only marginally roomier (29 chars a line
/// against 28), so there is no breakpoint where a longer body would be safe.
///
/// `icon`, `sides` and `route` are not copy. `route` is the router's own index -- 0 is the hero,
/// so the sections start at 1 and these have to match `rail::SECTIONS` by position.
pub(crate) const CAPABILITIES: [CardSpec; 7] = [
    CardSpec {
        title: "a leaf is everything",
        body: "Every element shares core logic to facilitate easy composition of visual effects",
        icon: crate::icons::IconHandles::Box,
        sides: 6.0,
        route: 2,
    },
    CardSpec {
        title: "layout that resolves",
        body: "Locations are resolved against a responsive grid adapting to any screen size.",
        icon: crate::icons::IconHandles::Grid,
        sides: 4.0,
        route: 3,
    },
    CardSpec {
        title: "visuals that compose",
        body: "Six renderers power all aesthetic elements. Configurable and composable.",
        icon: crate::icons::IconHandles::Layers,
        sides: 5.0,
        route: 4,
    },
    CardSpec {
        title: "motion that belongs",
        body: "Animations are sequenced, ease-able, and tied to the lifetime of what they animate.",
        icon: crate::icons::IconHandles::Repeat,
        sides: 7.0,
        route: 5,
    },
    CardSpec {
        title: "interaction is key",
        body: "input is read for scroll-wheels, mouse-clicks, touchpads, and more...",
        icon: IconHandles::Play,
        sides: 3.0,
        route: 6,
    },
    CardSpec {
        title: "communication is essential",
        body: "font glyphs with responsive sizing help any app relay important information to users.",
        icon: crate::icons::IconHandles::BookOpen,
        sides: 8.0,
        route: 7,
    },
    CardSpec {
        title: "assets with ease",
        body: "images and icons can be shipped alongside your code to allow more sophisticated media.",
        icon: IconHandles::Box,
        sides: 5.0,
        route: 8,
    },
];

/// The overview's plate -- the figure under the lead.
///
/// - **`label.text`: one line, 16 chars.** The box is a fixed 128px at LABEL, centred on its
///   node. Over-wide is scissored on both sides, so it fails symmetrically and quietly.
/// - **`caption`: one line, 30 chars.** Its own strip below the field, 22px tall.
/// - **`scale` numbers: 3 chars.** Right-aligned into a 24px gutter.
///
/// The annotations are pseudo-metrics on purpose. They are not instrumentation -- they are the
/// *look* of instrumentation, which is what makes a still figure feel mid-calculation. Numbers
/// that claim to be real measurements would have to be, and this page has nothing to measure.
///
/// `at`, `sides` and `size` are geometry, not copy: the nodes are placed where they compose and
/// where their labels have room. A labelled node is a turning point with its label thrown into
/// the open side of the turn, so moving a label to the other `Side` generally means moving the
/// node too.
pub(crate) const PLATE: PlateSpec = PlateSpec {
    nodes: &[
        Node {
            at: (0.14, 0.62),
            sides: 6.0,
            size: 24,
            label: None,
        },
        Node {
            at: (0.28, 0.26),
            sides: 8.0,
            size: 38,
            label: Some(Label {
                text: "resolve 0.4ms",
                side: Side::Above,
            }),
        },
        Node {
            at: (0.45, 0.66),
            sides: 5.0,
            size: 26,
            label: None,
        },
        Node {
            at: (0.59, 0.40),
            sides: 7.0,
            size: 20,
            label: None,
        },
        Node {
            at: (0.75, 0.72),
            sides: 6.0,
            size: 34,
            label: Some(Label {
                text: "reflow xs -> md",
                side: Side::Under,
            }),
        },
        Node {
            at: (0.91, 0.34),
            sides: 5.0,
            size: 24,
            label: None,
        },
    ],
    scale: &[
        (0.06, "32"),
        (0.20, ""),
        (0.34, "24"),
        (0.48, ""),
        (0.62, "16"),
        (0.76, ""),
        (0.90, "8"),
    ],
    caption: "fig. 01 | shape resolve",
};

// ---- leaf -------------------------------------------------------------------------------------

pub(crate) mod leaf {
    /// The page's opener. **Free.**
    ///
    /// The overview's own lead already says everything is a leaf and that the stem is what
    /// things resolve against, so this one has nowhere to go if it starts there too.
    pub(crate) const LEAD: &str = "Every element is a leaf. This provides core logic to each element \
        that enables visibility, clipping, responsive-locations and more. \
        The `Leaf` type is a handle to this element to allow commands to target it. Below is the core\
        logic illustrated as interactive demos.";

    /// The paragraph above each board. **Free**, all four.
    pub(crate) const RESOLVING: &str = "A child states its box in percentages of its parent, so one declaration is a different \
         number of pixels in a different parent. The declaration is written once and never \
         rewritten; the readout is what the tree makes of it this frame.";
    pub(crate) const CLIPPING: &str = "A parent is a boundary as well as an origin. This child is declared in pixels, so it \
         cannot shrink along with its parent -- narrow the parent and the child is cut at the \
         edge, while the box it asked for stays exactly what it was.";
    pub(crate) const INHERITING: &str = "Some of what you write to a parent reaches everything beneath it, and some of it stops \
         there. Which is which is not guessable, so the board writes both to the same parent and \
         you read the child.";
    pub(crate) const LIFETIME: &str = "The branch links lifetimes of elements. When an element that is placed as a dependent of\
        another, pruning the root cleans all branched-leafs.";
}

// ---- layout -----------------------------------------------------------------------------------

pub(crate) mod layout {
    /// The page's opener. **Free.**
    pub(crate) const LEAD: &str = "Every box is a `Location`: a percentage, a pixel count, or a grid line, on each axis. \
         `Grid` is the coordinate system those numbers are read against, and `Anchor` is an \
         escape hatch from it -- resolving against another element instead of the parent. Below \
         are three ways the same kind of declaration ends up a different box.";

    /// The paragraph above each board. **Free**, all three.
    pub(crate) const GRID: &str = "A grid's columns and rows are lines, not percentages -- the second column is the second \
         line in from the left, whichever width the grid divided into. A box spanning several \
         columns spans the gaps between them too, so it is not simply the columns it crosses \
         added together.";
    pub(crate) const ANCHOR: &str = "A box can resolve against any named element instead of its parent. The target and the \
         box carrying the anchor need not be related at all -- not siblings, not parent and \
         child, just two entities where one reads the other's geometry.";
    pub(crate) const MEASURE: &str = "A percentage can still be given a ceiling. Below it the box tracks its parent like any \
         other percentage; past it, the parent keeps growing and the box stops -- centered in \
         whatever room that leaves, by default.";
}

// ---- motion -----------------------------------------------------------------------------------

/// The `motion` page's prose. `motion` rather than `anim` -- the module is named for the file
/// it feeds, and that one is `anim.rs` only because `site::motion` is already the site's own
/// timing tokens.
pub(crate) mod motion {
    /// The page's opener. **Free.**
    pub(crate) const LEAD: &str = "An animation is one value on one element, moved from wherever it currently is to a \
         target, over a window of time. Four numbers decide the whole of it: when it starts, \
         when it finishes, the curve between them, and how many times it runs. Everything is \
         counted against a sequence, which is what reports the group as done.";

    /// The paragraph above each board. **Free**, all five.
    pub(crate) const EASE: &str = "The curve is what the motion feels like, and it is independent of how long the motion \
         lasts. The same travel over the same window reads as arriving, as leaving, or as a \
         machine moving a part, depending only on which one is picked.";
    pub(crate) const TIMING: &str = "Both numbers are measured from the sequence's own beginning, so a start is a delay \
         rather than a duration -- a tween from 600 to 1400 waits 600ms and then runs for 800. \
         That is what lets a whole page be staggered by handing each element one number, and \
         it is also the wait you are watching before each of these moves at all.";
    pub(crate) const TWEENS: &str = "What can be tweened is a closed set: opacity, color, elevation, the box itself, a \
         polygon's shape, an outline. Anything outside it is tweened as plain numbers instead \
         -- the engine runs the clock and reports each frame's values for you to apply.";
    pub(crate) const REPEAT: &str = "A repeat replays the same pass. Left alone it snaps back to the start value each time, \
         which is invisible on a rotation and a jolt on anything else; backtracking runs each \
         replay the way it came instead. A loop lives inside the animation rather than in a \
         chain of callbacks, so it cannot outlive what it animates.";
    pub(crate) const SEQUENCE: &str = "Joining a sequence groups animations; it does not order them. Each keeps its own \
         timing and they may overlap freely -- what the sequence adds is a single report once \
         the last of them lands, which is the hook the next stage of motion hangs off.";
}

// ---- renderers --------------------------------------------------------------------------------

pub(crate) mod renderers {
    /// The page's opener. **Free.**
    pub(crate) const LEAD: &str = "Six renderers draw everything on screen: Panel, Polygon, Text, the line quad behind \
         Line and Polyline, Icon, and Image. One board each, pressing the one property that \
         says what the pipeline is for. What surrounds two of them -- Text's sizing, and the \
         assets Icon and Image are fed from -- gets a section of its own further on.";

    /// The paragraph above each board. **Free**, all six.
    pub(crate) const ROUNDING: &str = "A panel's corners are a bracket, not a raw radius -- a step resolves off the panel's \
         own shorter side, so the same step reads the same on a small chip and a large card.";
    pub(crate) const SIDES: &str = "Side count and rounding are what every entrance on this site already animates through \
         -- a shape resolving from a rough triangle into itself is exactly these two numbers, \
         tweened. Here they're a press instead of a timer.";
    pub(crate) const GLYPH: &str = "A glyph's own box is fixed: on this monospace face every character advances the same \
         distance regardless of its shape, which is what lets the whole site budget copy in \
         characters instead of measuring each string. What changes letter to letter is only \
         which slot in the font's own atlas gets sampled -- looked up by codepoint, not by \
         width.";
    pub(crate) const DRAW: &str = "A polyline's path can be revealed by length rather than snapped on whole. The fraction \
         is of the arc, not the point count, so it reads smoothly regardless of how many \
         vertices make up the shape.";
    pub(crate) const ICON: &str = "An icon is a distance field rather than a picture. One field is registered at startup \
         -- 48px here, for every icon on the site -- and every size on the page is drawn from \
         that same one, so an icon has no resolution to outgrow: making it larger is a change \
         to its box and nothing else.";
    pub(crate) const IMAGE: &str = "An image arrives with its own pixels and its own ratio, and the box it lands in rarely \
         agrees with either. The fit settles it: keep the ratio and leave room, keep the ratio \
         and clip what overhangs, or take the box exactly and wear the distortion. It is chosen \
         as the element is grown, so the three below are three elements and a press picks which \
         one is showing.";
}

// ---- input ------------------------------------------------------------------------------------

pub(crate) mod input {
    /// The page's opener. **Free.**
    pub(crate) const LEAD: &str = "Input is reported, not handled. A press is hit-tested against every element on screen \
         at once -- a flat scan ranked by draw order, not a walk up the tree -- and whichever \
         one wins is named in an emission your own code answers. Three of the boards below \
         report what the engine said about your own pointer as you use them.";

    /// The paragraph above each board. **Free**, all five.
    pub(crate) const HIT: &str = "A click has to land inside the element's own shape and inside every clip its ancestors \
         impose. Shape is not always the box: rounding a panel the whole way switches its hit \
         test to a circle, so a dot stops collecting the clicks that land in the square corners \
         it never drew.";
    pub(crate) const GRAB: &str = "Every element competes for a press by default, drawn or not -- a label lying over a \
         button wins on draw order and swallows it. Passing through is how something purely \
         visual withdraws: it is still told the gesture crossed it, but it can no longer take \
         it, so several elements can report one press and they arrive in hit-test order.";
    pub(crate) const GESTURE: &str = "A press comes apart into three reports: it went down, it moved, and it came back up. \
         Movement is reported from the first pixel, so on a mouse very nearly every press is \
         also a drag. The threshold of ten pixels decides something else -- whether the release \
         is also a click. A click is an outcome rather than an event, and a gesture that \
         wandered produces none.";
    pub(crate) const SCROLL: &str = "Any element laid out on a grid can scroll, and what it cannot absorb is handed outward \
         to the next one that can. That is what lets a small panel inside a page take the wheel \
         until it reaches its own end and then let the page have the rest, instead of trapping \
         the gesture where it landed.";
    pub(crate) const FIELD: &str = "The one assembled control: a field with a caret, selection, a hint while it is empty, \
         and the keyboard handling behind all three. It reports what it now holds after every \
         edit, and reports Enter separately -- on a single-line field that key is not a \
         newline, it is a submission for you to answer.";

    /// Written across the gesture board's pad. **One line, ~30 chars** -- it is centred across
    /// the stage, which is the width of the content column less the field's own padding.
    pub(crate) const PAD: &str = "press, drag, release";
}

// ---- text -------------------------------------------------------------------------------------

pub(crate) mod text {
    /// The page's opener. **Free.**
    pub(crate) const LEAD: &str = "The renderers page took the glyph -- one character, sampled out of an atlas. This is \
         everything around it: a size that changes with the window, boxes measured in \
         characters rather than pixels, a box taking its size from its own string, colour per \
         character, and the one thing about fonts that is decided before the app runs.";

    /// The paragraph above each board. **Free**, all five.
    pub(crate) const SIZE: &str = "A size is declared per breakpoint and resolved against the window, falling back down \
         the steps to whatever was stated last. A short viewport overrides all of them: a phone \
         on its side is a wide window with almost no height, and large type is the first thing \
         to run out of room there. The frame below stands in for the window, since the window \
         is the one thing on this page you cannot be handed a button for -- the declaration it \
         is resolving, and the size that comes out, are the real ones.";
    pub(crate) const LETTERS: &str = "A box can be stated in characters instead of pixels, and it resolves against the \
         element's own size -- so writing a new size is a change of geometry, not only of \
         drawing. This whole site's copy is budgeted that way: one glyph advances 0.6 of the \
         size, always, so a column's width is a character count.";
    pub(crate) const CONTENT: &str = "The other direction: the box takes its size from the string rather than the string \
         being fitted into the box. Sizing to content by width also turns wrapping off -- with \
         no fixed right edge there is nothing to wrap against, so the run stays on one line and \
         the box grows to hold it.";
    pub(crate) const COLOR: &str = "A run has one colour, and any character in it may be given another by its offset. The \
         wordmark on the hero and the mark at the top of the rail are the same string, coloured \
         this way -- the extension takes the accent and the name stays plain, without splitting \
         either into two elements that would then have to be kept adjacent.";
    pub(crate) const FONT: &str = "A second face is registered at startup and named per element. This page is set in two: \
         the upright face everything structural uses, and the italic cut of the same family the \
         prose is in -- same weight, same rhythm, so only the slant changes.";
}

// ---- assets -----------------------------------------------------------------------------------

pub(crate) mod assets {
    /// The page's opener. **Free.**
    pub(crate) const LEAD: &str = "Everything drawn from bytes -- images, and the fields icons are baked into -- comes \
         through one loader. Asking for an asset hands back a key immediately, before any bytes \
         exist, and the key is what everything else is written against. Loading is decoupled \
         from drawing, so nothing in your layout has to wait for a fetch or branch on whether \
         one is still in flight.";

    /// The paragraph above each board, and the two sections with no board of their own.
    /// **Free**, all four.
    pub(crate) const KEY: &str = "The key is minted before the load is even started, so an element can be grown with it \
         in the same frame you asked. What arrives later is the bytes, reported once, keyed by \
         the handle you already hold -- there is no second handle to reconcile and nothing to \
         poll. The two states below are one frame apart for this picture, which is compiled \
         into the page; the board holds them apart so the first one can be looked at, because \
         over a network it is the state you would spend the most time in.";
    pub(crate) const ARTWORK: &str = "An icon is registered under an id at startup and referenced by that id afterwards. \
         Which artwork an element draws is a write like any other, so one element can be moved \
         between fields without being torn down and regrown -- the renderers page runs the \
         other axis of this, one field drawn at three sizes.";
    pub(crate) const WHERE: &str = "Bytes in hand and bytes to fetch are two paths, not two halves of one. Which path a \
         build takes is settled at compile time: the native binary embeds the file, the web \
         build asks for it at a URL, and the fetching variant does not exist at all in a native \
         build that opted out of it -- so getting that wrong is a compile error at the call \
         site rather than a failure at load. They differ only at the first step, and the board \
         below draws one path at a time so that the two you cannot have at once are still two.";
    pub(crate) const FONTS: &str = "Fonts do not go through any of this. A face is registered before the app starts and \
         cannot be fetched, and it must be monospaced -- a proportional file is refused as it \
         is read rather than drawn badly. Bundle it with the binary; the text page has what \
         happens once you have.";
}

/// The words on a board: its step buttons, its readout, and the labels drawn into its stage.
///
/// Everything here is tight. A board is the width of the content column, and at a 320px viewport
/// that is 288px with the field's own padding taken out of it.
pub(crate) mod board {
    /// The step buttons under each board, left to right. **One line, and the budget is a
    /// function of how many there are**, because the row divides itself into equal slots:
    ///
    /// | steps | slot  | chars |
    /// |-------|-------|-------|
    /// | 2     | 128px | 15    |
    /// | 3     | 85px  | 9     |
    /// | 4     | 64px  | 7     |
    ///
    /// Four is the cap the row is designed around. Past that the words stop fitting before the
    /// shapes do.
    ///
    /// Each array is also the board's state machine: the button at index `i` puts the demo in
    /// state `i`, so these are *states*, not actions -- "empty" is where the board already is,
    /// not something you do to it. Adding or removing an entry means changing the match in
    /// `leaf.rs` that answers it.
    pub(crate) const RESOLVING_STEPS: [&str; 3] = ["100%", "66%", "42%"];
    pub(crate) const CLIPPING_STEPS: [&str; 3] = ["full", "75%", "45%"];
    pub(crate) const INHERITING_STEPS: [&str; 3] = ["reset", "opacity", "color"];
    pub(crate) const LIFETIME_STEPS: [&str; 4] = ["empty", "place", "grow", "prune"];

    /// The grid board's steps. Doubles as the cell's own on-shape label -- what a step presses
    /// is exactly what the highlighted cell says it is.
    pub(crate) const GRID_STEPS: [&str; 3] = ["col 1", "col 2", "col 1-3"];
    pub(crate) const ANCHOR_STEPS: [&str; 3] = ["below", "right", "corner"];
    pub(crate) const MEASURE_STEPS: [&str; 3] = ["40%", "70%", "100%"];

    pub(crate) const ROUNDING_STEPS: [&str; 4] = ["none", "sm", "lg", "full"];
    pub(crate) const SIDES_STEPS: [&str; 3] = ["3", "6", "12"];
    pub(crate) const GLYPH_STEPS: [&str; 3] = ["a", "g", "@"];
    pub(crate) const DRAW_STEPS: [&str; 3] = ["25%", "60%", "100%"];
    pub(crate) const ICON_STEPS: [&str; 3] = ["24", "48", "128"];
    pub(crate) const IMAGE_STEPS: [&str; 3] = ["aspect", "crop", "stretch"];

    /// The motion page's steps. Every one of these replays a tween rather than setting a
    /// state, so the board is back where it started the moment it finishes -- which is what
    /// lets the same button be pressed twice and mean it twice.
    ///
    /// `emph` is abbreviated where the others are not because four steps share the row at 7
    /// characters; the readout spells `Ease::EMPHASIS` out in full.
    pub(crate) const EASE_STEPS: [&str; 4] = ["linear", "decel", "accel", "emph"];
    pub(crate) const TIMING_STEPS: [&str; 3] = ["quick", "slow", "delayed"];
    pub(crate) const TWEENS_STEPS: [&str; 4] = ["opacity", "color", "move", "shape"];
    pub(crate) const REPEAT_STEPS: [&str; 4] = ["once", "twice", "bounce", "forever"];
    pub(crate) const SEQUENCE_STEPS: [&str; 2] = ["stagger", "together"];

    /// The two row names in each board's readout, top then bottom. **One line, 7 chars** -- the
    /// label column is a fixed 56px at LABEL. Rendered in caps.
    pub(crate) const RESOLVING_ROWS: [&str; 2] = ["parent", "child"];
    pub(crate) const CLIPPING_ROWS: [&str; 2] = ["parent", "child"];
    pub(crate) const INHERITING_ROWS: [&str; 2] = ["wrote", "child"];
    pub(crate) const LIFETIME_ROWS: [&str; 2] = ["called", "child"];
    pub(crate) const GRID_ROWS: [&str; 2] = ["stage", "cell"];
    pub(crate) const ANCHOR_ROWS: [&str; 2] = ["call", "sits"];
    pub(crate) const MEASURE_ROWS: [&str; 2] = ["frame", "cell"];
    pub(crate) const ROUNDING_ROWS: [&str; 2] = ["round", "corners"];
    pub(crate) const SIDES_ROWS: [&str; 2] = ["sides", "shape"];
    pub(crate) const GLYPH_ROWS: [&str; 2] = ["advance", "code"];
    pub(crate) const DRAW_ROWS: [&str; 2] = ["drawn", "call"];
    pub(crate) const ICON_ROWS: [&str; 2] = ["drawn", "field"];
    pub(crate) const IMAGE_ROWS: [&str; 2] = ["view", "result"];
    pub(crate) const EASE_ROWS: [&str; 2] = ["curve", "reads"];
    pub(crate) const TIMING_ROWS: [&str; 2] = ["start", "finish"];
    pub(crate) const TWEENS_ROWS: [&str; 2] = ["motion", "tween"];
    pub(crate) const REPEAT_ROWS: [&str; 2] = ["repeat", "passes"];
    pub(crate) const SEQUENCE_ROWS: [&str; 2] = ["starts", "group"];

    /// What a readout row says before anything has been measured.
    pub(crate) const EMPTY_VALUE: &str = "--";

    /// The inheriting board's readout, one pair per step: what was written, then what the child
    /// did about it. **One line, 20 chars** each -- the value column is whatever the strip has
    /// left after the label, which is 168px at a 320px viewport.
    pub(crate) const INHERITING_VALUES: [[&str; 2]; 3] = [
        ["nothing yet", "--"],
        ["opacity 0.6", "faded as well"],
        ["a new color", "kept its own"],
    ];

    /// The anchor board's readout, one pair per step: the call, in shorthand, then what it puts
    /// the dependent box against. **One line, 20 chars** each, like [`INHERITING_VALUES`].
    pub(crate) const ANCHOR_VALUES: [[&str; 2]; 3] = [
        [".bottom().as_top()", "below target"],
        [".right().as_left()", "right of target"],
        [".right()+.bottom()", "off target's corner"],
    ];

    /// The rounding board's readout, one pair per step. `corners` never changes -- `Side` is
    /// left at its default throughout -- so the pairing states that rather than leaving the
    /// row blank.
    pub(crate) const ROUNDING_VALUES: [[&str; 2]; 4] = [
        ["none", "all 4"],
        ["sm", "all 4"],
        ["lg", "all 4"],
        ["full", "all 4"],
    ];

    /// The sides board's readout, one pair per step: the declared count, then the shape it
    /// names.
    pub(crate) const SIDES_VALUES: [[&str; 2]; 3] =
        [["3", "triangle"], ["6", "hexagon"], ["12", "12-gon"]];

    /// The glyph board's readout, one pair per step: the advance every letter shares on this
    /// face, then the codepoint that picks which one is sampled from the atlas.
    pub(crate) const GLYPH_VALUES: [[&str; 2]; 3] =
        [["48px", "u+0061"], ["48px", "u+0067"], ["48px", "u+0040"]];

    /// The draw board's readout, one pair per step: the reveal fraction, then the call that
    /// set it.
    pub(crate) const DRAW_VALUES: [[&str; 2]; 3] = [
        ["25%", "draw_progress(0.25)"],
        ["60%", "draw_progress(0.6)"],
        ["100%", "draw_progress(1.0)"],
    ];

    /// The icon board's readout, one pair per step: the box the icon is drawn into, then the
    /// field it is drawn from. The field never changes, and that is the board's whole point --
    /// three sizes, one registered artwork, no second copy at any of them.
    pub(crate) const ICON_VALUES: [[&str; 2]; 3] = [
        ["24 x 24", "48px msdf"],
        ["48 x 48", "48px msdf"],
        ["128 x 128", "48px msdf"],
    ];

    /// The image board's readout, one pair per step: the view, then what it does to a near-2:1
    /// photograph in a 4:3 box.
    pub(crate) const IMAGE_VALUES: [[&str; 2]; 3] = [
        ["Aspect", "fits, leaves room"],
        ["Crop", "fills, clips sides"],
        ["Stretch", "fills, distorts"],
    ];

    /// The ease board's readout, one pair per step: the constant, then what that curve is for.
    /// **One line, 20 chars** each -- `Ease::DECELERATE` is the longest at 16.
    pub(crate) const EASE_VALUES: [[&str; 2]; 4] = [
        ["Ease::Linear", "constant rate"],
        ["Ease::DECELERATE", "arriving"],
        ["Ease::ACCELERATE", "leaving"],
        ["Ease::EMPHASIS", "a change worth seeing"],
    ];

    /// The timing board's readout, one pair per step: the two numbers the tween is declared
    /// with. Both are offsets into the sequence, which is why the last step's finish is
    /// larger than its duration.
    pub(crate) const TIMING_VALUES: [[&str; 2]; 3] =
        [["200ms", "420ms"], ["200ms", "1100ms"], ["600ms", "1400ms"]];

    /// The tweens board's readout, one pair per step: the variant, then the move it makes.
    ///
    /// Written both ways round because that is what the step does -- a press sends its value to
    /// the end it is not at, so the same button is the trip and the return.
    pub(crate) const TWEENS_VALUES: [[&str; 2]; 4] = [
        ["Motion::Opacity", "1.0 <-> 0.2"],
        ["Motion::Color", "accent <-> rose"],
        ["Motion::Location", "left <-> right"],
        ["Motion::Polygon", "6 sides <-> 3"],
    ];

    /// The repeat board's readout, one pair per step: what was asked for, then how many passes
    /// that is and what happens between them.
    ///
    /// `Times(1)` is two passes, not one -- the count is replays *after* the first, which is
    /// the one thing about this call worth putting on an instrument.
    pub(crate) const REPEAT_VALUES: [[&str; 2]; 4] = [
        ["Repeat::Once", "1"],
        ["Repeat::Times(1)", "2, snapping back"],
        ["Times(1), backtrack", "2, out and back"],
        ["Repeat::Forever", "until superseded"],
    ];

    /// The sequence board's `starts` row, one per step: the three tweens' own delays inside
    /// the one sequence they are joined to.
    pub(crate) const SEQUENCE_STARTS: [&str; 2] = ["200 / 460 / 720ms", "200 / 200 / 200ms"];

    /// The sequence board's `group` row, which is the tree's answer rather than the step's:
    /// what the sequence is doing right now. `FINISHED` is written when the emission naming
    /// this board's own sequence arrives, so it is the engine saying so and not a timer here
    /// guessing at the same number twice.
    pub(crate) const SEQUENCE_IDLE: &str = "not started";
    pub(crate) const SEQUENCE_RUNNING: &str = "running";
    pub(crate) const SEQUENCE_FINISHED: &str = "finished";

    /// The lifetime board's `called` row, one per step. **One line, 20 chars.**
    ///
    /// These are the calls the board is standing in for, so they should stay recognisably like
    /// the API rather than becoming prose about it.
    pub(crate) const LIFETIME_CALLS: [&str; 4] = [
        "nothing yet",
        "canopy.leaf(..)",
        "branch(parent)",
        "prune(parent)",
    ];

    /// The lifetime board's `child` row. Read back off the tree each frame rather than chosen by
    /// the step -- except `WITHERED`, which the step is the authority on, because for the frames
    /// between the prune command and its landing the handle still reports itself as planted.
    /// **One line, 20 chars.**
    pub(crate) const CHILD_NONE: &str = "none";
    pub(crate) const CHILD_GROWING: &str = "growing";
    pub(crate) const CHILD_LIVE: &str = "live";
    pub(crate) const CHILD_WITHERED: &str = "withered";

    /// The word drawn inside the child shape on every board. **One line**, centred across the
    /// child's own band, which is about half the parent -- so roughly 15 chars, less on the
    /// clipping board once the parent has been narrowed.
    pub(crate) const CHILD: &str = "child";

    /// The anchor board's two shapes. **One line** -- `TARGET` sits in a 64px shape, `DEPENDENT`
    /// in a 40px one, so it stays to what that smaller shape can hold.
    pub(crate) const TARGET: &str = "target";
    pub(crate) const DEPENDENT: &str = "dep";

    /// The parent's own label, inside the frame at its top-left. **One line, 11 chars** -- the
    /// box is stated as 11 letters, which is exactly `parent 100%`.
    pub(crate) fn frame(width: f32) -> String {
        format!("parent {}%", width as i32)
    }

    /// The clipping board's parent label, which says full or not rather than a percentage.
    /// **One line, 8 chars.**
    ///
    /// Tighter than [`frame`]'s 11 because this label lives *inside* a frame that the last step
    /// narrows to about 134px at the narrowest breakpoint -- so the words have to fit the
    /// narrowed parent, not just their own box. The readout says "parent" on the row above
    /// anyway.
    pub(crate) const CLIP_FULL: &str = "full";
    pub(crate) const CLIP_NARROWED: &str = "narrowed";

    /// The lifetime board's empty-slot markers, each sitting exactly where the thing it stands
    /// for will appear. **One line** -- `NO_PARENT` shares [`frame`]'s 11-character box,
    /// `NO_CHILD` is centred across the child's band.
    ///
    /// A word in the slot is the board saying nothing is there yet, and saying it in the place
    /// you are about to watch fill. Both go off once the child exists and stay off through the
    /// prune, so they can never contradict a readout that says "withered".
    pub(crate) const NO_PARENT: &str = "no parent";
    pub(crate) const NO_CHILD: &str = "no child";

    /// The heading on every reference card. **One line, 30 chars.** Caps, as written.
    pub(crate) const REFERENCE: &str = "REFERENCE";

    // ---- input ---------------------------------------------------------------------------

    /// The input page's steps. The gesture board has none -- its stage is the control, and a
    /// press, a drag and a release are not states you can pick from a row.
    pub(crate) const HIT_STEPS: [&str; 2] = ["none", "full"];
    pub(crate) const GRAB_STEPS: [&str; 2] = ["grab", "pass-through"];
    pub(crate) const SCROLL_STEPS: [&str; 3] = ["top", "middle", "end"];
    pub(crate) const FIELD_STEPS: [&str; 2] = ["single", "multi"];

    pub(crate) const HIT_ROWS: [&str; 2] = ["shape", "hit"];
    pub(crate) const GRAB_ROWS: [&str; 2] = ["child", "heard"];
    pub(crate) const GESTURE_ROWS: [&str; 2] = ["phase", "travel"];
    pub(crate) const SCROLL_ROWS: [&str; 2] = ["offset", "at"];
    pub(crate) const FIELD_ROWS: [&str; 2] = ["value", "report"];

    /// The hit board's `shape` row, one per step: what rounding the panel the whole way does to
    /// its hit test. Not a name this page invented -- it is the variant the engine writes.
    pub(crate) const HIT_SHAPES: [&str; 2] = ["Rectangle", "Circle"];
    /// Its `hit` row, which is live: what the last press actually landed on. The pad is a plain
    /// target filling the stage behind the shape, so a press the shape declines is caught
    /// rather than vanishing -- which is what makes a declined corner visible at all.
    pub(crate) const HIT_NONE: &str = "nothing yet";
    pub(crate) const HIT_SHAPE: &str = "the shape";
    pub(crate) const HIT_PAD: &str = "the pad behind it";

    /// The grab board's `child` row, one per step.
    pub(crate) const GRAB_MODES: [&str; 2] = ["grabs the press", "passes it through"];
    /// Its `heard` row: who reported the last press. Both names appear when the child passes
    /// through, in the order the emissions arrived.
    pub(crate) const GRAB_NONE: &str = "nobody yet";
    pub(crate) const GRAB_CHILD: &str = "child";
    pub(crate) const GRAB_PARENT: &str = "parent";

    /// The gesture board's `phase` row: the emission's own name, so what the board says and
    /// what the engine sent are the same word.
    pub(crate) const GESTURE_IDLE: &str = "waiting";
    pub(crate) const GESTURE_ENGAGED: &str = "Engaged";
    pub(crate) const GESTURE_DRAGGED: &str = "Dragged";
    pub(crate) const GESTURE_DISENGAGED: &str = "Disengaged";
    pub(crate) const GESTURE_CLICKED: &str = "Clicked";

    /// The gesture board's `travel` row: how far the pointer has moved from where it went down,
    /// and whether that is far enough to have cost the gesture its click. **One line, 20
    /// chars** -- four digits plus the note is 16.
    ///
    /// The note is about the *click*, not about `Dragged`. `Dragged` arrives on every move
    /// while an element holds the gesture, from the first pixel; the ten-pixel threshold is
    /// only what decides whether a `Clicked` follows the release, and `DragStarted` -- which
    /// is what turns this note on -- is the one emission that announces it.
    pub(crate) fn travel(px: f32, past: bool) -> String {
        format!(
            "{}px{}",
            (px.round() as i32).min(9999),
            if past { ", no click" } else { "" }
        )
    }

    /// The scroll board's two rows, both read back off the view every frame.
    pub(crate) fn scrolled(px: f32) -> String {
        format!("{}px down", px.round() as i32)
    }
    pub(crate) fn progress(fraction: f32) -> String {
        format!("{}% of the range", (fraction * 100.0).round() as i32)
    }

    /// The field board's `value` row: what has been typed, cut to the strip.
    ///
    /// The one value on the site written by the reader rather than budgeted here, so it is the
    /// one that has to be cut at the point of display. A marker rather than a hard stop,
    /// because a string that simply ends at the edge reads as the readout being broken; and
    /// newlines are folded to spaces, since the multi-line field can hold them and this row is
    /// one line tall.
    const VALUE_WIDEST: usize = 17;
    pub(crate) fn typed(value: &str) -> String {
        if value.is_empty() {
            return FIELD_EMPTY.to_string();
        }
        let flat = value.replace('\n', " ");
        let mut shown: String = flat.chars().take(VALUE_WIDEST).collect();
        if flat.chars().count() > VALUE_WIDEST {
            shown.push_str("..");
        }
        shown
    }

    /// What the `value` row says before anything has been typed, and the `report` row: the last
    /// thing the field itself said. Enter is called out because on a single-line field it is
    /// not a newline -- it is the submission.
    pub(crate) const FIELD_EMPTY: &str = "empty";
    pub(crate) const FIELD_HINT: &str = "type here";
    pub(crate) const FIELD_FOCUSED: &str = "Focused";
    pub(crate) const FIELD_UNFOCUSED: &str = "Unfocused";
    pub(crate) const FIELD_CHANGED: &str = "TextChanged";
    pub(crate) const FIELD_SUBMITTED: &str = "Enter -- submitted";
    pub(crate) const FIELD_NONE: &str = "nothing yet";

    // ---- text ----------------------------------------------------------------------------

    /// The size board's steps: the breakpoint the frame on its stage is standing in for.
    ///
    /// The board cannot resize the window and does not pretend to. It resizes a frame *inside*
    /// the stage instead, and states which breakpoint that width would be -- so the fallback is
    /// acted out rather than merely described, and it works on a phone, where resizing anything
    /// is not available at all.
    pub(crate) const SIZE_STEPS: [&str; 3] = ["xs", "md", "lg"];
    pub(crate) const LETTERS_STEPS: [&str; 3] = ["13", "20", "32"];
    pub(crate) const CONTENT_STEPS: [&str; 3] = ["short", "longer", "longest"];
    pub(crate) const COLOR_STEPS: [&str; 3] = ["plain", "extension", "ramp"];
    pub(crate) const FONT_STEPS: [&str; 2] = ["default", "italic"];

    pub(crate) const SIZE_ROWS: [&str; 2] = ["window", "size"];
    pub(crate) const LETTERS_ROWS: [&str; 2] = ["advance", "box"];
    pub(crate) const CONTENT_ROWS: [&str; 2] = ["chars", "box"];
    pub(crate) const COLOR_ROWS: [&str; 2] = ["run", "glyphs"];
    pub(crate) const FONT_ROWS: [&str; 2] = ["FontId", "face"];

    /// The one declaration all three of the size board's steps read, written into the stage so
    /// the thing being resolved is on screen beside its result. **One line, 26 chars** -- the
    /// stage is 256px at the narrowest breakpoint and this is set at LABEL.
    pub(crate) const SIZE_DECLARATION: &str = "new(14).md(20).lg(28)";
    /// What the frame is captioned with, per step. **One line, 11 chars**, like every other
    /// frame label on the site. The numbers are the engine's real thresholds.
    pub(crate) const SIZE_FRAMES: [&str; 3] = ["xs to 419", "md 600+", "lg 840+"];
    /// The `window` row, per step: the span of viewport widths the frame stands in for.
    pub(crate) const SIZE_WINDOWS: [&str; 3] = ["under 420 wide", "600 to 839", "840 and over"];
    /// The specimen inside the frame, set at whatever [`SIZE_DECLARATION`] resolves to for the
    /// step. Short enough to fit the narrowest frame at the largest of those sizes.
    pub(crate) const SIZE_SPECIMEN: &str = "specimen";
    pub(crate) fn points(size: u32) -> String {
        format!("{size}px")
    }

    /// The letters board's specimen: a box stated as this many characters wide, so what the
    /// box does when the size changes is the whole demonstration.
    pub(crate) const LETTERS_SPECIMEN: &str = "12 letters..";
    pub(crate) const LETTERS_WIDE: i32 = 12;
    pub(crate) fn advance(size: u32) -> String {
        format!("{:.1}px a glyph", size as f32 * 0.6)
    }

    /// A box's resolved width, for the two boards whose subject is what a box came out as.
    pub(crate) fn box_width(px: f32) -> String {
        format!("{}px wide", px.round() as i32)
    }

    /// The content board's strings, one per step. Each is the specimen *and* the step's own
    /// measurement -- the box is sized from the string, so the string is the input.
    ///
    /// Short on purpose, and shorter than they were. The box grows rightward from a fixed left
    /// edge, so a string that outruns the stage takes its own outline off the edge with it --
    /// at which point the board is demonstrating clipping rather than content sizing. The
    /// longest is 14 characters, which at HEADLINE is 185px inside a stage that is 256px at the
    /// narrowest breakpoint.
    pub(crate) const CONTENT_STRINGS: [&str; 3] = ["fits", "fits more", "fits much more"];
    pub(crate) fn characters(count: usize) -> String {
        format!("{count} characters")
    }

    /// The colour board's specimen. The site's own wordmark, so the board is colouring the
    /// exact string the hero and the rail already colour this way.
    pub(crate) const COLOR_SPECIMEN: &str = "foliage.rs";
    /// Its `run` row, one per step: what was asked for, in the terms the call takes.
    pub(crate) const COLOR_RUNS: [&str; 3] = ["one color", "add(7..10, accent)", "add(i..i+1, ..)"];
    /// Its `glyphs` row, one per step: how many characters ended up with a colour of their own.
    pub(crate) const COLOR_OVERRIDES: [&str; 3] =
        ["none overridden", "3 overridden", "all 10 overridden"];

    /// The font board's specimen, set in whichever face the step names.
    pub(crate) const FONT_SPECIMEN: &str = "the same sentence";
    /// Its two rows, per step: the handle, and what that handle was registered as.
    pub(crate) const FONT_IDS: [&str; 2] = ["FontId::default()", "the registered id"];
    pub(crate) const FONT_FACES: [&str; 2] = ["JetBrains Mono", "the italic cut"];

    // ---- assets --------------------------------------------------------------------------

    /// The key board's steps: the two states of a load, held apart so they can be looked at.
    ///
    /// For an asset compiled into the binary these are one frame apart -- which is the fact the
    /// section is teaching, and also the reason the board has to hold them apart itself. The key
    /// is real and the picture is real; the only thing stood in for is the wait, which over a
    /// network is as long as the network is slow.
    pub(crate) const KEY_STEPS: [&str; 2] = ["asked", "arrived"];
    pub(crate) const ARTWORK_STEPS: [&str; 4] = ["box", "code", "grid", "play"];
    /// The sources board's steps: which build you are looking at. Two paths, not two halves of
    /// one -- the step picks a path and the stage draws that one.
    pub(crate) const SOURCES_STEPS: [&str; 2] = ["native", "web"];

    pub(crate) const KEY_ROWS: [&str; 2] = ["key", "bytes"];
    pub(crate) const ARTWORK_ROWS: [&str; 2] = ["icon", "element"];
    pub(crate) const SOURCES_ROWS: [&str; 2] = ["source", "then"];

    /// The key board's `key` row -- the first digits of a handle the loader really did mint for
    /// this board, truncated because a key is 128 bits and the strip holds twenty characters.
    /// That it exists in both states is the point; what it says is not.
    pub(crate) fn key_digits(key: u128) -> String {
        format!("{:x}..", key >> 96)
    }
    /// Its `bytes` row, per step, and the word in the frame while there is no picture in it.
    pub(crate) const KEY_NO_BYTES: &str = "none yet";
    pub(crate) const KEY_SLOT: &str = "no bytes yet";
    pub(crate) fn key_bytes(len: usize) -> String {
        format!("{}kb, drawing", len / 1024)
    }

    /// The sources board's stage: the three shapes a path runs through, left to right. The
    /// first changes with the step and the other two do not -- which is the whole claim.
    ///
    /// **One line, 9 chars.** A shape is 30% of a stage that is 256px at the narrowest
    /// breakpoint -- 77px, so nine characters at LABEL and no more. The call itself is far
    /// longer than that and belongs on the readout, where there is room for it; these are what
    /// the bytes *are* at that point in the path.
    pub(crate) const SOURCES_ORIGINS: [&str; 2] = ["embedded", "fetched"];
    pub(crate) const SOURCES_KEY: &str = "one key";
    pub(crate) const SOURCES_ELEMENT: &str = "Image";
    /// Its two rows, per step: the variant the call takes, and what happens after it.
    pub(crate) const SOURCES_CALLS: [&str; 2] = ["AssetSource::Bytes", "AssetSource::Url"];
    pub(crate) const SOURCES_THEN: [&str; 2] = ["in hand, same frame", "fetched, then drawn"];

    /// The artwork board's `icon` row, one per step -- the generated handle, which is what an
    /// app writes rather than the number behind it.
    pub(crate) const ARTWORK_IDS: [&str; 4] = [
        "IconHandles::Box",
        "IconHandles::Code",
        "IconHandles::Grid",
        "IconHandles::Play",
    ];
    /// Its `element` row, which never changes across the row and is the point of the board.
    pub(crate) const ARTWORK_ELEMENT: &str = "grown once";
}

/// The reference table under each board: the call, and what it does.
///
/// - **`call`: one line, 30 chars.** Scissored past that. The binding case is a 320px viewport.
/// - **`gloss`: 80 chars over 2 lines.**
///
/// The gloss budget is quoted at `md` -- a 600px window -- which is the tightest step, not the
/// narrowest window. The rail arrives there and takes 188px of the width the window just gained,
/// leaving a 340px measure where a 599px window had 567px. `xs` gets a third line to compensate
/// and is roomier in total (111 chars at 360px); `lg` is roomier still. Write to `md`.
///
/// The card's height is computed from the line counts, so a gloss that needs a third line at
/// `md` does not get a taller card -- it runs under the entry below it.
pub(crate) mod reference {
    use crate::site::blueprint::Entry;

    pub(crate) const RESOLVING: [Entry; 4] = [
        Entry {
            call: "Location::new().xs(h, v)",
            gloss: "The box, per breakpoint. A percentage is of the parent; px is absolute.",
        },
        Entry {
            call: "22.pct().as_left().with(..)",
            gloss: "One edge, then the opposite one. Width and right are interchangeable.",
        },
        Entry {
            call: "canopy.section(leaf)",
            gloss: "What that declaration works out to right now, in real pixels.",
        },
        Entry {
            call: "anchor()",
            gloss: "Resolve against a named element rather than the parent, for stacking.",
        },
    ];

    pub(crate) const CLIPPING: [Entry; 3] = [
        Entry {
            call: "clip = every ancestor's box",
            gloss: "An element draws only where its own box and all of its parents' overlap.",
        },
        Entry {
            call: "canopy.section(leaf)",
            gloss: "Reports the box it asked for, whether or not all of it is drawn.",
        },
        Entry {
            call: "ClipToViewport",
            gloss: "Opts out, and is bounded by the window instead of by the parent chain.",
        },
    ];

    pub(crate) const INHERITING: [Entry; 4] = [
        Entry {
            call: "canopy.opacity(leaf, to)",
            gloss: "Inherited. Everything beneath is drawn through the parent's value.",
        },
        Entry {
            call: "canopy.color(leaf, to)",
            gloss: "Not inherited. It is one element's own component and stops there.",
        },
        Entry {
            call: "canopy.visible(leaf, yes)",
            gloss: "Inherited, and the subtree keeps its state while it is hidden.",
        },
        Entry {
            call: "canopy.disable(leaf)",
            gloss: "Inherited. The subtree still draws, and stops taking input.",
        },
    ];

    pub(crate) const LIFETIME: [Entry; 4] = [
        Entry {
            call: "canopy.prune(leaf)",
            gloss: "Removes it and everything beneath it, in the one call.",
        },
        Entry {
            call: "canopy.presence(leaf)",
            gloss: "Planted while it is still being grown, then Live, then Withered.",
        },
        Entry {
            call: "Bloom::Withered(leaf)",
            gloss: "Reported once per element that went, after the frame applies it.",
        },
        Entry {
            call: "a withered handle",
            gloss: "Takes writes and drops them. Nothing panics, and it is never reused.",
        },
    ];

    pub(crate) const GRID: [Entry; 3] = [
        Entry {
            call: "N.col() / N.row()",
            gloss: "A 1-based line in the parent's grid, not a percentage of it.",
        },
        Entry {
            call: "Grid::new(3.col().gap(8), ..)",
            gloss: "Divides the parent into 3 columns, 8px of gap between them.",
        },
        Entry {
            call: "1.col().with(3.col())",
            gloss: "A span crosses the gaps too -- wider than 3 lone columns summed.",
        },
    ];

    pub(crate) const ANCHOR: [Entry; 3] = [
        Entry {
            call: "leaf.anchored(target)",
            gloss: "Names the entity this box's anchor() values resolve against.",
        },
        Entry {
            call: "anchor().bottom().as_top()",
            gloss: "Reads the target's edge, then places this box's own edge there.",
        },
        Entry {
            call: "independent of parenting",
            gloss: "An anchor target can sit anywhere in the tree; it decides position only.",
        },
    ];

    pub(crate) const MEASURE: [Entry; 3] = [
        Entry {
            call: "ConfigurationDescriptor::max",
            gloss: "A ceiling on this axis's resolved extent, in logical pixels.",
        },
        Entry {
            call: "ConfigurationDescriptor::min",
            gloss: "A floor on the same axis -- the symmetric constraint.",
        },
        Entry {
            call: "Justify::Near / Far / Center",
            gloss: "Where the box sits in the room a max leaves over. Centered by default.",
        },
    ];

    pub(crate) const EASE: [Entry; 3] = [
        Entry {
            call: "Timing::over(ms).eased(e)",
            gloss: "The window a tween runs in, and the curve it runs on. Separate choices.",
        },
        Entry {
            call: "Ease::Linear",
            gloss: "A constant rate. Every other curve here is a cubic bezier through two \
                    points.",
        },
        Entry {
            call: "Ease::Bezier(ControlPoints)",
            gloss: "Your own curve. Both control points are clamped, so overshoot is not \
                    expressible.",
        },
    ];

    pub(crate) const TIMING: [Entry; 3] = [
        Entry {
            call: "Timing::over(finish)",
            gloss: "Milliseconds from the sequence's beginning at which the tween ends.",
        },
        Entry {
            call: ".after(start)",
            gloss: "A delay, not a duration -- both numbers are read from the same origin.",
        },
        Entry {
            call: "canopy.animate(leaf, to, t)",
            gloss: "Starts it. The start value is whatever the element holds when it begins.",
        },
    ];

    pub(crate) const TWEENS: [Entry; 3] = [
        Entry {
            call: "Motion::{Opacity, Color, ..}",
            gloss: "The closed set: opacity, color, elevation, location, polygon, outline.",
        },
        Entry {
            call: "canopy.tween(channels, t)",
            gloss: "Plain numbers on the same clock, reported back for you to apply yourself.",
        },
        Entry {
            call: "a second tween, same value",
            gloss: "Supersedes the first rather than fighting it over one component.",
        },
    ];

    pub(crate) const REPEAT: [Entry; 3] = [
        Entry {
            call: "Timing::repeat(Repeat::..)",
            gloss: "Once, Times(n) for n replays after the first pass, or Forever.",
        },
        Entry {
            call: "Timing::backtrack()",
            gloss: "Each replay runs back the way it came instead of snapping to the start.",
        },
        Entry {
            call: "canopy.prune(leaf)",
            gloss: "Ends any loop on it -- a runner cannot outlive what it animates.",
        },
    ];

    pub(crate) const SEQUENCE: [Entry; 3] = [
        Entry {
            call: "canopy.sequence()",
            gloss: "Opens one. Every animation is counted against a sequence, named or not.",
        },
        Entry {
            call: "animate_during(.., seq)",
            gloss: "Joins it. Grouping is all it does -- the entries still keep their own \
                    timing.",
        },
        Entry {
            call: "Bloom::SequenceFinished(s)",
            gloss: "Fires once the last of them lands. The hook for chaining the next stage.",
        },
    ];

    pub(crate) const ROUNDING: [Entry; 3] = [
        Entry {
            call: "canopy.rounding(leaf, to)",
            gloss: "A size bracket, not a raw radius -- resolved off the panel's own shorter \
                    side.",
        },
        Entry {
            call: "Side::left() / right() / ..",
            gloss: "Restricts which corners Rounding applies to.",
        },
        Entry {
            call: "Rounding::Full",
            gloss: "A true pill or circle; also switches the hit test to match.",
        },
    ];

    pub(crate) const SIDES: [Entry; 3] = [
        Entry {
            call: "canopy.polygon(leaf, to)",
            gloss: "Rewrites sides, rounding and rotation together -- no respawn needed.",
        },
        Entry {
            call: ".sides(n)",
            gloss: "Fractional values are valid -- they drive a shader blend, not a vertex \
                    count.",
        },
        Entry {
            call: ".rounding(0.0..=1.0)",
            gloss: "0 is sharp, 1 is a true circle regardless of side count.",
        },
    ];

    pub(crate) const GLYPH: [Entry; 3] = [
        Entry {
            call: "canopy.text(leaf, s)",
            gloss: "Replaces a Text element's whole string, even a lone character.",
        },
        Entry {
            call: "advance = 0.6 * size",
            gloss: "Fixed per glyph on this face -- copy is budgeted in characters because of \
                    it.",
        },
        Entry {
            call: "FontId / register_fonts",
            gloss: "A second face can be registered and selected per Text element.",
        },
    ];

    pub(crate) const DRAW: [Entry; 3] = [
        Entry {
            call: "canopy.draw_progress(leaf, t)",
            gloss: "Reveals the path by arc length, 0.0 to 1.0.",
        },
        Entry {
            call: "Polyline::new().points(v)",
            gloss: "A Line segment plus a rounded Polygon joint per vertex, chained.",
        },
        Entry {
            call: "canopy.points(leaf, v)",
            gloss: "Rewrites the whole vertex chain in one call.",
        },
    ];

    pub(crate) const ICON: [Entry; 3] = [
        Entry {
            call: "Icon::msdf(id, bytes, ..)",
            gloss: "Registers one field under an id, at startup. Every size draws from it.",
        },
        Entry {
            call: "canopy.location(leaf, to)",
            gloss: "An icon has no size of its own -- resizing one is resizing its box.",
        },
        Entry {
            call: "canopy.icon(leaf, id)",
            gloss: "Swaps which registered artwork is drawn, without regrowing the element.",
        },
    ];

    pub(crate) const IMAGE: [Entry; 3] = [
        Entry {
            call: "foliage.load_asset(source)",
            gloss: "Bytes in hand or a URL to fetch. The key it returns is usable at once.",
        },
        Entry {
            call: "Image::new(key).view(v)",
            gloss: "The fit, fixed as the element is grown -- there is no verb that rewrites \
                    it.",
        },
        Entry {
            call: "ImageView::Crop",
            gloss: "Fills both axes at the image's own ratio and clips whatever overhangs.",
        },
    ];

    pub(crate) const HIT: [Entry; 3] = [
        Entry {
            call: "Rounding::Full",
            gloss: "Switches the hit test to a circle as well as the corners it draws.",
        },
        Entry {
            call: "InteractionShape",
            gloss: "Rectangle or Circle. Written by Rounding rather than chosen separately.",
        },
        Entry {
            call: "the shape AND every clip",
            gloss: "A press must land inside both, so an ancestor's box can cut a target down.",
        },
    ];

    pub(crate) const GRAB: [Entry; 3] = [
        Entry {
            call: ".interactive()",
            gloss: "Competes for the press. The topmost element under the pointer wins it.",
        },
        Entry {
            call: ".pass_through()",
            gloss: "Told the gesture crossed it, never wins it -- for overlays and labels.",
        },
        Entry {
            call: "several per press",
            gloss: "The winner and every pass-through crossed, reported in hit-test order.",
        },
    ];

    pub(crate) const GESTURE: [Entry; 5] = [
        Entry {
            call: "Bloom::Engaged",
            gloss: "The pointer went down on this element.",
        },
        Entry {
            call: "Bloom::Dragged",
            gloss: "Every move of the pointer while this element holds the gesture.",
        },
        Entry {
            call: "Bloom::DragStarted",
            gloss: "Once, when the gesture passes the threshold and stops being a click.",
        },
        Entry {
            call: "Bloom::Disengaged",
            gloss: "The release, however it ended. Always follows an Engaged.",
        },
        Entry {
            call: "Bloom::Clicked",
            gloss: "The release, if the gesture stayed within 10px on both axes throughout.",
        },
    ];

    pub(crate) const SCROLL: [Entry; 3] = [
        Entry {
            call: "a Grid on an element",
            gloss: "Makes it a view: what overflows can be dragged, wheeled and flung.",
        },
        Entry {
            call: "canopy.scroll(leaf, to)",
            gloss: "Puts it at a fraction of its own range. Either axis left alone stays.",
        },
        Entry {
            call: ".overscroll(false)",
            gloss: "Traps the gesture here instead of handing the remainder further out.",
        },
    ];

    pub(crate) const FIELD: [Entry; 4] = [
        Entry {
            call: "TextInput::new()",
            gloss: "Field, caret, selection and the keyboard handling, as one element.",
        },
        Entry {
            call: ".hint_text(..)",
            gloss: "Shown only while the field is empty, in the field's own foreground tone.",
        },
        Entry {
            call: "Bloom::TextChanged",
            gloss: "The whole value after every edit -- typed, pasted, or written to it.",
        },
        Entry {
            call: "TextInputAction::Enter",
            gloss: "A submission on a single-line field, where the key is not a newline.",
        },
    ];

    pub(crate) const SIZE: [Entry; 3] = [
        Entry {
            call: "FontSize::new(20).md(28)",
            gloss: "One size, then overrides from a breakpoint up. Unset steps fall back down.",
        },
        Entry {
            call: ".short(14)",
            gloss: "Wins outright while the viewport is cramped, whatever its width says.",
        },
        Entry {
            call: "canopy.font_size(leaf, to)",
            gloss: "Rewrites it later. The glyphs are laid out again where they already are.",
        },
    ];

    pub(crate) const LETTERS: [Entry; 3] = [
        Entry {
            call: "12.letters().as_width()",
            gloss: "A box stated in characters of this element's own size, not in pixels.",
        },
        Entry {
            call: ".size(FontSize::new(n))",
            gloss: "Gives an element carrying no text the cell its .letters() measure against.",
        },
        Entry {
            call: "advance = 0.6 * size",
            gloss: "Fixed on this face, which is what makes a character count a width.",
        },
    ];

    pub(crate) const CONTENT: [Entry; 3] = [
        Entry {
            call: "text_content().as_width()",
            gloss: "Takes that axis from the laid-out glyphs instead of from the parent.",
        },
        Entry {
            call: "width also ends wrapping",
            gloss: "With no fixed right edge there is nothing left to wrap the run against.",
        },
        Entry {
            call: "anchor() on a neighbour",
            gloss: "How a box with no text of its own follows a run that measures itself.",
        },
    ];

    pub(crate) const COLOR: [Entry; 3] = [
        Entry {
            call: "GlyphColors::new().add(r, c)",
            gloss: "Colors a range of character offsets on top of the run's own color.",
        },
        Entry {
            call: "canopy.glyph_colors(leaf, to)",
            gloss: "Replaces the whole set of overrides in one write.",
        },
        Entry {
            call: "later ranges win",
            gloss: "Where two added ranges overlap, the one added last decides.",
        },
    ];

    pub(crate) const FONT: [Entry; 3] = [
        Entry {
            call: "foliage.font(bytes)",
            gloss: "Registers a face at startup and hands back the id that names it.",
        },
        Entry {
            call: ".font(id)",
            gloss: "Draws this element in that face. Everything else keeps the built-in one.",
        },
        Entry {
            call: "monospace only",
            gloss: "A proportional file is refused as it is read, rather than drawn badly.",
        },
    ];

    pub(crate) const KEY: [Entry; 4] = [
        Entry {
            call: "canopy.load_asset(source)",
            gloss: "Hands back a key at once. The bytes arrive whenever they arrive.",
        },
        Entry {
            call: "Image::new(key)",
            gloss: "Growable the same frame, before anything exists behind the key.",
        },
        Entry {
            call: "Bloom::AssetLoaded",
            gloss: "Reported once the bytes land, naming the key you were already holding.",
        },
        Entry {
            call: "canopy.asset(key)",
            gloss: "The bytes themselves, undecoded, once there are some.",
        },
    ];

    pub(crate) const ARTWORK: [Entry; 3] = [
        Entry {
            call: "foliage.icon(Icon::msdf(..))",
            gloss: "Registers a baked field under an id, before the app runs.",
        },
        Entry {
            call: "canopy.icon(leaf, id)",
            gloss: "Points an element at another registered field, with no regrow.",
        },
        Entry {
            call: "#[icon_handle] enum",
            gloss: "Generated from a directory of svgs: one variant per file, ids and all.",
        },
    ];

    pub(crate) const SOURCES: [Entry; 4] = [
        Entry {
            call: "AssetSource::Bytes(v)",
            gloss: "Already in hand. Resolves in the same frame it was asked for.",
        },
        Entry {
            call: "AssetSource::Url(s)",
            gloss: "Fetched, and never blocking: its own thread natively, fetch on the web.",
        },
        Entry {
            call: "bundled_asset!(f, path)",
            gloss: "States the path once and picks the variant for the platform being built.",
        },
        Entry {
            call: "remote-assets, off",
            gloss: "Drops the Url variant outright -- a compile error, not a load failure.",
        },
    ];
}
