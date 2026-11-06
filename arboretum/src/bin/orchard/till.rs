//! The till: checkout in four steps, the order placed through the courier, and the receipt that
//! comes back -- stamped, sealed in wax, with a file to take away for every good on it.
//!
//! The steps are a chain ([`lichen::gate`]): each one's press is armed when it is the next to do,
//! at rest once done, and out of reach before then. Placing the order is a [`Confirm`], and what
//! the till does with it is staged -- authorising, issuing licences, packaging -- with an address at
//! `fail.test` turned down at the first, and a way on from there.
//!
//! An overlay under the bar, as the detail is: the bag stays in reach over it, and the library
//! comes back to the latest receipt.

use foliage::{
    Axes, Boxed, Ease, Elevation, Field as Mark, FontSize, Grove, Grow, Leaf, Length, Location,
    Motion, Palette, Panel, Place, Pollen, Position, ScrollTo, Source, Stem, Text, Timing,
    anchor, bottom, center_x, center_y, content, left, right, top,
};
use lichen::{Chip, Confirm, Field, Mosaic, Overlay, Pick, Ping, Press, Say, Scatter, Silhouette, Stamp, Step, Voice, gate, measure};

use arboretum::sprout::{Courier, Print, Printed, Progress};

use crate::api::{self, Bought, Declined, Order, Receipt};
use crate::cart::Cart;
use crate::catalog::{money, weight};
use crate::grid::clip;
use crate::icons::Icons;
use crate::label::{self, ROW};
use crate::theme::{self, BAR_NARROW, BAR_WIDE, Faces, HAIR, HEAVY, column};

const STEPS: [&str; 4] = ["email", "holder", "payment", "review"];
const ASKS: [(&str, &str); 4] = [
    (
        "Where should the receipt go?",
        "the address the receipt and the files go to. nothing is sent: this is a demo",
    ),
    (
        "Whose name goes on the licences?",
        "every file is printed with the name of whoever holds its licence",
    ),
    (
        "How will it be paid?",
        "a demo till: nothing is charged, and no card is asked for",
    ),
    (
        "Check the label before it prints.",
        "once it is placed, the order is authorised, licensed and packaged",
    ),
];
/// How tall each step stands: the review, with its label and the press under it, the tallest.
const PANELS: [f32; 4] = [236.0, 236.0, 236.0, 380.0];
/// How long each stage of the till takes.
const AUTHORISE_MS: u64 = 1300;
const ISSUE_MS: u64 = 950;
const PACK_MS: u64 = 850;
/// How tall a good stands on the docket, and on the receipt.
const DOCKET_LINE: f32 = 46.0;
const TAKEN: f32 = 100.0;

/// What the till asks of the shop.
#[derive(Clone, Debug, PartialEq)]
pub enum Asked {
    /// Back to the bag.
    Bag,
    /// The order went through: here is its receipt.
    Paid(Receipt),
}

/// What comes back from the till, stage by stage.
enum Stage {
    Authorised(Result<Receipt, Declined>),
    Issued(Receipt),
    Packaged(Receipt),
}

/// The docket beside the steps: what is being bought, and what it comes to.
struct Docket {
    panel: Leaf,
    inner: Leaf,
    lines: Vec<Leaf>,
    sums: Option<Leaf>,
    foot: Leaf,
}

/// One line of the receipt, and its press to take the file away.
struct Taken {
    root: Leaf,
    download: Chip,
    bought: Bought,
}

/// The receipt: the slip, and the seal beside it.
struct Slip {
    pane: Leaf,
    number: Leaf,
    stamp: Stamp,
    facts: Vec<Leaf>,
    lines: Leaf,
    taken: Vec<Taken>,
    sums: Vec<Leaf>,
    seal_box: Leaf,
    seal: Option<(Mosaic, Leaf)>,
    initials: Leaf,
    sealed: Vec<Leaf>,
    shop: Chip,
    ping: Ping,
    say: Say,
    printed: Vec<Printed>,
}

pub struct Till {
    overlay: Overlay,
    body: Leaf,
    back: Chip,
    close: Chip,
    title: Leaf,
    checkout: Leaf,
    steps_col: Leaf,
    steps: Vec<Chip>,
    marks: [Mark; 4],
    check: Mark,
    done: [bool; 4],
    at: usize,
    panels: Vec<Leaf>,
    email: Field,
    holder: Field,
    method: Pick,
    nexts: Vec<Chip>,
    says: Vec<Say>,
    review: Vec<Leaf>,
    review_printed: Vec<Printed>,
    reprint: bool,
    confirm: Confirm,
    retry: Chip,
    change: Chip,
    progress: Progress,
    docket: Docket,
    courier: Courier<Stage>,
    placing: bool,
    placed: u32,
    slip: Slip,
    receipt: Option<Receipt>,
    /// Whether the till shows a receipt rather than the checkout.
    on_receipt: bool,
    faces: Faces,
    download: Mark,
}

