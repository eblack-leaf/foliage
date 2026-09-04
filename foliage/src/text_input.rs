//! TextInput -- an editable run, and the first [`Frond`](crate::frond): a leaf that is divided.
//!
//! # What a field is made of
//!
//! Everything else foliage grows is one element. A field is four, because the parts move
//! independently of each other and each is already something the engine draws:
//!
//! | | Part | Is |
//! |---|---|---|
//! | | `run` | the value, as an ordinary [`Text`](crate::Text) |
//! | | `hint` | the placeholder, shown only while the value is empty |
//! | | `caret` | a [`Panel`](crate::Panel) [`CARET`] wide, shown only while the field holds focus |
//! | | `selection` | a [`Panel`](crate::Panel) behind the run, from one end of the selection to the other |
//!
//! The app names the field and nothing else. Every verb and every read is addressed to it, and the
//! parts are grown, placed and hidden from here -- so a field is one [`Leaf`] to hold, and what it
//! is made of is not a surface to keep in step with.
//!
//! # The caret is placed in the ordinary grammar
//!
//! A caret at character `n` is `anchor().left() + anchor().letters(n)` against the run, and a
//! selection is the span between two of those. That is the whole of the geometry: no pass measures
//! a caret, because [`letters`](crate::Source::letters) already resolves a character count against
//! the font the run composes in, and it moves when the run moves because an anchor does.
//!
//! The run is drawn in front of both marks. A caret sits on the boundary between two character
//! cells and is as wide as it needs to be seen, so a caret drawn over the run would take a bite out
//! of the glyph it stands before -- at a small size, a quarter of it.
//!
//! # The field is a scrolling region
//!
//! One line, as wide as its own value, inside a box that clips it. That is a region, so it is
//! declared as one -- which is also what makes the caret stay in view: every edit asks the field to
//! [`show`](crate::ScrollTo::show) the caret, and R4 answers it against the extent the same frame
//! measured.

use core::ops::Range;

use bevy_ecs::component::Component;
use tracing::debug;

use crate::coordinate::{Axes, Position};
use crate::elevation::Elevation;
use crate::elm::{Chlorophyll, PanelPigment, Pigment};
use crate::grove::Grove;
use crate::interaction::Gestures;
use crate::interaction::input::{Key, Keystroke};
use crate::leaf::Leaf;
use crate::lifecycle::Visible;
use crate::frond::{Fronds, Sprouts};
use crate::op::{Bud, Op};
use crate::palette::{Fill, Palette};
use crate::place::{Anchored, Boxed, Caller, Manner, Placement, Places};
use crate::placement::basis::anchor;
use crate::placement::location::Location;
use crate::placement::role::{center_y, left};
use crate::placement::source::{Source, content};
use crate::seed::Buds;
use crate::text::Lettering;
use crate::text::TextPigment;
use crate::view::{Scroll, ScrollTo, Scrolls};

/// How wide the caret is drawn, in logical pixels.
///
/// Stated rather than derived from the character cell: a caret is a mark between two characters and
/// not a character, so it is the same width in a heading as in a caption.
const CARET: f32 = 2.0;

/// An editable run of glyphs.
///
/// One line. What it holds is a value, a placeholder shown while that value is empty, and a caret
/// with a selection behind it -- and the app holds one [`Leaf`] for the whole of it:
///
/// ```no_run
/// # use foliage::{Boxed, FontSize, Location, Palette, Place, Source, TextInput, left, top};
/// TextInput::new()
///     .value("")
///     .placeholder("Search")
///     .color(Palette::Ink)
///     .font_size(FontSize::new().xs(14))
///     .at(Location::new().xs(
///         left(0.px()).right(100.pct()),
///         top(0.px()).height(32.px()),
///     ));
/// ```
///
/// It draws no border and no ground of its own. A field is a run and a caret inside whatever box an
/// app puts it in, and a chrome the engine drew would be one more thing to talk an app out of.
///
/// A tap puts the caret where it landed and takes focus -- the second because focus goes to
/// whatever a tap lands on, which a field is by virtue of receiving at all. An app that wants focus
/// somewhere else writes [`focus`](crate::Grow::focus) from [`clicked`](crate::Pollen::clicked) and
/// wins, because the tap settled focus a frame before the app is handed it.
///
/// What it types is reported as [`edited`](crate::Pollen::edited), and an `Enter` as
/// [`submitted`](crate::Pollen::submitted).
#[derive(Clone, Debug)]
pub struct TextInput {
    pub(crate) placement: Placement,
    pub(crate) value: String,
    pub(crate) placeholder: String,
    pub(crate) fill: Fill,
    pub(crate) hint: Fill,
    pub(crate) caret: Fill,
    pub(crate) selection: Fill,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// An empty field, read in [`Palette::Ink`] with an [`Palette::Accent`] caret.
    pub fn new() -> Self {
        Self {
            placement: Placement::default(),
            value: String::new(),
            placeholder: String::new(),
            fill: Fill::Role(Palette::Ink),
            hint: Fill::Role(Palette::Muted),
            caret: Fill::Role(Palette::Accent),
            selection: Fill::Role(Palette::Muted),
        }
    }

