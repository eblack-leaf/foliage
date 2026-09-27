//! A blog, drawn with some care: a title set large, a cover made of the same polygons the leaf is,
//! a quote with a rule beside it, and the few presses a reader makes.

use foliage::{
    Boxed, Cap, Corners, Elevation, Font, FontSize, Grove, Grow, HAIRLINE, Icon, Leaf, Line,
    Location, Motion, Palette, Panel, Place, Point, Pollen, Polygon, Rounding, Source, Stem, Text,
    center_x, center_y, content, left, right, top,
};
use lichen::{Chip, Cluster, Ping, Press, Ramp, Say, Scatter, Trail, Voice, measure};

use crate::icons::Icons;
use crate::parts::{self, Stack, at, in_lane, lane, line};
use crate::theme;

/// The essay's title, and who wrote it.
const TITLE: &str = "On the patience of lichen";
const BYLINE: &str = "words and drawings by a. moss · 7 min read";

/// What the essay stops to say.
const QUOTE: &str = "\u{201c}A lichen is not a plant. It is an agreement, kept for a very \
                     long time.\u{201d}";

/// What it is filed under.
const TAGS: [&str; 3] = ["essays", "botany", "slow"];

/// How far one press on reading on takes a reader, in percent.
const READ_ON: u32 = 22;

/// How many have liked it before this reader.
const LIKES: u32 = 12;

/// Where sharing points.
const SHARED: &str = "https://eblack-leaf.github.io/foliage/";

/// What else there is to read.
const MORE: [&str; 3] = [
    "The shape of a leaf, traced by hand",
    "Grids that do not look like grids",
    "Colour from one ramp",
];

/// The cover's blues.
const TIDE: [(f32, f32, f32); 3] = [(0.62, 0.80, 0.95), (0.27, 0.47, 0.70), (0.15, 0.28, 0.46)];

pub(crate) struct Blog {
    tags: Vec<(Chip, bool)>,
    read_on: Chip,
    progress: Leaf,
    progress_said: Leaf,
    read: u32,
    like: Chip,
    liked: bool,
    keep: Chip,
    kept: bool,
    share: Chip,
    shared: Ping,
    divider: Trail,
    email: lichen::Field,
    subscribe: Chip,
    subscribed: Say,
}

