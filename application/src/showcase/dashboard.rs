//! A dashboard: how a service is doing, over a range of time, and the few things done about it.

use foliage::{
    Boxed, Cap, Corners, Elevation, Font, FontSize, Grove, Grow, HAIRLINE, Leaf, Line, Location,
    Motion, Palette, Panel, Place, Point, Pollen, Polygon, Rounding, Source, Text, Trace, center_x,
    center_y, content, left, right, top,
};
use lichen::{
    Badge, Chip, Confirm, Pick, Ping, Press, Say, Scatter, Step, Switch, Voice, gate, measure,
};

use crate::icons::Icons;
use crate::parts::{self, Stack, at, in_lane, lane, line};

/// The ranges the dashboard reads over.
const RANGES: [&str; 4] = ["1h", "24h", "7d", "30d"];

/// What each tile says over each range: requests, error rate, and p95 latency, each with how far it
/// moved and whether that is good.
const TILES: [[(&str, &str, bool); 3]; 4] = [
    [
        ("1.2k", "+4%", true),
        ("0.3%", "-0.1", true),
        ("212ms", "+18", false),
    ],
    [
        ("31k", "+9%", true),
        ("0.4%", "+0.1", false),
        ("198ms", "-6", true),
    ],
    [
        ("204k", "+12%", true),
        ("0.2%", "-0.2", true),
        ("205ms", "-2", true),
    ],
    [
        ("880k", "+31%", true),
        ("0.5%", "+0.3", false),
        ("231ms", "+25", false),
    ],
];

/// How tall a tile is: what it is, the number, and how far it moved, one under the other -- a
/// tile is a third of a phone's width, which has no room for two of them side by side.
const TILE: f32 = 80.0;

/// What each tile is.
const NAMES: [&str; 3] = ["requests", "errors", "p95"];

/// How many points the line is drawn through.
const POINTS: usize = 18;

/// The services, and whether each is well.
const SERVICES: [(&str, bool); 3] = [("api", true), ("worker", false), ("database", true)];

/// The flags that can be flipped.
const FLAGS: [&str; 3] = ["new checkout", "dark launch", "beta search"];

/// What a deploy goes through.
const DEPLOY: [&str; 3] = ["build", "test", "ship"];

/// What happened lately, newest first.
const ACTIVITY: [(&str, &str); 4] = [
    ("12:04", "deploy 482 shipped by ana"),
    ("11:52", "worker queue depth over 1k"),
    ("11:30", "flag beta search turned on"),
    ("10:17", "database failover drill passed"),
];

pub(crate) struct Dashboard {
    range: Pick,
    tiles: Vec<(Leaf, Leaf)>,
    segments: Vec<Leaf>,
    dot: Leaf,
    services: Vec<(Chip, Option<Badge>)>,
    spike: Chip,
    alarm: Ping,
    alert: Say,
    flags: Vec<Switch>,
    deploy: Vec<Chip>,
    deployed: [bool; DEPLOY.len()],
    shipped: Say,
    restart: Confirm,
    restarted: Say,
}

impl Dashboard {
    pub(crate) fn grow(grove: &mut Grove, page: Leaf, icons: &Icons, italic: Font) -> Self {
        let m = measure();
        let mut stack = Stack::new(page);
        stack.heading(
            grove,
            "dashboard",
            "How a service is doing over a range of time, and the few things done about it.",
            italic,
        );

        let body = stack.card(grove, "time range", m.height);
        let range = Pick::grow(grove, body, Location::new(), 6.0, "range", &RANGES, 1);

        // Three numbers, each with how far it moved under it: the move in the positive hue where that is
        // good news, and the danger hue where it is not.
        let body = stack.card(grove, "at a glance", TILE);
        let tiles = NAMES
            .iter()
            .enumerate()
            .map(|(n, &name)| {
                let third = 100.0 / NAMES.len() as f32;
                let tile = grove.branch(
                    body,
                    Panel::new()
                        .color(Palette::Surface)
                        .rounding(Corners::all(Rounding::Sm))
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(at(
                            left((n as f32 * third).pct() + (n as f32 * m.gap / 3.0).px())
                                .width(third.pct() - (2.0 * m.gap / 3.0).px()),
                            0.0,
                            TILE,
                        )),
                );
                line(grove, tile, name, 10.0, 8.0, lichen::INERT.ink);
                let value = grove.branch(
                    tile,
                    Text::new("")
                        .color(Palette::Ink)
                        .font_size(FontSize::new().xs(18))
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new().xs(
                            left(10.px()).width(content()),
                            top(26.px()).height(content()),
                        )),
                );
                let delta = parts::words(
                    grove,
                    tile,
                    "",
                    Location::new().xs(
                        left(10.px()).width(content()),
                        top((TILE - 24.0).px()).height(1.letters()),
                    ),
                    Palette::Positive.mark(),
                );
                (value, delta)
            })
            .collect();

