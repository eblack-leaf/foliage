//! The bag: what is in it, for how many seats, under which licence -- and what it all comes to,
//! in money and in bytes.

use crate::catalog::{GOODS, Good};

/// The terms a good is sold under.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Licence {
    Personal,
    Studio,
    Extended,
}

impl Licence {
    pub const ALL: [Licence; 3] = [Licence::Personal, Licence::Studio, Licence::Extended];
    /// The names the licence pick offers, in the order of [`Licence::ALL`].
    pub const NAMES: &'static [&'static str] = &["personal", "studio", "extended"];

    /// How many times a personal licence's price it costs.
    pub fn times(self) -> u32 {
        match self {
            Licence::Personal => 1,
            Licence::Studio => 3,
            Licence::Extended => 8,
        }
    }

    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    /// What it allows, in a line.
    pub fn terms(self) -> &'static str {
        match self {
            Licence::Personal => "your own work, not for sale",
            Licence::Studio => "client work, up to ten people",
            Licence::Extended => "products for sale, unlimited",
        }
    }
}

/// A code taken off the total.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Promo {
    pub code: &'static str,
    /// How much is taken off, in percent.
    pub percent: u32,
}

/// One line in the bag: a good, under a licence, as a format, for some seats.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub good: usize,
    pub licence: Licence,
    /// Which of the good's aisle's formats.
    pub format: usize,
    pub seats: u32,
}

impl Line {
    pub fn good(&self) -> &'static Good {
        &GOODS[self.good]
    }

    /// The format, as its file extension.
    pub fn format(&self) -> &'static str {
        let formats = self.good().aisle.formats();
        formats[self.format.min(formats.len() - 1)]
    }

    /// What one seat costs.
    pub fn each(&self) -> u32 {
        self.good().cents * self.licence.times()
    }

    /// What the line costs.
    pub fn cents(&self) -> u32 {
        self.each() * self.seats
    }
}

/// The most seats one line can hold.
pub const MOST_SEATS: u32 = 99;

/// The bag.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cart {
    pub lines: Vec<Line>,
    pub promo: Option<Promo>,
}

impl Cart {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a seat of `good` under `licence` as `format`: another seat on the line that already
    /// holds the same, or a new line at the end. Which line it went on.
    pub fn add(&mut self, good: usize, licence: Licence, format: usize) -> usize {
        if let Some(n) = self
            .lines
            .iter()
            .position(|line| line.good == good && line.licence == licence && line.format == format)
        {
            self.lines[n].seats = (self.lines[n].seats + 1).min(MOST_SEATS);
            return n;
        }
        self.lines.push(Line {
            good,
            licence,
            format,
            seats: 1,
        });
        self.lines.len() - 1
    }

    /// Changes line `n`'s seats by `by`, keeping it between one and [`MOST_SEATS`]. What it holds
    /// now.
    pub fn seat(&mut self, n: usize, by: i32) -> u32 {
        let Some(line) = self.lines.get_mut(n) else {
            return 0;
        };
        line.seats = (line.seats as i32 + by).clamp(1, MOST_SEATS as i32) as u32;
        line.seats
    }

    /// Takes line `n` out.
    pub fn remove(&mut self, n: usize) {
        if n < self.lines.len() {
            self.lines.remove(n);
        }
        if self.lines.is_empty() {
            self.promo = None;
        }
    }

    /// Empties the bag, as a placed order does.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.promo = None;
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// How many seats, across every line: what the bag's tally counts.
    pub fn count(&self) -> u32 {
        self.lines.iter().map(|line| line.seats).sum()
    }

    pub fn subtotal(&self) -> u32 {
        self.lines.iter().map(Line::cents).sum()
    }

    /// What the promo takes off, rounded to the nearest cent.
    pub fn discount(&self) -> u32 {
        match self.promo {
            Some(promo) => (self.subtotal() * promo.percent + 50) / 100,
            None => 0,
        }
    }

    pub fn total(&self) -> u32 {
        self.subtotal() - self.discount()
    }

    /// What the bag weighs: each good once, however many seats -- a seat is a licence, and a
    /// licence weighs nothing.
    pub fn bytes(&self) -> u64 {
        self.lines.iter().map(|line| line.good().bytes).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_the_same_again_adds_a_seat() {
        let mut cart = Cart::new();
        assert_eq!(cart.add(5, Licence::Personal, 0), 0);
        assert_eq!(cart.add(5, Licence::Personal, 0), 0);
        assert_eq!(cart.lines.len(), 1);
        assert_eq!(cart.lines[0].seats, 2);
        // Another licence or format is another line.
        assert_eq!(cart.add(5, Licence::Studio, 0), 1);
        assert_eq!(cart.add(5, Licence::Personal, 1), 2);
        assert_eq!(cart.count(), 4);
    }

    #[test]
    fn totals_follow_licences_seats_and_promo() {
        let mut cart = Cart::new();
        // Aurora, €22, personal ×1; Tape Hiss, €7, studio ×3, two seats.
        cart.add(5, Licence::Personal, 0);
        let hiss = cart.add(15, Licence::Studio, 1);
        cart.seat(hiss, 1);
        assert_eq!(cart.subtotal(), 2200 + 700 * 3 * 2);
        assert_eq!(cart.discount(), 0);
        cart.promo = Some(Promo {
            code: "FERN",
            percent: 15,
        });
        // 15% of €64.00 is €9.60.
        assert_eq!(cart.subtotal(), 6400);
        assert_eq!(cart.discount(), 960);
        assert_eq!(cart.total(), 5440);
    }

    #[test]
    fn a_discount_rounds_to_the_nearest_cent() {
        let mut cart = Cart::new();
        cart.add(15, Licence::Personal, 0); // €7.00
        cart.promo = Some(Promo {
            code: "FERN",
            percent: 15,
        });
        assert_eq!(cart.discount(), 105);
        cart.add(0, Licence::Personal, 0); // + €9.00 = €16.00, 15% = €2.40
        assert_eq!(cart.discount(), 240);
        assert_eq!(cart.total(), 1360);
    }

    #[test]
    fn seats_stay_between_one_and_the_most() {
        let mut cart = Cart::new();
        cart.add(0, Licence::Personal, 0);
        assert_eq!(cart.seat(0, -5), 1);
        assert_eq!(cart.seat(0, 500), MOST_SEATS);
        assert_eq!(cart.seat(3, 1), 0, "no such line");
    }

    #[test]
    fn a_bag_weighs_each_good_once() {
        let mut cart = Cart::new();
        cart.add(12, Licence::Personal, 0);
        cart.seat(0, 3);
        assert_eq!(cart.bytes(), GOODS[12].bytes);
        cart.add(0, Licence::Extended, 2);
        assert_eq!(cart.bytes(), GOODS[12].bytes + GOODS[0].bytes);
        assert_eq!(cart.lines[1].format(), ".css");
    }

    #[test]
    fn emptying_the_bag_lets_go_of_the_promo() {
        let mut cart = Cart::new();
        cart.add(0, Licence::Personal, 0);
        cart.promo = Some(Promo {
            code: "FERN",
            percent: 15,
        });
        cart.remove(0);
        assert!(cart.is_empty());
        assert_eq!(cart.promo, None);
        assert_eq!(cart.total(), 0);
    }
}
