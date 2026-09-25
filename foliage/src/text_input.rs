//! TextInput -- an editable run, and the first [`Frond`](crate::frond): a leaf that is divided.
//!
//! # What a field is made of
//!
//! Everything else foliage grows is one element. A field is six, because the parts move
//! independently of each other and each is already something the engine draws:
//!
//! | | Part | Is |
//! |---|---|---|
//! | | `run` | the value, as an ordinary [`Text`](crate::Text) |
//! | | `hint` | the placeholder, shown only while the value is empty |
//! | | `caret` | a [`Panel`](crate::Panel) [`CARET`] wide, shown only while the field holds focus |
//! | | `head`, `body`, `tail` | the selection: [`Panel`](crate::Panel)s behind the run, one per part of a span that crosses lines |
//!
//! The app names the field and nothing else. Every verb and every read is addressed to it, and the
//! parts are grown, placed and hidden from here -- so a field is one [`Leaf`] to hold, and what it
//! is made of is not a surface to keep in step with.
//!
//! # The caret is placed in the ordinary grammar
//!
//! A caret at character `n` is `anchor().left() + anchor().character(n)` across and
//! `anchor().top() + anchor().character(n)` down, against the run. That is the whole of the
//! geometry: no pass measures a caret, because [`character`](crate::Anchor::character) resolves an
//! index to the cell the run wrapped that character into, and it moves when the run moves because
//! an anchor does. It is exact on the frame the run wraps differently, because it is read off the
//! wrap rather than off where the caret was last seen.
//!
//! A selection is the span between two of those. On one line it is one box; across lines it is
//! three -- the rest of the first line, every whole line between, and the start of the last -- and
//! which of the three are drawn is the one thing decided here rather than by the layout, against the
//! width the run was last drawn at.
//!
//! The run is drawn in front of all of them. A caret sits on the boundary between two character
//! cells and is as wide as it needs to be seen, so a caret drawn over the run would take a bite out
//! of the glyph it stands before -- at a small size, a quarter of it.
//!
//! # The field is a scrolling region
//!
//! A [`TextInput`] is one line, as wide as its own value, inside a box that clips it and scrolls
//! across. A [`TextArea`] wraps at its box and scrolls down. Either is a region, so it is declared as
//! one -- which is also what makes the caret stay in view: every edit asks the field to
//! [`show`](crate::ScrollTo::show) the caret, and R4 answers it against the extent the same frame
//! measured.
//!
//! # Read-only is a field that keeps its value
//!
//! A field [`read_only`](TextInput::read_only) is still a field: it receives, so a drag scrolls it
//! and a hold selects in it; it takes focus, so the arrows walk it and `Ctrl+C` copies out of it.
//! What it refuses is every keystroke that would change the value -- typing, deleting, cutting
//! and pasting -- which is the one thing that separates reading a value from editing it.
//! Disabling a field would refuse far more: a disabled region takes no gesture at all, so a value
//! longer than its box could not be scrolled to be read.

use core::ops::Range;

use bevy_ecs::component::Component;
use tracing::debug;

use crate::coordinate::{Axes, Position};
use crate::elevation::Elevation;
use crate::elm::{Chlorophyll, PanelPigment, Pigment};
use crate::frond::{Fronds, Sprouts};
use crate::grove::Grove;
use crate::interaction::focus::Intent;
use crate::interaction::input::{Key, Keystroke};
use crate::interaction::{self, Gestures};
use crate::keyboard::Keypad;
use crate::leaf::Leaf;
use crate::lifecycle::Visible;
use crate::op::{Bud, Op};
use crate::palette::{Fill, Palette};
use crate::place::{Anchored, Boxed, Caller, Manner, Placement, Places};
use crate::placement::basis::anchor;
use crate::placement::location::Location;
use crate::placement::role::{center_y, left, top};
use crate::placement::source::{Source, content};
use crate::seed::Buds;
use crate::text::Lettering;
use crate::text::TextPigment;
use crate::text::shape::{Shaped, Wrap, shape};
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
/// A drag across a field **scrolls its value**, because a field declares no drags and so is left
/// to the region it is. Selecting is a press that was [`held`](crate::Pollen::held) and then
/// dragged: the two motions are identical until the hold separates them, which is the only thing
/// that can.
///
/// What it types is reported as [`edited`](crate::Pollen::edited), and an `Enter` as
/// [`submitted`](crate::Pollen::submitted).
///
/// Its value is what [`text`](crate::Grow::text) rewrites and [`Vein::Text`](crate::Vein::Text)
/// reads, and what [`color`](crate::Grow::color) refills -- moved with a
/// [`Motion::Color`](crate::Motion::Color) or a [`Motion::Palette`](crate::Motion::Palette) like
/// any other fill, and read back as [`Vein::Color`](crate::Vein::Color). The placeholder, the
/// caret and the selection keep the fills they were grown with.
///
/// It answers the clipboard itself: `Ctrl+C` and `Ctrl+X` put the selected span on it and `Ctrl+V`
/// asks for what is there, which lands **in a later frame** because what a clipboard holds is the
/// host's to say. A paste is reported as [`edited`](crate::Pollen::edited) like anything else the
/// person at the keyboard did, so an app hears it where it hears the typing.
///
/// On a platform with a soft keyboard it raises the [`keypad`](TextInput::keypad) it named
/// whenever it holds focus, and nothing declares that beyond being a field.
///
/// A value of more than one line is a [`TextArea`].
#[derive(Clone, Debug)]
pub struct TextInput {
    pub(crate) placement: Placement,
    pub(crate) value: String,
    pub(crate) placeholder: String,
    pub(crate) fill: Fill,
    pub(crate) hint: Fill,
    pub(crate) caret: Fill,
    pub(crate) selection: Fill,
    pub(crate) keypad: Keypad,
    pub(crate) read_only: bool,
}

