//! What the pointer is resting on.

use foliage::{Grove, Leaf, Position, Sap, Section, Vein};

/// The hover layer lichen does not have: which of a set of watched elements the cursor is over,
/// and when that changes.
///
/// foliage reports no hover -- a pointer passing over an element is not a gesture -- but it does
/// say where a hovering pointer is ([`Grove::cursor`]), and every element says where it was drawn
/// ([`Vein::Drawn`]). This reads the one against the other, once a frame.
///
/// It knows only what it is told about what covers what: `Drawn` is where an element was drawn,
/// not whether something was drawn over it. So the owner suspends it while an overlay is up or a
/// drag is underway, by handing [`frame`](Hover::frame) `open: false`, and what was under the
/// pointer is left as it would be if the pointer had gone.
///
/// On a touchscreen there is no hovering pointer, so nothing is ever entered. A hover effect is
/// decoration on top of a press, never the only way to reach something.
pub struct Hover {
    watched: Vec<Leaf>,
    over: Option<Leaf>,
    entered: Option<Leaf>,
    left: Option<Leaf>,
}

impl Hover {
    /// An empty hover layer. Asks the engine to run a frame for every move of the cursor, which a
    /// hover needs and an engine without one would rather not do.
    pub fn new(grove: &mut Grove) -> Self {
        grove.rouse_on_cursor(true);
        Self {
            watched: Vec::new(),
            over: None,
            entered: None,
            left: None,
        }
    }

    /// Watches `leaf`. Where two watched elements overlap, the one watched later wins, so watch in
    /// the order things are stacked.
    pub fn watch(&mut self, leaf: Leaf) {
        if !self.watched.contains(&leaf) {
            self.watched.push(leaf);
        }
    }

    /// Stops watching `leaf`, leaving it if the pointer was over it.
    pub fn unwatch(&mut self, leaf: Leaf) {
        self.watched.retain(|watched| *watched != leaf);
        if self.over == Some(leaf) {
            self.over = None;
            self.left = Some(leaf);
        }
    }

    /// Reads where the cursor is against what is watched. With `open` false, the layer is suspended:
    /// nothing is over anything until it opens again.
    pub fn frame(&mut self, grove: &Grove, open: bool) {
        self.entered = None;
        self.left = None;
        let now = match (open, grove.cursor()) {
            (true, Some(cursor)) => {
                let boxes = self.watched.iter().filter_map(|&leaf| {
                    let shown = !matches!(grove.tap(leaf, Vein::Visible), Some(Sap::Visible(false)));
                    let seen = !matches!(grove.tap(leaf, Vein::Opacity), Some(Sap::Opacity(o)) if o <= 0.0);
                    match (shown && seen, grove.tap(leaf, Vein::Drawn)) {
                        (true, Some(Sap::Section(drawn))) => Some((leaf, drawn)),
                        _ => None,
                    }
                });
                topmost(boxes, cursor)
            }
            _ => None,
        };
        if now != self.over {
            self.left = self.over;
            self.entered = now;
            self.over = now;
        }
    }

    /// What the pointer is over, if anything watched.
    pub fn over(&self) -> Option<Leaf> {
        self.over
    }

    /// Whether the pointer is over `leaf`.
    pub fn is_over(&self, leaf: Leaf) -> bool {
        self.over == Some(leaf)
    }

    /// Whether the pointer came onto `leaf` this frame.
    pub fn entered(&self, leaf: Leaf) -> bool {
        self.entered == Some(leaf)
    }

    /// Whether the pointer left `leaf` this frame.
    pub fn left(&self, leaf: Leaf) -> bool {
        self.left == Some(leaf)
    }

    /// Whatever the pointer came onto this frame.
    pub fn arrived(&self) -> Option<Leaf> {
        self.entered
    }

    /// Whatever the pointer left this frame.
    pub fn departed(&self) -> Option<Leaf> {
        self.left
    }
}

/// The last of `boxes` containing `at`: what a pointer at `at` is over, when later boxes stand in
/// front of earlier ones.
pub fn topmost<T: Copy>(boxes: impl IntoIterator<Item = (T, Section)>, at: Position) -> Option<T> {
    boxes
        .into_iter()
        .filter(|(_, section)| section.contains(at))
        .last()
        .map(|(item, _)| item)
}

#[cfg(test)]
mod tests {
    use super::topmost;
    use foliage::{Area, Position, Section};

    fn boxed(x: f32, y: f32, w: f32, h: f32) -> Section {
        Section::new(Position::new(x, y), Area::new(w, h))
    }

    #[test]
    fn the_later_box_stands_in_front() {
        let boxes = [(1, boxed(0.0, 0.0, 100.0, 100.0)), (2, boxed(50.0, 50.0, 100.0, 100.0))];
        assert_eq!(topmost(boxes, Position::new(75.0, 75.0)), Some(2));
        assert_eq!(topmost(boxes, Position::new(10.0, 10.0)), Some(1));
        assert_eq!(topmost(boxes, Position::new(400.0, 10.0)), None);
    }
}
