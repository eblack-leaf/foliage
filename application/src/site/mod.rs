//! The site's shared vocabulary: design tokens, the entrance motion, and the content
//! column everything is written into.
//!
//! Values follow Material 3's scales rather than being picked per call site -- a 4px
//! spacing grid, a type scale with real steps between roles, and colors named by role
//! instead of hue. `Color::slate(n)` is already a tonal palette in M3's sense, so `role`
//! below just names the tones this app should use.

pub(crate) mod hero;
pub(crate) mod overview;
pub(crate) mod rail;
pub(crate) mod shell;
pub(crate) mod stub;

use foliage::{
    Anchor, Animation, ClipToViewport, Color, ConfigurationDescriptor, Ease, EcsExtension,
    Elevation, Entity,
    FontSize, Grid, GridExt, HorizontalAlignment, Location, Opacity, Panel, Polygon, Rounding,
    Sprout, Text, TextContentHeight, Tree, VerticalAlignment, anchor,
};

/// Named by role, so retheming is this block rather than every call site.
pub(crate) mod role {
    /// Raised surfaces -- cards, the rail.
    pub(crate) const SURFACE: i32 = 800;
    /// Hairlines and card borders. Present, never competing.
    pub(crate) const OUTLINE: i32 = 600;
    /// Headings and anything that must read first.
    pub(crate) const ON_SURFACE: i32 = 200;
    /// Prose, captions, inactive rail entries.
    pub(crate) const ON_SURFACE_VARIANT: i32 = 400;
}
/// The one accent. Used for the active rail entry, primary actions, and the motif shapes --
/// scarce enough that it always means "this one".
pub(crate) const ACCENT: i32 = 400; // green

pub(crate) mod type_scale {
    pub(crate) const DISPLAY: u32 = 32;
    pub(crate) const HEADLINE: u32 = 22;
    pub(crate) const TITLE: u32 = 16;
    pub(crate) const BODY: u32 = 14;
    pub(crate) const LABEL: u32 = 12;
}

/// 4px grid. Every offset on the site should be one of these.
pub(crate) mod space {
    pub(crate) const XS: i32 = 4;
    pub(crate) const SM: i32 = 8;
    pub(crate) const MD: i32 = 16;
    pub(crate) const LG: i32 = 24;
    pub(crate) const XL: i32 = 40;
}

/// Crisp, not floaty. Entrances are short and decisively eased; nothing drifts in from
/// offscreen -- things resolve in place, from a rough shape into their real one.
pub(crate) mod motion {
    pub(crate) const ENTRANCE: u64 = 340;
    pub(crate) const FADE: u64 = 200;
    /// Each successive element in a group starts this much later. Small enough to read as
    /// one gesture rather than a queue.
    pub(crate) const STAGGER: u64 = 60;
}

/// The site's entrance: a polygon resolves from a rough triangle into its real shape while
/// unwinding a slight rotation, in place.
///
/// In place is the point -- nothing slides in from offscreen. The shape is already where it
/// belongs and simply becomes itself, which reads as emphatic rather than as travel.
pub(crate) fn morph_in(
    tree: &mut Tree,
    entity: Entity,
    seq: Entity,
    sides: f32,
    rounding: f32,
    start: u64,
) {
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(entity)
            .during(seq)
            .start(start)
            .finish(start + motion::FADE)
            .eased(Ease::Linear),
    );
    tree.animate(
        Animation::new(Polygon {
            sides,
            rounding,
            rotation: 0.0,
        })
        .targeting(entity)
        .during(seq)
        .start(start)
        .finish(start + motion::ENTRANCE)
        .eased(Ease::EMPHASIS),
    );
}

/// The page background, which the renderer clears to (`ash/mod.rs`'s `color_attachment`).
/// [`cutout_badge`] depends on matching it exactly -- if the clear color moves, this must
/// move with it or the cutouts stop reading as holes.
pub(crate) fn background() -> Color {
    Color::gray(900)
}

/// A shape badged onto a card's corner, ringed in the page background so it reads as
/// *punched through* the card rather than stuck on top of it.
///
/// Three layers: the card, then a backdrop shape in the background tone slightly larger on
/// every side, then the badge itself. The backdrop is the whole trick -- it interrupts the
/// card's edge, so the eye reads a hole with something sitting in it.
///
/// Both layers carry [`ClipToViewport`]: they deliberately render outside the column that
/// contains them, and without it the column's own box slices the overhang clean off -- the
/// half-badges. The marker keeps them ordinary `Stem` children (normal elevation, normal
/// removal cascade) while bounding them by the window instead of by their parent.
pub(crate) fn cutout_badge(
    tree: &mut Tree,
    parent: Entity,
    card: Entity,
    sides: f32,
    size: i32,
    seq: Entity,
    start: u64,
) -> Entity {
    let ring = space::XS;
    let backdrop_size = size + ring * 2;
    let corner = |w: i32| {
        Location::new().xs(
            anchor().right().as_center_x().with(w.px().as_width()),
            anchor().top().as_center_y().with(w.px().as_height()),
        )
    };
    let backdrop = tree.branch(
        parent,
        Polygon::new()
            .sides(sides)
            .rounding(0.35)
            .rotation(0.0)
            .color(background())
            .at(corner(backdrop_size))
            .elevate(Elevation::up(3))
            .with((Anchor::new(card), Opacity::new(0.0), ClipToViewport)),
    );
    let badge = tree.branch(
        parent,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(-0.12)
            .color(Color::green(ACCENT))
            .at(corner(size))
            .elevate(Elevation::up(4))
            .with((Anchor::new(card), Opacity::new(0.0), ClipToViewport)),
    );
    // the backdrop just appears -- morphing it would animate the hole itself, which reads
    // as the card tearing rather than as something arriving in it
    fade_in(tree, backdrop, seq, start);
    morph_in(tree, badge, seq, sides, 0.35, start);
    badge
}

