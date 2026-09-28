# Fronds

Almost everything foliage grows is one element. A few things are several: the app names one `Leaf`,
and parts are grown under it, placed and hidden on its behalf. A frond is a leaf divided into
leaflets that is still one leaf, and [`frond.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/frond.rs)
is the whole of what the frame knows about them. Today there is one kind, the text field, in
[`text_input.rs`](https://github.com/eblack-leaf/foliage/blob/main/foliage/src/text_input.rs).

## Three questions

```rust,ignore
// frond.rs
pub(crate) trait Sprouts: Send + Sync + 'static {
    fn sprout(self: Box<Self>, grove: &mut Grove, leaf: Leaf);
}

pub(crate) trait Fronds: Sync {
    fn gestured(&self, grove: &mut Grove);
    fn settled(&self, grove: &mut Grove);
}

pub(crate) const FRONDS: &[&dyn Fronds] = &[&text_input::Field];
```

| | Asked | For |
|---|---|---|
| `Sprouts::sprout` | in the drain step that grew the leaf | grow my leaflets |
| `Fronds::gestured` | straight after dispatch, before the app's `frame` | what did this frame's gestures mean to me |
| `Fronds::settled` | at the end of the drain | put my leaflets back in step |

The last two are asked of a *kind* rather than of an element, because each is about every element of
that kind at once. `FRONDS` is the only place a kind is named: `fern` walks the list and names none
of them, so adding a kind is a seed, an implementation and one line, and no pass of the engine grows
a branch for whatever one of them happens to want.

Neither hook is a place for arbitrary work, and each sits where it does for a reason. `gestured` runs
while this frame's reports are still being collected and before the app runs, so a frond reads
interaction on exactly the terms an app does, and whatever the app queues afterwards has the last
word. `settled` runs once focus and every queued write are final, so leaflets are put in step with
state that cannot change again this frame. Both **queue or write like anything else**, and neither
reaches into resolution: nothing about a frond is an input to a pass that would otherwise not know it
existed.

## What a field is made of

| Part | Is |
|---|---|
| the field | the element the app named: it receives, and it is a scrolling region |
| `run` | the value, as an ordinary run of text |
| `hint` | the placeholder, visible only while the value is empty |
| `caret` | a panel two pixels wide, visible only while the field holds focus |
| `head`, `body`, `tail` | the selection: one panel per part of a span that crosses lines |

The field's own seed declares two things an app never has to. It is **interactive**, which is what
lets a gesture reach it and focus rest on it. And it **scrolls** along the axis its value grows on,
across for a `TextInput` and down for a `TextArea`, containing that axis. That one declaration is
what clips a value longer than its box and what the caret is kept in view by. It declares no drags,
so a plain drag across a field is the region's and scrolls the value; a selection comes out of a
hold, which claims a drag whatever an element declared ([Interaction](interaction.md#a-press-that-was-held)).

## Sprouting

The seed carries a `Sprout` (the value, the placeholder, the four fills, the keypad, whether it is
read-only, and how many lines it has) in its `Bud`. When the drain grows the field, it takes the
sprout and hands it the new leaf, and `sprout` grows the six parts directly into the tree, in the same
drain step, with names from the same allocator. Nothing downstream can tell them from anything else
grown that frame, and the frame that planted the field is the frame all of it is live in.

Every part is `intangible`. The hit test stops at the top of the stack, so if any part could be the
top, the field itself never would be. The selection sits at the back, the caret in front of it, and
the run in front of both, because a caret drawn over the run would take a bite out of the glyph it
stands before.

## The caret is placed in the ordinary grammar

Nothing measures a caret. It is anchored to the run and placed with the same grammar an app writes:

```rust,ignore
// text_input.rs
fn caret_at(index: usize) -> Location {
    Location::new().xs(
        left(anchor().left() + anchor().character(index)).width(CARET.px()),
        top(anchor().top() + anchor().character(index)).height(anchor().letters(1.0)),
    )
}
```

`anchor().character(n)` resolves to the cell the run wrapped character `n` into, read off the wrap at
the width the run has *this* frame ([the resolver](resolver.md#how-each-source-reads)). So the caret
is right on the frame the run wraps differently, and it moves when the run moves, because an anchor
does.

A selection is the span between two such positions. On one line it is one box; across lines it is
three: the rest of the first line, every whole line between, and the start of the last. All three
are always placed, and the ones a span does not need resolve to nothing: the tail has no width when
both ends share a line, and the body has no height unless there are whole lines between them. Which
lines the ends fell on is the layout's to read off the wrap, exactly as the caret's line is, so a
frame that rewraps under a selection draws it divided the new way with nothing decided here.

## Gestured: what a tap means to a field

Dispatch knows nothing about fields. It reports a tap, a hold or a drag like any other, and
`gestured` reads what was reported about each field:

- a **tap** queues a `select` of an empty range at the character it landed on, placing the caret and
  collapsing any selection;
- a **hold** queues focus and the same empty `select`, where the selection will begin;
- a **drag** out of a hold queues a `select` from the selection's anchor to the character the pointer
  has reached, and while the pointer is past the field's edge it asks for another frame, because a
  reader holding still out there is still asking for more of the value.

These are the same `select` ops an app would write, queued before the app runs, so an app that wants
something else writes it in `frame` and wins. Which character a point falls on rounds the column (a
press past the middle of a character puts the caret after it) and floors the line, then asks the
wrap which index that cell is.

## Typing

A key for a focused field reaches it in the drain, as a `Keyed` op, and the field applies it through
one pure function:

```rust,ignore
// text_input.rs
pub(crate) fn applied(
    value: &str,
    editing: Editing,
    stroke: Keystroke,
    wrap: Option<Wrap<'_>>,
) -> Applied
```

It takes the value, the caret and selection, the key, and (for an area) the value as it wrapped, and
says what the key does: a new value and caret, a moved caret, a copy, a cut, a paste to ask for, a
submission, or nothing. Everything editing means is in that one function, as arithmetic over
character indices, which is why every case of it can be tested as one line of a table. A read-only
field passes the answer through a filter that turns writes into nothing and a cut into a copy.

A write sets the run's text and the caret, records `edited`, and calls `refresh`, which re-places the
caret, shows or hides the placeholder, and asks the field to `show` the caret, so R4 scrolls the
value in the same frame the caret passed the edge. A paste is the one key that cannot finish in the
frame it arrived: the clipboard is asked, and the answer comes back as a `Pasted` op in a later drain,
where it is written exactly as typing would have been.

## Settled: putting the parts in step

At the end of every drain, `settled` walks the fields once focus is final:

- the caret is visible if the field holds focus and is not read-only;
- the selection is placed and shown if the field holds focus and something is selected, and hidden
  otherwise. It is not cleared when focus leaves: a selection is state and focus is not, so a field
  stepped away from and back into is as it was left;
- the soft keyboard is raised for the focused field's keypad, or lowered if nothing typeable holds
  focus. Nothing about a phone is declared anywhere; being a field is the whole of it.

Every one of those is an ordinary `visible` or `at` write, recorded like any other and resolved by R7
and R2 in the same frame. That is the payoff of settling focus before resolution runs: a caret is not
a product patched up after the pass that composes it.

## Addressed as one

Every verb and every read an app aims at a field is redirected to the part it concerns. `text` writes
the run and moves the caret to the end; `color` refills the run; `tap(field, Vein::Text)` reads the
run; `Vein::Selection` reads the field's `Editing`. The field carries a `Parts` component naming its
six parts, and that component's presence is the one test for "is this a field", asked once per op.
So what a field is made of is never a name an app has to hold, and never a surface an app has to keep
in step.
