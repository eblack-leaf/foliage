//! The shop's backend, which is not there: the answers a server would give, worked out here.
//!
//! Pure functions over the catalogue and the bag. The page never shows one of their answers at
//! once -- each is carried through a [`Courier`](arboretum::sprout::Courier), so a promo code is
//! checked and an order authorised with the wait and the chance of a refusal a real one has. A real
//! backend would replace this module and nothing else.

use arboretum::genart::Seed;

use crate::cart::{Cart, Line, Promo};
use crate::catalog::{Aisle, Lch, Look, weight};

/// Why a promo code was not taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// Nothing was typed.
    Empty,
    /// It was a code, and it lapsed on the day given.
    Expired(&'static str),
    /// It was never a code.
    Unknown(String),
}

impl Refused {
    /// What the bag says about it.
    pub fn words(&self) -> String {
        match self {
            Refused::Empty => "type a code first".to_string(),
            Refused::Expired(when) => format!("that code lapsed on {when}"),
            Refused::Unknown(code) => format!("“{code}” is not a code we know"),
        }
    }
}

/// Checks a promo code, as the till would.
pub fn promo(code: &str) -> Result<Promo, Refused> {
    let code = code.trim().to_uppercase();
    match code.as_str() {
        "" => Err(Refused::Empty),
        "FERN" => Ok(Promo {
            code: "FERN",
            percent: 15,
        }),
        "EXPIRED" => Err(Refused::Expired("31.10.2026")),
        _ => Err(Refused::Unknown(code)),
    }
}

/// Whether `email` is an address a receipt could be sent to: something, an `@`, a domain with a
/// dot in it, and no spaces. What is wrong with it, if anything.
pub fn email(email: &str) -> Result<(), &'static str> {
    let email = email.trim();
    if email.is_empty() {
        return Err("the receipt needs an address to go to");
    }
    if email.contains(char::is_whitespace) {
        return Err("an address has no spaces in it");
    }
    let Some((name, domain)) = email.split_once('@') else {
        return Err("that address is missing its @");
    };
    if name.is_empty() || domain.contains('@') {
        return Err("that address has its @ in the wrong place");
    }
    match domain.split_once('.') {
        Some((host, tld)) if !host.is_empty() && !tld.is_empty() && !tld.ends_with('.') => Ok(()),
        _ => Err("that address's domain needs a dot, as in fernwood.studio"),
    }
}

/// Whether `holder` is a name a licence could be printed with.
pub fn holder(holder: &str) -> Result<(), &'static str> {
    match holder.trim().chars().count() {
        0 => Err("a licence is printed with a name: whose is it?"),
        1 => Err("a little more of the name, please"),
        2..=40 => Ok(()),
        _ => Err("that name is longer than the label"),
    }
}

/// The ways an order can be paid. None of them charges anything.
pub const METHODS: &[&str] = &["demo card", "voucher", "invoice"];

/// What an order is placed with.
#[derive(Clone, Debug, PartialEq)]
pub struct Order {
    pub email: String,
    pub holder: String,
    pub method: usize,
    pub cart: Cart,
}

/// Why an order was not taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declined {
    /// The bag was empty by the time it got here.
    Empty,
    /// The payment was refused, and what the bank said.
    Bank(&'static str),
}

impl Declined {
    pub fn words(&self) -> &'static str {
        match self {
            Declined::Empty => "the bag was empty by the time it reached the till",
            Declined::Bank(why) => why,
        }
    }
}

/// One line of a receipt: what was bought, and the key to it.
#[derive(Clone, Debug, PartialEq)]
pub struct Bought {
    pub line: Line,
    pub key: String,
}

/// An order taken: what comes back from the till.
#[derive(Clone, Debug, PartialEq)]
pub struct Receipt {
    pub number: String,
    pub email: String,
    pub holder: String,
    pub method: &'static str,
    pub bought: Vec<Bought>,
    pub subtotal: u32,
    pub discount: u32,
    pub promo: Option<Promo>,
    pub total: u32,
    pub bytes: u64,
}

/// Places `order`, the shop's `placed`th: a receipt, or why not.
///
/// An address at `fail.test` is declined by its bank, which is how the demo shows a refusal; the
/// same order from another address goes through.
pub fn place(order: &Order, placed: u32) -> Result<Receipt, Declined> {
    if order.cart.is_empty() {
        return Err(Declined::Empty);
    }
    if order.email.trim().to_lowercase().ends_with("@fail.test") {
        return Err(Declined::Bank(
            "declined: the bank behind fail.test said no. nothing was charged",
        ));
    }
    let mut seed = Seed::of(&format!(
        "{}|{}|{placed}",
        order.email.trim(),
        order.holder.trim()
    ));
    let number = number(&mut seed);
    let bought = order
        .cart
        .lines
        .iter()
        .map(|&line| Bought {
            line,
            key: key(&line, &mut seed),
        })
        .collect();
    Ok(Receipt {
        number,
        email: order.email.trim().to_string(),
        holder: order.holder.trim().to_string(),
        method: METHODS[order.method.min(METHODS.len() - 1)],
        bought,
        subtotal: order.cart.subtotal(),
        discount: order.cart.discount(),
        promo: order.cart.promo,
        total: order.cart.total(),
        bytes: order.cart.bytes(),
    })
}