/// An editable run of glyphs that wraps.
///
/// A [`TextInput`] with as many lines as its value takes: the run wraps at the box's width exactly
/// as a [`Text`](crate::Text) does, and the field scrolls **down** rather than across to keep the
/// caret in view.
///
/// ```no_run
/// # use foliage::{Boxed, FontSize, Location, Palette, Place, Source, TextArea, left, top};
/// TextArea::new()
///     .placeholder("Notes")
///     .color(Palette::Ink)
///     .font_size(FontSize::new().xs(14))
///     .at(Location::new().xs(
///         left(0.px()).right(100.pct()),
///         top(0.px()).height(160.px()),
///     ));
/// ```
///
/// Where the two differ is what the keys that are about lines mean. `Enter` puts a newline in the
/// value where a field would submit, and `Ctrl+Enter` is what submits. `Up` and `Down` move the
/// caret by a line, and `Home` and `End` go to the ends of the line the caret is on rather than of
/// the whole value -- the line as it wrapped, which is the line a reader sees. A newline that comes
/// in on the clipboard is kept, where a field drops it.
///
/// Everything else -- the placeholder, the fills, the clipboard, the keypad, what a tap and a hold
/// do, and what it reports -- is as it is on a [`TextInput`], and it is read with the same
/// [`Vein::Text`](crate::Vein::Text) and [`Vein::Selection`](crate::Vein::Selection).
#[derive(Clone, Debug)]
pub struct TextArea {
    pub(crate) placement: Placement,
    pub(crate) value: String,
    pub(crate) placeholder: String,
    pub(crate) fill: Fill,
    pub(crate) hint: Fill,
    pub(crate) caret: Fill,
    pub(crate) selection: Fill,
    pub(crate) keypad: Keypad,
    pub(crate) read_only: bool,
}

/// The builders a field and an area share, which are all of them: the two are the same thing to
/// describe and differ only in what they do with a line.
macro_rules! field {
    ($seed:ident, $lines:expr) => {
        impl Default for $seed {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $seed {
            /// An empty one, read in [`Palette::Ink`] with an [`Palette::Accent`] caret.
            pub fn new() -> Self {
                Self {
                    placement: Placement::default(),
                    value: String::new(),
                    placeholder: String::new(),
                    fill: Fill::Role(Palette::Ink),
                    hint: Fill::Role(Palette::Muted),
                    caret: Fill::Role(Palette::Accent),
                    selection: Fill::Role(Palette::Muted),
                    keypad: Keypad::Text,
                    read_only: false,
                }
            }

            /// What the field starts out saying.
            pub fn value(mut self, value: impl Into<String>) -> Self {
                self.value = value.into();
                self
            }

            /// What is read in the field's place while it says nothing.
            ///
            /// Drawn in [`hint`](Self::hint) rather than in the value's own fill, and never part of
            /// the value: it is absent from [`Vein::Text`](crate::Vein::Text) and a field showing
            /// one is empty.
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

            /// Which soft keyboard the field raises while it holds focus.
            ///
            /// A hint about what is easy to type and not a rule about what the field takes: a
            /// [`Keypad::Number`] field can still be pasted a word into, so what a value is allowed
            /// to be is the app's to check either way. Ignored where the platform raises no
            /// keyboard of its own.
            pub fn keypad(mut self, keypad: Keypad) -> Self {
                self.keypad = keypad;
                self
            }

            /// Whether the value can be read but not changed.
            ///
            /// Read-only, the field still receives: a drag scrolls it, a hold selects in it, and
            /// with focus the arrows walk it and `Ctrl+C` copies out of it. Every keystroke that
            /// would change the value does nothing -- a cut is only a copy -- no caret is drawn,
            /// and no soft keyboard is raised. What the app writes with
            /// [`text`](crate::Grow::text) is written either way; this is about the person at the
            /// keyboard. Changed later with [`read_only`](crate::Grow::read_only).
            pub fn read_only(mut self, read_only: bool) -> Self {
                self.read_only = read_only;
                self
            }
        }

        impl Places for $seed {
            fn placement(&mut self) -> &mut Placement {
                &mut self.placement
            }
        }

        impl Boxed for $seed {}

        impl Buds for $seed {
            fn bud(mut self, at: Caller) -> Bud {
                let lines: Lines = $lines;
                // A field is measured in characters throughout -- its caret, its selection and its
                // own value -- so it carries a typeface whether or not one was named, exactly as a
                // run does.
                self.placement.typeface.get_or_insert_default();
                // Declared here rather than left to the app, because both are what make a field a
                // field rather than a preference about one. It receives, so a gesture can reach it
                // and focus can rest on it -- which is what makes a tap focus it, since that is
                // where focus goes for anything that receives; and it is a region along the axis
                // its value grows on, which is what clips a value larger than its box and what the
                // caret is kept in view by.
                //
                // It declares no drags. A drag across a field is then the region's, and the region
                // is the field, so the value moves under its box; selection comes out of a hold,
                // which claims a drag whatever an element declared.
                self.placement.manner.gestures.receives = true;
                let axes = lines.axes();
                self.placement.manner.scrolls = Some(Scrolls(Scroll::new(axes).contain(axes)));
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
                        keypad: self.keypad,
                        read_only: self.read_only,
                        lines,
                    })),
                    at,
                    ..Bud::bare()
                }
            }
        }
    };
}