impl Blog {
    pub(crate) fn grow(grove: &mut Grove, page: Leaf, icons: &Icons, italic: Font) -> Self {
        let m = measure();
        let mut scatter = Scatter::new();
        let mut stack = Stack::new(page);
        stack.heading(
            grove,
            "artistic blog",
            "An essay's page: set large, quiet where it can be, and drawn in the leaf's own \
             polygons.",
            italic,
        );

        // The title, large enough to wrap on a phone, and the byline in the quiet italic.
        let body = stack.card(grove, "title", 112.0);
        grove.branch(
            body,
            Text::new(TITLE)
                .color(Palette::Ink)
                .font_size(FontSize::new().xs(24).md(30))
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new()
                    .xs(left(0.px()).width(100.pct()), top(0.px()).height(content()))),
        );
        grove.branch(
            body,
            Text::new(BYLINE)
                .color(lichen::INERT.ink)
                .font(italic)
                .font_size(lichen::caption())
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(0.px()).width(100.pct()),
                    top(100.pct() - 1.letters()).height(1.letters()),
                )),
        );

        // A cover: three puffs of polygons over a horizon, ember, blip green and tide blue, in
        // the proportions of a sun, a hill and a pond.
        let body = stack.card(grove, "cover", 150.0);
        grove.branch(
            body,
            Line::new()
                .between(
                    Point::new(0.pct(), 72.pct()),
                    Point::new(100.pct(), 72.pct()),
                )
                .weight(HAIRLINE)
                .color(Palette::Muted)
                .elevate(Elevation::up(1)),
        );
        let scheme = grove.scheme();
        let puffs = [
            (62.0, 30.0, 84.0, Ramp::of(&scheme.stops(theme::SPECIMEN))),
            (8.0, 58.0, 110.0, Ramp::of(&scheme.stops(theme::BLIP))),
            (38.0, 78.0, 64.0, Ramp::new(&TIDE)),
        ];
        for (across, down, size, ramp) in puffs {
            let pad = grove.branch(
                body,
                Stem::new()
                    .intangible()
                    .elevate(Elevation::up(2))
                    .at(Location::new().xs(
                        left(across.pct()).width(size.px()),
                        center_y(down.pct()).height(size.px()),
                    )),
            );
            Cluster::grow(grove, pad, size, &ramp, &mut scatter);
        }

        // A quote the essay stops for, with a rule in the accent beside it.
        let body = stack.card(grove, "pull quote", 64.0);
        grove.branch(
            body,
            Line::new()
                .between(Point::new(2.px(), 0.px()), Point::new(2.px(), 100.pct()))
                .weight(HAIRLINE * 2.5)
                .cap(Cap::Round)
                .color(Palette::Accent.mark())
                .elevate(Elevation::up(1)),
        );
        grove.branch(
            body,
            Text::new(QUOTE)
                .color(Palette::Ink)
                .font(italic)
                .font_size(lichen::caption())
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    left(18.px()).right(100.pct()),
                    center_y(50.pct()).height(content()),
                )),
        );

        // What it is filed under, each a press that follows the tag.
        let body = stack.card(grove, "tags", m.height);
        let mut along = 0.px();
        let mut tags = Vec::with_capacity(TAGS.len());
        for name in TAGS {
            let chip = parts::chip(
                grove,
                body,
                in_lane(left(along.clone()).width(Chip::width(name)), 0),
                icons.tag,
                name,
            );
            along = along + Chip::width(name) + m.gap.px();
            tags.push((chip, false));
        }

        // How far through a reader is, moved along by a press rather than by the scroll, since a
        // showcase is not an essay.
        let body = stack.card(grove, "reading progress", lane(1) + 6.0);
        let read_on = parts::chip(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("read on")), 0),
            icons.arrow_right,
            "read on",
        );
        let progress_said = parts::words(
            grove,
            body,
            "",
            Location::new().xs(
                right(100.pct()).width(content()),
                center_y((m.height / 2.0).px()).height(1.letters()),
            ),
            lichen::INERT.ink,
        );
        let track = grove.branch(
            body,
            Panel::new()
                .color(Palette::Surface)
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .elevate(Elevation::up(1))
                .at(at(left(0.px()).width(100.pct()), lane(1), 6.0)),
        );
        let progress = grove.branch(
            track,
            Panel::new()
                .color(Palette::Accent.mark())
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .elevate(Elevation::up(1))
                .at(read_to(0)),
        );

        // What a reader does with it: likes it, keeps it, passes it on.
        let body = stack.card(grove, "reactions", m.height);
        let like = parts::chip(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("000")), 0),
            icons.heart,
            &LIKES.to_string(),
        );
        let keep = parts::mark(
            grove,
            body,
            in_lane(
                left(Chip::width("000") + m.gap.px()).width(m.height.px()),
                0,
            ),
            icons.bookmark,
        );
        let share = parts::chip(
            grove,
            body,
            in_lane(
                right(100.pct() - (m.height + 4.0).px()).width(Chip::width("share")),
                0,
            ),
            icons.share_2,
            "share",
        );
        let shared = Ping::bare(
            grove,
            body,
            in_lane(right(100.pct()).width(m.height.px()), 0),
            icons.check,
        );

        // Who wrote it: a disc in the ember with their initials, a name, and a line about them.
        let body = stack.card(grove, "author", 56.0);
        let avatar = grove.branch(
            body,
            Polygon::circle()
                .color(theme::HUES[1])
                .intangible()
                .elevate(Elevation::up(1))
                .at(at(left(0.px()).width(56.px()), 0.0, 56.0)),
        );
        grove.branch(
            avatar,
            Text::new("am")
                .color(theme::ON_HUE)
                .font_size(FontSize::new().xs(18))
                .intangible()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    center_x(50.pct()).width(content()),
                    center_y(50.pct()).height(content()),
                )),
        );
        line(grove, body, "a. moss", 72.0, 6.0, Palette::Ink);
        parts::words(
            grove,
            body,
            "Draws leaves, and writes about slow things.",
            Location::new().xs(
                left(72.px()).right(100.pct()),
                top(28.px()).height(content()),
            ),
            lichen::INERT.ink,
        );

        // A break in the page, as a string of beads.
        let body = stack.card(grove, "divider", 24.0);
        let mut divider = Trail::new(Ramp::of(&scheme.stops(theme::SPECIMEN)));
        divider.grow(
            grove,
            body,
            body,
            (0.pct().into(), 50.pct().into()),
            (100.pct().into(), 50.pct().into()),
            24,
            &mut scatter,
        );
        divider.expand(grove);

        let body = stack.card(grove, "subscribe", lane(1) + 16.0);
        let email = lichen::Field::grow(
            grove,
            body,
            in_lane(
                left(0.px()).right(100.pct() - Chip::width("subscribe") - m.gap.px()),
                0,
            ),
            6.0,
            "email",
            false,
        );
        let mut subscribe = parts::chip(
            grove,
            body,
            in_lane(right(100.pct()).width(Chip::width("subscribe")), 0),
            icons.mail,
            "subscribe",
        );
        subscribe.arm(grove, Press::Inert);
        let subscribed = Say::grow(
            grove,
            body,
            at(left(0.px()).width(100.pct()), lane(1), 16.0),
            Some(italic),
        );

        // What else there is, a line each, each pointing on.
        let body = stack.card(grove, "more stories", MORE.len() as f32 * 24.0 - 8.0);
        for (n, title) in MORE.iter().enumerate() {
            let down = n as f32 * 24.0;
            line(grove, body, *title, 0.0, down, lichen::REST.ink);
            grove.branch(
                body,
                Icon::new(icons.arrow_right)
                    .color(lichen::INERT.ink)
                    .intangible()
                    .elevate(Elevation::up(1))
                    .at(at(right(100.pct()).width(14.px()), down + 1.0, 14.0)),
            );
        }

        let mut blog = Self {
            tags,
            read_on,
            progress,
            progress_said,
            read: 0,
            like,
            liked: false,
            keep,
            kept: false,
            share,
            shared,
            divider,
            email,
            subscribe,
            subscribed,
        };
        blog.reading(grove);
        blog
    }

    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        self.divider.frame(pollen);
        for (chip, followed) in &mut self.tags {
            if chip.pressed(pollen) {
                *followed = !*followed;
                chip.arm(grove, chosen(*followed));
            }
        }
        if self.read_on.pressed(pollen) {
            self.read = match self.read >= 100 {
                true => 0,
                false => (self.read + READ_ON).min(100),
            };
            self.reading(grove);
        }
        if self.like.pressed(pollen) {
            self.liked = !self.liked;
            self.like.arm(grove, chosen(self.liked));
            grove.text(self.like.label(), (LIKES + self.liked as u32).to_string());
        }
        if self.keep.pressed(pollen) {
            self.kept = !self.kept;
            self.keep.arm(grove, chosen(self.kept));
        }
        self.shared.frame(grove, pollen);
        if self.share.pressed(pollen) {
            grove.copy(SHARED);
            self.shared.fire(grove);
        }
        if self.email.frame(grove, pollen) {
            let looks = self.email.value(grove).contains('@');
            self.subscribe.arm(grove, Press::armed_if(looks));
        }
        if (self.subscribe.pressed(pollen) || pollen.submitted(self.email.input()))
            && self.email.value(grove).contains('@')
        {
            self.subscribed.tell(
                grove,
                "subscribed. one letter a month, no more.",
                Voice::Hint,
            );
            self.email.clear(grove);
            self.subscribe.arm(grove, Press::Inert);
        }
    }

    /// Fills the progress bar to how far the reader is, and says so.
    fn reading(&mut self, grove: &mut Grove) {
        grove.animate(
            self.progress,
            Motion::Location(read_to(self.read)),
            lichen::timing(),
        );
        let said = match self.read {
            0 => "not started".to_string(),
            100 => "finished".to_string(),
            read => format!("{read}% read"),
        };
        grove.text(self.progress_said, said);
        self.read_on.arm(
            grove,
            match self.read >= 100 {
                true => Press::Chosen,
                false => Press::Rest,
            },
        );
    }
}

/// Chosen where `on`, at rest where not.
fn chosen(on: bool) -> Press {
    match on {
        true => Press::Chosen,
        false => Press::Rest,
    }
}

/// The progress bar's fill at `read` percent.
fn read_to(read: u32) -> Location {
    Location::new().xs(
        left(0.px()).width((read as f32).pct()),
        top(0.px()).bottom(100.pct()),
    )
}