    /// What the field starts out saying.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// What is read in the field's place while it says nothing.
    ///
    /// Drawn in [`hint`](TextInput::hint) rather than in the value's own fill, and never part of
    /// the value: it is absent from [`Vein::Text`](crate::Vein::Text) and a field showing one is
    /// empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// What the value is filled with.
    pub fn color(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();
        self
    }

    /// What the placeholder is filled with.
    pub fn hint(mut self, fill: impl Into<Fill>) -> Self {
        self.hint = fill.into();
        self
    }

    /// What the caret is filled with.
    pub fn caret(mut self, fill: impl Into<Fill>) -> Self {
        self.caret = fill.into();
        self
    }

    /// What is drawn behind a selected span.
    pub fn selection(mut self, fill: impl Into<Fill>) -> Self {
        self.selection = fill.into();
        self
    }
}

impl Places for TextInput {
    fn placement(&mut self) -> &mut Placement {
        &mut self.placement
    }
}

impl Boxed for TextInput {}

impl Buds for TextInput {
    fn bud(mut self, at: Caller) -> Bud {
        // A field is measured in characters throughout -- its caret, its selection and its own
        // value -- so it carries a typeface whether or not one was named, exactly as a run does.
        self.placement.typeface.get_or_insert_default();
        // Declared here rather than left to the app, because each of the three is what makes a
        // field a field rather than a preference about one. It receives, so a gesture can reach it
        // and focus can rest on it -- which is also the whole of what makes a tap focus it, since
        // that is where focus goes for anything that receives; it takes drags across, so a drag
        // over the value selects while a drag down still scrolls whatever the field is sitting in;
        // and it is a region on the same axis, which is what clips a value longer than the box and
        // what the caret is kept in view by.
        self.placement.manner.gestures.receives = true;
        self.placement.manner.gestures.drags = Some(Axes::Horizontal);
        self.placement.manner.scrolls = Some(Scrolls(
            Scroll::new(Axes::Horizontal).contain(Axes::Horizontal),
        ));
        Bud {
            chlorophyll: Chlorophyll::None,
            placement: self.placement,
            sprout: Some(Box::new(Sprout {
                value: self.value,
                placeholder: self.placeholder,
                fill: self.fill,
                hint: self.hint,
                caret: self.caret,
                selection: self.selection,
            })),
            at,
            ..Bud::bare()
        }
    }
}

/// What a field's parts are grown from, carried by the [`Bud`] until the drain grows them.
#[derive(Clone, Debug)]
pub(crate) struct Sprout {
    pub(crate) value: String,
    pub(crate) placeholder: String,
    pub(crate) fill: Fill,
    pub(crate) hint: Fill,
    pub(crate) caret: Fill,
    pub(crate) selection: Fill,
}

/// The four elements a field is made of.
///
/// Carried on the field, which is what makes every verb addressed to the field able to reach the
/// part it actually changes -- and what makes an element a field at all, since nothing else grows
/// one.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Parts {
    pub(crate) run: Leaf,
    pub(crate) hint: Leaf,
    pub(crate) caret: Leaf,
    pub(crate) selection: Leaf,
}

