//! Where the herbarium stands the key and what each of its tips opens.
//!
//! The herbarium's own reading, on what lichen's specimen hands it. The key stands at the top of
//! the room, as large as its share of the room lets it be. A chosen tip's controls stand on the
//! ground its cut cleared -- the box [`key::CONTROLS`] draws for it, just under the line, clear of
//! the mosaic and right where the key just went -- and what they open stands past them the way the
//! cut faces, out to the room's edge, where there is room: a column across the room where the cut
//! faces up or down, the room's height where it faces to a side. Longer than that, it scrolls
//! there.
//!
//! The section comes in once the key has drawn in to the tip, so the controls stand on ground and
//! not on tiles still going, and goes at once when the tip is let go.

use foliage::{
    Area, Axes, Boxed, Ease, Elevation, Grove, Grow, Leaf, Location, Motion, Place, Pollen,
    ScrollTo, Source, Stem, Timing, Tween, left, top,
};
use lichen::{Cleared, Silhouette, Specimen};

use super::key;

/// How far what a section's controls open stands off them, in pixels.
pub(crate) const STANDOFF: f32 = 24.0;

/// How wide what a section opens is at most where its cut faces up or down: a phone's width and
/// some, so on a desk a form is not a strip.
const COLUMN: f32 = 640.0;

/// How long a section takes to come in, and to go.
const FADE_MS: u64 = 260;

/// The room the key stands in, and each tip's section on it.
pub(crate) struct Sections {
    /// What everything is grown on: the room.
    page: Leaf,
    tips: Vec<Section>,
    shown: Option<usize>,
}

/// One tip's section: its controls, what they open, which scrolls, and the fade it is going out
/// on, after which both are hidden.
struct Section {
    controls: Leaf,
    details: Leaf,
    going: Option<Tween>,
}

/// A box in pixels, in the room: its left, top, width and height.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Rect(f32, f32, f32, f32);

impl Rect {
    fn location(self) -> Location {
        let Rect(x, y, width, height) = self;
        Location::new().xs(
            left(x.px()).width(width.max(0.0).px()),
            top(y.px()).height(height.max(0.0).px()),
        )
    }
}

impl Sections {
    /// The room, grown on `under`: what the key is grown into, before there are sections.
    pub(crate) fn page(grove: &mut Grove, under: Leaf) -> Leaf {
        grove.branch(under, Stem::new().at(Location::new()))
    }

    /// Where the key stands in a room `room` in size, for growing it.
    pub(crate) fn shape(room: Area, shape: &Silhouette) -> Location {
        shaped(room, shape).location()
    }

    /// Grows a section for every tip of `key`, hidden, on `page`, in a room `room` in size.
    pub(crate) fn grow(grove: &mut Grove, page: Leaf, key: &Specimen, room: Area) -> Self {
        let laid = Laid::of(room, key);
        let tips = (0..key.len())
            .map(|n| {
                // In front of the key, whose cleared ground the controls stand on. Intangible, so
                // where they are merely there they take no press meant for anything else.
                let controls = grove.branch(
                    page,
                    Stem::new()
                        .at(laid.controls[n].location())
                        .elevate(Elevation::up(4))
                        .intangible()
                        .visible(false)
                        .opacity(0.0)
                        .focus_scope(),
                );
                // Solid, not intangible: a wheel notch goes to what is on top under the pointer,
                // and over the room between what a section grew that has to be this, or the notch
                // scrolls nothing. It stands only where the cut cleared.
                let details = grove.branch(
                    page,
                    Stem::new()
                        .at(laid.details[n].location())
                        .elevate(Elevation::up(4))
                        .scrolls(Axes::Vertical)
                        .visible(false)
                        .opacity(0.0)
                        .focus_scope(),
                );
                Section {
                    controls,
                    details,
                    going: None,
                }
            })
            .collect();
        Self {
            page,
            tips,
            shown: None,
        }
    }

    /// Where tip `n`'s section grows its controls.
    pub(crate) fn controls(&self, n: usize) -> Leaf {
        self.tips[n].controls
    }

    /// Where tip `n`'s section grows what its controls open.
    pub(crate) fn details(&self, n: usize) -> Leaf {
        self.tips[n].details
    }

    /// The room is now `room` in size: the key and every section are placed again for it.
    pub(crate) fn fit(&self, grove: &mut Grove, key: &Specimen, room: Area) {
        let laid = Laid::of(room, key);
        grove.at(key.root(), laid.shape.location());
        for (n, tip) in self.tips.iter().enumerate() {
            grove.at(tip.controls, laid.controls[n].location());
            grove.at(tip.details, laid.details[n].location());
        }
    }

    /// Shows what the key has chosen, if anything: its section comes in once the key has drawn in
    /// to it, and any other goes at once.
    pub(crate) fn show(&mut self, grove: &mut Grove, chosen: Option<usize>) {
        if chosen == self.shown {
            return;
        }
        for (n, tip) in self.tips.iter_mut().enumerate() {
            let leaves = [tip.controls, tip.details];
            if chosen == Some(n) {
                tip.going = None;
                grove.scroll(tip.details, ScrollTo::start());
                for leaf in leaves {
                    grove.visible(leaf, true);
                    grove.animate(
                        leaf,
                        Motion::Opacity(1.0),
                        Timing::ms(FADE_MS)
                            .after(Specimen::GATHERED)
                            .ease(Ease::Decelerate),
                    );
                }
            } else if self.shown == Some(n) {
                for leaf in leaves {
                    grove.animate(
                        leaf,
                        Motion::Opacity(0.0),
                        Timing::ms(FADE_MS).ease(Ease::Accelerate),
                    );
                }
                tip.going = Some(grove.timer(Timing::ms(FADE_MS)));
            }
        }
        if chosen.is_none() {
            grove.scroll(self.page, ScrollTo::start());
        }
        self.shown = chosen;
    }