field!(TextInput, Lines::One);
field!(TextArea, Lines::Many);

/// How many lines a field has: one, or as many as its value wraps to.
///
/// The one thing that tells a [`TextInput`] from a [`TextArea`]. Everything a field does reads the
/// same way in either -- what differs is which axis the value grows along, and so which axis the
/// field scrolls, which keys are about lines, and whether a newline is a character it holds.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Lines {
    /// One line, as wide as its value, scrolling across.
    #[default]
    One,
    /// As many lines as the value wraps to at the box's width, scrolling down.
    Many,
}

impl Lines {
    /// The axis the value grows along, which is the one the field scrolls.
    fn axes(self) -> Axes {
        match self {
            Lines::One => Axes::Horizontal,
            Lines::Many => Axes::Vertical,
        }
    }

    /// How many cells across the run wraps at, given the width it was last drawn at.
    ///
    /// A field of one line never wraps: its run is as wide as its own value by declaration, so the
    /// answer is not read off a width that may not have been resolved yet.
    fn columns(self, shaped: &Shaped, width: f32) -> usize {
        match self {
            Lines::One => usize::MAX,
            Lines::Many => shaped.columns(width),
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
    pub(crate) keypad: Keypad,
    pub(crate) read_only: bool,
    pub(crate) lines: Lines,
}

/// Whether a field refuses the keystrokes that would change its value.
///
/// Carried on the field beside its [`Keypad`], and read wherever a keystroke is about to write:
/// the whole of read-only is that those writes do not happen.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReadOnly(pub(crate) bool);

/// The six elements a field is made of, and how many lines it has.
///
/// Carried on the field, which is what makes every verb addressed to the field able to reach the
/// part it actually changes -- and what makes an element a field at all, since nothing else grows
/// one.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Parts {
    pub(crate) run: Leaf,
    pub(crate) hint: Leaf,
    pub(crate) caret: Leaf,
    /// The selection on the line it begins: from its start to its end where the two share a line,
    /// and to the run's right edge where they do not.
    pub(crate) head: Leaf,
    /// Every whole line the selection covers between the one it begins on and the one it ends on.
    pub(crate) body: Leaf,
    /// The selection on the line it ends, from the run's left edge, where that is not the line it
    /// began on.
    pub(crate) tail: Leaf,
    pub(crate) lines: Lines,
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
    /// The selected span, to go on the clipboard. The value is unchanged.
    Copied(String),
    /// The same span, with the value it came out of and where that leaves the caret.
    Cut {
        text: String,
        written: String,
        editing: Editing,
    },
    /// The clipboard was asked for, to be written in at the caret when it answers.
    ///
    /// The one keystroke a field cannot finish on its own: what is on the clipboard is the host's
    /// to say and it does not say it now, so the write happens in the frame the answer lands.
    Pasting,
    /// `Enter` on a field, or `Ctrl+Enter` on an area.
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
///
/// `wrap` is the value as it wrapped, for a field of more than one line, and `None` for a field of
/// one. It is what the keys that are about lines are answered against -- a line is a fact about the
/// wrap and not about the value -- and its absence is what makes `Enter` a submission and `Up` and
/// `Down` nothing at all.
pub(crate) fn applied(
    value: &str,
    editing: Editing,
    stroke: Keystroke,
    wrap: Option<Wrap<'_>>,
) -> Applied {
    let characters: Vec<char> = value.chars().collect();
    let editing = editing.clamped(characters.len());
    let extend = stroke.modifiers.shift;
    let span = editing.span();
    let length = characters.len();
    // A command rather than a character. Held, a key says what to do with the value instead of
    // what to put in it, so it is answered before the key's own meaning is.
    if stroke.modifiers.control {
        // A selection is what the clipboard verbs work over, and an empty one is nothing to work
        // over: a field with its caret between two characters has no span to copy and none to cut.
        // Taking the whole value instead would be a rule nobody asked for.
        let selected = || characters[span.start..span.end].iter().collect::<String>();
        return match stroke.key {
            Key::Typed('a') | Key::Typed('A') => Applied::Moved(Editing {
                anchor: 0,
                caret: characters.len(),
            }),
            Key::Typed('c') | Key::Typed('C') if !editing.collapsed() => {
                Applied::Copied(selected())
            }
            Key::Typed('x') | Key::Typed('X') if !editing.collapsed() => Applied::Cut {
                text: selected(),
                written: spliced(&characters, span.clone(), ""),
                editing: Editing::at(span.start),
            },
            Key::Typed('v') | Key::Typed('V') => Applied::Pasting,
            // What submits an area, whose bare `Enter` is a newline. Answered for a field too, so
            // the chord means one thing wherever it is pressed.
            Key::Enter => Applied::Submitted,
            _ => Applied::Nothing,
        };
    }
    match stroke.key {
        Key::Typed(character) => Applied::Wrote(
            spliced(&characters, span.clone(), &character.to_string()),
            Editing::at(span.start + 1),
        ),
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
                spliced(&characters, removed.clone(), ""),
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
                spliced(&characters, removed.clone(), ""),
                Editing::at(removed.start),
            )
        }
        Key::Left => Applied::Moved(moved(editing, length, Toward::Left, extend, wrap)),
        Key::Right => Applied::Moved(moved(editing, length, Toward::Right, extend, wrap)),
        Key::Home => Applied::Moved(moved(editing, length, Toward::Start, extend, wrap)),
        Key::End => Applied::Moved(moved(editing, length, Toward::End, extend, wrap)),
        // A field is one line, so there is no line above or below to move to. An app reading its
        // own keys is what steps a list of them.
        Key::Up | Key::Down if wrap.is_none() => Applied::Nothing,
        Key::Up => Applied::Moved(moved(editing, length, Toward::Up, extend, wrap)),
        Key::Down => Applied::Moved(moved(editing, length, Toward::Down, extend, wrap)),
        // On a field, not an edit and not a caret: the app is told, and what that means is the
        // app's. On an area it is the one character a key cannot otherwise put in the value.
        Key::Enter => match wrap {
            None => Applied::Submitted,
            Some(_) => Applied::Wrote(
                spliced(&characters, span.clone(), "\n"),
                Editing::at(span.start + 1),
            ),
        },
        // Both answered before a field is ever asked, because both move focus wherever focus is:
        // `Tab` steps it and `Escape` takes it away. A field never sees either.
        Key::Tab | Key::Escape => Applied::Nothing,
    }
}

