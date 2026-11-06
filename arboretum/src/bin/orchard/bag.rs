//! The bag, in from the right: what is in it, for how many seats, and what it all comes to -- the
//! scale's reading, in money and in bytes.
//!
//! A promo code is asked of the till, through the courier, so it is checked with a wait and can be
//! turned down. The totals roll like a scale settling.

use foliage::{
    Axes, Boxed, Ease, Elevation, FontSize, Grove, Grow, Leaf, Location, Motion, Palette, Panel,
    Place, Pollen, Position, ScrollTo, Source, Stem, Text, Timing, Tween, anchor, center_x,
    center_y, content, left, right, top,
};
use lichen::{Chip, Field, Press, Say, Voice, measure};

use arboretum::sprout::{Closed, Courier, Drawer, Edge, Roll};

use crate::api::{self, Refused};
use crate::art::{Art, Scale};
use crate::cart::{Cart, Promo};
use crate::catalog::{money, weight};
use crate::icons::Icons;
use crate::label;
use crate::theme::{self, Faces, HAIR, HEAVY};

/// How wide the drawer is, where the window allows.
const WIDE: f32 = 440.0;
/// How tall a line of the bag stands.
const LINE: f32 = 102.0;
/// How large a line's picture is.
const THUMB: f32 = 64.0;
/// How far in from a line's left its words start.
const TEXT_X: f32 = THUMB + 14.0;
/// How tall the bag stands when there is nothing in it.
const EMPTY: f32 = 132.0;
/// How long the till takes to answer a code.
const TILL_MS: u64 = 750;

/// What the bag asks of the shop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    /// To go to the till.
    Checkout,
    /// To go and find something to put in it.
    Browse,
    /// The bag changed: its count, for the tally.
    Changed(u32),
}

struct Row {
    root: Leaf,
    art: Art,
    name: Leaf,
    meta: Leaf,
    seats: Leaf,
    price: Leaf,
    minus: Chip,
    plus: Chip,
    remove: Chip,
}

pub struct Bag {
    drawer: Drawer,
    scroll: Leaf,
    close: Chip,
    summary: Leaf,
    list: Leaf,
    rows: Vec<Row>,
    going: Vec<(Leaf, Tween)>,
    empty: Leaf,
    browse: Chip,
    code: Field,
    apply: Chip,
    say: Say,
    subtotal: Roll,
    discount_key: Leaf,
    discount: Roll,
    total: Roll,
    checkout: Chip,
    courier: Courier<Result<Promo, Refused>>,
    faces: Faces,
    /// A seat fewer, a seat more, and out.
    marks: (foliage::Field, foliage::Field, foliage::Field),
}