impl Till {
    pub fn grow(grove: &mut Grove, shop: Leaf, page: Leaf, faces: &Faces, icons: &Icons) -> Self {
        let m = measure();
        let mut overlay = Overlay::grow(
            grove,
            shop,
            page,
            Location::new()
                .xs(
                    left(0.px()).right(100.pct()),
                    top(BAR_NARROW.px()).bottom(100.pct()),
                )
                .lg(
                    left(0.px()).right(100.pct()),
                    top(BAR_WIDE.px()).bottom(100.pct()),
                ),
            Elevation::up(32),
        );
        let body = grove.branch(
            overlay.sheet(),
            Stem::new()
                .interactive()
                .scrolls(Axes::Vertical)
                .elevate(Elevation::up(1))
                .at(Location::new()),
        );
        // The body holds focus while it is up, so the keys that scroll it reach it.
        overlay.hold(body);
        let col = grove.branch(
            body,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(column(|| top(22.px()).height(10.px()))),
        );
        let back = Chip::grow(
            grove,
            col,
            Location::new().xs(
                left(0.px()).width(Chip::width("the bag")),
                top(0.px()).height(m.height.px()),
            ),
            Elevation::up(1),
            icons.arrow_left,
            "the bag",
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
        let title = grove.branch(
            col,
            Text::new("The till")
                .font(faces.roman)
                .font_size(theme::heading())
                .color(Palette::Ink)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top((m.height + 28.0).px()).height(1.letters()),
                )),
        );
        let rule = grove.branch(
            col,
            Panel::new()
                .color(Palette::Ink)
                .intangible()
                .anchored(title)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 12.px()).height(HEAVY.px()),
                )),
        );
        // The checkout: the steps, and beside them (or under them) the docket.
        let checkout = grove.branch(
            col,
            Stem::new()
                .intangible()
                .anchored(rule)
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 24.px()).height(10.px()),
                )),
        );
        let rail = 2.0 * m.height + m.gap;
        let steps_col = grove.branch(
            checkout,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(steps_at(0)),
        );
        let marks = [icons.mail, icons.user, icons.credit_card, icons.tag];
        let steps: Vec<Chip> = STEPS
            .iter()
            .enumerate()
            .map(|(n, name)| {
                Chip::grow(
                    grove,
                    steps_col,
                    step_at(n),
                    Elevation::up(1),
                    marks[n],
                    name,
                )
            })
            .collect();
        let panel_at = Location::new()
            .xs(
                left(0.px()).right(100.pct()).at_most(480.px()),
                top((rail + 24.0).px()).height(PANELS[3].px()),
            )
            .md(
                left(0.px()).right(100.pct()).at_most(480.px()),
                top((m.height + 24.0).px()).height(PANELS[3].px()),
            );
        // Each step: what it asks, why, and the form under that.
        let mut panels = Vec::new();
        let mut whys = Vec::new();
        let mut forms = Vec::new();
        for (n, (ask, why)) in ASKS.iter().enumerate() {
            let panel = grove.branch(
                steps_col,
                Stem::new()
                    .intangible()
                    .visible(n == 0)
                    .font_size(lichen::caption())
                    .at(panel_at.clone()),
            );
            let heading = grove.branch(
                panel,
                Text::new(*ask)
                    .font(faces.roman)
                    .font_size(FontSize::new().xs(21).md(24))
                    .color(Palette::Ink)
                    .intangible()
                    .at(Location::new()
                        .xs(left(0.px()).right(100.pct()), top(0.px()).height(content()))),
            );
            let why = label::words(
                grove,
                panel,
                heading,
                8.0,
                *why,
                None,
                lichen::caption(),
                Palette::Muted,
            );
            let form = grove.branch(
                panel,
                Stem::new()
                    .intangible()
                    .anchored(why)
                    .font_size(lichen::caption())
                    .at(Location::new().xs(
                        left(0.px()).right(100.pct()),
                        top(anchor().bottom() + 20.px()).height(10.px()),
                    )),
            );
            panels.push(panel);
            whys.push(why);
            if n < 3 {
                forms.push(form);
            }
        }
        let field_at = || {
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(0.px()).height(m.height.px()),
            )
        };
        let email = Field::grow(grove, forms[0], field_at(), 8.0, "email", false);
        email.hint(grove, "ada@fernwood.studio");
        let holder = Field::grow(grove, forms[1], field_at(), 8.0, "holder", false);
        holder.hint(grove, "Ada Fernwood");
        let method = Pick::grow(
            grove,
            forms[2],
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(0.px()).height(Pick::height(1).px()),
            ),
            8.0,
            "method",
            api::METHODS,
            1,
        );
        let mut nexts = Vec::new();
        let mut says = Vec::new();
        for (n, &form) in forms.iter().enumerate() {
            let below = if n == 2 { Pick::height(1) } else { m.height } + 14.0;
            let mut next = Chip::grow(
                grove,
                form,
                Location::new().xs(
                    left(0.px()).width(Chip::width("continue")),
                    top(below.px()).height(m.height.px()),
                ),
                Elevation::up(1),
                icons.arrow_right,
                "continue",
            );
            next.arm(grove, Press::Armed);
            nexts.push(next);
            says.push(Say::grow(
                grove,
                form,
                Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top((below + m.height + 12.0).px()).height(content()),
                ),
                None,
            ));
        }
        // The review: the label as it will print, then the press that places it.
        let (rows, review, review_printed) = label::rows(
            grove,
            panels[3],
            whys[3],
            20.0,
            &["to", "holder", "paid by", "goods", "total"],
            84.0,
        );
        let press = grove.branch(
            panels[3],
            Stem::new()
                .intangible()
                .anchored(rows)
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(anchor().bottom() + 20.px()).height(10.px()),
                )),
        );
        let row_at = |x: Length, w: Length| {
            Location::new().xs(left(x).width(w), top(0.px()).height(m.height.px()))
        };
        let confirm = Confirm::grow(
            grove,
            press,
            row_at(0.px(), Chip::width("place the order")),
            Location::new().xs(
                right(100.pct()).width(Chip::width("pay €0000.00")),
                top(0.px()).height(m.height.px()),
            ),
            (icons.package, "place the order"),
            (icons.x, "not yet"),
            (icons.lock, "pay"),
        );
        let retry = Chip::grow(
            grove,
            press,
            row_at(0.px(), Chip::width("try again")),
            Elevation::up(3),
            icons.send,
            "try again",
        );
        let change = Chip::grow(
            grove,
            press,
            row_at(
                Chip::width("try again") + m.gap.px(),
                Chip::width("change the address"),
            ),
            Elevation::up(3),
            icons.mail,
            "change the address",
        );
        grove.visible(retry.leaf(), false);
        grove.visible(change.leaf(), false);
        let progress = Progress::grow(
            grove,
            press,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top((m.height + 16.0).px()).height(4.px()),
            ),
            3,
            Palette::Muted,
            Palette::Accent,
        );
        grove.opacity(progress.leaf(), 0.0);
        says.push(Say::grow(
            grove,
            press,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top((m.height + 30.0).px()).height(content()),
            ),
            None,
        ));
        let docket = docket(grove, checkout, steps_col, faces);
        let slip = slip(grove, col, rule, faces, icons);
        let mut till = Self {
            overlay,
            body,
            back,
            close,
            title,
            checkout,
            steps_col,
            steps,
            marks,
            check: icons.check,
            done: [false; 4],
            at: 0,
            panels,
            email,
            holder,
            method,
            nexts,
            says,
            review,
            review_printed,
            reprint: false,
            confirm,
            retry,
            change,
            progress,
            docket,
            courier: Courier::new(0x7111),
            placing: false,
            placed: 0,
            slip,
            receipt: None,
            on_receipt: false,
            faces: *faces,
            download: icons.download,
        };
        till.wear(grove);
        till
    }

    pub fn is_open(&self) -> bool {
        self.overlay.is_open()
    }

    /// Opens the till on the checkout for `cart`.
    pub fn checkout(&mut self, grove: &mut Grove, cart: &Cart) {
        self.show(grove);
        self.on_receipt = false;
        grove.text(self.title, "The till");
        grove.visible(self.checkout, true);
        grove.visible(self.slip.pane, false);
        grove.visible(self.back.leaf(), true);
        self.courier.cancel_all(grove);
        self.placing = false;
        self.progress.set(grove, 0.0, 0);
        grove.opacity(self.progress.leaf(), 0.0);
        self.says[3].clear(grove);
        self.declined(grove, false);
        self.write_docket(grove, cart);
        self.write_review(grove, cart);
        // Back to the first step not done.
        let first = self.done.iter().position(|d| !d).unwrap_or(3);
        self.go(grove, first);
    }

    /// Writes the checkout over for `cart`, changed in the bag while the till is up.
    pub fn refresh(&mut self, grove: &mut Grove, cart: &Cart) {
        if !self.overlay.is_open() || self.on_receipt {
            return;
        }
        if cart.is_empty() {
            self.close(grove);
            return;
        }
        self.write_docket(grove, cart);
        self.write_review(grove, cart);
    }

    /// Opens the till on a receipt: the one just paid, or one from the library.
    pub fn receipt(&mut self, grove: &mut Grove, receipt: &Receipt, print: &mut Print) {
        self.show(grove);
        self.on_receipt = true;
        grove.text(self.title, "Receipt");
        grove.visible(self.checkout, false);
        grove.visible(self.slip.pane, true);
        grove.visible(self.back.leaf(), false);
        self.write_receipt(grove, receipt, print);
    }

    fn show(&mut self, grove: &mut Grove) {
        self.overlay.open(grove);
        grove.scroll(self.body, ScrollTo::start());
    }

    /// Takes the till down. Safe to repeat.
    pub fn close(&mut self, grove: &mut Grove) {
        self.overlay.close(grove);
        // An order half placed is not placed: the till lets it go.
        if self.placing {
            self.courier.cancel_all(grove);
            self.placing = false;
        }
        self.confirm.disarm(grove);
    }

    /// Carries the till for a frame.
    pub fn frame(
        &mut self,
        grove: &mut Grove,
        pollen: &Pollen,
        cart: &Cart,
        print: &mut Print,
    ) -> Option<Asked> {
        let escaped = self.overlay.frame(grove, pollen).is_some();
        if let Some((seal, _)) = &mut self.slip.seal {
            seal.frame(grove, pollen);
        }
        if !escaped && !self.overlay.is_open() {
            return None;
        }
        if escaped || self.close.pressed(pollen) || self.slip.shop.pressed(pollen) {
            self.close(grove);
            return None;
        }
        if self.back.pressed(pollen) {
            self.close(grove);
            return Some(Asked::Bag);
        }
        if self.reprint {
            self.reprint = false;
            print.unprint(grove, &self.review_printed);
            print.run(grove, &self.review_printed, 80, 30);
        }
        self.slip.ping.frame(grove, pollen);
        self.take(grove, pollen);
        // The chain: a step's press goes back to it, once it is in reach.
        if !self.placing
            && let Some(n) = self.steps.iter().position(|s| s.pressed(pollen))
        {
            self.go(grove, n);
        }
        // Words changed on a step done open it again, and close every step after it.
        if self.email.frame(grove, pollen) {
            self.reopen(grove, 0);
        }
        if self.holder.frame(grove, pollen) {
            self.reopen(grove, 1);
        }
        if self.method.frame(grove, pollen) {
            self.write_review(grove, cart);
        }
        let submitted =
            pollen.submitted(self.email.input()) || pollen.submitted(self.holder.input());
        let next = self
            .nexts
            .iter()
            .position(|c| c.pressed(pollen))
            .or(submitted.then_some(self.at));
        if let Some(n) = next
            && n < 3
        {
            self.advance(grove, n, cart);
        }
        // Placing the order.
        match self.confirm.frame(grove, pollen) {
            Some(Step::Confirmed) => self.place(grove, cart),
            Some(Step::Armed) => {
                grove.text(
                    self.confirm.commit().label(),
                    format!("pay {}", money(cart.total())),
                );
                self.says[3].tell(
                    grove,
                    "ready: pay places it, not yet puts it back",
                    Voice::Quiet,
                );
            }
            Some(Step::Kept) => self.says[3].clear(grove),
            None => {}
        }
        if self.retry.pressed(pollen) {
            self.declined(grove, false);
            self.place(grove, cart);
        }
        if self.change.pressed(pollen) {
            self.declined(grove, false);
            self.progress.set(grove, 0.0, 0);
            grove.opacity(self.progress.leaf(), 0.0);
            self.says[3].clear(grove);
            self.done[0] = false;
            self.go(grove, 0);
        }
        for (_, stage) in self.courier.frame(pollen) {
            match stage {
                Stage::Authorised(Ok(receipt)) => {
                    self.progress.set(grove, 0.45, 500);
                    self.says[3].tell(grove, "authorised · issuing licences…", Voice::Quiet);
                    self.courier.send(grove, ISSUE_MS, Stage::Issued(receipt));
                }
                Stage::Authorised(Err(declined)) => {
                    self.placing = false;
                    self.progress.set(grove, 0.33, 200);
                    self.progress.tint(grove, Palette::Danger, 200);
                    self.says[3].tell(grove, declined.words(), Voice::Warn);
                    self.declined(grove, true);
                    self.wear(grove);
                }
                Stage::Issued(receipt) => {
                    self.progress.set(grove, 0.78, 500);
                    self.says[3].tell(grove, "licences issued · packaging bits…", Voice::Quiet);
                    self.courier.send(grove, PACK_MS, Stage::Packaged(receipt));
                }
                Stage::Packaged(receipt) => {
                    self.progress.set(grove, 1.0, 300);
                    self.placing = false;
                    self.placed += 1;
                    // The address, the name and the way of paying stand for the next order.
                    self.done = [true, true, true, false];
                    self.receipt(grove, &receipt, print);
                    return Some(Asked::Paid(receipt));
                }
            }
        }
        None
    }

    /// A file off the receipt: downloaded where that can be done, and copied either way.
    fn take(&mut self, grove: &mut Grove, pollen: &Pollen) {
        let Some(receipt) = &self.receipt else {
            return;
        };
        let Some(taken) = self.slip.taken.iter().find(|t| t.download.pressed(pollen)) else {
            return;
        };
        let (name, mime, text) = api::file(&taken.bought, receipt);
        grove.download(api::data_url(mime, &text));
        grove.copy(text);
        self.slip.ping.fire(grove);
        // Only a browser saves a file; anywhere else it is on the clipboard and nowhere more.
        let words = match cfg!(target_arch = "wasm32") {
            true => format!("{name} · downloading, and on the clipboard"),
            false => format!("{name} · on the clipboard"),
        };
        self.slip.say.tell(grove, &words, Voice::Hint);
    }

    /// Checks step `n` and, if it holds, moves on to the next.
    fn advance(&mut self, grove: &mut Grove, n: usize, cart: &Cart) {
        let checked = match n {
            0 => api::email(&self.email.value(grove)),
            1 => api::holder(&self.holder.value(grove)),
            _ => Ok(()),
        };
        match checked {
            Ok(()) => {
                self.says[n].clear(grove);
                self.done[n] = true;
                self.write_review(grove, cart);
                let next = self.done.iter().position(|d| !d).unwrap_or(3);
                self.go(grove, next);
            }
            Err(why) => {
                self.says[n].tell(grove, why, Voice::Warn);
                match n {
                    0 => self.email.focus(grove),
                    _ => self.holder.focus(grove),
                }
            }
        }
    }

    /// Opens step `n` again, its words having changed, and closes every step after it.
    fn reopen(&mut self, grove: &mut Grove, n: usize) {
        if self.done[n] {
            self.done[n] = false;
            self.wear(grove);
            self.confirm.open(grove, false);
        }
    }

    /// Shows step `n`, if it is in reach: done, or the next to do.
    fn go(&mut self, grove: &mut Grove, n: usize) {
        let reachable = n == 0 || self.done[..n].iter().all(|d| *d);
        if !reachable {
            return;
        }
        let moved = self.at != n || !self.overlay.is_open();
        self.at = n;
        for (k, &panel) in self.panels.iter().enumerate() {
            grove.visible(panel, k == n);
        }
        let panel = self.panels[n];
        // The docket under the steps follows the step showing up and down.
        grove.animate(
            self.steps_col,
            Motion::Location(steps_at(n)),
            Timing::ms(300).ease(Ease::Decelerate),
        );
        grove.opacity(panel, 0.0);
        grove.shift(panel, Position::new(0.0, 8.0));
        let timing = Timing::ms(300).ease(Ease::Decelerate);
        grove.animate(panel, Motion::Opacity(1.0), timing);
        grove.animate(panel, Motion::Shift(Position::default()), timing);
        self.confirm.open(grove, n == 3 && !self.placing);
        if n == 3 && moved {
            self.reprint = true;
        }
        match n {
            0 => self.email.focus(grove),
            1 => self.holder.focus(grove),
            _ => grove.focus(self.body),
        }
        self.wear(grove);
    }

    /// Dresses the chain: done at rest with a tick, the step showing chosen, the next armed, the
    /// rest out of reach.
    fn wear(&mut self, grove: &mut Grove) {
        let wears: Vec<Press> = gate(&self.done).collect();
        for (n, (step, press)) in self.steps.iter_mut().zip(wears).enumerate() {
            let press = if n == self.at && press != Press::Inert {
                Press::Chosen
            } else {
                press
            };
            step.arm(grove, press);
            step.remark(
                grove,
                if self.done[n] {
                    self.check
                } else {
                    self.marks[n]
                },
            );
        }
    }

    /// Places the order: worked out by the till at once, and answered stage by stage.
    fn place(&mut self, grove: &mut Grove, cart: &Cart) {
        if self.placing || cart.is_empty() {
            return;
        }
        let order = Order {
            email: self.email.value(grove),
            holder: self.holder.value(grove),
            method: self.method.index(),
            cart: cart.clone(),
        };
        let answer = api::place(&order, self.placed);
        self.placing = true;
        self.confirm.open(grove, false);
        grove.opacity(self.progress.leaf(), 1.0);
        self.progress.tint(grove, Palette::Accent, 0);
        self.progress.set(grove, 0.0, 0);
        self.progress.set(grove, 0.15, 400);
        self.says[3].tell(grove, "authorising…", Voice::Quiet);
        self.courier
            .send(grove, AUTHORISE_MS, Stage::Authorised(answer));
    }

    /// Shows the way on from a refusal, or puts it away.
    fn declined(&mut self, grove: &mut Grove, declined: bool) {
        grove.visible(self.retry.leaf(), declined);
        grove.visible(self.change.leaf(), declined);
        grove.visible(self.confirm.ask().leaf(), !declined);
        if !declined {
            self.confirm.open(grove, self.at == 3 && !self.placing);
        }
    }

    /// What the review step says the label will.
    fn write_review(&mut self, grove: &mut Grove, cart: &Cart) {
        let or_dash = |s: String| {
            if s.trim().is_empty() {
                "—".to_string()
            } else {
                s.trim().to_string()
            }
        };
        let goods = cart.lines.len();
        let values = [
            or_dash(self.email.value(grove)),
            or_dash(self.holder.value(grove)),
            api::METHODS[self.method.index()].to_string(),
            format!(
                "{goods} good{} · net {}",
                if goods == 1 { "" } else { "s" },
                weight(cart.bytes())
            ),
            money(cart.total()),
        ];
        for (leaf, value) in self.review.iter().zip(values) {
            grove.text(*leaf, value);
        }
    }

    /// Writes the docket for `cart`: a line a good, then the sums.
    fn write_docket(&mut self, grove: &mut Grove, cart: &Cart) {
        let d = &mut self.docket;
        for leaf in d.lines.drain(..).chain(d.sums.take()) {
            grove.prune(leaf);
        }
        let mut y = 40.0;
        for line in &cart.lines {
            let row = grove.branch(
                d.inner,
                Stem::new()
                    .intangible()
                    .font_size(lichen::caption())
                    .at(Location::new().xs(
                        left(0.px()).right(100.pct()),
                        top(y.px()).height(DOCKET_LINE.px()),
                    )),
            );
            grove.branch(
                row,
                Text::new(clip(line.good().name, 24))
                    .font(self.faces.roman)
                    .font_size(FontSize::new().xs(17))
                    .color(Palette::Ink)
                    .intangible()
                    .at(Location::new().xs(
                        left(0.px()).right(100.pct() - 84.px()),
                        top(5.px()).height(1.letters()),
                    )),
            );
            grove.branch(
                row,
                Text::new(format!(
                    "{} · {} seat{}",
                    line.licence.name(),
                    line.seats,
                    if line.seats == 1 { "" } else { "s" }
                ))
                .font_size(theme::small())
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct() - 84.px()),
                    top(26.px()).height(1.letters()),
                )),
            );
            grove.branch(
                row,
                Text::new(money(line.cents()))
                    .font(self.faces.italic)
                    .font_size(FontSize::new().xs(17))
                    .color(Palette::Ink)
                    .intangible()
                    .at(Location::new().xs(
                        right(100.pct()).width(content()),
                        center_y((DOCKET_LINE / 2.0).px()).height(1.letters()),
                    )),
            );
            label::fixed_rule(grove, row, DOCKET_LINE - HAIR, HAIR);
            d.lines.push(row);
            y += DOCKET_LINE;
        }
        let sums = grove.branch(
            d.inner,
            Stem::new()
                .intangible()
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(y.px()).height((3.0 * ROW + 10.0).px()),
                )),
        );
        let discount = match cart.discount() {
            0 => money(0),
            off => format!("−{}", money(off)),
        };
        let rows = [
            ("subtotal", money(cart.subtotal()), FontSize::new().xs(16)),
            ("discount", discount, FontSize::new().xs(16)),
            ("total", money(cart.total()), FontSize::new().xs(22)),
        ];
        for (n, (name, value, size)) in rows.into_iter().enumerate() {
            let at = n as f32 * ROW + if n == 2 { 10.0 } else { 0.0 };
            let (_, v) = label::sum_row(
                grove,
                sums,
                at,
                name,
                size,
                Some(self.faces.italic),
                Palette::Ink,
            );
            grove.text(v, value);
        }
        label::fixed_rule(grove, sums, ROW - HAIR, HAIR);
        label::fixed_rule(grove, sums, 2.0 * ROW, HEAVY);
        d.sums = Some(sums);
        grove.text(
            d.foot,
            format!("net {} · yours the moment it is paid", weight(cart.bytes())),
        );
        grove.at(d.panel, docket_at(cart.lines.len()));
    }

    /// Writes the receipt, prints it in, stamps it and seals it.
    fn write_receipt(&mut self, grove: &mut Grove, receipt: &Receipt, print: &mut Print) {
        self.receipt = Some(receipt.clone());
        let faces = self.faces;
        let download = self.download;
        let slip = &mut self.slip;
        print.unprint(grove, &slip.printed);
        print.retype(
            grove,
            slip.number,
            &format!("order {}", receipt.number),
            160,
        );
        let facts = [
            receipt.holder.clone(),
            receipt.email.clone(),
            receipt.method.to_string(),
            api::summary(receipt),
        ];
        for (leaf, value) in slip.facts.iter().zip(facts) {
            grove.text(*leaf, value);
        }
        let sealed = [
            receipt.holder.clone(),
            weight(receipt.bytes),
            "nothing · a demo till".to_string(),
        ];
        for (leaf, value) in slip.sealed.iter().zip(sealed) {
            grove.text(*leaf, value);
        }
        print.run(grove, &slip.printed, 260, 40);
        // A line a good, each with its key and its file.
        for taken in slip.taken.drain(..) {
            grove.prune(taken.root);
        }
        let m = measure();
        for (n, bought) in receipt.bought.iter().enumerate() {
            let root = grove.branch(
                slip.lines,
                Stem::new()
                    .intangible()
                    .font_size(lichen::caption())
                    .opacity(0.0)
                    .at(Location::new().xs(
                        left(0.px()).right(100.pct()),
                        top((n as f32 * TAKEN).px()).height(TAKEN.px()),
                    )),
            );
            grove.animate(
                root,
                Motion::Opacity(1.0),
                Timing::ms(320)
                    .after(500 + n as u64 * 90)
                    .ease(Ease::Decelerate),
            );
            let line = bought.line;
            grove.branch(
                root,
                Text::new(clip(line.good().name, 26))
                    .font(faces.roman)
                    .font_size(FontSize::new().xs(18))
                    .color(Palette::Ink)
                    .intangible()
                    .at(Location::new().xs(
                        left(0.px()).right(100.pct() - 90.px()),
                        top(10.px()).height(1.letters()),
                    )),
            );
            grove.branch(
                root,
                Text::new(money(line.cents()))
                    .font(faces.italic)
                    .font_size(FontSize::new().xs(18))
                    .color(Palette::Ink)
                    .intangible()
                    .at(Location::new().xs(
                        right(100.pct()).width(content()),
                        top(10.px()).height(1.letters()),
                    )),
            );
            grove.branch(
                root,
                Text::new(format!(
                    "{} · {} · {} seat{}",
                    line.format(),
                    line.licence.name(),
                    line.seats,
                    if line.seats == 1 { "" } else { "s" }
                ))
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top(36.px()).height(1.letters()),
                )),
            );
            grove.branch(
                root,
                Text::new(format!("key {}", bought.key))
                    .font_size(lichen::caption())
                    .color(Palette::Ink)
                    .intangible()
                    .at(Location::new().xs(
                        left(0.px()).right(100.pct() - 96.px()),
                        center_y((58.0 + m.height / 2.0).px()).height(1.letters()),
                    )),
            );
            let chip = Chip::grow(
                grove,
                root,
                Location::new().xs(
                    right(100.pct()).width(Chip::width("file")),
                    top(58.px()).height(m.height.px()),
                ),
                Elevation::up(1),
                download,
                "file",
            );
            label::fixed_rule(grove, root, TAKEN - HAIR, HAIR);
            slip.taken.push(Taken {
                root,
                download: chip,
                bought: bought.clone(),
            });
        }
        grove.at(slip.lines, lines_at(receipt.bought.len()));
        let discount = match receipt.discount {
            0 => money(0),
            off => format!("−{}", money(off)),
        };
        let sums = [money(receipt.subtotal), discount, money(receipt.total)];
        for (leaf, value) in slip.sums.iter().zip(sums) {
            grove.text(*leaf, value);
        }
        // The stamp, and the seal pressed into wax with the holder's initials.
        slip.stamp.lift(grove);
        slip.stamp.press(grove, 700);
        if let Some((_, root)) = slip.seal.take() {
            grove.prune(root);
        }
        let mut seal = wax();
        let mut scatter = Scatter::new();
        for _ in 0..(self.placed * 7 + receipt.number.len() as u32) {
            scatter.next();
        }
        let root = seal.grow(grove, slip.seal_box, Location::new(), &mut scatter);
        seal.form(grove);
        slip.seal = Some((seal, root));
        grove.text(slip.initials, initials(&receipt.holder));
        grove.opacity(slip.initials, 0.0);
        grove.animate(
            slip.initials,
            Motion::Opacity(1.0),
            Timing::ms(500).after(1700).ease(Ease::Decelerate),
        );
        slip.say.clear(grove);
    }
}