        // Requests over the range, as a line through the points and a dot on the latest.
        let body = stack.card(grove, "requests / min", 96.0);
        grove.branch(
            body,
            Line::new()
                .between(
                    Point::new(0.pct(), 100.pct()),
                    Point::new(100.pct(), 100.pct()),
                )
                .weight(HAIRLINE)
                .color(Palette::Muted)
                .elevate(Elevation::up(1)),
        );
        let first = series(0);
        let segments = first
            .windows(2)
            .map(|pair| {
                grove.branch(
                    body,
                    Line::new()
                        .trace(traced(pair[0], pair[1]))
                        .weight(HAIRLINE * 2.0)
                        .cap(Cap::Round)
                        .color(Palette::Accent.mark())
                        .elevate(Elevation::up(2)),
                )
            })
            .collect();
        let dot = grove.branch(
            body,
            Polygon::circle()
                .color(Palette::Accent.mark())
                .intangible()
                .elevate(Elevation::up(3))
                .at(spot(first[POINTS - 1])),
        );

        // The services, one to a row, with a dot on whatever is not well until it is seen to.
        let body = stack.card(grove, "services", parts::lane(SERVICES.len()) - m.gap);
        let services = SERVICES
            .iter()
            .enumerate()
            .map(|(n, &(name, well))| {
                let state = match well {
                    true => "healthy",
                    false => "degraded",
                };
                let chip = parts::chip(
                    grove,
                    body,
                    in_lane(left(0.px()).width(100.pct()), n),
                    icons.server,
                    &format!("{name} · {state}"),
                );
                let badge = (!well).then(|| {
                    let mut badge = Badge::on(grove, &chip);
                    badge.show(grove, true);
                    badge
                });
                (chip, badge)
            })
            .collect();