impl Bag {
    pub fn grow(grove: &mut Grove, layer: Leaf, shop: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let m = measure();
        let drawer = Drawer::grow(grove, layer, shop, Edge::Right, WIDE);
        // The sheet's edge, in ink, so it stands off the page as a label stands off a shelf.
        grove.branch(
            drawer.sheet(),
            Panel::new()
                .color(Palette::Ink)
                .intangible()
                .elevate(Elevation::up(2))
                .at(Location::new()
                    .xs(left(0.px()).width(HAIR.px()), top(0.px()).bottom(100.pct()))),
        );
        // The body scrolls, and takes a stray press itself so that focus stays in the sheet.
        let scroll = grove.branch(
            drawer.body(),
            Stem::new()
                .interactive()
                .scrolls(Axes::Vertical)
                .elevate(Elevation::up(1))
                .font_size(lichen::caption())
                .at(Location::new()),
        );
        let col = grove.branch(
            scroll,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(Location::new().xs(left(0.px()).right(100.pct()), top(4.px()).height(10.px()))),
        );
        let title = grove.branch(
            col,
            Text::new("Your bag")
                .font(faces.roman)
                .font_size(FontSize::new().xs(28).md(30))
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(0.px()).height(1.letters()),
                )),
        );
        let close = Chip::mark(
            grove,
            col,
            Location::new().xs(
                right(100.pct()).width(Chip::square()),
                top(0.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.x,
        );
        let summary = label::words(
            grove,
            col,
            title,
            6.0,
            "",
            None,
            lichen::caption(),
            Palette::Muted,
        );
        let head = rule(grove, col, summary, 14.0, HEAVY);
        let list = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(head)
                .font_size(lichen::caption())
                .at(list_at(0)),
        );
        // What the bag says with nothing in it.
        let empty = grove.branch(
            list,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(0.px()).height(EMPTY.px()),
                )),
        );
        grove.branch(
            empty,
            Text::new("The bag weighs nothing yet.")
                .font(faces.italic)
                .font_size(FontSize::new().xs(20).md(22))
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(22.px()).height(content()),
                )),
        );
        let browse = Chip::grow(
            grove,
            empty,
            Location::new().xs(
                left(0.px()).width(Chip::width("browse the aisles")),
                top(70.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.arrow_left,
            "browse the aisles",
        );
        let foot = rule(grove, col, list, 0.0, HEAVY);
        // A code, asked of the till.
        let promo = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(foot)
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 18.px()).height(m.height.px()),
                )),
        );
        let apply_w = Chip::width("apply");
        let code = Field::grow(
            grove,
            promo,
            Location::new().xs(
                left(0.px()).right(100.pct() - apply_w.clone() - m.gap.px()),
                top(0.px()).height(100.pct()),
            ),
            6.0,
            "code",
            false,
        );
        code.hint(grove, "a promo code");
        let apply = Chip::grow(
            grove,
            promo,
            Location::new().xs(
                right(100.pct()).width(apply_w),
                top(0.px()).height(100.pct()),
            ),
            Elevation::up(1),
            icons.percent,
            "apply",
        );
        let say = Say::grow(
            grove,
            col,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + 10.px()).height(1.letters()),
            ),
            None,
        );
        grove.anchor(say.leaf(), promo);
        // The totals, as the foot of a label prints them.
        let sums = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(say.leaf())
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 18.px()).height((3.0 * label::ROW + 22.0).px()),
                )),
        );
        let figure = |grove: &mut Grove, y: f32, size: FontSize| {
            Roll::grow(
                grove,
                sums,
                Location::new().xs(
                    right(100.pct() + 1.px()).width(9.letters()),
                    center_y((y + label::ROW / 2.0).px()).height(1.letters()),
                ),
                9,
                "€.−",
                size,
                Some(faces.italic),
                Palette::Ink,
            )
        };
        let small = FontSize::new().xs(18);
        let key = |grove: &mut Grove, y: f32, name: &str| {
            grove.branch(
                sums,
                Text::new(name)
                    .font_size(lichen::caption())
                    .color(Palette::Ink)
                    .intangible()
                    .at(Location::new().xs(
                        left(0.px()).width(content()),
                        center_y((y + label::ROW / 2.0).px()).height(1.letters()),
                    )),
            )
        };
        key(grove, 0.0, "subtotal");
        let subtotal = figure(grove, 0.0, small);
        label::fixed_rule(grove, sums, label::ROW - HAIR, HAIR);
        let discount_key = key(grove, label::ROW, "discount");
        let discount = figure(grove, label::ROW, small);
        label::fixed_rule(grove, sums, 2.0 * label::ROW - HEAVY, HEAVY);
        key(grove, 2.0 * label::ROW + 8.0, "total");
        let total = figure(grove, 2.0 * label::ROW + 8.0, theme::figure());
        let mut checkout = Chip::grow(
            grove,
            col,
            Location::new().xs(
                left(0.px()).width(Chip::width("go to the till")),
                top(anchor().bottom() + 24.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.credit_card,
            "go to the till",
        );
        grove.anchor(checkout.leaf(), sums);
        checkout.arm(grove, Press::Inert);
        // A margin under the till's press, so it is never against the sheet's foot.
        grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(checkout.leaf())
                .at(Location::new().xs(
                    left(0.px()).width(1.px()),
                    top(anchor().bottom()).height(24.px()),
                )),
        );
        let mut bag = Self {
            drawer,
            scroll,
            close,
            summary,
            list,
            rows: Vec::new(),
            going: Vec::new(),
            empty,
            browse,
            code,
            apply,
            say,
            subtotal,
            discount_key,
            discount,
            total,
            checkout,
            courier: Courier::new(0xba9),
            faces: *faces,
            marks: (icons.minus, icons.plus, icons.trash_2),
        };
        bag.sums(grove, &Cart::new(), false);
        bag
    }

    pub fn is_open(&self) -> bool {
        self.drawer.is_open()
    }

    pub fn open(&mut self, grove: &mut Grove) {
        grove.scroll(self.scroll, ScrollTo::start());
        self.drawer.open(grove);
        grove.focus(self.scroll);
    }

    pub fn close(&mut self, grove: &mut Grove) {
        self.drawer.close(grove);
    }

    /// Puts a line the cart grew, or changed, on the bag: a new line at the foot, or line `n`'s
    /// seats and price written again.
    pub fn sync(&mut self, grove: &mut Grove, cart: &Cart, n: usize) {
        if n >= self.rows.len() {
            let row = self.row(grove, cart, n);
            grove.opacity(row.root, 0.0);
            grove.animate(
                row.root,
                Motion::Opacity(1.0),
                Timing::ms(320).after(120).ease(Ease::Decelerate),
            );
            self.rows.push(row);
            grove.animate(
                self.list,
                Motion::Location(list_at(self.rows.len())),
                Timing::ms(320).ease(Ease::Decelerate),
            );
        } else {
            self.write(grove, cart, n);
        }
        grove.visible(self.empty, self.rows.is_empty());
        self.sums(grove, cart, true);
    }

    /// Empties the bag's lines, as a placed order does.
    pub fn empty(&mut self, grove: &mut Grove, cart: &Cart) {
        for row in self.rows.drain(..) {
            grove.prune(row.root);
        }
        grove.at(self.list, list_at(0));
        grove.visible(self.empty, true);
        self.code.clear(grove);
        self.say.clear(grove);
        self.sums(grove, cart, false);
    }

    /// Carries the bag for a frame.
    pub fn frame(
        &mut self,
        grove: &mut Grove,
        pollen: &Pollen,
        cart: &mut Cart,
        budget: &mut u32,
    ) -> Option<Asked> {
        self.going.retain(|&(leaf, done)| {
            let gone = pollen.finished(done);
            if gone {
                grove.prune(leaf);
            }
            !gone
        });
        self.subtotal.frame(grove);
        self.discount.frame(grove);
        self.total.frame(grove);
        for row in &mut self.rows {
            row.art.frame(grove, budget);
        }
        let closed = self.drawer.frame(grove, pollen);
        if !self.drawer.is_open() {
            return None;
        }
        if matches!(closed, Some(Closed::Escape | Closed::Scrim)) {
            return None;
        }
        if self.close.pressed(pollen) {
            self.close(grove);
            return None;
        }
        if self.browse.pressed(pollen) {
            self.close(grove);
            return Some(Asked::Browse);
        }
        // The lines: a seat more, a seat fewer, or the line taken out.
        let mut asked = None;
        let mut gone = None;
        for (n, row) in self.rows.iter().enumerate() {
            if row.plus.pressed(pollen) {
                cart.seat(n, 1);
                asked = Some(n);
            }
            if row.minus.pressed(pollen) {
                cart.seat(n, -1);
                asked = Some(n);
            }
            if row.remove.pressed(pollen) {
                gone = Some(n);
            }
        }
        if let Some(n) = asked {
            self.write(grove, cart, n);
            self.sums(grove, cart, true);
        }
        if let Some(n) = gone {
            cart.remove(n);
            self.take_out(grove, n);
            if cart.promo.is_none() {
                self.code.clear(grove);
                self.say.clear(grove);
            }
            self.sums(grove, cart, true);
        }
        // A code, asked of the till.
        self.code.frame(grove, pollen);
        if (self.apply.pressed(pollen) || pollen.submitted(self.code.input()))
            && !self.courier.busy()
        {
            let code = self.code.value(grove);
            match api::promo(&code) {
                Err(Refused::Empty) => self.say.tell(grove, &Refused::Empty.words(), Voice::Warn),
                answer => {
                    self.courier.send(grove, TILL_MS, answer);
                    self.apply.arm(grove, Press::Inert);
                    self.say.tell(grove, "asking the till…", Voice::Quiet);
                }
            }
        }
        for (_, answer) in self.courier.frame(pollen) {
            self.apply.arm(grove, Press::Rest);
            match answer {
                Ok(promo) => {
                    cart.promo = Some(promo);
                    let words = match cart.is_empty() {
                        true => format!(
                            "{} takes {}% off whatever goes in the bag",
                            promo.code, promo.percent
                        ),
                        false => format!("{} takes {}% off the bag", promo.code, promo.percent),
                    };
                    self.say.tell(grove, &words, Voice::Hint);
                    self.sums(grove, cart, true);
                }
                Err(refused) => {
                    self.say.tell(grove, &refused.words(), Voice::Warn);
                }
            }
        }
        if self.checkout.pressed(pollen) && !cart.is_empty() {
            return Some(Asked::Checkout);
        }
        if asked.is_some() || gone.is_some() {
            return Some(Asked::Changed(cart.count()));
        }
        None
    }

    /// Takes row `n` out: it slides away, and every row under it moves up into its place.
    fn take_out(&mut self, grove: &mut Grove, n: usize) {
        let row = self.rows.remove(n);
        let timing = Timing::ms(240).ease(Ease::Accelerate);
        grove.animate(row.root, Motion::Opacity(0.0), timing);
        grove.animate(row.root, Motion::Shift(Position::new(40.0, 0.0)), timing);
        self.going.push((row.root, grove.timer(Timing::ms(260))));
        for (k, row) in self.rows.iter().enumerate().skip(n) {
            grove.animate(
                row.root,
                Motion::Location(line_at(k)),
                Timing::ms(380)
                    .after(160 + (k - n) as u64 * 40)
                    .ease(Ease::Emphasis),
            );
        }
        grove.animate(
            self.list,
            Motion::Location(list_at(self.rows.len())),
            Timing::ms(380).after(160).ease(Ease::Emphasis),
        );
        grove.visible(self.empty, self.rows.is_empty());
        if self.rows.is_empty() {
            grove.opacity(self.empty, 0.0);
            grove.animate(
                self.empty,
                Motion::Opacity(1.0),
                Timing::ms(300).after(300).ease(Ease::Decelerate),
            );
        }
    }

    /// The totals, the summary under the title, and whether the till can be gone to.
    fn sums(&mut self, grove: &mut Grove, cart: &Cart, roll: bool) {
        let figures = [
            (&mut self.subtotal, money(cart.subtotal())),
            (
                &mut self.discount,
                if cart.discount() > 0 {
                    format!("−{}", money(cart.discount()))
                } else {
                    money(0)
                },
            ),
            (&mut self.total, money(cart.total())),
        ];
        for (figure, says) in figures {
            match roll {
                true => figure.set(grove, &says),
                false => figure.place(grove, &says),
            }
        }
        grove.text(
            self.discount_key,
            match cart.promo {
                Some(promo) => format!("discount · {} −{}%", promo.code, promo.percent),
                None => "discount".to_string(),
            },
        );
        let seats = cart.count();
        let goods = cart.lines.len();
        grove.text(
            self.summary,
            match goods {
                0 => "nothing in it · net 0 B".to_string(),
                _ => format!(
                    "{goods} good{} · {seats} seat{} · net {}",
                    if goods == 1 { "" } else { "s" },
                    if seats == 1 { "" } else { "s" },
                    weight(cart.bytes())
                ),
            },
        );
        self.checkout.arm(grove, Press::armed_if(!cart.is_empty()));
    }

    /// Writes line `n`'s seats and price.
    fn write(&mut self, grove: &mut Grove, cart: &Cart, n: usize) {
        if let (Some(row), Some(line)) = (self.rows.get_mut(n), cart.lines.get(n)) {
            written(grove, row, line);
        }
    }

    /// Grows the row for line `n`.
    fn row(&mut self, grove: &mut Grove, cart: &Cart, n: usize) -> Row {
        let m = measure();
        let line = cart.lines[n];
        let root = grove.branch(
            self.list,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(line_at(n)),
        );
        let framed = grove.branch(
            root,
            Panel::new()
                .color(Palette::Raised)
                .outline(HAIR, Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(THUMB.px()),
                    top(14.px()).height(THUMB.px()),
                )),
        );
        let mut art = Art::grow(
            grove,
            framed,
            Location::new().xs(
                left(4.px()).right(100.pct() - 4.px()),
                top(4.px()).bottom(100.pct() - 4.px()),
            ),
            line.good,
            Scale::Mini,
            &self.faces,
        );
        grove.elevate(art.leaf(), Elevation::up(1));
        art.fit(grove, grove.viewport().width);
        let name = grove.branch(
            root,
            Text::new("")
                .font(self.faces.roman)
                .font_size(FontSize::new().xs(18))
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(TEXT_X.px()).right(100.pct()),
                    top(10.px()).height(1.letters()),
                )),
        );
        let meta = grove.branch(
            root,
            Text::new("")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(
                    left(TEXT_X.px()).right(100.pct()),
                    top(36.px()).height(1.letters()),
                )),
        );
        let y = 58.0;
        let step = |grove: &mut Grove, x: f32, mark| {
            Chip::mark(
                grove,
                root,
                Location::new().xs(
                    left(x.px()).width(Chip::square()),
                    top(y.px()).height(m.height.px()),
                ),
                Elevation::up(1),
                mark,
            )
        };
        let minus = step(grove, TEXT_X, self.marks.0);
        let seats_leaf = grove.branch(
            root,
            Text::new("")
                .font_size(lichen::caption())
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    center_x((TEXT_X + m.height + 34.0).px()).width(content()),
                    center_y((y + m.height / 2.0).px()).height(1.letters()),
                )),
        );
        let plus = step(grove, TEXT_X + m.height + 68.0, self.marks.1);
        let remove = step(grove, TEXT_X + 2.0 * m.height + 68.0 + m.gap, self.marks.2);
        let price = grove.branch(
            root,
            Text::new("")
                .font(self.faces.italic)
                .font_size(FontSize::new().xs(18))
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    right(100.pct()).width(content()),
                    center_y((y + m.height / 2.0).px()).height(1.letters()),
                )),
        );
        label::fixed_rule(grove, root, LINE - HAIR, HAIR);
        let mut row = Row {
            root,
            art,
            name,
            meta,
            seats: seats_leaf,
            price,
            minus,
            plus,
            remove,
        };
        written(grove, &mut row, &line);
        row
    }
}