/// What a keystroke comes to on a field that is read-only: the same, less anything that would
/// change the value. A write is nothing, a paste is not asked for, and a cut is the copy it
/// contains -- the person asked for the span, and taking it out is the part refused. Moving,
/// selecting, copying and submitting are reading, and go through.
pub(crate) fn held(applied: Applied) -> Applied {
    match applied {
        Applied::Wrote(..) | Applied::Pasting => Applied::Nothing,
        Applied::Cut { text, .. } => Applied::Copied(text),
        other => other,
    }
}

/// Text put in at the caret, replacing whatever was selected.
///
/// The rule a paste is applied by, and the general case of the one a typed character is: what
/// arrives is a string rather than a character, and it is written in at exactly the same place.
///
/// Control characters are dropped, which is the rule a keystroke already arrives under. A newline
/// is the exception on an area, where it is a character the value holds; on a field it is dropped
/// with the rest, because a field is one line and a newline in it would be a character no cell can
/// be measured for. A carriage return is a newline's other spelling, and is read as one.
pub(crate) fn inserted(value: &str, editing: Editing, text: &str, lines: Lines) -> Applied {
    let characters: Vec<char> = value.chars().collect();
    let span = editing.clamped(characters.len()).span();
    let written: String = text
        .replace("\r\n", "\n")
        .chars()
        .filter(|character| match (lines, character) {
            (Lines::Many, '\n') => true,
            _ => !character.is_control(),
        })
        .collect();
    if written.is_empty() {
        return Applied::Nothing;
    }
    Applied::Wrote(
        spliced(&characters, span.clone(), &written),
        Editing::at(span.start + written.chars().count()),
    )
}