/// An order number: `BB-` and two groups of four hex digits.
fn number(seed: &mut Seed) -> String {
    let n = seed.next_u64();
    format!("BB-{:04X}-{:04X}", (n >> 16) & 0xffff, n & 0xffff)
}

/// A licence key for `line`: the good's first three letters, its licence's first, and twelve hex
/// digits.
fn key(line: &Line, seed: &mut Seed) -> String {
    let stem: String = line
        .good()
        .name
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .take(3)
        .collect::<String>()
        .to_uppercase();
    let licence = line
        .licence
        .name()
        .chars()
        .next()
        .unwrap_or('p')
        .to_ascii_uppercase();
    let n = seed.next_u64();
    format!(
        "{stem}{licence}-{:04X}-{:04X}-{:04X}",
        (n >> 32) & 0xffff,
        (n >> 16) & 0xffff,
        n & 0xffff
    )
}

/// A file to hand over for one line of a receipt: its name, its type, and what is in it. A
/// palette's is real -- the colours as CSS custom properties -- and anything else's is the readme
/// that would come in its box.
pub fn file(bought: &Bought, receipt: &Receipt) -> (String, &'static str, String) {
    let good = bought.line.good();
    let slug: String = good
        .name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let header = format!(
        "bits & bytes · {} · {}\nlicensed to {} ({} licence, {} seat{}) · key {}\norder {} · {}\n",
        good.name,
        good.aisle.single(),
        receipt.holder,
        bought.line.licence.name(),
        bought.line.seats,
        if bought.line.seats == 1 { "" } else { "s" },
        bought.key,
        receipt.number,
        bought.line.licence.terms(),
    );
    match good.look {
        Look::Palette(swatches) => {
            let mut css = format!(
                "/* {} */\n:root {{\n",
                header.trim_end().replace('\n', "\n   ")
            );
            for (n, &swatch) in swatches.iter().enumerate() {
                css.push_str(&format!("  --{slug}-{}: {};\n", n + 1, hex(swatch)));
            }
            css.push_str("}\n");
            (format!("{slug}.css"), "text/css", css)
        }
        _ => {
            let body = format!(
                "{header}\n{} -- {}\ncontents: {} · {} files · {} as {}\n\nthis is a demo: the shop is a fiction, and so is the file this readme came with.\n",
                good.name,
                good.blurb,
                good.contents,
                good.files,
                weight(good.bytes),
                bought.line.format(),
            );
            (format!("{slug}-readme.txt"), "text/plain", body)
        }
    }
}

/// A `data:` URL for `text` as `mime`, base64 so that no `#` or newline cuts it short.
pub fn data_url(mime: &str, text: &str) -> String {
    format!(
        "data:{mime};charset=utf-8;base64,{}",
        base64(text.as_bytes())
    )
}

/// Standard base64, padded.
pub fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// A colour as a hex triplet, for a palette's file and for copying a swatch.
pub fn hex((l, c, h): Lch) -> String {
    let color = foliage::Color::oklch(l, c, h);
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        byte(color.red),
        byte(color.green),
        byte(color.blue)
    )
}

/// What a receipt holds, in a line: how many goods, and what they weigh.
pub fn summary(receipt: &Receipt) -> String {
    let goods = receipt.bought.len();
    format!(
        "{goods} good{} · net {}",
        if goods == 1 { "" } else { "s" },
        weight(receipt.bytes)
    )
}