/// Writes what a row says from its line: the name, what it is, its seats and its price.
fn written(grove: &mut Grove, row: &mut Row, line: &crate::cart::Line) {
    grove.text(row.seats, seats(line.seats));
    grove.text(row.price, money(line.cents()));
    grove.text(
        row.meta,
        format!(
            "{} · {} · {}",
            line.good().aisle.single(),
            line.format(),
            line.licence.name()
        ),
    );
    grove.text(row.name, crate::grid::clip(line.good().name, 26));
    row.minus.arm(grove, Press::rest_if(line.seats > 1));
}

/// "1 seat", "3 seats".
fn seats(n: u32) -> String {
    format!("{n} seat{}", if n == 1 { "" } else { "s" })
}

/// Where line `n` stands in the list.
fn line_at(n: usize) -> Location {
    Location::new().xs(
        left(0.px()).right(100.pct()),
        top((n as f32 * LINE).px()).height(LINE.px()),
    )
}

/// The list, holding `n` lines -- or the word for none.
fn list_at(n: usize) -> Location {
    let height = if n == 0 { EMPTY } else { n as f32 * LINE };
    Location::new().xs(
        left(0.px()).right(100.pct()),
        top(anchor().bottom()).height(height.px()),
    )
}

/// A rule under `col`, `gap` below `after`, always there.
fn rule(grove: &mut Grove, col: Leaf, after: Leaf, gap: f32, weight: f32) -> Leaf {
    grove.branch(
        col,
        Panel::new()
            .color(Palette::Ink)
            .intangible()
            .anchored(after)
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + gap.px()).height(weight.px()),
            )),
    )
}
