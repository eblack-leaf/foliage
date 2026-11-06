//! The grammar every label in the shop is set in: heavy rules and hairlines, and rows of a name on
//! the left and its value beside it -- the panel on the side of a box that says what is in it.
//!
//! Every word the shop says stands in one of these rows, under one of these rules, or in a press:
//! there are no loose captions anywhere.

use foliage::{
    Boxed, Elevation, Fill, Font, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place,
    Source, Stem, Text, anchor, center_y, content, left, right, top,
};

use arboretum::sprout::{Print, Printed};

/// How tall a row of a panel is.
pub const ROW: f32 = 30.0;

/// A rule `weight` thick across `under`, `gap` below `after`, printed in. The holder, to anchor
/// the next thing to, and what prints it.
pub fn rule(grove: &mut Grove, under: Leaf, after: Leaf, gap: f32, weight: f32) -> Printed {
    let holder = grove.branch(
        under,
        Stem::new()
            .intangible()
            .anchored(after)
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + gap.px()).height(weight.px()),
            )),
    );
    Print::rule_in(grove, holder, Palette::Ink)
}

/// A rule across `under` at `y`, not printed: always there.
pub fn fixed_rule(grove: &mut Grove, under: Leaf, y: f32, weight: f32) -> Leaf {
    grove.branch(
        under,
        Panel::new()
            .color(Palette::Ink)
            .intangible()
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(y.px()).height(weight.px()),
            )),
    )
}

/// Words under `under`, anchored `gap` below `after`, as wide as `under`, as tall as they wrap.
#[allow(clippy::too_many_arguments)]
pub fn words(
    grove: &mut Grove,
    under: Leaf,
    after: Leaf,
    gap: f32,
    text: impl Into<String>,
    font: Option<Font>,
    size: FontSize,
    fill: impl Into<Fill>,
) -> Leaf {
    let text = Text::new(text).font_size(size).color(fill).intangible();
    let text = match font {
        Some(font) => text.font(font),
        None => text,
    };
    grove.branch(
        under,
        text.anchored(after).at(Location::new().xs(
            left(0.px()).right(100.pct()),
            top(anchor().bottom() + gap.px()).height(content()),
        )),
    )
}

/// A panel of rows under `under`, `gap` below `after`: each row a name and a value, a hairline
/// under every row but the last and a heavy rule under that. Hands back the panel and the values,
/// to be written over, and what prints it all.
pub fn rows(
    grove: &mut Grove,
    under: Leaf,
    after: Leaf,
    gap: f32,
    names: &[&str],
    key: f32,
) -> (Leaf, Vec<Leaf>, Vec<Printed>) {
    let panel = grove.branch(
        under,
        Stem::new()
            .intangible()
            .anchored(after)
            .font_size(lichen::caption())
            .at(Location::new().xs(
                left(0.px()).right(100.pct()),
                top(anchor().bottom() + gap.px()).height((names.len() as f32 * ROW).px()),
            )),
    );
    let mut printed = Vec::new();
    let values = names
        .iter()
        .enumerate()
        .map(|(n, name)| {
            let y = n as f32 * ROW;
            let middle = || center_y((y + ROW / 2.0).px()).height(1.letters());
            let key_leaf = grove.branch(
                panel,
                Text::new(*name)
                    .font_size(lichen::caption())
                    .color(Palette::Muted)
                    .intangible()
                    .at(Location::new().xs(left(0.px()).width(content()), middle())),
            );
            let value = grove.branch(
                panel,
                Text::new("")
                    .font_size(lichen::caption())
                    .color(Palette::Ink)
                    .intangible()
                    .at(Location::new().xs(left(key.px()).right(100.pct()), middle())),
            );
            let last = n + 1 == names.len();
            let weight = if last {
                crate::theme::HEAVY
            } else {
                crate::theme::HAIR
            };
            printed.push(Print::words(grove, key_leaf));
            printed.push(Print::rule(
                grove,
                panel,
                Location::new().xs(
                    left(0.px()).right(100.pct()),
                    top((y + ROW - weight).px()).height(weight.px()),
                ),
                Palette::Ink,
            ));
            printed.push(Print::plain(grove, value));
            value
        })
        .collect();
    (panel, values, printed)
}

/// A line of a total: a name on the left and a figure on the right, at `y` in `under`.
pub fn sum_row(
    grove: &mut Grove,
    under: Leaf,
    y: f32,
    name: &str,
    size: FontSize,
    font: Option<Font>,
    fill: impl Into<Fill>,
) -> (Leaf, Leaf) {
    let fill = fill.into();
    let middle = || center_y((y + ROW / 2.0).px()).height(1.letters());
    let key = grove.branch(
        under,
        Text::new(name)
            .font_size(lichen::caption())
            .color(fill)
            .intangible()
            .elevate(Elevation::up(1))
            .at(Location::new().xs(left(0.px()).width(content()), middle())),
    );
    let text = Text::new("")
        .font_size(size)
        .color(fill)
        .intangible()
        .elevate(Elevation::up(1));
    let text = match font {
        Some(font) => text.font(font),
        None => text,
    };
    let value = grove.branch(
        under,
        text.at(Location::new().xs(right(100.pct()).width(content()), middle())),
    );
    (key, value)
}