/// What one licence of `aisle` allows, in a line, for the contents panel.
pub fn formats_line(aisle: Aisle) -> String {
    aisle.formats().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cart::Licence;
    use crate::catalog::GOODS;

    fn order(email: &str) -> Order {
        let mut cart = Cart::new();
        cart.add(5, Licence::Personal, 0);
        cart.add(2, Licence::Studio, 2);
        Order {
            email: email.to_string(),
            holder: "Ada Fernwood".to_string(),
            method: 0,
            cart,
        }
    }

    #[test]
    fn fern_takes_fifteen_and_expired_is_told_so() {
        assert_eq!(
            promo(" fern "),
            Ok(Promo {
                code: "FERN",
                percent: 15
            })
        );
        assert_eq!(promo("EXPIRED"), Err(Refused::Expired("31.10.2026")));
        assert_eq!(promo("bogus"), Err(Refused::Unknown("BOGUS".to_string())));
        assert_eq!(promo("  "), Err(Refused::Empty));
        assert!(promo("expired").unwrap_err().words().contains("lapsed"));
    }

    #[test]
    fn an_address_needs_an_at_and_a_dotted_domain() {
        assert!(email("ada@fernwood.studio").is_ok());
        assert!(
            email(" ada@fail.test ").is_ok(),
            "fail.test is a fine address; its bank is the problem"
        );
        assert!(email("").is_err());
        assert!(email("ada fernwood@x.y").is_err());
        assert!(email("ada.fernwood.studio").is_err());
        assert!(email("@fernwood.studio").is_err());
        assert!(email("ada@fernwood").is_err());
        assert!(email("ada@fernwood.").is_err());
        assert!(email("a@b@c.d").is_err());
    }

    #[test]
    fn a_holder_needs_a_name() {
        assert!(holder("Ada Fernwood").is_ok());
        assert!(holder(" ").is_err());
        assert!(holder("A").is_err());
        assert!(holder(&"x".repeat(41)).is_err());
    }

    #[test]
    fn an_order_is_receipted_with_keys_and_totals() {
        let order = order("ada@fernwood.studio");
        let receipt = place(&order, 0).unwrap();
        assert!(
            receipt.number.starts_with("BB-") && receipt.number.len() == 12,
            "{}",
            receipt.number
        );
        assert_eq!(receipt.bought.len(), 2);
        assert!(
            receipt.bought[0].key.starts_with("AURP-"),
            "{}",
            receipt.bought[0].key
        );
        assert!(
            receipt.bought[1].key.starts_with("PAPS-"),
            "{}",
            receipt.bought[1].key
        );
        assert_ne!(receipt.bought[0].key, receipt.bought[1].key);
        assert_eq!(receipt.total, order.cart.total());
        assert_eq!(receipt.subtotal, 2200 + 900 * 3);
        assert_eq!(receipt.bytes, order.cart.bytes());
        // The same order again is the same receipt; the next order is another.
        assert_eq!(place(&order, 0).unwrap(), receipt);
        assert_ne!(place(&order, 1).unwrap().number, receipt.number);
        assert!(summary(&receipt).starts_with("2 goods"));
    }

    #[test]
    fn fail_test_is_declined_and_an_empty_bag_is_refused() {
        assert_eq!(
            place(&order("ada@FAIL.test"), 0).unwrap_err(),
            Declined::Bank("declined: the bank behind fail.test said no. nothing was charged")
        );
        let mut empty = order("ada@fernwood.studio");
        empty.cart.clear();
        assert_eq!(place(&empty, 0), Err(Declined::Empty));
    }

    #[test]
    fn a_palette_downloads_as_css_and_anything_else_as_a_readme() {
        let receipt = place(&order("ada@fernwood.studio"), 0).unwrap();
        let (name, mime, css) = file(&receipt.bought[1], &receipt);
        assert_eq!(name, "paper-rust.css");
        assert_eq!(mime, "text/css");
        assert!(css.contains(":root {"));
        assert_eq!(css.matches("--paper-rust-").count(), 6);
        assert!(css.contains("#"), "colours as hex");
        let (name, mime, readme) = file(&receipt.bought[0], &receipt);
        assert_eq!(name, "aurora-readme.txt");
        assert_eq!(mime, "text/plain");
        assert!(readme.contains("licensed to Ada Fernwood"));
        assert!(readme.contains(&receipt.bought[0].key));
    }

    #[test]
    fn base64_is_standard_and_padded() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        let url = data_url("text/css", "a#b\nc");
        assert!(url.starts_with("data:text/css;charset=utf-8;base64,"));
        assert!(!url.contains('#') && !url.contains('\n'));
    }

    #[test]
    fn hex_is_a_hash_and_six_digits() {
        let ink = hex((0.21, 0.012, 45.0));
        assert_eq!(ink.len(), 7);
        assert!(ink.starts_with('#'));
        assert_eq!(hex((1.0, 0.0, 0.0)), "#ffffff");
        for good in &GOODS {
            if let Look::Palette(swatches) = good.look {
                assert!(swatches.iter().all(|&s| hex(s).len() == 7));
            }
        }
    }

    #[test]
    fn a_receipt_keeps_each_line_s_licence_and_sums_up() {
        let receipt = place(&order("ada@fernwood.studio"), 0).unwrap();
        assert!(
            receipt
                .bought
                .iter()
                .any(|b| b.line.licence == Licence::Studio)
        );
        assert!(
            !receipt
                .bought
                .iter()
                .any(|b| b.line.licence == Licence::Extended)
        );
        assert_eq!(
            summary(&receipt),
            format!("2 goods · net {}", weight(receipt.bytes))
        );
        assert_eq!(formats_line(Aisle::Shaders), ".wgsl .glsl");
    }
}