/// Where the caret is and what is selected, in characters of the value.
///
/// Two indices rather than a range, because which end moves is what a shifted arrow key needs to
/// know: `anchor` is where the selection was begun and `caret` is where it has reached, so the two
/// are equal exactly when nothing is selected.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Editing {
    pub(crate) caret: usize,
    pub(crate) anchor: usize,
}

impl Editing {
    /// A caret at `index`, with nothing selected.
    pub(crate) fn at(index: usize) -> Self {
        Self {
            caret: index,
            anchor: index,
        }
    }

    /// The selected span, low end first.
    pub(crate) fn span(self) -> Range<usize> {
        match self.caret <= self.anchor {
            true => self.caret..self.anchor,
            false => self.anchor..self.caret,
        }
    }

    /// Whether anything is selected.
    fn collapsed(self) -> bool {
        self.caret == self.anchor
    }

    /// The same, held inside a value `length` characters long.
    fn clamped(self, length: usize) -> Self {
        Self {
            caret: self.caret.min(length),
            anchor: self.anchor.min(length),
        }
    }
}

/// What one keystroke did.
pub(crate) enum Applied {
    /// The value changed, and this is what it and the caret became.
    Wrote(String, Editing),
    /// The caret moved and the value did not. A selection is a caret with its anchor left behind,
    /// so selecting is this as well.
    Moved(Editing),
    /// `Enter`.
    Submitted,
    /// Nothing a field does.
    Nothing,
}

/// One keystroke against a value and a caret. The whole of what editing means.
///
/// Pure, and the only place the rules live: what a key does to a value is arithmetic over character
/// indices, and separating it from the tree is what lets every case be stated as one.
///
/// Indices are **characters**, not bytes -- the space [`Shaped`](crate::text::shape) lays a run out
/// in and the space a [`tint`](crate::Grow::tint) is written in, so a caret, a highlight and a
/// range all mean the same thing by the same number.
pub(crate) fn applied(value: &str, editing: Editing, stroke: Keystroke) -> Applied {
    let characters: Vec<char> = value.chars().collect();
    let editing = editing.clamped(characters.len());
    let extend = stroke.modifiers.shift;
    let span = editing.span();
    // A command rather than a character. Held, a key says what to do with the value instead of
    // what to put in it, so it is answered before the key's own meaning is.
    if stroke.modifiers.control {
        return match stroke.key {
            Key::Typed('a') | Key::Typed('A') => Applied::Moved(Editing {
                anchor: 0,
                caret: characters.len(),
            }),
            _ => Applied::Nothing,
        };
    }
    match stroke.key {
        Key::Typed(character) => {
            let mut written: String = characters[..span.start].iter().collect();
            written.push(character);
            written.extend(&characters[span.end..]);
            Applied::Wrote(written, Editing::at(span.start + 1))
        }
        Key::Backspace => {
            // A selection is what is removed where there is one; otherwise the character before the
            // caret. The two are one rule -- delete the span -- with an empty span reaching back by
            // one first.
            let removed = match editing.collapsed() {
                true => span.start.saturating_sub(1)..span.end,
                false => span,
            };
            if removed.is_empty() {
                return Applied::Nothing;
            }
            Applied::Wrote(
                without(&characters, removed.clone()),
                Editing::at(removed.start),
            )
        }
        Key::Delete => {
            let removed = match editing.collapsed() {
                true => span.start..(span.end + 1).min(characters.len()),
                false => span,
            };
            if removed.is_empty() {
                return Applied::Nothing;
            }
            Applied::Wrote(
                without(&characters, removed.clone()),
                Editing::at(removed.start),
            )
        }
        Key::Left => Applied::Moved(moved(editing, characters.len(), Toward::Left, extend)),
        Key::Right => Applied::Moved(moved(editing, characters.len(), Toward::Right, extend)),
        Key::Home => Applied::Moved(moved(editing, characters.len(), Toward::Start, extend)),
        Key::End => Applied::Moved(moved(editing, characters.len(), Toward::End, extend)),
        // Not an edit and not a caret: the app is told, and what that means is the app's.
        Key::Enter => Applied::Submitted,
        // Both answered before a field is ever asked, because both move focus wherever focus is:
        // `Tab` steps it and `Escape` takes it away. A field never sees either.
        Key::Tab | Key::Escape => Applied::Nothing,
    }
}