        // An alert, as it arrives: a mark that goes by itself, and the line that stays until the
        // next thing happens.
        let body = stack.card(grove, "alerts", lane(1) + 16.0);
        let spike = parts::chip(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("simulate spike")), 0),
            icons.trending_up,
            "simulate spike",
        );
        let alarm = parts::ping(
            grove,
            body,
            in_lane(
                left(Chip::width("simulate spike") + 4.px()).width(m.height.px()),
                0,
            ),
            icons.alert_triangle,
        );
        let alert = Say::grow(
            grove,
            body,
            at(left(0.px()).width(100.pct()), lane(1), 16.0),
            Some(italic),
        );

        let body = stack.card(grove, "feature flags", lane(FLAGS.len()) - m.gap);
        let flags = FLAGS
            .iter()
            .enumerate()
            .map(|(n, &name)| {
                Switch::grow(
                    grove,
                    body,
                    in_lane(left(0.px()).width(Switch::width(name)), n),
                    Elevation::up(1),
                    name,
                )
            })
            .collect();

        let body = stack.card(grove, "deploy", lane(1) + 16.0);
        let marks = [icons.r#box, icons.check, icons.git_branch];
        let deploy = DEPLOY
            .iter()
            .zip(marks)
            .enumerate()
            .map(|(n, (&name, mark))| {
                let third = 100.0 / DEPLOY.len() as f32;
                let across = left((n as f32 * third).pct() + (n as f32 * m.gap / 3.0).px())
                    .width(third.pct() - (2.0 * m.gap / 3.0).px());
                parts::chip(grove, body, in_lane(across, 0), mark, name)
            })
            .collect();
        let shipped = Say::grow(
            grove,
            body,
            at(left(0.px()).width(100.pct()), lane(1), 16.0),
            Some(italic),
        );

        let body = stack.card(grove, "restart", lane(1) + 16.0);
        let mut restart = Confirm::grow(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("restart api")), 0),
            in_lane(right(100.pct()).width(Chip::width("restart")), 0),
            (icons.power, "restart api"),
            (icons.x, "keep"),
            (icons.refresh_cw, "restart"),
        );
        restart.open(grove, true);
        let restarted = Say::grow(
            grove,
            body,
            at(left(0.px()).width(100.pct()), lane(1), 16.0),
            Some(italic),
        );

        // What happened lately: when, in the grey between, and what, in ink.
        let body = stack.card(grove, "activity", ACTIVITY.len() as f32 * 22.0 - 6.0);
        for (n, &(when, what)) in ACTIVITY.iter().enumerate() {
            let down = n as f32 * 22.0;
            line(grove, body, when, 0.0, down, lichen::INERT.ink);
            line(grove, body, what, 56.0, down, lichen::REST.ink);
        }

        let mut dashboard = Self {
            range,
            tiles,
            segments,
            dot,
            services,
            spike,
            alarm,
            alert,
            flags,
            deploy,
            deployed: [false; DEPLOY.len()],
            shipped,
            restart,
            restarted,
        };
        dashboard.read(grove, 0, false);
        dashboard.ship(grove);
        dashboard
    }

    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if self.range.frame(grove, pollen) {
            self.read(grove, self.range.index(), true);
        }
        for (n, (chip, badge)) in self.services.iter_mut().enumerate() {
            if let Some(shown) = badge
                && chip.pressed(pollen)
                && shown.shown()
            {
                shown.show(grove, false);
                grove.text(chip.label(), format!("{} · seen to", SERVICES[n].0));
            }
        }
        self.alarm.frame(grove, pollen);
        if self.spike.pressed(pollen) {
            self.alarm.fire(grove);
            self.alert
                .tell(grove, "p95 over 400ms on api, 11:58.", Voice::Warn);
        }
        for flag in &mut self.flags {
            flag.frame(grove, pollen);
        }
        if let Some(n) = self.deploy.iter().position(|chip| chip.pressed(pollen)) {
            match self.deployed.iter().all(|&done| done) {
                true => self.deployed = [false; DEPLOY.len()],
                false => self.deployed[n] = true,
            }
            self.ship(grove);
        }
        match self.restart.frame(grove, pollen) {
            Some(Step::Confirmed) => self.restarted.tell(grove, "api restarting.", Voice::Hint),
            Some(Step::Kept) => self.restarted.set(grove, "left running."),
            Some(Step::Armed) => self.restarted.clear(grove),
            None => {}
        }
    }

    /// Reads the dashboard over range `n`: the tiles say what it came to, and the line is drawn
    /// through it -- moved there, where `moved`, rather than put.
    fn read(&mut self, grove: &mut Grove, n: usize, moved: bool) {
        for ((value, delta), &(said, moved_by, good)) in self.tiles.iter().zip(&TILES[n]) {
            grove.text(*value, said);
            grove.text(*delta, moved_by);
            let fill = match good {
                true => Palette::Positive.mark(),
                false => Palette::Danger.mark(),
            };
            grove.color(*delta, fill);
        }
        let points = series(n);
        for (segment, pair) in self.segments.iter().zip(points.windows(2)) {
            let trace = traced(pair[0], pair[1]);
            match moved {
                true => {
                    grove.animate(*segment, Motion::Trace(trace), lichen::timing());
                }
                false => grove.trace(*segment, trace),
            }
        }
        let last = spot(points[POINTS - 1]);
        match moved {
            true => {
                grove.animate(self.dot, Motion::Location(last), lichen::timing());
            }
            false => grove.at(self.dot, last),
        }
    }

    /// Dresses the deploy steps for which have run, and says what came of them once all have.
    fn ship(&mut self, grove: &mut Grove) {
        let all = self.deployed.iter().all(|&done| done);
        for (n, (chip, press)) in self.deploy.iter_mut().zip(gate(&self.deployed)).enumerate() {
            let press = match (all, n == DEPLOY.len() - 1) {
                (true, true) => Press::Chosen,
                _ => press,
            };
            chip.arm(grove, press);
        }
        match all {
            true => self.shipped.tell(grove, "deploy 483 is live.", Voice::Hint),
            false => self.shipped.set(grove, "press each step as it passes."),
        }
    }
}

/// The line over range `n`, as points across the card: `x` and `y` both fractions of it, `y` from
/// the top. A walk from the scatter, run further along for each range, so every range is its own
/// line and the same line every time it is read.
fn series(n: usize) -> [(f32, f32); POINTS] {
    let mut scatter = Scatter::new();
    for _ in 0..n * POINTS {
        scatter.next();
    }
    let mut level = 0.55;
    let mut points = [(0.0, 0.0); POINTS];
    for (k, point) in points.iter_mut().enumerate() {
        level = (level + scatter.between(-0.16, 0.14)).clamp(0.12, 0.88);
        *point = (k as f32 / (POINTS - 1) as f32, level);
    }
    points
}

/// A segment of the line, as the ends a stroke is placed by.
fn traced(from: (f32, f32), to: (f32, f32)) -> Trace {
    let point = |(x, y): (f32, f32)| Point::new((x * 100.0).pct(), (y * 100.0).pct());
    Trace::new().xs(point(from), point(to))
}

/// The dot on the latest point.
fn spot((x, y): (f32, f32)) -> Location {
    Location::new().xs(
        center_x((x * 100.0).pct()).width(8.px()),
        center_y((y * 100.0).pct()).height(8.px()),
    )
}