/// The value with a span of it replaced by `text`, which is empty where the span is being removed.
fn spliced(characters: &[char], span: Range<usize>, text: &str) -> String {
    characters[..span.start]
        .iter()
        .copied()
        .chain(text.chars())
        .chain(characters[span.end..].iter().copied())
        .collect()
}

/// Which way a caret was asked to go.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Toward {
    Left,
    Right,
    Start,
    End,
    Up,
    Down,
}

/// Where the caret lands, and what that leaves selected.
///
/// An unshifted arrow against a selection **collapses it to the edge it points at** rather than
/// stepping from the caret. That is what every editor does and what a reader means by it: the
/// selection was the thing being moved away from.
///
/// The keys that are about lines read the wrap. `Home` and `End` go to the ends of the line the
/// caret is on; `Up` and `Down` go to the same column of the line above or below, or to the ends of
/// the value where there is no such line -- and a column past the end of a shorter line is that
/// line's end, which is where a reader expects to land. Without a wrap there is one line, and its
/// ends are the value's.
fn moved(
    editing: Editing,
    length: usize,
    toward: Toward,
    extend: bool,
    wrap: Option<Wrap<'_>>,
) -> Editing {
    let span = editing.span();
    let caret = match (toward, extend, editing.collapsed()) {
        (Toward::Left, false, false) => span.start,
        (Toward::Right, false, false) => span.end,
        (Toward::Left, _, _) => editing.caret.saturating_sub(1),
        (Toward::Right, _, _) => (editing.caret + 1).min(length),
        (Toward::Start, _, _) => match wrap {
            Some(wrap) => wrap.ends(wrap.cell_of(editing.caret).1).0,
            None => 0,
        },
        (Toward::End, _, _) => match wrap {
            Some(wrap) => wrap.ends(wrap.cell_of(editing.caret).1).1,
            None => length,
        },
        (Toward::Up, _, _) => match wrap {
            Some(wrap) => match wrap.cell_of(editing.caret) {
                (_, 0) => 0,
                (column, line) => wrap.index_at(column, line - 1),
            },
            None => editing.caret,
        },
        (Toward::Down, _, _) => match wrap {
            Some(wrap) => {
                let (column, line) = wrap.cell_of(editing.caret);
                match line >= wrap.cell_of(length).1 {
                    true => length,
                    false => wrap.index_at(column, line + 1),
                }
            }
            None => editing.caret,
        },
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

/// Grows a field's six parts under it, in the drain that grew the field.
fn sprout(grove: &mut Grove, field: Leaf, sprout: Sprout) {
    let at = grove
        .tree
        .spawned_at(field)
        .unwrap_or(core::panic::Location::caller());
    let typeface = grove.tree.typeface(field);
    let lines = sprout.lines;
    // Every name first, so the parts can be placed against each other as they are grown.
    let (run, run_growth) = grove.naming.leaf();
    let (hint, hint_growth) = grove.naming.leaf();
    let (caret, caret_growth) = grove.naming.leaf();
    let (head, head_growth) = grove.naming.leaf();
    let (body, body_growth) = grove.naming.leaf();
    let (tail, tail_growth) = grove.naming.leaf();
    let parts = Parts {
        run,
        hint,
        caret,
        head,
        body,
        tail,
        lines,
    };
    // A run sized along the axis the value grows on and held to the box on the other: as wide as
    // its own value and one line tall, centred, on a field; as wide as the box and as tall as it
    // wraps to, from the top, on an area. Its extent along that axis is what the field's is
    // measured from, so a value larger than the box is what makes the field scrollable and nothing
    // has to say so. It sits in front of every mark, because a mark drawn over it would cut into
    // the glyph it stands against.
    let line = |elevation: i32| Placement {
        location: Some(match lines {
            Lines::One => Location::new().xs(
                left(0.px()).width(content()),
                center_y(50.pct()).height(1.letters()),
            ),
            Lines::Many => {
                Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(content()))
            }
        }),
        elevation: Some(Elevation::up(elevation)),
        typeface,
        manner: Manner {
            // The hit test reads the top of the stack and stops, so every part of a field has to be
            // intangible or the field itself would never be the top of it.
            gestures: Gestures {
                intangible: true,
                ..Gestures::default()
            },
            ..Manner::default()
        },
        ..Placement::default()
    };
    // The caret and the selection are spans of the run's own character cells, so all are anchored
    // to it: the caret is rewritten by `refresh` whenever it moves, and the selection by `settled`
    // every frame it is drawn.
    let against = |elevation: i32, visible: bool| Placement {
        location: Some(head_at(0, Some(0))),
        anchor: Some(Anchored { to: run, at }),
        elevation: Some(Elevation::up(elevation)),
        manner: Manner {
            gestures: Gestures {
                intangible: true,
                ..Gestures::default()
            },
            visible: Visible(visible),
            ..Manner::default()
        },
        ..Placement::default()
    };
    let mark = |fill: Fill, placement: Placement| Bud {
        chlorophyll: Chlorophyll::Panel,
        pigment: Some(Pigment::Panel(PanelPigment {
            fill,
            rounding: Default::default(),
        })),
        placement,
        at,
        ..Bud::bare()
    };
    // The selection says it is hidden, because nothing is selected yet. The caret always has
    // something to show, so it declares itself shown and lets focus be the only thing that hides
    // it.
    for (leaf, growth) in [
        (head, head_growth),
        (body, body_growth),
        (tail, tail_growth),
    ] {
        grove.tree.grow(
            leaf,
            growth,
            Some(field),
            mark(sprout.selection, against(0, false)),
        );
    }
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
        mark(sprout.caret, against(1, true)),
    );
    grove.tree.set_parts(field, parts);
    grove.tree.set_editing(field, Editing::default());
    grove.tree.set_keypad(field, sprout.keypad);
    grove.tree.set_read_only(field, sprout.read_only);
    refresh(grove, field);
    debug!(leaf = field.id(), "field sprouted");
}