/// Where step `n`'s press stands: two by two on a narrow window, in a row on a wider one.
fn step_at(n: usize) -> Location {
    let m = measure();
    let along = (0..n).fold(0.px(), |x: Length, j| {
        x + Chip::width(STEPS[j]) + m.gap.px()
    });
    let narrow = match n % 2 {
        0 => 0.px(),
        _ => Chip::width(STEPS[n - 1]) + m.gap.px(),
    };
    Location::new()
        .xs(
            left(narrow).width(Chip::width(STEPS[n])),
            top(((n / 2) as f32 * (m.height + m.gap)).px()).height(m.height.px()),
        )
        .md(
            left(along).width(Chip::width(STEPS[n])),
            top(0.px()).height(m.height.px()),
        )
}

/// Where the steps stand, as tall as step `n` needs.
fn steps_at(n: usize) -> Location {
    let m = measure();
    let rail = 2.0 * m.height + m.gap;
    Location::new()
        .xs(
            left(0.px()).right(100.pct()),
            top(0.px()).height((rail + 24.0 + PANELS[n]).px()),
        )
        .md(
            left(0.px()).right(100.pct()),
            top(0.px()).height((m.height + 24.0 + PANELS[n]).px()),
        )
        .lg(
            left(0.px()).width(56.pct()),
            top(0.px()).height((m.height + 24.0 + PANELS[n]).px()),
        )
}

