//! The internals tip: the engine, as one frame read top to bottom.
//!
//! A rail down the left with a node on it for every system, in the order `fern::run` runs them.
//! Under each node, what it does, and what it hands the next one on down the rail. What writes into
//! the frame from off it -- a worker, an asset arriving -- joins the rail from the side just before
//! the queue it writes into. The frame is a loop twice over, and both are said where they happen
//! rather than drawn back up the page: resolution leaves the box stack the next dispatch reads, and
//! the last node hands on to the first.
//!
//! Every placement is read off the one before it, so a note that wraps to four lines on a phone and
//! one on a desk moves everything under it either way, and nothing is ever drawn over anything.
//! A press on a node lights it and the rail on either side of it; a press on a part of the frame in
//! the controls lights every system in it.

use core::f32::consts::{FRAC_PI_2, PI};

use foliage::{
    Boxed, Corners, Ease, Elevation, Field, Font, Grove, Grow, Icon, Leaf, Location, Motion,
    Palette, Panel, Place, Pollen, Polygon, Rounding, Source, Text, Timing, anchor, center_x,
    center_y, content, left, top,
};
use lichen::{Chip, Mosaic, Press};

use crate::icons::Icons;
use crate::parts::{self, Stack};
use crate::theme::GAP;

/// Where the rail runs, from the left edge, and how thick it is.
const RAIL: f32 = 18.0;
const WEIGHT: f32 = 1.5;

/// How far in from the rail a node's note, and what the node hands on, stand.
const INDENT: f32 = 40.0;

/// How tall a node is, and how wide at most. How wide a note is at most: a line of reading.
const NODE: f32 = 36.0;
const NODE_WIDEST: f32 = 320.0;
const NOTE_WIDEST: f32 = 540.0;

/// The room above a node, under its note, and under what it hands on.
const ABOVE: f32 = 22.0;
const UNDER: f32 = 8.0;
const HANDED: f32 = 4.0;

/// How large an arrowhead is.
const HEAD: f32 = 9.0;

/// How long one thing takes to arrive, and how far apart arrivals are down the rail.
const ARRIVE_MS: u64 = 280;
const ARRIVE_STEP: u64 = 28;

/// One system.
struct System {
    /// What its node says.
    name: &'static str,
    /// What its note says.
    note: &'static str,
    /// What it hands the next system on the rail, if it says. The last one's is where the frame
    /// goes after it.
    hands: &'static str,
    /// Whether it joins the rail from the side rather than standing on it.
    feeds: bool,
}

const SYSTEMS: [System; 13] = [
    System {
        name: "willow · the window",
        note: "Step 1, intake. Window and input events become input state, and the clock is \
               sampled once, here.",
        hands: "events",
        feeds: false,
    },
    System {
        name: "interaction · dispatch",
        note: "Step 2. What is under a point, read from the box stack the last frame left; who \
               receives it, from what asked to.",
        hands: "pollen",
        feeds: false,
    },
    System {
        name: "root · the app",
        note: "Step 3. The app reads Pollen and settled state, and queues ops against the names it \
               holds. Nothing is drawn from here.",
        hands: "ops",
        feeds: false,
    },
    System {
        name: "sprig",
        note: "A worker thread or a promise, writing through the same queue.",
        hands: "",
        feeds: true,
    },
    System {
        name: "asset",
        note: "Fonts, marks and pictures, arriving as ops -- from a file or a URL alike.",
        hands: "",
        feeds: true,
    },
    System {
        name: "queue · drain",
        note: "Step 4. The single apply: every op, first in, first out, whatever it came from.",
        hands: "plants and writes",
        feeds: false,
    },
    System {
        name: "tree · the ecs",
        note: "One element is one bevy_ecs entity, kept inside the engine. An app holds names, \
               never borrows.",
        hands: "",
        feeds: false,
    },
    System {
        name: "aspen · animate",
        note: "Step 5. Tweens advance, and a moving placement resolves both of its ends every \
               frame.",
        hands: "tweened values",
        feeds: false,
    },
    System {
        name: "rowan · resolve",
        note: "Steps 6 and 7: measure, place, scroll, clip, rank, inherit. What it leaves is the \
               box stack the next frame's dispatch reads.",
        hands: "resolved boxes",
        feeds: false,
    },
    System {
        name: "elm · extract",
        note: "Step 8. What resolved is compared with what the backend holds, and only what \
               differs becomes an instance.",
        hands: "changed instances",
        feeds: false,
    },
    System {
        name: "ash · render",
        note: "Step 9. One renderer per kind -- panel, text, icon, line, polygon, image -- and \
               every batch is applied.",
        hands: "draws",
        feeds: false,
    },
    System {
        name: "ginkgo · the gpu",
        note: "The device and the surface. Logical pixels everywhere above; the scale factor is \
               applied here, and only here.",
        hands: "a frame",
        feeds: false,
    },
    System {
        name: "photosynthesize",
        note: "The platform's loop around all of it, which first asks whether another frame is \
               owed at all.",
        hands: "and back to willow, for the next",
        feeds: false,
    },
];