/// The value with a span of it taken out.
fn without(characters: &[char], span: Range<usize>) -> String {
    characters[..span.start]
        .iter()
        .chain(&characters[span.end..])
        .collect()
}

/// Which way a caret was asked to go.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Toward {
    Left,
    Right,
    Start,
    End,
}

/// Where the caret lands, and what that leaves selected.
///
/// An unshifted arrow against a selection **collapses it to the edge it points at** rather than
/// stepping from the caret. That is what every editor does and what a reader means by it: the
/// selection was the thing being moved away from.
fn moved(editing: Editing, length: usize, toward: Toward, extend: bool) -> Editing {
    let span = editing.span();
    let caret = match (toward, extend, editing.collapsed()) {
        (Toward::Left, false, false) => span.start,
        (Toward::Right, false, false) => span.end,
        (Toward::Left, _, _) => editing.caret.saturating_sub(1),
        (Toward::Right, _, _) => (editing.caret + 1).min(length),
        (Toward::Start, _, _) => 0,
        (Toward::End, _, _) => length,
    };
    match extend {
        true => Editing {
            caret,
            anchor: editing.anchor,
        },
        false => Editing::at(caret),
    }
}

impl Sprouts for Sprout {
    fn sprout(self: Box<Self>, grove: &mut Grove, leaf: Leaf) {
        sprout(grove, leaf, *self);
    }
}

/// Grows a field's four parts under it, in the drain that grew the field.
fn sprout(grove: &mut Grove, field: Leaf, sprout: Sprout) {
    let at = grove
        .tree
        .spawned_at(field)
        .unwrap_or(core::panic::Location::caller());
    let typeface = grove.tree.typeface(field);
    // Every name first, so the parts can be placed against each other as they are grown.
    let (run, run_growth) = grove.tree.allocate();
    let (hint, hint_growth) = grove.tree.allocate();
    let (caret, caret_growth) = grove.tree.allocate();
    let (selection, selection_growth) = grove.tree.allocate();
    let parts = Parts {
        run,
        hint,
        caret,
        selection,
    };
    // A run as wide as its own value, one line tall, centred in whatever box the field was given.
    // Its width is what the field's extent is measured from, so a value longer than the box is what
    // makes the field scrollable and nothing has to say so. It sits in front of both marks, because
    // a mark drawn over it would cut into the glyph it stands against.
    let line = |elevation: i32| Placement {
        location: Some(Location::new().xs(
            left(0.px()).width(content()),
            center_y(50.pct()).height(1.letters()),
        )),
        elevation: Some(Elevation::up(elevation)),
        typeface,
        manner: Manner {
            // The hit test reads the top of the stack and stops, so every part of a field has to be
            // out of that stack or the field itself would never be the top of it.
            gestures: Gestures {
                transparent: true,
                ..Gestures::default()
            },
            ..Manner::default()
        },
        ..Placement::default()
    };
    // The caret and the selection are spans of the run's own character cells, so both are anchored
    // to it and both are rewritten by `refresh` whenever the caret moves.
    let against = |elevation: i32, visible: bool| Placement {
        location: Some(span(0, 0)),
        anchor: Some(Anchored { to: run, at }),
        elevation: Some(Elevation::up(elevation)),
        manner: Manner {
            gestures: Gestures {
                transparent: true,
                ..Gestures::default()
            },
            visible: Visible(visible),
            ..Manner::default()
        },
        ..Placement::default()
    };
    grove.tree.grow(
        selection,
        selection_growth,
        Some(field),
        Bud {
            chlorophyll: Chlorophyll::Panel,
            pigment: Some(Pigment::Panel(PanelPigment {
                fill: sprout.selection,
                rounding: Default::default(),
            })),
            placement: against(0, false),
            at,
            ..Bud::bare()
        },
    );
    let empty = sprout.value.is_empty();
    grove.tree.grow(
        run,
        run_growth,
        Some(field),
        Bud {
            chlorophyll: Chlorophyll::Text,
            pigment: Some(Pigment::Text(TextPigment { fill: sprout.fill })),
            lettering: Some(Lettering(sprout.value)),
            placement: line(2),
            at,
            ..Bud::bare()
        },
    );
    let mut hint_placement = line(2);
    hint_placement.manner.visible = Visible(empty);
    grove.tree.grow(
        hint,
        hint_growth,
        Some(field),
        Bud {
            chlorophyll: Chlorophyll::Text,
            pigment: Some(Pigment::Text(TextPigment { fill: sprout.hint })),
            lettering: Some(Lettering(sprout.placeholder)),
            placement: hint_placement,
            at,
            ..Bud::bare()
        },
    );
    grove.tree.grow(
        caret,
        caret_growth,
        Some(field),
        Bud {
            chlorophyll: Chlorophyll::Panel,
            pigment: Some(Pigment::Panel(PanelPigment {
                fill: sprout.caret,
                rounding: Default::default(),
            })),
            // A caret always has something to show, so it declares itself shown and lets focus be
            // the only thing that hides it. The selection says otherwise for itself, below.
            placement: against(1, true),
            at,
            ..Bud::bare()
        },
    );
    grove.tree.set_parts(field, parts);
    grove.tree.set_editing(field, Editing::default());
    refresh(grove, field);
    debug!(leaf = field.id(), "field sprouted");
}

