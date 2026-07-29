//! The site's shared vocabulary: design tokens, the entrance motion, and the content
//! column everything is written into.
//!
//! Values follow Material 3's scales rather than being picked per call site -- a 4px
//! spacing grid, a type scale with real steps between roles, and colors named by role
//! instead of hue. `Color::slate(n)` is already a tonal palette in M3's sense, so `role`
//! below just names the tones this app should use.

pub(crate) mod cards;
pub(crate) mod drawer;
pub(crate) mod figure;
pub(crate) mod hero;
pub(crate) mod overview;
pub(crate) mod rail;
pub(crate) mod shell;
pub(crate) mod stub;

use foliage::{
    Anchor, Animation, Color, ConfigurationDescriptor, Ease, EcsExtension, Elevation, Entity,
    FontId, FontSize, Grid, GridExt, HorizontalAlignment, HrefLink, Icon, IconId,
    InteractionListener, InteractionPropagation, InteractionShape, Leaf, Location, OnClick,
    Opacity, Panel, Polygon, Rounding, Sprout, Text, TextContentHeight, Tree, Trigger,
    VerticalAlignment, anchor,
};

/// Roles return whole `Color`s rather than tone numbers, so the palette genuinely lives
/// here -- as bare tones, every call site still spelled out `Color::slate(..)`, and changing
/// hue meant touching all of them.
///
/// Neutrals are `stone`: warm and sandy, where `slate` carries a blue cast.
pub(crate) mod role {
    use foliage::Color;

    /// Raised surfaces -- the rail, the drawer's menu backing.
    pub(crate) fn surface() -> Color {
        Color::stone(800)
    }
    /// Large filled areas -- cards. One step off the page and warm where the background is
    /// neutral, so a card is a sand-toned plane rather than an empty outline.
    ///
    /// Deliberately not [`surface`]: at card size that tone is a slab, and the accent has to
    /// stay the brightest thing on the page.
    pub(crate) fn surface_container() -> Color {
        Color::stone(900)
    }
    /// Hairlines and card borders. Present, never competing.
    pub(crate) fn outline() -> Color {
        Color::stone(600)
    }
    /// Headings and anything that must read first.
    pub(crate) fn on_surface() -> Color {
        Color::stone(200)
    }
    /// The page title only. A step down from [`on_surface`] because at `DISPLAY` size the
    /// full tone shouts -- a title that large is already the loudest thing on the page from
    /// its size alone, and does not need the brightness too.
    pub(crate) fn on_surface_title() -> Color {
        Color::stone(400)
    }
    /// Prose, captions, inactive rail entries.
    pub(crate) fn on_surface_variant() -> Color {
        Color::stone(400)
    }
    /// The one accent: active rail entry, primary actions, motif shapes. Scarce enough that
    /// it always means "this one".
    pub(crate) fn accent() -> Color {
        Color::orange(400)
    }
    /// Text and icons sitting *on* the accent.
    pub(crate) fn on_accent() -> Color {
        Color::stone(950)
    }
}

pub(crate) mod type_scale {
    pub(crate) const DISPLAY: u32 = 32;
    pub(crate) const HEADLINE: u32 = 22;
    pub(crate) const TITLE: u32 = 16;
    /// A page's opening paragraph. One step over body -- enough that it is visibly the first
    /// thing to read, not so much that a paragraph starts behaving like a heading.
    pub(crate) const LEAD: u32 = 16;
    pub(crate) const BODY: u32 = 14;
    pub(crate) const LABEL: u32 = 12;
}

/// The prose italic, registered once at startup.
///
/// A `OnceLock` because the id has to reach the route builders, and they only ever get a
/// `Tree` -- there is no resource read from `Commands`, and threading a `FontId` through every
/// `build(tree, slot)` signature to reach two call sites is worse than a global that is
/// written once before the first route runs.
static ITALIC: std::sync::OnceLock<foliage::FontId> = std::sync::OnceLock::new();

/// Registers the site's fonts. Must run before any route builds.
///
/// `JetBrainsMonoNL-MediumItalic` -- the italic cut of the family `foliage` already bundles,
/// at the same weight, so prose set in it stays the same colour and rhythm on the page and
/// only the slant changes. NL is the no-ligatures variant, matching the bundled default.
pub(crate) fn register_fonts(foliage: &mut foliage::Foliage) {
    let id =
        foliage.font(include_bytes!("../assets/fonts/JetBrainsMonoNL-MediumItalic.ttf").as_slice());
    let _ = ITALIC.set(id);
}