    /// Hides a section that was let go once it has gone. Call once a frame.
    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        for tip in &mut self.tips {
            if let Some(going) = tip.going
                && pollen.finished(going)
            {
                tip.going = None;
                for leaf in [tip.controls, tip.details] {
                    grove.visible(leaf, false);
                }
            }
        }
    }
}

/// Where the key, each tip's controls and what each opens stand, for one size of room.
struct Laid {
    shape: Rect,
    controls: Vec<Rect>,
    details: Vec<Rect>,
}

impl Laid {
    fn of(room: Area, key: &Specimen) -> Self {
        let shaped = shaped(room, key.shape());
        let Rect(x, y, width, _) = shaped;
        let controls: Vec<Rect> = key::CONTROLS
            .iter()
            .map(|&((cx, cy), (cw, ch))| {
                Rect(x + cx * width, y + cy * width, cw * width, ch * width)
            })
            .collect();
        let details = controls
            .iter()
            .enumerate()
            .map(|(n, &controls)| beyond(room, controls, key.cut(n)))
            .collect();
        Self {
            shape: shaped,
            controls,
            details,
        }
    }
}

/// The key's box: its share of the room inside the margin, as large as both sides let it be,
/// across the middle and at the top.
fn shaped(room: Area, shape: &Silhouette) -> Rect {
    let margin = key::MARGIN;
    let inner = (
        (room.width - 2.0 * margin).max(1.0),
        (room.height - 2.0 * margin).max(1.0),
    );
    let aspect = shape.aspect();
    let width = (inner.0 * key::SHARE)
        .min(inner.1 * key::SHARE / aspect)
        .max(1.0);
    Rect((room.width - width) / 2.0, margin, width, width * aspect)
}

/// Where what a section's controls open stands: past them the way the cut faces, out to the
/// room's margin. Facing up or down, a column no wider than [`COLUMN`], centred on the controls and
/// kept inside the margins; facing to a side, the room's height, margin to margin.
fn beyond(room: Area, Rect(x, y, width, height): Rect, cut: Cleared) -> Rect {
    let margin = key::MARGIN;
    let (nx, ny) = cut.facing;
    if nx.abs() > ny.abs() {
        let tall = room.height - 2.0 * margin;
        return match nx > 0.0 {
            true => {
                let from = x + width + STANDOFF;
                Rect(from, margin, room.width - margin - from, tall)
            }
            false => {
                let to = x - STANDOFF;
                Rect(margin, margin, to - margin, tall)
            }
        };
    }
    let column = (room.width - 2.0 * margin).min(COLUMN);
    let from = (x + width / 2.0 - column / 2.0).clamp(margin, room.width - margin - column);
    match ny > 0.0 {
        true => {
            let below = y + height + STANDOFF;
            Rect(from, below, column, room.height - margin - below)
        }
        false => {
            let above = y - STANDOFF;
            Rect(from, margin, column, above - margin)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cut(facing: (f32, f32)) -> Cleared {
        Cleared {
            from: (0.0, 0.0),
            to: (1.0, 0.0),
            facing,
        }
    }

    /// Facing down, what the controls open runs from under them to the room's foot, a column
    /// centred on them.
    #[test]
    fn facing_down_it_opens_below() {
        let controls = Rect(400.0, 100.0, 200.0, 80.0);
        let Rect(x, y, width, height) = beyond(Area::new(1000.0, 600.0), controls, cut((0.0, 1.0)));
        assert_eq!(y, 100.0 + 80.0 + STANDOFF);
        assert_eq!(y + height, 600.0 - key::MARGIN);
        assert_eq!(width, COLUMN);
        assert_eq!(x + width / 2.0, 500.0);
    }

    /// Facing up, from the room's top to above the controls.
    #[test]
    fn facing_up_it_opens_above() {
        let controls = Rect(400.0, 300.0, 200.0, 80.0);
        let Rect(_, y, _, height) = beyond(Area::new(1000.0, 600.0), controls, cut((0.0, -1.0)));
        assert_eq!(y, key::MARGIN);
        assert_eq!(y + height, 300.0 - STANDOFF);
    }

    /// Facing to a side, from past the controls to the margin that way, the room's height.
    #[test]
    fn facing_sideways_it_opens_to_that_side() {
        let controls = Rect(400.0, 100.0, 200.0, 80.0);
        let Rect(x, y, width, height) = beyond(Area::new(1000.0, 600.0), controls, cut((1.0, 0.0)));
        assert_eq!(x, 600.0 + STANDOFF);
        assert_eq!(x + width, 1000.0 - key::MARGIN);
        assert_eq!((y, height), (key::MARGIN, 600.0 - 2.0 * key::MARGIN));
    }

    /// On a phone, a column is the room's width inside the margins.
    #[test]
    fn a_column_keeps_inside_the_margins() {
        let controls = Rect(100.0, 100.0, 190.0, 80.0);
        let Rect(x, _, width, _) = beyond(Area::new(390.0, 844.0), controls, cut((0.0, 1.0)));
        assert_eq!((x, width), (key::MARGIN, 390.0 - 2.0 * key::MARGIN));
    }
}
