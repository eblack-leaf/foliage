//! A storefront: one product, and everything between looking at it and having ordered it.

use foliage::{
    Boxed, Elevation, Font, Grove, Grow, Icon, Leaf, Location, Palette, Place, Pollen, Source,
    Stem, center_x, center_y, content, left, right,
};
use lichen::{
    Badge, Chip, Cluster, Confirm, Pick, Ping, Press, Ramp, Say, Scatter, Step, Voice, gate,
    measure,
};

use crate::icons::Icons;
use crate::parts::{self, Stack, at, in_lane, lane, line};
use crate::theme;

/// What one of the product costs, in cents.
const PRICE: u32 = 8_400;

/// The code the promo field takes, and what it takes off, in percent.
const CODE: &str = "LEAF";
const OFF: u32 = 10;

/// How many stars a rating is out of, and how far apart they stand.
const STARS: usize = 5;
const STAR: f32 = 22.0;
const STAR_PITCH: f32 = 28.0;

/// The steps an order goes through, and what each is marked with.
const SHIPPING: [&str; 4] = ["ordered", "packed", "shipped", "delivered"];

pub(crate) struct Storefront {
    wish: Chip,
    wished: bool,
    size: Pick,
    fewer: Chip,
    more: Chip,
    count: Leaf,
    total: Leaf,
    quantity: u32,
    add: Chip,
    added: Ping,
    cart: Chip,
    carted: Badge,
    in_cart: u32,
    stars: Vec<(Leaf, Leaf)>,
    rating: Leaf,
    code: lichen::Field,
    apply: Chip,
    promo: Say,
    discounted: bool,
    shipping: Vec<Chip>,
    shipped: [bool; SHIPPING.len()],
    checkout: Confirm,
    ordered: Say,
}