/// The registered italic, or the default font if [`register_fonts`] has not run.
fn italic() -> foliage::FontId {
    ITALIC.get().copied().unwrap_or_default()
}

/// How far past its last element a page can scroll. Roughly a thumb's worth on a phone, so
/// the final row of buttons is never pinned against the bottom edge while you are reading it.
pub(crate) const SCROLL_TAIL: i32 = 96;

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
    /// Long enough to watch a shape resolve. The morph is the site's one piece of real
    /// character, and at a third of a second it was over before it registered -- a fade
    /// with extra steps.
    pub(crate) const ENTRANCE: u64 = 1400;
    pub(crate) const FADE: u64 = 260;
    /// Each successive element in a group starts this much later. Small enough to read as
    /// one gesture rather than a queue.
    pub(crate) const STAGGER: u64 = 90;
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

/// How far a badge reaches past the corner it is badged onto -- half the backdrop, plus slack
/// so the outermost pixel is never flush with the clip it sits in.
///
/// A cell has to reserve this much beyond its card on the sides the badge overhangs. On the
/// 4px grid, and sized from the badge rather than picked: change [`cutout_badge`]'s size and
/// this must move with it.
pub(crate) const BADGE_OVERHANG: i32 = 20;

/// A shape badged onto a corner, ringed in the page background so it reads as *punched
/// through* the surface rather than stuck on top of it.
///
/// Three layers: the surface, then a backdrop shape in the background tone slightly larger on
/// every side, then the badge itself. The backdrop is the whole trick -- it interrupts the
/// surface's edge, so the eye reads a hole with something sitting in it. Interrupting the edge
/// is the effect; a badge that stays politely inside the card is just a badge.
///
/// `cell` is a box reaching [`BADGE_OVERHANG`] past the card on the top and right, and the
/// badge is centered on the card's corner *inside* it. That indirection is the point: the
/// badge has to cross the card's boundary, so it cannot be the card's child, and the obvious
/// alternative -- parenting it to the scroll container with an `Anchor` back at the card --
/// anchors across a scroll boundary and desyncs under the wheel, stranding badges at their
/// pre-scroll positions while the cards move out from under them. A cell keeps the whole thing
/// local: one clip, no anchor, and nothing to fall out of step with.
///
/// Interaction passes through both layers -- they are decoration, and a shape that swallows
/// the scroll wheel where it happens to sit is a worse bug than the one it was drawn to fix.
pub(crate) fn cutout_badge(
    tree: &mut Tree,
    cell: Entity,
    sides: f32,
    size: i32,
    seq: Entity,
    start: u64,
) -> Entity {
    let ring = space::XS;
    let backdrop_size = size + ring * 2;
    // Both layers share one center -- the card's own top-right corner -- so the backdrop rings
    // the badge evenly. By center rather than by edge precisely because the two differ in
    // size: matching their right and top edges would seat the larger one off to one side.
    let corner = |w: i32| {
        Location::new().xs(
            100.pct()
                .as_center_x()
                .adjust(-BADGE_OVERHANG)
                .with(w.px().as_width()),
            0.pct()
                .as_center_y()
                .adjust(BADGE_OVERHANG)
                .with(w.px().as_height()),
        )
    };
    let backdrop = tree.branch(
        cell,
        Polygon::new()
            .sides(sides)
            .rounding(0.35)
            .rotation(0.0)
            .color(background())
            .at(corner(backdrop_size))
            .elevate(Elevation::up(3))
            .with((Opacity::new(0.0), InteractionPropagation::pass_through())),
    );
    let badge = tree.branch(
        cell,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(-0.12)
            .color(role::accent())
            .at(corner(size))
            .elevate(Elevation::up(4))
            .with((Opacity::new(0.0), InteractionPropagation::pass_through())),
    );
    // the backdrop just appears -- morphing it would animate the hole itself, which reads
    // as the card tearing rather than as something arriving in it
    fade_in(tree, backdrop, seq, start);
    morph_in(tree, badge, seq, sides, 0.35, start);
    badge
}

/// The site's one button: a shadowed polygon with an icon in it and a label beneath.
///
/// The offset shadow gives depth without a blur, and the shape morphing in is the site's
/// signature. Shared rather than duplicated so the hero and the sections cannot drift into
/// two different-looking buttons.
pub(crate) struct PolyButton {
    pub(crate) label: &'static str,
    pub(crate) icon: crate::icons::IconHandles,
    pub(crate) href: &'static str,
    /// Final side count. Each button in a row gets its own, so they resolve into visibly
    /// different shapes rather than one repeated three times.
    pub(crate) sides: f32,
    pub(crate) face: Color,
}