/// Where the docket stands, holding `lines` lines: under the steps on a narrow window, beside them
/// on a wide one.
fn docket_at(lines: usize) -> Location {
    let height = 16.0 + 40.0 + lines as f32 * DOCKET_LINE + 3.0 * ROW + 10.0 + 44.0 + 16.0;
    Location::new()
        .xs(
            left(0.px()).right(100.pct()).at_most(480.px()),
            top(anchor().bottom() + 12.px()).height(height.px()),
        )
        .lg(
            right(100.pct()).width(38.pct()),
            top(anchor().top()).height(height.px()),
        )
}

/// Where the receipt's lines stand, `n` of them.
fn lines_at(n: usize) -> Location {
    Location::new().xs(
        left(0.px()).right(100.pct()),
        top(anchor().bottom() + 6.px()).height((n as f32 * TAKEN).px()),
    )
}

/// The holder's initials, for the seal: the first letter of the first two words.
fn initials(holder: &str) -> String {
    holder
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

/// A wax seal: a scalloped disc, twenty-four lobes round, in the wax's colours.
fn wax() -> Mosaic {
    let stops = theme::wax();
    let outline: Vec<(f32, f32)> = (0..144)
        .map(|n| {
            let theta = n as f32 / 144.0 * std::f32::consts::TAU;
            let r = 1.0 + 0.04 * (24.0 * theta).cos();
            (r * theta.cos(), r * theta.sin())
        })
        .collect();
    let edge = lichen::rgb(stops[0]);
    Mosaic::new(Silhouette::new(&outline), lichen::Ramp::of(&stops))
        .pitch(0.085)
        .edge(edge)
        .along((0.3, 1.0))
}

/// Grows the docket under `under`, beside or under `steps`.
fn docket(grove: &mut Grove, under: Leaf, steps: Leaf, faces: &Faces) -> Docket {
    let panel = grove.branch(
        under,
        Panel::new()
            .color(Palette::Raised)
            .outline(HAIR, Palette::Ink)
            .intangible()
            .anchored(steps)
            .at(docket_at(0)),
    );
    let inner = grove.branch(
        panel,
        Stem::new()
            .intangible()
            .elevate(Elevation::up(1))
            .font_size(lichen::caption())
            .at(Location::new().xs(
                left(18.px()).right(100.pct() - 18.px()),
                top(16.px()).bottom(100.pct() - 16.px()),
            )),
    );
    grove.branch(
        inner,
        Text::new("Your order")
            .font(faces.roman)
            .font_size(FontSize::new().xs(21).md(22))
            .color(Palette::Ink)
            .intangible()
            .at(Location::new().xs(
                left(0.px()).width(content()),
                top(0.px()).height(1.letters()),
            )),
    );
    label::fixed_rule(grove, inner, 36.0, HEAVY);
    let foot = grove.branch(
        inner,
        Text::new("")
            .font_size(theme::small())
            .color(Palette::Muted)
            .intangible()
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                bottom(100.pct()).height(content()),
            )),
    );
    Docket {
        panel,
        inner,
        lines: Vec::new(),
        sums: None,
        foot,
    }
}