impl Storefront {
    pub(crate) fn grow(grove: &mut Grove, page: Leaf, icons: &Icons, italic: Font) -> Self {
        let m = measure();
        let mut scatter = Scatter::new();
        let mut stack = Stack::new(page);
        stack.heading(
            grove,
            "storefront",
            "One product, and every step from looking at it to having ordered it.",
            italic,
        );

        // A product: a puff of the leaf's own colours for a photograph, what it is and what it
        // costs, and a heart to keep it.
        let body = stack.card(grove, "product", 96.0);
        let pad = grove.branch(
            body,
            Stem::new()
                .intangible()
                .at(at(left(0.px()).width(96.px()), 0.0, 96.0)),
        );
        let ember = Ramp::of(&grove.scheme().stops(theme::SPECIMEN));
        Cluster::grow(grove, pad, 96.0, &ember, &mut scatter);
        let beside = 112.0;
        line(grove, body, "linen overshirt", beside, 4.0, Palette::Ink);
        line(
            grove,
            body,
            dollars(PRICE),
            beside,
            28.0,
            Palette::Accent.mark(),
        );
        line(
            grove,
            body,
            "in stock · ships in 2 days",
            beside,
            52.0,
            lichen::INERT.ink,
        );
        let wish = parts::mark(
            grove,
            body,
            in_lane(right(100.pct()).width(m.height.px()), 0),
            icons.heart,
        );

        let body = stack.card(grove, "size", m.height);
        let size = Pick::grow(
            grove,
            body,
            Location::new(),
            6.0,
            "size",
            &["xs", "s", "m", "l", "xl"],
            1,
        );

        // How many, and what that comes to.
        let body = stack.card(grove, "quantity", m.height);
        let fewer = parts::mark(
            grove,
            body,
            in_lane(left(0.px()).width(m.height.px()), 0),
            icons.minus,
        );
        let count = parts::words(
            grove,
            body,
            "1",
            Location::new().xs(
                center_x((m.height + 24.0).px()).width(content()),
                center_y(50.pct()).height(1.letters()),
            ),
            Palette::Ink,
        );
        let more = parts::mark(
            grove,
            body,
            in_lane(left((m.height + 48.0).px()).width(m.height.px()), 0),
            icons.plus,
        );
        let total = parts::words(
            grove,
            body,
            "",
            Location::new().xs(
                right(100.pct()).width(16.letters()),
                center_y(50.pct()).height(1.letters()),
            ),
            lichen::REST.ink,
        );

        // Adding it: the press to make here, a mark that says it landed, and the cart keeping
        // count with a dot on it until it is looked at.
        let body = stack.card(grove, "add to cart", m.height);
        let mut add = parts::chip(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("add")), 0),
            icons.plus,
            "add",
        );
        add.arm(grove, Press::Armed);
        let added = parts::ping(
            grove,
            body,
            in_lane(left(Chip::width("add") + 4.px()).width(m.height.px()), 0),
            icons.check,
        );
        let cart = parts::chip(
            grove,
            body,
            in_lane(right(100.pct()).width(Chip::width("cart 00")), 0),
            icons.shopping_cart,
            "cart 0",
        );
        let carted = Badge::on(grove, &cart);

        // A rating, pressed onto the stars.
        let body = stack.card(grove, "rating", m.height);
        let stars = (0..STARS)
            .map(|n| {
                let cell = grove.branch(
                    body,
                    Stem::new().interactive().at(Location::new().xs(
                        left((n as f32 * STAR_PITCH).px()).width(STAR.px()),
                        center_y(50.pct()).height(STAR.px()),
                    )),
                );
                let star = grove.branch(
                    cell,
                    Icon::new(icons.star)
                        .color(Palette::Muted)
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new()),
                );
                (cell, star)
            })
            .collect();
        let rating = parts::words(
            grove,
            body,
            "not rated",
            Location::new().xs(
                left((STARS as f32 * STAR_PITCH + 12.0).px()).width(12.letters()),
                center_y(50.pct()).height(1.letters()),
            ),
            lichen::INERT.ink,
        );

        // A code, and the line that says what it came to.
        let body = stack.card(grove, "promo code", lane(1) + 16.0);
        let apply_width = Chip::width("apply");
        let code = lichen::Field::grow(
            grove,
            body,
            in_lane(
                left(0.px()).right(100.pct() - apply_width.clone() - m.gap.px()),
                0,
            ),
            5.0,
            "code",
            false,
        );
        let mut apply = parts::chip(
            grove,
            body,
            in_lane(right(100.pct()).width(apply_width), 0),
            icons.tag,
            "apply",
        );
        apply.arm(grove, Press::Inert);
        let mut promo = Say::grow(
            grove,
            body,
            at(left(0.px()).width(100.pct()), lane(1), 16.0),
            Some(italic),
        );
        promo.set(grove, &format!("try {CODE}"));

        // Where the order is. Each step is pressed when it happens; the next is the one to press,
        // and the ones after it cannot be pressed yet.
        let body = stack.card(grove, "shipping", lane(2) - m.gap);
        let marks = [icons.check, icons.package, icons.truck, icons.check];
        let shipping = SHIPPING
            .iter()
            .zip(marks)
            .enumerate()
            .map(|(n, (&name, mark))| {
                let across = match n % 2 {
                    0 => left(0.px()).width(50.pct() - (m.gap / 2.0).px()),
                    _ => left(50.pct() + (m.gap / 2.0).px()).right(100.pct()),
                };
                parts::chip(grove, body, in_lane(across, n / 2), mark, name)
            })
            .collect();

        // Placing it: the press that cannot be taken back, made as two.
        let body = stack.card(grove, "checkout", lane(1) + 16.0);
        let mut checkout = Confirm::grow(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("place order")), 0),
            in_lane(right(100.pct()).width(Chip::width("confirm")), 0),
            (icons.credit_card, "place order"),
            (icons.x, "keep"),
            (icons.check, "confirm"),
        );
        checkout.open(grove, true);
        let ordered = Say::grow(
            grove,
            body,
            at(left(0.px()).width(100.pct()), lane(1), 16.0),
            Some(italic),
        );

        let mut storefront = Self {
            wish,
            wished: false,
            size,
            fewer,
            more,
            count,
            total,
            quantity: 1,
            add,
            added,
            cart,
            carted,
            in_cart: 0,
            stars,
            rating,
            code,
            apply,
            promo,
            discounted: false,
            shipping,
            shipped: [false; SHIPPING.len()],
            checkout,
            ordered,
        };
        storefront.recount(grove);
        storefront.ship(grove);
        storefront
    }

    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if self.wish.pressed(pollen) {
            self.wished = !self.wished;
            self.wish.arm(
                grove,
                match self.wished {
                    true => Press::Chosen,
                    false => Press::Rest,
                },
            );
        }
        self.size.frame(grove, pollen);
        if self.fewer.pressed(pollen) && self.quantity > 1 {
            self.quantity -= 1;
            self.recount(grove);
        }
        if self.more.pressed(pollen) && self.quantity < 9 {
            self.quantity += 1;
            self.recount(grove);
        }
        self.added.frame(grove, pollen);
        if self.add.pressed(pollen) {
            self.in_cart += self.quantity;
            self.added.fire(grove);
            self.carted.show(grove, true);
            grove.text(self.cart.label(), format!("cart {}", self.in_cart));
        }
        // Looking at the cart is what the dot was asking for.
        if self.cart.pressed(pollen) {
            self.carted.show(grove, false);
        }
        if let Some(n) = self
            .stars
            .iter()
            .position(|(cell, _)| pollen.clicked(*cell))
        {
            for (k, (_, star)) in self.stars.iter().enumerate() {
                let fill = match k <= n {
                    true => Palette::Accent.mark(),
                    false => Palette::Muted,
                };
                grove.animate(*star, fill.into(), lichen::timing());
            }
            grove.text(self.rating, format!("{} of {STARS}", n + 1));
        }
        if self.code.frame(grove, pollen) {
            let typed = !self.code.value(grove).trim().is_empty();
            self.apply.arm(grove, Press::rest_if(typed));
        }
        if self.apply.pressed(pollen) || pollen.submitted(self.code.input()) {
            let right = self.code.value(grove).trim().eq_ignore_ascii_case(CODE);
            match right {
                true => self
                    .promo
                    .tell(grove, &format!("applied: {OFF}% off"), Voice::Hint),
                false => self.promo.tell(grove, "not a code we know", Voice::Warn),
            }
            self.discounted = right;
            self.recount(grove);
        }
        if let Some(n) = self.shipping.iter().position(|chip| chip.pressed(pollen)) {
            // Pressing the last step again starts another order.
            match self.shipped.iter().all(|&done| done) {
                true => self.shipped = [false; SHIPPING.len()],
                false => self.shipped[n] = true,
            }
            self.ship(grove);
        }
        match self.checkout.frame(grove, pollen) {
            Some(Step::Confirmed) => {
                self.ordered
                    .tell(grove, "order placed. thank you.", Voice::Hint);
            }
            Some(Step::Kept) => self.ordered.set(grove, "nothing ordered."),
            Some(Step::Armed) => self.ordered.clear(grove),
            None => {}
        }
    }

    /// Writes the count, and what it comes to after whatever the code took off.
    fn recount(&mut self, grove: &mut Grove) {
        let mut cents = PRICE * self.quantity;
        if self.discounted {
            cents -= cents * OFF / 100;
        }
        grove.text(self.count, format!("{}", self.quantity));
        grove.text(self.total, format!("total {}", dollars(cents)));
        self.fewer.arm(grove, Press::rest_if(self.quantity > 1));
        self.more.arm(grove, Press::rest_if(self.quantity < 9));
    }

    /// Dresses the shipping steps for which of them are done. Once all are, the last is left in
    /// reach, to start again.
    fn ship(&mut self, grove: &mut Grove) {
        let all = self.shipped.iter().all(|&done| done);
        for (n, (chip, press)) in self
            .shipping
            .iter_mut()
            .zip(gate(&self.shipped))
            .enumerate()
        {
            let press = match (all, n == SHIPPING.len() - 1) {
                (true, true) => Press::Chosen,
                _ => press,
            };
            chip.arm(grove, press);
        }
    }
}

/// Cents, as a price.
fn dollars(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}