/// Where the queue stands among [`SYSTEMS`]: what everything that feeds the rail writes into.
const QUEUE: usize = 5;

/// The parts of the frame the controls light, and the systems each is.
const PARTS: [(&str, &[usize]); 5] = [
    ("input", &[0, 1]),
    ("the app", &[2, 3, 4, 5]),
    ("resolve", &[6, 7, 8]),
    ("draw", &[9, 10, 11]),
    ("loop", &[12]),
];

/// What the controls hold, one to a row.
pub(crate) fn column() -> Vec<&'static str> {
    PARTS.iter().map(|&(name, _)| name).collect()
}

/// One system, grown.
struct Node {
    ground: Leaf,
    name: Leaf,
    mark: Leaf,
    note: Leaf,
    hands: Option<Leaf>,
}

/// A stretch of the rail, or a feed into it, grown: its strokes and its head, and the two systems
/// it runs between.
struct Link {
    from: usize,
    to: usize,
    parts: Vec<Leaf>,
}

/// The internals tip, grown, and what is lit.
pub(crate) struct Internals {
    legend: Vec<Chip>,
    /// Which of the legend is lit, where one is.
    part: Option<usize>,
    nodes: Vec<Node>,
    links: Vec<Link>,
    lit: [bool; SYSTEMS.len()],
    /// Everything on the rail, top to bottom, for it to arrive in that order.
    order: Vec<Leaf>,
    arrived: bool,
}