/// Where the caret sits at character `index` of the run: the cell that character wrapped into.
fn caret_at(index: usize) -> Location {
    Location::new().xs(
        left(anchor().left() + anchor().character(index)).width(CARET.px()),
        top(anchor().top() + anchor().character(index)).height(anchor().letters(1.0)),
    )
}

/// The selection on the line character `from` is on: to character `to` where it is on the same
/// line, and to the run's right edge where it is not.
fn head_at(from: usize, to: Option<usize>) -> Location {
    let across = left(anchor().left() + anchor().character(from));
    Location::new().xs(
        match to {
            Some(to) => across.right(anchor().left() + anchor().character(to)),
            None => across.right(anchor().right()),
        },
        top(anchor().top() + anchor().character(from)).height(anchor().letters(1.0)),
    )
}

/// Every whole line between the one character `from` is on and the one character `to` is on.
fn body_at(from: usize, to: usize) -> Location {
    Location::new().xs(
        left(anchor().left()).right(anchor().right()),
        top(anchor().top() + anchor().character(from) + anchor().letters(1.0))
            .bottom(anchor().top() + anchor().character(to)),
    )
}

/// The line character `to` is on, from the run's left edge up to it.
fn tail_at(to: usize) -> Location {
    Location::new().xs(
        left(anchor().left()).right(anchor().left() + anchor().character(to)),
        top(anchor().top() + anchor().character(to)).height(anchor().letters(1.0)),
    )
}

/// Puts the parts back in step with the value and the caret.
///
/// Runs at the drain, after whatever changed either of them, so the frame that took the keystroke is
/// the frame the caret has moved in. The selection is not placed here: it is placed at the end of
/// the drain, where focus is final and so is what it is drawn against.
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
    grove
        .tree
        .set_location(parts.caret, caret_at(editing.caret));
    grove.tree.set_visible(parts.hint, length == 0);
    // The caret is kept in view by the region the field already is, against the extent this frame's
    // R3 measures rather than the one the last frame left -- which is what makes typing past the
    // edge of the box scroll it in the same frame.
    grove.sought.push((field, ScrollTo::show(parts.caret)));
}

/// One keystroke, delivered to the element holding focus.
///
/// Anything but a field makes nothing of one, which is ordinary rather than a drop: the key was
/// delivered and reported, and having no use for it is what most elements do.
pub(crate) fn typed(grove: &mut Grove, field: Leaf, stroke: Keystroke) {
    let Some(parts) = grove.tree.parts(field) else {
        return;
    };
    let value = value(grove, parts);
    let shaped = shaped(grove, parts, &value);
    let wrap = match parts.lines {
        Lines::One => None,
        Lines::Many => Some(shaped.wrap(columns(grove, parts, &shaped))),
    };
    let mut applied = applied(&value, grove.tree.editing(field), stroke, wrap);
    if grove.tree.read_only(field) {
        applied = held(applied);
    }
    match applied {
        Applied::Wrote(written, editing) => wrote(grove, field, parts, written, editing),
        Applied::Moved(editing) => {
            grove.tree.set_editing(field, editing);
            refresh(grove, field);
        }
        Applied::Copied(text) => grove.clipboard.write(text),
        Applied::Cut {
            text,
            written,
            editing,
        } => {
            grove.clipboard.write(text);
            wrote(grove, field, parts, written, editing);
        }
        // The one keystroke that finishes in another frame. The read is started here and the write
        // happens where its answer is drained, which is `pasted`.
        Applied::Pasting => {
            let (queue, wake) = (grove.queue.clone(), grove.wake.clone());
            grove.clipboard.read(&queue, &wake, Some(field));
        }
        Applied::Submitted => {
            grove.drift.submitted.insert(field);
            debug!(leaf = field.id(), "submitted");
        }
        Applied::Nothing => {}
    }
}