/// Button diameter, and the row height a caller should reserve for one plus its label.
pub(crate) const POLY_BUTTON: i32 = 56;
pub(crate) const POLY_BUTTON_ROW_H: i32 = POLY_BUTTON + space::SM + 24;
const POLY_SHADOW_OFF: i32 = 7;
const POLY_ICON_SCALE: f32 = 0.44;

/// Places one at `center_pct` across `row`, which should be [`POLY_BUTTON_ROW_H`] tall.
pub(crate) fn poly_button(
    tree: &mut Tree,
    row: Entity,
    spec: &PolyButton,
    center_pct: f32,
    seq: Entity,
    start: u64,
) -> Entity {
    let shadow = tree.branch(
        row,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(-0.16)
            .color(role::surface())
            .at(Location::new().xs(
                center_pct
                    .pct()
                    .as_center_x()
                    .adjust(-POLY_SHADOW_OFF)
                    .with(POLY_BUTTON.px().as_width()),
                POLY_SHADOW_OFF
                    .px()
                    .as_top()
                    .with(POLY_BUTTON.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .with(Opacity::new(0.0)),
    );
    morph_in(tree, shadow, seq, spec.sides, 0.15, start);

    let button = tree.branch(
        row,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(-0.16)
            .color(spec.face)
            .at(Location::new().xs(
                center_pct
                    .pct()
                    .as_center_x()
                    .with(POLY_BUTTON.px().as_width()),
                0.px().as_top().with(POLY_BUTTON.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with((
                InteractionListener::new(),
                InteractionShape::Circle,
                Opacity::new(0.0),
            )),
    );
    morph_in(tree, button, seq, spec.sides, 0.15, start);
    let href = spec.href;
    tree.on_click(button, move |_: Trigger<OnClick>, _: Tree| {
        HrefLink::new(href).navigate();
    });

    let icon = tree.branch(
        row,
        Icon::new(IconId::from(spec.icon))
            .color(role::on_accent())
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .with((anchor().width() * POLY_ICON_SCALE).as_width()),
                anchor()
                    .center_y()
                    .as_center_y()
                    .with((anchor().height() * POLY_ICON_SCALE).as_height()),
            ))
            .elevate(Elevation::up(4))
            .with((
                Anchor::new(button),
                // the icon draws above the button, so without this it wins the hit-test and
                // swallows the click meant for the shape under it
                InteractionPropagation::pass_through(),
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, icon, seq, start);

    let label = tree.branch(
        row,
        Text::new(spec.label)
            .size(FontSize::new(type_scale::TITLE))
            .color(role::on_surface_variant())
            .at(Location::new().xs(
                center_pct.pct().as_center_x().with(90.px().as_width()),
                anchor()
                    .bottom()
                    .as_top()
                    .adjust(space::SM)
                    .with(24.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with((
                HorizontalAlignment::Center,
                VerticalAlignment::Middle,
                Anchor::new(button),
                Opacity::new(0.0),
            )),
    );
    fade_in(tree, label, seq, start);
    button
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
    fn text(
        &mut self,
        tree: &mut Tree,
        value: &str,
        size: u32,
        tone: Color,
        gap: i32,
        font: FontId,
    ) -> Entity {
        let start = self.stagger();
        let entity = tree.branch(
            self.parent,
            Text::new(value)
                .size(FontSize::new(size))
                .color(tone)
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
                    font,
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
            role::on_surface_title(),
            space::XL,
            FontId::default(),
        )
    }
    /// A section heading within the page.
    pub(crate) fn heading(&mut self, tree: &mut Tree, value: &str) -> Entity {
        self.text(
            tree,
            value,
            type_scale::HEADLINE,
            role::on_surface(),
            space::XL,
            FontId::default(),
        )
    }
    /// A paragraph, in the italic. Wraps freely -- everything below follows it down.
    ///
    /// The slant is the site's only way to separate prose from everything else: in a
    /// monospaced family there is no proportional cut to switch to, and weight is already
    /// spent on the type scale. It also reads as commentary next to the upright labels and
    /// numbers in the plates, which is what prose on these pages is.
    pub(crate) fn prose(&mut self, tree: &mut Tree, value: &str) -> Entity {
        self.text(
            tree,
            value,
            type_scale::BODY,
            role::on_surface_variant(),
            space::MD,
            italic(),
        )
    }
    /// A page's opening paragraph: set larger than body prose and marked with an accent rule
    /// down its left edge.
    ///
    /// Deliberately not a filled surface. The slabs on these pages are the card grids, and an
    /// opener built as one more slab reads as the first card rather than as the lead -- so
    /// the emphasis here is a rule and a type step, which are the two things a slab is not.
    pub(crate) fn lead(&mut self, tree: &mut Tree, value: &str) -> Entity {
        let start = self.stagger();
        let entity = tree.branch(
            self.parent,
            Text::new(value)
                .size(FontSize::new(type_scale::LEAD).short(type_scale::TITLE))
                .color(role::on_surface())
                .at(Location::new().xs(
                    space::LG.px().as_left().with(100.pct().as_right()),
                    self.below(space::MD, type_scale::LEAD as i32 + space::SM),
                ))
                .elevate(Elevation::up(2))
                .with((
                    HorizontalAlignment::Left,
                    VerticalAlignment::Top,
                    TextContentHeight(true),
                    self.anchor_to_last(),
                    Opacity::new(0.0),
                    italic(),
                )),
        );
        fade_in(tree, entity, self.seq, start);
        // Anchored to the paragraph rather than given a height: the wrap count changes with
        // every breakpoint, and a rule that stops short of the text it marks looks like a
        // mistake at exactly the width nobody tested.
        let rule = tree.branch(
            self.parent,
            Panel::new()
                .color(role::accent())
                .rounding(Rounding::None)
                .at(Location::new().xs(
                    // width first: the resolver takes `(Width, Right)`, not `(Right, Width)`
                    3.px()
                        .as_width()
                        .with(anchor().left().as_right().adjust(-space::SM)),
                    anchor().top().as_top().with(anchor().bottom().as_bottom()),
                ))
                .elevate(Elevation::up(2))
                .with((Anchor::new(entity), Opacity::new(0.0))),
        );
        fade_in(tree, rule, self.seq, start);
        self.last = Some(entity);
        entity
    }
    /// A hairline across the column. Separates one kind of content from another without
    /// spending a heading on it.
    pub(crate) fn rule(&mut self, tree: &mut Tree) -> Entity {
        let start = self.stagger();
        let entity = tree.branch(
            self.parent,
            Panel::new()
                .color(role::outline())
                .rounding(Rounding::None)
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    self.below(space::XL, 1),
                ))
                .elevate(Elevation::up(1))
                .with((self.anchor_to_last(), Opacity::new(0.0))),
        );
        fade_in(tree, entity, self.seq, start);
        self.last = Some(entity);
        entity
    }
    /// A bare region in the stack -- no surface of its own, just space the caller owns.
    /// A bare region whose height differs by breakpoint -- for content that reflows into
    /// fewer rows as the column widens, like a card grid going one-up to two-up.
    ///
    /// All three steps are spelled out because the two callers reflow at different widths: a
    /// plate just gets taller at `md`, while the card grid stays one-up until `lg`.
    pub(crate) fn region(&mut self, tree: &mut Tree, heights: (i32, i32, i32), gap: i32) -> Entity {
        let start = self.stagger();
        let full = || 0.pct().as_left().with(100.pct().as_right());
        let entity = tree.branch(
            self.parent,
            Leaf::sprout()
                .at(Location::new()
                    .xs(full(), self.below(gap, heights.0))
                    .md(full(), self.below(gap, heights.1))
                    .lg(full(), self.below(gap, heights.2)))
                .elevate(Elevation::up(1))
                .with((
                    Grid::new(1.col().gap(0), 1.row().gap(0)),
                    self.anchor_to_last(),
                )),
        );
        let _ = start;
        self.last = Some(entity);
        entity
    }
    pub(crate) fn surface_plain(&mut self, tree: &mut Tree, height: i32, gap: i32) -> Entity {
        let start = self.stagger();
        let entity = tree.branch(
            self.parent,
            Panel::new()
                .color(background())
                .rounding(Rounding::Xs)
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
    /// Empty space after the last element, so a page can be scrolled clear of its own bottom
    /// edge.
    ///
    /// `extent_check` grows the scroll range to cover the content and no further, so the last
    /// thing on a page ends flush with the viewport -- there is no overscroll to give it
    /// breathing room. This is that room, made of the only thing the extent understands: more
    /// content.
    ///
    /// Interaction passes through, or a full-width invisible box below the buttons would eat
    /// clicks aimed at whatever the layout puts near it.
    pub(crate) fn tail(&mut self, tree: &mut Tree, height: i32) -> Entity {
        let entity = tree.branch(
            self.parent,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    self.below(0, height),
                ))
                .elevate(Elevation::up(1))
                .with((
                    self.anchor_to_last(),
                    InteractionPropagation::pass_through(),
                )),
        );
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
            role::on_surface_variant(),
            space::SM,
            italic(),
        )
    }
}
