//! The site's shared vocabulary: design tokens, the entrance motion, and the content
//! column everything is written into.
//!
//! Values follow Material 3's scales rather than being picked per call site -- a 4px
//! spacing grid, a type scale with real steps between roles, and colors named by role
//! instead of hue. `Color::slate(n)` is already a tonal palette in M3's sense, so `role`
//! below just names the tones this app should use.

pub(crate) mod drawer;
pub(crate) mod hero;
pub(crate) mod overview;
pub(crate) mod rail;
pub(crate) mod shell;
pub(crate) mod stub;

use foliage::{
    Anchor, Animation, Color, ConfigurationDescriptor, Ease, EcsExtension, Elevation, Entity,
    FontSize, Grid, GridExt, HorizontalAlignment, HrefLink, Icon, IconId, InteractionListener,
    InteractionPropagation, InteractionShape, Location, OnClick, Opacity, Panel, Polygon, Rounding,
    Sprout, Text, TextContentHeight, Tree, Trigger, VerticalAlignment, anchor,
};

/// Roles return whole `Color`s rather than tone numbers, so the palette genuinely lives
/// here -- as bare tones, every call site still spelled out `Color::slate(..)`, and changing
/// hue meant touching all of them.
///
/// Neutrals are `stone`: warm and sandy, where `slate` carries a blue cast.
pub(crate) mod role {
    use foliage::Color;

    /// Raised surfaces -- cards, the rail.
    pub(crate) fn surface() -> Color {
        Color::stone(800)
    }
    /// Hairlines and card borders. Present, never competing.
    pub(crate) fn outline() -> Color {
        Color::stone(600)
    }
    /// Headings and anything that must read first.
    pub(crate) fn on_surface() -> Color {
        Color::stone(200)
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

/// A shape badged onto a card's corner, ringed in the page background so it reads as
/// *punched through* the card rather than stuck on top of it.
///
/// Three layers: the card, then a backdrop shape in the background tone slightly larger on
/// every side, then the badge itself. The backdrop is the whole trick -- it interrupts the
/// card's edge, so the eye reads a hole with something sitting in it.
///
/// `parent` should be a container *wider* than the card -- the scroll container, not the
/// measured column -- since the badge overhangs the card's edge and would otherwise be
/// sliced by the column's own box.
///
/// Deliberately not `ClipToViewport`: that marker also resets the entity's elevation prefix
/// into the front overlay tier (see `coordinate/elevation.rs`), which floats the badges over
/// everything including the hero. It is for dropdowns and popovers, not for a shape that
/// merely overhangs its neighbour.
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
            .with((Anchor::new(card), Opacity::new(0.0))),
    );
    let badge = tree.branch(
        parent,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(-0.12)
            .color(role::accent())
            .at(corner(size))
            .elevate(Elevation::up(4))
            .with((Anchor::new(card), Opacity::new(0.0))),
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
    fn text(&mut self, tree: &mut Tree, value: &str, size: u32, tone: Color, gap: i32) -> Entity {
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
            role::on_surface(),
            space::XL,
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
        )
    }
    /// A paragraph. Wraps freely -- everything below follows it down.
    pub(crate) fn prose(&mut self, tree: &mut Tree, value: &str) -> Entity {
        self.text(
            tree,
            value,
            type_scale::BODY,
            role::on_surface_variant(),
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
        self.panel(tree, height, gap, Some(role::outline()))
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
            role::on_surface_variant(),
            space::SM,
        )
    }
}