/// What the clipboard answered, written in at the caret of the field that asked for it.
///
/// A frame or more after the `Ctrl+V` that asked, which is the whole reason the answer is an op:
/// what a clipboard holds is the host's to say and it does not say it now. Everything from here is
/// the ordinary write the keystroke would have made, so the paste is
/// [`edited`](crate::Pollen::edited) like anything else the person at the keyboard did.
pub(crate) fn pasted(grove: &mut Grove, field: Leaf, text: &str) {
    let Some(parts) = grove.tree.parts(field) else {
        return;
    };
    // Asked for before the field was made read-only, and answered after: it lands on a field that
    // no longer takes it.
    if grove.tree.read_only(field) {
        debug!(leaf = field.id(), "paste dropped: read-only");
        return;
    }
    if let Applied::Wrote(written, editing) = inserted(
        &value(grove, parts),
        grove.tree.editing(field),
        text,
        parts.lines,
    ) {
        wrote(grove, field, parts, written, editing);
    }
}

/// What the field currently says.
fn value(grove: &Grove, parts: Parts) -> String {
    grove
        .tree
        .lettering(parts.run)
        .unwrap_or_default()
        .to_string()
}

/// The field's value, shaped in the run's own cell.
///
/// Shaped here rather than read out of the cache, because the value may have been written in this
/// same drain and R1 has not seen it yet -- two keystrokes in one frame are the ordinary case. The
/// cell is the run's as R1 last measured it, which changes only with the font.
fn shaped(grove: &Grove, parts: Parts, value: &str) -> Shaped {
    shape(value, grove.tree.cell(parts.run))
}

/// How many cells across the field's run wraps at, as it was last drawn.
///
/// Last drawn, because the drain runs before this frame's layout: it is the width a reader is
/// looking at, which is the one a key or a point means. On a field of one line it is unbounded.
fn columns(grove: &Grove, parts: Parts, shaped: &Shaped) -> usize {
    parts
        .lines
        .columns(shaped, grove.tree.drawn(parts.run).width())
}