/// Fades an entity in without a shape change, for text and panels.
pub(crate) fn fade_in(tree: &mut Tree, entity: Entity, seq: Entity, start: u64) {
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(entity)
            .during(seq)
            .start(start)
            .finish(start + motion::FADE)
            .eased(Ease::Linear),
    );
}

/// A vertical stack of content inside a scrollable column.
///
/// Each element anchors to the bottom of the one before it rather than sitting at a
/// computed offset, so prose that wraps to three lines on a phone pushes everything below
/// it down instead of being overlapped. Nothing here needs to know the viewport width.
pub(crate) struct Column {
    parent: Entity,
    last: Option<Entity>,
    /// Runs the entrance animations, so a whole page arrives as one staggered gesture.
    seq: Entity,
    step: u64,
}

impl Column {
    pub(crate) fn new(tree: &mut Tree, parent: Entity) -> Self {
        let seq = tree.sequence();
        Self {
            parent,
            last: None,
            seq,
            step: 0,
        }
    }
    pub(crate) fn sequence(&self) -> Entity {
        self.seq
    }
    /// The next element's vertical placement: below the previous element, or at the top of
    /// the column for the first one.
    fn below(&self, gap: i32, seed_height: i32) -> ConfigurationDescriptor {
        if self.last.is_some() {
            anchor()
                .bottom()
                .as_top()
                .adjust(gap)
                .with(seed_height.px().as_height())
        } else {
            gap.px().as_top().with(seed_height.px().as_height())
        }
    }
    fn anchor_to_last(&self) -> Anchor {
        Anchor::new(self.last.unwrap_or(self.parent))
    }
    fn stagger(&mut self) -> u64 {
        let at = self.step;
        self.step += motion::STAGGER;
        at
    }
    fn text(&mut self, tree: &mut Tree, value: &str, size: u32, tone: i32, gap: i32) -> Entity {
        let start = self.stagger();
        let entity = tree.branch(
            self.parent,
            Text::new(value)
                .size(FontSize::new(size))
                .color(Color::slate(tone))
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    self.below(gap, size as i32 + space::SM),
                ))
                .elevate(Elevation::up(2))
                .with((
                    HorizontalAlignment::Left,
                    VerticalAlignment::Top,
                    TextContentHeight(true),
                    self.anchor_to_last(),
                    Opacity::new(0.0),
                )),
        );
        fade_in(tree, entity, self.seq, start);
        self.last = Some(entity);
        entity
    }
    /// The page's own title. One per page.
    pub(crate) fn display(&mut self, tree: &mut Tree, value: &str) -> Entity {
        self.text(
            tree,
            value,
            type_scale::DISPLAY,
            role::ON_SURFACE,
            space::XL,
        )
    }
    /// A section heading within the page.
    pub(crate) fn heading(&mut self, tree: &mut Tree, value: &str) -> Entity {
        self.text(
            tree,
            value,
            type_scale::HEADLINE,
            role::ON_SURFACE,
            space::XL,
        )
    }
    /// A paragraph. Wraps freely -- everything below follows it down.
    pub(crate) fn prose(&mut self, tree: &mut Tree, value: &str) -> Entity {
        self.text(
            tree,
            value,
            type_scale::BODY,
            role::ON_SURFACE_VARIANT,
            space::MD,
        )
    }
    /// A card: an outlined surface of fixed height, joining the stack like anything else.
    /// Returns it for the caller to fill.
    ///
    /// Outlined rather than filled because `Rounding` is proportional (`Md` is half the
    /// short side), so a filled card at this size would read as a pill. `Xs` is the only
    /// step that looks like a corner.
    pub(crate) fn surface(&mut self, tree: &mut Tree, height: i32, gap: i32) -> Entity {
        self.panel(tree, height, gap, Some(Color::slate(role::OUTLINE)))
    }
    /// A bare region in the stack -- no surface of its own, just space the caller owns.
    pub(crate) fn surface_plain(&mut self, tree: &mut Tree, height: i32, gap: i32) -> Entity {
        self.panel(tree, height, gap, None)
    }
    fn panel(
        &mut self,
        tree: &mut Tree,
        height: i32,
        gap: i32,
        outline: Option<Color>,
    ) -> Entity {
        let start = self.stagger();
        let mut sprout = Panel::new()
            .color(outline.unwrap_or(background()))
            .rounding(Rounding::Xs);
        if outline.is_some() {
            sprout = sprout.outline(1);
        }
        let entity = tree.branch(
            self.parent,
            sprout
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    self.below(gap, height),
                ))
                .elevate(Elevation::up(1))
                .with((
                    Grid::new(1.col().gap(0), 1.row().gap(0)),
                    self.anchor_to_last(),
                    Opacity::new(0.0),
                )),
        );
        fade_in(tree, entity, self.seq, start);
        self.last = Some(entity);
        entity
    }
    /// A caption under a figure. Unused until the sections carrying figures are written.
    #[allow(dead_code)]
    pub(crate) fn caption(&mut self, tree: &mut Tree, value: &str) -> Entity {
        self.text(
            tree,
            value,
            type_scale::LABEL,
            role::ON_SURFACE_VARIANT,
            space::SM,
        )
    }
}