/// Where the caret sits at character `index` of the run.
fn caret_at(index: usize) -> Location {
    Location::new().xs(
        left(anchor().left() + anchor().letters(index as f32)).width(CARET.px()),
        center_y(anchor().center_y()).height(anchor().letters(1.0)),
    )
}

/// The box covering characters `from..to` of the run.
fn span(from: usize, to: usize) -> Location {
    Location::new().xs(
        left(anchor().left() + anchor().letters(from as f32))
            .right(anchor().left() + anchor().letters(to as f32)),
        center_y(anchor().center_y()).height(anchor().letters(1.0)),
    )
}

/// Puts the parts back in step with the value and the caret.
///
/// Runs at the drain, after whatever changed either of them, so the frame that took the keystroke is
/// the frame the caret has moved in.
pub(crate) fn refresh(grove: &mut Grove, field: Leaf) {
    let Some(parts) = grove.tree.parts(field) else {
        return;
    };
    let length = grove
        .tree
        .lettering(parts.run)
        .map(|value| value.chars().count())
        .unwrap_or_default();
    let editing = grove.tree.editing(field).clamped(length);
    grove.tree.set_editing(field, editing);
    let selected = editing.span();
    grove
        .tree
        .set_location(parts.caret, caret_at(editing.caret));
    grove
        .tree
        .set_location(parts.selection, span(selected.start, selected.end));
    grove
        .tree
        .set_visible(parts.selection, !selected.is_empty());
    grove.tree.set_visible(parts.hint, length == 0);
    // The caret is kept in view by the region the field already is, against the extent this frame's
    // R3 measures rather than the one the last frame left -- which is what makes typing past the
    // right edge of the box scroll it in the same frame.
    grove.sought.push((field, ScrollTo::show(parts.caret)));
}

/// One keystroke, delivered to the field holding focus.
pub(crate) fn typed(grove: &mut Grove, field: Leaf, stroke: Keystroke) {
    let Some(parts) = grove.tree.parts(field) else {
        debug!(leaf = field.id(), "type dropped: not a field");
        return;
    };
    let value = grove
        .tree
        .lettering(parts.run)
        .unwrap_or_default()
        .to_string();
    match applied(&value, grove.tree.editing(field), stroke) {
        Applied::Wrote(written, editing) => {
            grove.tree.set_lettering(parts.run, written);
            grove.tree.set_editing(field, editing);
            grove.drift.edited.insert(field);
            refresh(grove, field);
            debug!(leaf = field.id(), "edited");
        }
        Applied::Moved(editing) => {
            grove.tree.set_editing(field, editing);
            refresh(grove, field);
        }
        Applied::Submitted => {
            grove.drift.submitted.insert(field);
            debug!(leaf = field.id(), "submitted");
        }
        Applied::Nothing => {}
    }
}

/// The kind, for the two questions [`Fronds`] asks of every field at once.
pub(crate) struct Field;

impl Fronds for Field {
    fn gestured(&self, grove: &mut Grove) {
        gestured(grove)
    }

    fn settled(&self, grove: &mut Grove) {
        settled(grove)
    }
}