/// Puts a new value and caret on a field, however it was arrived at.
///
/// Typing, cutting and pasting are one write with three causes, and each is what the person at the
/// keyboard did to the value -- so each is one [`edited`](crate::Pollen::edited).
fn wrote(grove: &mut Grove, field: Leaf, parts: Parts, written: String, editing: Editing) {
    grove.tree.set_lettering(parts.run, written);
    grove.tree.set_editing(field, editing);
    grove.drift.edited.insert(field);
    refresh(grove, field);
    debug!(leaf = field.id(), "edited");
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
/// about where the caret goes, a hold is one about where a selection begins, and a drag is one
/// about how far it reaches -- but each is a field's reading of an ordinary gesture, not something
/// a gesture carries. So this asks what was reported about the leaves it owns and queues the same
/// [`select`](crate::Grow::select) an app would write.
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
        // A hold says the same thing a tap does about where the caret goes, and takes focus with
        // it: a held press is not a tap, so nothing has moved either. It is what begins a
        // selection, since the drag out of a hold is the field's whatever it declared.
        if let Some(at) = grove.drift.held.get(&field).copied() {
            let index = index_at(grove, parts, at);
            grove.queue.push(Op::Focus(Intent::To(field)));
            grove.queue.push(Op::Select {
                leaf: field,
                range: index..index,
            });
        }
        // A drag selects from where it began to where it has reached. Stated as one span rather
        // than as two moves, because that is what it is -- and low end last where the drag went
        // leftwards, which is how the anchor stays the end that was pressed.
        //
        // The anchor is read from the field and not from where the drag began. It is a character,
        // and the hold that started the selection already put it there; a *point* is not, because
        // the value scrolls under it the moment the drag reaches the edge -- so re-reading one
        // would move the end the selection was measured from as the field followed the caret.
        //
        // Where it has reached is the **open gesture** where there is one, and the move where the
        // gesture has already closed -- which is the release, and is the last of the movement. A
        // pointer held still reports nothing, and a drag that stopped moving is still a drag.
        let reached = grove
            .drift
            .dragged
            .get(&field)
            .map(|drag| drag.current)
            .or_else(|| interaction::dragging(grove, field));
        if let Some(to) = reached {
            let anchor = grove.tree.editing(field).anchor;
            let caret = index_at(grove, parts, to);
            grove.queue.push(Op::Select {
                leaf: field,
                range: anchor..caret,
            });
            // A pointer past the edge is a reader still asking for more of the value, and the
            // caret it is being taken to is further out every frame. So the field owes itself the
            // frames that get there, the way anything driving its own motion does.
            if beyond(grove, field, parts, to) {
                grove.again();
            }
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
///
/// The selection is placed here as well as shown, every frame it is drawn. Its geometry is the
/// layout's, read off the wrap by the same [`character`](crate::Anchor::character) the caret is --
/// but *how many boxes* it takes is a fact about which lines its two ends fell on, and that is read
/// here against the width the run was last drawn at. A frame that changes the wrap under a
/// selection is answered on the frame after, which is the one frame the drawn and the declared can
/// disagree.
///
/// The soft keyboard is settled here too, and it is the same statement the caret is: a platform's
/// own keyboard is raised for the field that holds focus and lowered for everything else, so
/// nothing about a phone is declared anywhere -- being a field is the whole of it.
fn settled(grove: &mut Grove) {
    let focused = grove.focus.held();
    for (field, parts) in grove.tree.fields() {
        let showing = Some(field) == focused;
        // A caret says where typing goes, and a read-only field takes none -- so it has none to
        // show. Its selection is reading, and is drawn like any other.
        grove
            .tree
            .set_visible(parts.caret, showing && !grove.tree.read_only(field));
        let selected = grove.tree.editing(field).span();
        if !showing || selected.is_empty() {
            for mark in [parts.head, parts.body, parts.tail] {
                grove.tree.set_visible(mark, false);
            }
            continue;
        }
        let value = value(grove, parts);
        let shaped = shaped(grove, parts, &value);
        let wrap = shaped.wrap(columns(grove, parts, &shaped));
        let (_, first) = wrap.cell_of(selected.start);
        let (_, last) = wrap.cell_of(selected.end);
        let same = first == last;
        grove.tree.set_location(
            parts.head,
            head_at(selected.start, same.then_some(selected.end)),
        );
        grove.tree.set_visible(parts.head, true);
        grove
            .tree
            .set_location(parts.body, body_at(selected.start, selected.end));
        grove.tree.set_visible(parts.body, last > first + 1);
        grove.tree.set_location(parts.tail, tail_at(selected.end));
        grove.tree.set_visible(parts.tail, !same);
    }
    // Nothing is raised to type into a field that takes no typing.
    let wanted = focused
        .filter(|&leaf| !grove.tree.read_only(leaf))
        .and_then(|leaf| grove.tree.keypad(leaf));
    grove.keyboard.raise(wanted);
}

/// Makes a field read-only, or editable again.
///
/// Dropped, like any op naming something it does not apply to, if the element is not a field. The
/// caret and the soft keyboard follow at the end of the drain, where focus is settled.
pub(crate) fn read_only(grove: &mut Grove, field: Leaf, read_only: bool) {
    if grove.tree.parts(field).is_none() {
        debug!(leaf = field.id(), "read-only dropped: not a field");
        return;
    }
    grove.tree.set_read_only(field, read_only);
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

/// Whether a point has left the field along the axis it scrolls.
///
/// That axis and not the whole box: a pointer below a one-line field is at a character like any
/// other, and the field has nowhere to go for it -- and a pointer beside an area is on the line it
/// is level with.
fn beyond(grove: &Grove, field: Leaf, parts: Parts, at: Position) -> bool {
    let box_of = grove.tree.drawn(field);
    match parts.lines {
        Lines::One => at.x < box_of.left() || at.x > box_of.right(),
        Lines::Many => at.y < box_of.top() || at.y > box_of.bottom(),
    }
}

/// Which character of the run a point falls on.
///
/// The column is rounded rather than floored, so pressing past the middle of a character puts the
/// caret after it -- which is where a hand aiming between two characters means. The line is floored,
/// because a hand is on the line it is on. Which index that cell is, on a run that wraps, is the
/// wrap's to say.
fn index_at(grove: &Grove, parts: Parts, at: Position) -> usize {
    let cell = grove.tree.cell(parts.run);
    if cell.width <= 0.0 || cell.height <= 0.0 {
        return 0;
    }
    let drawn = grove.tree.drawn(parts.run);
    let column = ((at.x - drawn.left()) / cell.width).round().max(0.0) as usize;
    let line = ((at.y - drawn.top()) / cell.height).floor().max(0.0) as usize;
    let value = value(grove, parts);
    let shaped = shaped(grove, parts, &value);
    shaped
        .wrap(columns(grove, parts, &shaped))
        .index_at(column, line)
}