impl Internals {
    pub(crate) fn grow(
        grove: &mut Grove,
        controls: Leaf,
        details: Leaf,
        icons: &Icons,
        italic: Font,
    ) -> Self {
        let presses: Vec<_> = PARTS
            .iter()
            .zip([
                icons.mouse_pointer,
                icons.code,
                icons.layout,
                icons.layers,
                icons.refresh_cw,
            ])
            .map(|(&(name, _), mark)| (mark, name))
            .collect();
        let legend = parts::chips(grove, controls, &presses);

        let mut stack = Stack::new(details);
        stack.heading(
            grove,
            "internals",
            "One frame of the engine, top to bottom. Press a system, or a part of the frame, to \
             light it.",
            italic,
        );
        let marks = marks(icons);
        let mut nodes = Vec::with_capacity(SYSTEMS.len());
        let mut links = Vec::new();
        let mut order = Vec::new();
        // The rail's stretches under the last system that stood on it, waiting for the next one
        // to stand on it so they can be said to run between the two.
        let mut pending: Vec<Leaf> = Vec::new();
        let mut on_rail = 0;
        for (n, (system, mark)) in SYSTEMS.iter().zip(marks).enumerate() {
            let indent = if system.feeds { INDENT } else { 0.0 };
            let gap = match n {
                0 => 2.0 * GAP,
                _ => ABOVE,
            };
            // Down from whatever was placed last to the top of this node.
            if let Some(above) = stack.last().filter(|_| n > 0) {
                let stretch = rail(grove, details, above, Reach::Below(gap));
                pending.push(stretch);
                order.push(stretch);
            }
            let ground = stack.place(
                grove,
                Panel::new()
                    .color(Palette::Raised)
                    .rounding(Corners::all(Rounding::Sm))
                    .interactive()
                    .opacity(0.0)
                    .elevate(Elevation::up(2)),
                left(indent.px())
                    .width(100.pct() - indent.px())
                    .at_most(NODE_WIDEST.px()),
                NODE.px(),
                gap,
            );
            order.push(ground);
            let mark = grove.branch(
                ground,
                Icon::new(mark)
                    .color(Palette::Ink)
                    .intangible()
                    .elevate(Elevation::up(1))
                    .at(Location::new().xs(
                        left(10.px()).width(16.px()),
                        center_y(50.pct()).height(16.px()),
                    )),
            );
            let name = grove.branch(
                ground,
                Text::new(system.name)
                    .color(Palette::Ink)
                    .intangible()
                    .elevate(Elevation::up(1))
                    .at(Location::new().xs(
                        left(34.px()).right(100.pct() - 8.px()),
                        center_y(50.pct()).height(1.letters()),
                    )),
            );
            match system.feeds {
                // Across from the rail into the side of the node, headed at the rail: what it
                // writes goes on down with everything else.
                true => {
                    // The rail runs on past a node that feeds it, beside it rather than behind.
                    let stretch = rail(grove, details, ground, Reach::Alongside(0.0));
                    pending.push(stretch);
                    order.push(stretch);
                    let into = feed(grove, details, ground);
                    order.extend(&into);
                    links.push(Link {
                        from: n,
                        to: QUEUE,
                        parts: into,
                    });
                }
                // Down the rail into the top of the node, from whatever stood on it last.
                false => {
                    if n > 0 {
                        let head = head(grove, details, ground);
                        pending.push(head);
                        order.push(head);
                        links.push(Link {
                            from: on_rail,
                            to: n,
                            parts: std::mem::take(&mut pending),
                        });
                    }
                    on_rail = n;
                }
            }
            let last = n + 1 == SYSTEMS.len();
            let note = stack.place(
                grove,
                Text::new(system.note)
                    .color(lichen::INERT.ink)
                    .font(italic)
                    .font_size(lichen::caption())
                    .intangible()
                    .opacity(0.0)
                    .elevate(Elevation::up(1)),
                left((indent + INDENT).px())
                    .width(100.pct() - (indent + INDENT).px())
                    .at_most(NOTE_WIDEST.px()),
                content(),
                UNDER,
            );
            order.push(note);
            if !last {
                let stretch = rail(grove, details, note, Reach::Alongside(UNDER));
                pending.push(stretch);
                order.push(stretch);
            }
            let hands = (!system.hands.is_empty()).then(|| {
                let hands = stack.place(
                    grove,
                    Text::new(system.hands)
                        .color(lichen::INERT.ink)
                        .font_size(lichen::caption())
                        .intangible()
                        .opacity(0.0)
                        .elevate(Elevation::up(1)),
                    left(INDENT.px()).width(content()),
                    1.letters(),
                    HANDED,
                );
                order.push(hands);
                if !last {
                    let stretch = rail(grove, details, hands, Reach::Alongside(HANDED));
                    pending.push(stretch);
                    order.push(stretch);
                }
                hands
            });
            nodes.push(Node {
                ground,
                name,
                mark,
                note,
                hands,
            });
        }
        Self {
            legend,
            part: None,
            nodes,
            links,
            lit: [false; SYSTEMS.len()],
            order,
            arrived: false,
        }
    }

    /// The tip was chosen: the rail arrives, top to bottom, the first time.
    pub(crate) fn opened(&mut self, grove: &mut Grove) {
        if self.arrived {
            return;
        }
        self.arrived = true;
        for (n, &leaf) in self.order.iter().enumerate() {
            grove.animate(
                leaf,
                Motion::Opacity(1.0),
                Timing::ms(ARRIVE_MS)
                    .after(Mosaic::CHANGE_MS + n as u64 * ARRIVE_STEP)
                    .ease(Ease::Decelerate),
            );
        }
    }

    /// Carries the tip for a frame. What was just lit, if something was: the first node of it, to
    /// be brought into view.
    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> Option<Leaf> {
        let mut lit = [false; SYSTEMS.len()];
        let part = match self.legend.iter().position(|chip| chip.pressed(pollen)) {
            // A second press on what is lit puts it out.
            Some(n) => {
                let part = (self.part != Some(n)).then_some(n);
                for &system in part.map(|n| PARTS[n].1).unwrap_or_default() {
                    lit[system] = true;
                }
                part
            }
            None => {
                let n = self
                    .nodes
                    .iter()
                    .position(|node| pollen.clicked(node.ground))?;
                let alone = self.lit[n] && self.lit.iter().filter(|&&on| on).count() == 1;
                lit[n] = !alone;
                None
            }
        };
        self.light(grove, lit, part);
        let first = lit.iter().position(|&on| on)?;
        Some(self.nodes[first].ground)
    }