/// Grows the receipt under `col`, below `after`: the slip, and the seal beside it on a wide
/// window or under it on a narrow one. Hidden until there is a receipt.
fn slip(grove: &mut Grove, col: Leaf, after: Leaf, faces: &Faces, icons: &Icons) -> Slip {
    let m = measure();
    let mut printed = Vec::new();
    let pane = grove.branch(
        col,
        Stem::new()
            .intangible()
            .anchored(after)
            .visible(false)
            .font_size(lichen::caption())
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + 24.px()).height(10.px()),
            )),
    );
    let paper = grove.branch(
        pane,
        Panel::new()
            .color(Palette::Raised)
            .outline(HAIR, Palette::Ink)
            .intangible()
            .at(Location::new().xs(left(0.px()).width(10.px()), top(0.px()).height(10.px()))),
    );
    let inner = grove.branch(
        pane,
        Stem::new()
            .intangible()
            .elevate(Elevation::up(1))
            .font_size(lichen::caption())
            .at(Location::new()
                .xs(
                    left(22.px()).right(100.pct() - 22.px()).at_most(596.px()),
                    top(24.px()).height(10.px()),
                )
                .lg(
                    left(22.px()).width(56.pct() - 44.px()),
                    top(24.px()).height(10.px()),
                )),
    );
    let wordmark = grove.branch(
        inner,
        Text::new("bits & bytes")
            .font(faces.italic)
            .font_size(FontSize::new().xs(24))
            .color(Palette::Ink)
            .intangible()
            .at(Location::new().xs(
                left(0.px()).width(content()),
                top(0.px()).height(1.letters()),
            )),
    );
    let number = label::words(
        grove,
        inner,
        wordmark,
        6.0,
        "",
        None,
        lichen::caption(),
        Palette::Ink,
    );
    let stamp_box = grove.branch(
        inner,
        Stem::new().intangible().at(Location::new().xs(
            right(100.pct() - 6.px()).width(92.px()),
            top(6.px()).height(40.px()),
        )),
    );
    let stamp = Stamp::grow(
        grove,
        stamp_box,
        Location::new(),
        "PAID",
        FontSize::new().xs(20),
        Some(faces.roman),
        Palette::Accent,
    );
    let heavy = label::rule(grove, inner, number, 16.0, HEAVY);
    printed.push(heavy);
    let (facts_panel, facts, fact_printed) = label::rows(
        grove,
        inner,
        heavy.leaf(),
        0.0,
        &["for", "sent to", "paid by", "holds"],
        80.0,
    );
    printed.extend(fact_printed);
    let lines = grove.branch(
        inner,
        Stem::new()
            .intangible()
            .anchored(facts_panel)
            .font_size(lichen::caption())
            .at(lines_at(0)),
    );
    let sums_box = grove.branch(
        inner,
        Stem::new()
            .intangible()
            .anchored(lines)
            .font_size(lichen::caption())
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + 4.px()).height((3.0 * ROW + 12.0).px()),
            )),
    );
    let mut sums = Vec::new();
    for (n, name) in ["subtotal", "discount", "total paid"].iter().enumerate() {
        let size = if n == 2 {
            FontSize::new().xs(26)
        } else {
            FontSize::new().xs(17)
        };
        let at = n as f32 * ROW + if n == 2 { 12.0 } else { 0.0 };
        let (_, value) = label::sum_row(
            grove,
            sums_box,
            at,
            name,
            size,
            Some(faces.italic),
            Palette::Ink,
        );
        sums.push(value);
    }
    label::fixed_rule(grove, sums_box, ROW - HAIR, HAIR);
    label::fixed_rule(grove, sums_box, 2.0 * ROW, HEAVY);
    // The paper runs to just under the sums.
    grove.anchor(paper, sums_box);
    grove.at(
        paper,
        Location::new()
            .xs(
                left(0.px()).right(100.pct()).at_most(640.px()),
                top(0.px()).bottom(anchor().bottom() + 24.px()),
            )
            .lg(
                left(0.px()).width(56.pct()),
                top(0.px()).bottom(anchor().bottom() + 24.px()),
            ),
    );
    // The seal: under the slip on a narrow window, beside it on a wide one.
    let seal_pane = grove.branch(
        pane,
        Stem::new()
            .intangible()
            .anchored(paper)
            .font_size(lichen::caption())
            .at(Location::new()
                .xs(
                    left(0.px()).right(100.pct()).at_most(640.px()),
                    top(anchor().bottom() + 32.px()).height(10.px()),
                )
                .lg(
                    right(100.pct()).width(38.pct()),
                    top(anchor().top()).height(10.px()),
                )),
    );
    let heading = grove.branch(
        seal_pane,
        Text::new("Sealed in wax")
            .font(faces.roman)
            .font_size(FontSize::new().xs(21).md(22))
            .color(Palette::Ink)
            .intangible()
            .at(Location::new().xs(
                left(0.px()).width(content()),
                top(0.px()).height(1.letters()),
            )),
    );
    printed.push(Print::words(grove, heading));
    printed.push(Print::rule(
        grove,
        seal_pane,
        Location::new().xs(
            left(0.px()).right(100.pct()),
            top(36.px()).height(HEAVY.px()),
        ),
        Palette::Ink,
    ));
    let seal_box = grove.branch(
        seal_pane,
        Stem::new()
            .intangible()
            .elevate(Elevation::up(1))
            .at(Location::new()
                .xs(
                    center_x(50.pct()).width(168.px()),
                    top(64.px()).height(168.px()),
                )
                .lg(
                    center_x(50.pct()).width(212.px()),
                    top(64.px()).height(212.px()),
                )),
    );
    let initials = grove.branch(
        seal_box,
        Text::new("")
            .font(faces.italic)
            .font_size(FontSize::new().xs(40).lg(52))
            .color(Palette::Raised)
            .intangible()
            .elevate(Elevation::up(24))
            .at(Location::new().xs(
                center_x(50.pct()).width(content()),
                center_y(50.pct()).height(1.letters()),
            )),
    );
    let (sealed_panel, sealed, sealed_printed) = label::rows(
        grove,
        seal_pane,
        seal_box,
        28.0,
        &["for", "weighs", "charged"],
        80.0,
    );
    printed.extend(sealed_printed);
    let chip_row = grove.branch(
        seal_pane,
        Stem::new()
            .intangible()
            .anchored(sealed_panel)
            .font_size(lichen::caption())
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + 20.px()).height(m.height.px()),
            )),
    );
    let mut shop = Chip::grow(
        grove,
        chip_row,
        Location::new().xs(
            left(0.px()).width(Chip::width("back to the shop")),
            top(0.px()).height(100.pct()),
        ),
        Elevation::up(1),
        icons.home,
        "back to the shop",
    );
    shop.arm(grove, Press::Armed);
    let ping = Ping::bare(
        grove,
        chip_row,
        Location::new().xs(
            right(100.pct()).width(Chip::square()),
            top(0.px()).height(100.pct()),
        ),
        icons.check,
    );
    let say = Say::grow(
        grove,
        seal_pane,
        Location::new().xs(
            left(0.px()).right(100.pct()),
            top(anchor().bottom() + 10.px()).height(content()),
        ),
        None,
    );
    grove.anchor(say.leaf(), chip_row);
    // A margin under whichever ends lower.
    for last in [paper, say.leaf()] {
        grove.branch(
            pane,
            Stem::new()
                .intangible()
                .anchored(last)
                .at(Location::new().xs(
                    left(0.px()).width(1.px()),
                    top(anchor().bottom()).height(72.px()),
                )),
        );
    }
    Slip {
        pane,
        number,
        stamp,
        facts,
        lines,
        taken: Vec::new(),
        sums,
        seal_box,
        seal: None,
        initials,
        sealed,
        shop,
        ping,
        say,
        printed,
    }
}