/// What a field makes of the gestures dispatch reported this frame.
///
/// **A field reads interaction; interaction knows nothing about fields.** A tap is a statement
/// about where the caret goes and a drag is one about what is selected, but that is a field's
/// reading of an ordinary gesture, not something a gesture carries. So this asks the two questions
/// a field has of what was reported -- was I tapped, and where; am I being dragged, and to where --
/// and queues the same [`select`](crate::Grow::select) an app would write.
fn gestured(grove: &mut Grove) {
    for (field, parts) in grove.tree.fields() {
        // A tap says where the caret goes and collapses whatever was selected. A gesture that
        // became a drag was never a tap, so a drag out of a field to scroll the page behind it
        // leaves the field exactly as it found it.
        if let Some(at) = grove.drift.clicked.get(&field).copied() {
            let index = index_at(grove, parts, at);
            grove.queue.push(Op::Select {
                leaf: field,
                range: index..index,
            });
        }
        // A drag selects from where it began to where it has reached. Stated as one span rather
        // than as two moves, because that is what it is -- and low end last where the drag went
        // leftwards, which is how the anchor stays the end that was pressed.
        if let Some(drag) = grove.drift.dragged.get(&field).copied() {
            let anchor = index_at(grove, parts, drag.start);
            let caret = index_at(grove, parts, drag.current);
            grove.queue.push(Op::Select {
                leaf: field,
                range: anchor..caret,
            });
        }
    }
}

/// Puts every field's parts back in step with focus.
///
/// At the end of the drain, where focus is already final: a tap settles it at dispatch and an
/// [`Op::Focus`] settles it here, so by now there is nothing left that could move it. That is what
/// keeps a caret an ordinary [`visible`](crate::Grow::visible) write inherited by R7 like anything
/// else, instead of a product patched up after the pass that composes it.
///
/// What is selected is not cleared with focus. A selection is state and focus is not, so a field
/// stepped away from and back into is as it was left -- while a *tap* back into it collapses the
/// selection, because a tap says where the caret goes.
fn settled(grove: &mut Grove) {
    let focused = grove.focus.held();
    for (field, parts) in grove.tree.fields() {
        let showing = Some(field) == focused;
        grove.tree.set_visible(parts.caret, showing);
        let selected = grove.tree.editing(field).span();
        grove
            .tree
            .set_visible(parts.selection, showing && !selected.is_empty());
    }
}

/// Selects a span of the value outright.
///
/// `range` is read as anchor-then-caret rather than low-then-high, so a span whose end precedes its
/// start is a selection reaching backwards -- which is what a drag leftwards is, and what a shifted
/// arrow continues from.
pub(crate) fn select(grove: &mut Grove, field: Leaf, range: Range<usize>) {
    if grove.tree.parts(field).is_none() {
        debug!(leaf = field.id(), "select dropped: not a field");
        return;
    }
    grove.tree.set_editing(
        field,
        Editing {
            anchor: range.start,
            caret: range.end,
        },
    );
    refresh(grove, field);
}

/// Rewrites the whole value, leaving the caret at the end of it.
///
/// Takes the parts the drain already looked up to tell a field from a run, so the one question that
/// answers it is asked once.
///
/// Not reported as [`edited`](crate::Pollen::edited): that is what the person at the keyboard did,
/// and an app that wrote the value already knows what it wrote.
pub(crate) fn lettered(grove: &mut Grove, field: Leaf, parts: Parts, value: String) {
    let length = value.chars().count();
    grove.tree.set_lettering(parts.run, value);
    grove.tree.set_editing(field, Editing::at(length));
    refresh(grove, field);
}

/// Which character of the run a point falls on.
///
/// Rounded rather than floored, so pressing past the middle of a character puts the caret after it
/// -- which is where a hand aiming between two characters means.
fn index_at(grove: &Grove, parts: Parts, at: Position) -> usize {
    let cell = grove.tree.cell(parts.run);
    if cell.width <= 0.0 {
        return 0;
    }
    let length = grove
        .tree
        .lettering(parts.run)
        .map(|value| value.chars().count())
        .unwrap_or_default();
    let across = at.x - grove.tree.drawn(parts.run).left();
    ((across / cell.width).round().max(0.0) as usize).min(length)
}