    /// Lights every system in `lit` -- its node, its note, what it hands on -- and every stretch
    /// of the rail between two systems either of which is lit, and puts out the rest.
    fn light(&mut self, grove: &mut Grove, lit: [bool; SYSTEMS.len()], part: Option<usize>) {
        let timing = lichen::timing();
        for (node, &on) in self.nodes.iter().zip(&lit) {
            let (ground, ink) = match on {
                true => (Palette::Accent, Palette::Contrast),
                false => (Palette::Raised, Palette::Ink),
            };
            let read = match on {
                true => Motion::from(Palette::Ink),
                false => Motion::from(lichen::INERT.ink),
            };
            grove.animate(node.ground, ground.into(), timing);
            grove.animate(node.name, ink.into(), timing);
            grove.animate(node.mark, ink.into(), timing);
            grove.animate(node.note, read.clone(), timing);
            if let Some(hands) = node.hands {
                let handed = match on {
                    true => Motion::from(Palette::Accent.mark()),
                    false => Motion::from(lichen::INERT.ink),
                };
                grove.animate(hands, handed, timing);
            }
        }
        for link in &self.links {
            let fill = match lit[link.from] || lit[link.to] {
                true => Palette::Accent.mark(),
                false => Palette::Muted,
            };
            for &part in &link.parts {
                grove.animate(part, fill.into(), timing);
            }
        }
        for (n, chip) in self.legend.iter_mut().enumerate() {
            chip.arm(
                grove,
                match part == Some(n) {
                    true => Press::Chosen,
                    false => Press::Rest,
                },
            );
        }
        self.lit = lit;
        self.part = part;
    }
}

/// The marks each system's node is marked with, in [`SYSTEMS`]' order.
fn marks(icons: &Icons) -> [Field; SYSTEMS.len()] {
    [
        icons.monitor,
        icons.mouse_pointer,
        icons.code,
        icons.zap,
        icons.image,
        icons.list,
        icons.git_branch,
        icons.wind,
        icons.layout,
        icons.filter,
        icons.layers,
        icons.cpu,
        icons.refresh_cw,
    ]
}

/// How far a stretch of the rail runs, against what it is placed by.
enum Reach {
    /// Beside it, from this far over its top -- the room it was placed under -- to its foot.
    Alongside(f32),
    /// Under it, from its foot this far down -- to the top of what was placed under it.
    Below(f32),
}

/// A stretch of the rail, placed by `by`. Behind everything, so a node standing on the rail
/// covers it.
fn rail(grove: &mut Grove, under: Leaf, by: Leaf, reach: Reach) -> Leaf {
    let down = match reach {
        Reach::Alongside(above) => top(anchor().top() - above.px()).bottom(anchor().bottom()),
        Reach::Below(gap) => top(anchor().bottom()).bottom(anchor().bottom() + gap.px()),
    };
    grove.branch(
        under,
        Panel::new()
            .color(Palette::Muted)
            .intangible()
            .opacity(0.0)
            .elevate(Elevation::up(1))
            .anchored(by)
            .at(Location::new().xs(left((RAIL - WEIGHT / 2.0).px()).width(WEIGHT.px()), down)),
    )
}

/// The head of the rail where it reaches the top of `node`, pointing down into it.
fn head(grove: &mut Grove, under: Leaf, node: Leaf) -> Leaf {
    grove.branch(
        under,
        Polygon::new()
            .sides(3.0)
            .rounding(0.12)
            .rotation(PI)
            .color(Palette::Muted)
            .intangible()
            .opacity(0.0)
            .elevate(Elevation::up(3))
            .anchored(node)
            .at(Location::new().xs(
                center_x(RAIL.px()).width(HEAD.px()),
                center_y(anchor().top() - (HEAD / 2.0).px()).height(HEAD.px()),
            )),
    )
}

/// A feed from the side of `node` across to the rail, headed at the rail.
fn feed(grove: &mut Grove, under: Leaf, node: Leaf) -> Vec<Leaf> {
    let stroke = grove.branch(
        under,
        Panel::new()
            .color(Palette::Muted)
            .intangible()
            .opacity(0.0)
            .elevate(Elevation::up(1))
            .anchored(node)
            .at(Location::new().xs(
                left(RAIL.px()).right(anchor().left()),
                center_y(anchor().center_y()).height(WEIGHT.px()),
            )),
    );
    let head = grove.branch(
        under,
        Polygon::new()
            .sides(3.0)
            .rounding(0.12)
            .rotation(-FRAC_PI_2)
            .color(Palette::Muted)
            .intangible()
            .opacity(0.0)
            .elevate(Elevation::up(3))
            .anchored(node)
            .at(Location::new().xs(
                center_x((RAIL + HEAD / 2.0 + 1.0).px()).width(HEAD.px()),
                center_y(anchor().center_y()).height(HEAD.px()),
            )),
    );
    vec![stroke, head]
}
