//! The site's shared vocabulary: design tokens, the entrance motion, and the content
//! column everything is written into.
//!
//! Values follow Material 3's scales rather than being picked per call site -- a 4px
//! spacing grid, a type scale with real steps between roles, and colors named by role
//! instead of hue. `Color::slate(n)` is already a tonal palette in M3's sense, so `role`
//! below just names the tones this app should use.

pub(crate) mod anim;
pub(crate) mod assets;
pub(crate) mod blueprint;
pub(crate) mod cards;
pub(crate) mod copy;
pub(crate) mod drawer;
pub(crate) mod figure;
pub(crate) mod hero;
pub(crate) mod input;
pub(crate) mod layout;
pub(crate) mod leaf;
pub(crate) mod overview;
pub(crate) mod rail;
pub(crate) mod renderers;
pub(crate) mod router;
pub(crate) mod shell;
pub(crate) mod text;

use foliage::{
    Bare, Canopy, Color, ConfigurationDescriptor, Ease, Elevation, FontId, FontSize, Grid, GridExt,
    Grows, HorizontalAlignment, Icon, IconId, Leaf, Location, LocationValue, Motion, Panel,
    Polygon, Rounding, Sprout, Text, TextInputAction, Timing, VerticalAlignment, anchor,
    text_content,
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
        Color::stone(700)
    }
    /// Large filled areas -- cards. One step off the page, so a card is a plane rather than an
    /// empty outline.
    ///
    /// Deliberately not [`surface`]: at card size that tone is a slab, and the accent has to
    /// stay the brightest thing on the page.
    pub(crate) fn surface_container() -> Color {
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
    /// Section headings. Structure, not content -- they say where you are, and the prose under
    /// them is what you came to read.
    pub(crate) fn on_surface_heading() -> Color {
        Color::stone(400)
    }
    /// The page title only. Quieter still: set in caps at `DISPLAY` size it is already the
    /// loudest thing on the page from its shape alone, and the brightness on top of that made
    /// it shout over everything it was introducing.
    pub(crate) fn on_surface_title() -> Color {
        Color::stone(500)
    }
    /// Prose and captions -- text on the page and on cards. Not on [`surface`], which is light
    /// enough that this tone stops holding against it; that takes [`on_surface`].
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
    /// A page's opening paragraph -- the same size as body.
    ///
    /// It was a step larger, and at that size a paragraph starts behaving like a heading and
    /// crowds the one under it. The lead is marked by its accent rule and its indent instead,
    /// which say "start here" without competing for the type scale.
    pub(crate) const LEAD: u32 = BODY;
    pub(crate) const BODY: u32 = 14;
    /// Was 12, which on a phone is where the type stops surviving the device grid rather than
    /// where it stops being readable. `xmin` is a whole number of device pixels, so a bearing
    /// worth ~8% of the em floors to 1px at small sizes and the gap between letters loses a
    /// third of its width -- measurably, not as an impression. Measured on the bundled face,
    /// that gap is 1px at 20 and 21 physical px and 2px at 25.
    ///
    /// 13 rather than 14 because the reference tables are what this size is mostly carrying,
    /// and 14 stopped fitting them. It keeps a step below [`BODY`] at the cost of landing
    /// nearer the low end of that range on a fractional scale factor.
    pub(crate) const LABEL: u32 = 13;
}

/// The prose italic, registered once at startup.
///
/// A `OnceLock` because the id has to reach the route builders, and a `FontId` threaded
/// through every `build(grow, slot)` signature to reach two call sites is worse than a global
/// that is written once before the first route runs.
static ITALIC: std::sync::OnceLock<FontId> = std::sync::OnceLock::new();

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
pub(crate) fn italic() -> FontId {
    ITALIC.get().copied().unwrap_or_default()
}

/// The one image the site draws, keyed once at startup. A `OnceLock` for the same reason
/// [`ITALIC`] is one -- the key has to reach a route builder, and threading an `AssetKey`
/// through every `build(grow, slot)` to reach one call site is worse than a global written
/// before the first route runs.
static SAMPLE: std::sync::OnceLock<foliage::AssetKey> = std::sync::OnceLock::new();

/// Registers the site's images. Must run before any route builds.
///
/// A 708x363 photograph -- near 2:1, and nothing like the 4:3 boxes the renderers page's
/// `fit` board puts it in, which is what makes the three views three different pictures.
///
/// Embedded rather than fetched, like the icons and the italic. `AssetSource::Bytes` is the
/// one variant that behaves identically on every platform, so the web build needs no hosting
/// convention for it.
pub(crate) fn register_assets(foliage: &mut foliage::Foliage) {
    let key = foliage.load_asset(foliage::AssetSource::Bytes(
        include_bytes!("../assets/images/sample.jpg").to_vec(),
    ));
    let _ = SAMPLE.set(key);
}

/// The registered test card's key. Zero if [`register_assets`] has not run, which names no
/// asset and draws nothing rather than failing.
pub(crate) fn sample_image() -> foliage::AssetKey {
    SAMPLE.get().copied().unwrap_or_default()
}

/// What a route's builder writes into besides the tree itself.
///
/// The engine reports *that* something was clicked or that a timer ran out; which of this
/// page's elements that `Leaf` was, and what it means, is the app's own bookkeeping. This is
/// that bookkeeping, and it is torn down and rebuilt with the page it describes -- which is
/// what keeps a stale entry from a previous section out of the current one's handlers.
#[derive(Default)]
pub(crate) struct Page {
    /// The live demos on this page, each owning its own elements and its own state.
    pub(crate) demos: Vec<Box<dyn Demo>>,
    /// Clicking one of these goes to a route.
    pub(crate) nav: Vec<(Leaf, usize)>,
    /// Clicking one of these leaves the site.
    pub(crate) links: Vec<(Leaf, &'static str)>,
    /// `(timer, target)` -- the target is out of the hit test until the timer reports. See
    /// [`arm_at`].
    pub(crate) arming: Vec<(Leaf, Leaf)>,
    /// Paths whose vertices are re-resolved as their field changes size.
    pub(crate) traverses: Vec<figure::Traverse>,
    /// The hero's live readout, on the one route that has one.
    pub(crate) readout: Option<hero::Readout>,
}

/// Where a gesture currently is, as the engine reports it.
///
/// A click is not one of these: `Clicked` is the *outcome* of a gesture that stayed put, and it
/// arrives through [`Demo::clicked`] alongside every other press on the site. These four are
/// the gesture itself, which only the `input` section has any use for.
#[derive(Copy, Clone, PartialEq)]
pub(crate) enum Phase {
    /// The pointer went down on this element.
    Engaged,
    /// It moved, however little. Every move, from the first pixel.
    Dragged,
    /// It has travelled past the drag threshold, so the release will not be a click. Once per
    /// gesture, and the only announcement of the threshold there is.
    DragStarted,
    /// It came back up, however the gesture ended.
    Disengaged,
}

/// Something on a page the reader drives.
///
/// A trait rather than one more `Option<..>` field on [`Page`], because a section that explains
/// a part of the library by letting you operate it is the shape every section is heading for,
/// and the alternative is a field per section plus a match in `entry.rs` that grows with the
/// site. A demo owns its own elements and its own state; the only thing it shares with the
/// frame is the emission.
///
/// Every hook but [`clicked`](Demo::clicked) has a default, because a board that presses one
/// property wants exactly one of them and a page should not have to write five empty methods to
/// say so. All of them answer the same way: `true` means "that was mine, stop offering it
/// around".
pub(crate) trait Demo {
    /// Answers a click. `false` if the `Leaf` is not one of this demo's own, which is what lets
    /// several demos sit on one page and the page still fall through to navigation.
    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) -> bool;
    /// Answers a sequence's completion, the same way [`clicked`](Demo::clicked) answers a
    /// press: `false` if the sequence is not one this demo opened.
    ///
    /// Named sequences all report on one channel and the page's own entrance is one of them, so
    /// a demo that cares has to hold the handle it opened and compare. Most do not care, which
    /// is why this has a default.
    fn finished(&mut self, _canopy: &mut Canopy, _seq: Leaf) -> bool {
        false
    }
    /// Answers one step of a gesture on one of this demo's elements.
    ///
    /// The `input` section is the only thing on the site that needs the phases rather than the
    /// click they add up to.
    fn gesture(&mut self, _canopy: &mut Canopy, _leaf: Leaf, _phase: Phase) -> bool {
        false
    }
    /// Answers keyboard focus arriving at or leaving one of this demo's elements.
    fn focus(&mut self, _canopy: &mut Canopy, _leaf: Leaf, _has: bool) -> bool {
        false
    }
    /// Answers an edit to one of this demo's text inputs -- typed, pasted, or written.
    fn typed(&mut self, _canopy: &mut Canopy, _leaf: Leaf, _value: &str) -> bool {
        false
    }
    /// Answers a binding one of this demo's text inputs matched. Submission is
    /// [`TextInputAction::Enter`](foliage::TextInputAction::Enter) on a single-line field.
    fn acted(&mut self, _canopy: &mut Canopy, _leaf: Leaf, _action: TextInputAction) -> bool {
        false
    }
    /// Once a frame, for anything a demo displays that only the tree knows -- a resolved box,
    /// or whether an element is still there.
    fn drive(&mut self, _canopy: &mut Canopy) {}
}

/// What a builder gets: somewhere to grow, and somewhere to record what it grew.
///
/// The two travel together because nearly everything on this site is both -- a button is a
/// shape *and* a destination, an entrance is an animation *and* a control that must not be
/// clickable until it finishes.
pub(crate) struct Grow<'a, 'w, 's> {
    pub(crate) canopy: &'a mut Canopy<'w, 's>,
    pub(crate) page: &'a mut Page,
}

impl<'a, 'w, 's> Grow<'a, 'w, 's> {
    pub(crate) fn new(canopy: &'a mut Canopy<'w, 's>, page: &'a mut Page) -> Self {
        Self { canopy, page }
    }
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

/// Where a page's title starts, measured from the top of the scroll container.
///
/// Sized from the drawer's menu button rather than picked, because that is what it is clearing:
/// at `xs` the button floats over the content in the top-left corner, and a title at [`space::XL`]
/// started 8px inside its footprint -- so every page opened with its first word under the button.
///
/// Applied at every breakpoint, not just `xs`. The button is parked off-canvas at `md`+ so the
/// clearance is unnecessary there, but a title that changes height with the window is a jump for
/// nothing, and the extra air above a page's first line reads as deliberate at any width.
pub(crate) const PAGE_TOP: i32 = drawer::MENU_FOOTPRINT + space::MD;

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

/// A tween running from `start` to `start + length` milliseconds out, on `ease`.
///
/// Every entrance on this site is expressed as an offset into the page's own arrival rather
/// than as a duration from now, which is what lets a whole page be staggered by passing one
/// number down.
pub(crate) fn timing(start: u64, length: u64, ease: Ease) -> Timing {
    Timing::over(start + length).after(start).eased(ease)
}

/// The site's entrance: a polygon resolves from a rough triangle into its real shape while
/// unwinding a slight rotation, in place.
///
/// In place is the point -- nothing slides in from offscreen. The shape is already where it
/// belongs and simply becomes itself, which reads as emphatic rather than as travel.
pub(crate) fn morph_in(
    canopy: &mut Canopy,
    leaf: Leaf,
    seq: Leaf,
    sides: f32,
    rounding: f32,
    start: u64,
) {
    fade_in(canopy, leaf, seq, start);
    canopy.animate_during(
        leaf,
        Motion::Polygon(Polygon {
            sides,
            rounding,
            rotation: 0.0,
        }),
        timing(start, motion::ENTRANCE, Ease::EMPHASIS),
        seq,
    );
}

/// The page background. Handed to the engine as its `ClearColor` in `run`, so this function
/// is the one place it is decided -- [`cutout_badge`] rings its backdrop in this same tone and
/// depends on matching the cleared surface exactly, or the cutouts stop reading as holes.
///
/// The same tone the engine defaults to -- tuned anyway, so the match is stated rather than
/// inherited.
pub(crate) fn background() -> Color {
    Color::stone(900)
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
/// badge is centered on the card's corner *inside* it. That indirection is the point: the badge
/// has to cross the card's boundary, so it cannot be the card's child.
///
/// The obvious alternative is to parent it to the scroll container and anchor it back at the
/// card -- the workaround `ClipToViewport`'s own TODO describes, forced by there being no way
/// to escape one clip without also being lifted into the front overlay tier. It puts the badge
/// on a *wider clip than the thing it belongs to*: the badge tracks its card exactly, but once
/// the card scrolls past the edge of the box that clips it, the card stops being drawn and the
/// badge does not. Badges go on floating over empty page, still perfectly in position, above
/// cards that are no longer visible.
///
/// A cell keeps the badge on the same clip as its card, so the two can only ever appear and
/// disappear together.
///
/// Interaction passes through both layers -- they are decoration, and a shape that swallows
/// the scroll wheel where it happens to sit is a worse bug than the one it was drawn to fix.
pub(crate) fn cutout_badge(
    canopy: &mut Canopy,
    cell: Leaf,
    sides: f32,
    size: i32,
    seq: Leaf,
    start: u64,
) -> Leaf {
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
    let backdrop = canopy.branch(
        cell,
        Polygon::new()
            .sides(sides)
            .rounding(0.35)
            .rotation(0.0)
            .color(background())
            .at(corner(backdrop_size))
            .elevate(Elevation::up(3))
            .opacity(0.0)
            .pass_through(),
    );
    let badge = canopy.branch(
        cell,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(-0.12)
            .color(role::accent())
            .at(corner(size))
            .elevate(Elevation::up(4))
            .opacity(0.0)
            .pass_through(),
    );
    // the backdrop just appears -- morphing it would animate the hole itself, which reads
    // as the card tearing rather than as something arriving in it
    fade_in(canopy, backdrop, seq, start);
    morph_in(canopy, badge, seq, sides, 0.35, start);
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
    /// Handed to [`HrefLink`](foliage::HrefLink), which is a browser escape hatch rather than
    /// any kind of route: it synthesises an anchor click on the web and compiles away to
    /// nothing everywhere else. A button carrying one is inert on desktop and Android.
    pub(crate) href: &'static str,
    /// Final side count. Each button in a row gets its own, so they resolve into visibly
    /// different shapes rather than one repeated three times.
    pub(crate) sides: f32,
    pub(crate) face: Color,
}

/// The same control, answering to the page it sits on instead of leaving the site.
///
/// The split is between the control and what pressing it means: this is the whole button, and
/// [`PolyButton`] is this plus a destination. A demo keeps the `Leaf` it gets back and answers
/// the press itself.
pub(crate) struct PolyAction {
    pub(crate) label: &'static str,
    pub(crate) icon: crate::icons::IconHandles,
    pub(crate) sides: f32,
    pub(crate) face: Color,
}

/// Button diameter, and the row height a caller should reserve for one plus its label.
pub(crate) const POLY_BUTTON: i32 = 56;
pub(crate) const POLY_BUTTON_ROW_H: i32 = POLY_BUTTON + space::SM + 24;
const POLY_SHADOW_OFF: i32 = 7;
/// Floor on a poly control's tap target, wider than [`POLY_STEP`] draws -- a drawn shape that
/// size reads fine in a row of four, but a *hit box* that size is missed on a phone more often
/// than it should be. [`POLY_BUTTON`] already clears this, so only the step row is affected.
const MIN_HIT: i32 = 44;
const POLY_ICON_SCALE: f32 = 0.44;

/// Places one at `center_pct` across `row`, which should be [`POLY_BUTTON_ROW_H`] tall, and
/// records it as leaving the site.
pub(crate) fn poly_button(
    g: &mut Grow,
    row: Leaf,
    spec: &PolyButton,
    center_pct: f32,
    seq: Leaf,
    start: u64,
) -> Leaf {
    let button = poly_action(
        g,
        row,
        &PolyAction {
            label: spec.label,
            icon: spec.icon,
            sides: spec.sides,
            face: spec.face,
        },
        center_pct,
        seq,
        start,
    );
    g.page.links.push((button, spec.href));
    button
}

/// The shape and the offset shadow under it -- everything a poly control is before whatever it
/// carries. Returns `(shadow, button)`; only the button takes a press.
///
/// Shared by both sizes rather than written twice, so the one thing that makes these read as
/// the site's own button -- an unblurred shadow at a fixed offset, and a triangle resolving
/// into the real shape -- cannot drift between the hero's destinations and a board's steps.
///
/// `button` is a plain, undrawn tap target rather than the visible triangle itself, sized to at
/// least [`MIN_HIT`] and centered on the same point the triangle resolves to. The two are
/// separate entities because they want different sizes for different reasons -- the triangle's
/// is a drawing decision, the tap target's is a thumb's -- and one entity can only have one.
fn poly_shape(
    g: &mut Grow,
    row: Leaf,
    sides: f32,
    face: Color,
    center_pct: f32,
    size: i32,
    offset: i32,
    seq: Leaf,
    start: u64,
) -> (Leaf, Leaf) {
    let shadow = g.canopy.branch(
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
                    .adjust(-offset)
                    .with(size.px().as_width()),
                offset.px().as_top().with(size.px().as_height()),
            ))
            .elevate(Elevation::up(2))
            .opacity(0.0),
    );
    morph_in(g.canopy, shadow, seq, sides, 0.15, start);

    let shape = g.canopy.branch(
        row,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(-0.16)
            .color(face)
            .at(Location::new().xs(
                center_pct.pct().as_center_x().with(size.px().as_width()),
                0.px().as_top().with(size.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .opacity(0.0),
    );
    morph_in(g.canopy, shape, seq, sides, 0.15, start);

    let hit_size = size.max(MIN_HIT);
    let button = g.canopy.branch(
        row,
        Bare::new()
            .at(Location::new().xs(
                center_pct
                    .pct()
                    .as_center_x()
                    .with(hit_size.px().as_width()),
                ((size - hit_size) / 2)
                    .px()
                    .as_top()
                    .with(hit_size.px().as_height()),
            ))
            .elevate(Elevation::up(4))
            .interactive()
            .round_hit_area(),
    );
    // armed on the fade, not the morph: the shape keeps resolving for another second after
    // the button is plainly visible, and a button you can see but cannot press is its own bug
    arm_at(g, button, start + motion::FADE);
    (shadow, button)
}

/// The control itself, with nothing attached to the press. The caller keeps the returned
/// `Leaf` and decides what it means.
pub(crate) fn poly_action(
    g: &mut Grow,
    row: Leaf,
    spec: &PolyAction,
    center_pct: f32,
    seq: Leaf,
    start: u64,
) -> Leaf {
    let (_shadow, button) = poly_shape(
        g,
        row,
        spec.sides,
        spec.face,
        center_pct,
        POLY_BUTTON,
        POLY_SHADOW_OFF,
        seq,
        start,
    );

    let icon = g.canopy.branch(
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
            .anchored(button)
            // the icon draws above the button, so without this it wins the hit-test and
            // swallows the click meant for the shape under it
            .pass_through()
            .opacity(0.0),
    );
    fade_in(g.canopy, icon, seq, start);

    let label = g.canopy.branch(
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
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .anchored(button)
            .opacity(0.0),
    );
    fade_in(g.canopy, label, seq, start);
    button
}

/// The step control's shape, and the row height a board must reserve for one.
///
/// Smaller than a destination's on purpose. A board carries up to four of these across a column
/// that is about 256px inside its own padding at the narrowest breakpoint -- a 64px slot -- and
/// at [`POLY_BUTTON`] the shapes touch before their words do.
pub(crate) const POLY_STEP: i32 = 36;
const POLY_STEP_SHADOW_OFF: i32 = 5;
const POLY_STEP_LABEL_H: i32 = 16;
/// The bar under the selected step's word. Three pixels, because it is read as *which one* and
/// never as a mark in its own right.
const POLY_STEP_MARK_H: i32 = 3;
const POLY_STEP_MARK_W: i32 = 20;
pub(crate) const POLY_STEP_ROW_H: i32 =
    POLY_STEP + space::SM + POLY_STEP_LABEL_H + space::XS + POLY_STEP_MARK_H;

/// The faces a control row cycles through, and the shapes to go with them.
///
/// The hero's own three tones and no new ones. Alternating rather than one repeated: a row of
/// identical chips reads as a segmented control, which is a thing you pick a *setting* from --
/// where these are the steps of a demonstration, and each one is its own move.
const STEP_FACES: [fn() -> Color; 3] = [role::accent, || Color::amber(400), || Color::rose(400)];
/// Side counts, so no two neighbours resolve into the same polygon.
const STEP_SIDES: [f32; 4] = [6.0, 5.0, 7.0, 4.0];

/// One step control: the shape that takes the press, and the marks that say it is the live one.
pub(crate) struct StepControl {
    /// The one leaf of the three that answers a click.
    pub(crate) button: Leaf,
    /// The bar under the word, shown for the selected step alone.
    mark: Leaf,
    label: Leaf,
}

impl StepControl {
    /// Lit or not. Colour and visibility rather than opacity, because the entrance is animating
    /// opacity up from zero on every one of these and a resting value written underneath it
    /// would be overwritten the moment the tween's next frame lands.
    fn select(&self, canopy: &mut Canopy, on: bool) {
        canopy.visible(self.mark, on);
        canopy.color(
            self.label,
            if on {
                role::on_surface()
            } else {
                role::on_surface_variant()
            },
        );
    }
}

/// One step of a demo's control row: shape, word, and selection mark.
///
/// The row divides itself into `count` equal slots and centres a control in each, so two steps
/// spread across exactly the field four steps fill. A board with fewer of them reads as a
/// complete row rather than as a full one with the end missing, which is what a fixed slot
/// width would have made of it.
///
/// `i` picks the face and the shape as well as the position -- a board declares its steps as a
/// list of words, and cannot put the row out of the site's palette by writing them down.
pub(crate) fn poly_step(
    g: &mut Grow,
    row: Leaf,
    text: &'static str,
    i: usize,
    count: usize,
    seq: Leaf,
    start: u64,
) -> StepControl {
    let slot = 100.0 / count as f32;
    let center = slot * i as f32 + slot / 2.0;
    let sides = STEP_SIDES[i % STEP_SIDES.len()];
    let (_shadow, button) = poly_shape(
        g,
        row,
        sides,
        STEP_FACES[i % STEP_FACES.len()](),
        center,
        POLY_STEP,
        POLY_STEP_SHADOW_OFF,
        seq,
        start,
    );

    // The ordinal, where a destination carries an icon. These steps are a sequence and the words
    // under them are too short to say so; the number is what makes "prune" the third thing you
    // do rather than one of four things you may.
    let numeral = g.canopy.branch(
        row,
        Text::new((i + 1).to_string())
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_accent())
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .with(anchor().width().as_width()),
                anchor()
                    .center_y()
                    .as_center_y()
                    .with(1.letters().as_height()),
            ))
            .elevate(Elevation::up(4))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .anchored(button)
            // the shape is the target; a numeral that won the hit-test would put a dead spot in
            // the middle of every button
            .pass_through()
            .opacity(0.0),
    );
    fade_in(g.canopy, numeral, seq, start);

    // Placed off the row rather than anchored to the button, so every word in the row sits on
    // one baseline whatever the shapes above them resolved to.
    let label = g.canopy.branch(
        row,
        Text::new(text)
            .size(FontSize::new(type_scale::LABEL))
            .color(role::on_surface_variant())
            .at(Location::new().xs(
                center.pct().as_center_x().with(slot.pct().as_width()),
                (POLY_STEP + space::SM)
                    .px()
                    .as_top()
                    .with(POLY_STEP_LABEL_H.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .pass_through()
            .opacity(0.0),
    );
    fade_in(g.canopy, label, seq, start);

    // In the step's own face, not the accent: the mark says *this shape* is live, and a tone the
    // shape above it does not have makes it a second thing to account for.
    let mark = g.canopy.branch(
        row,
        Panel::new()
            .color(STEP_FACES[i % STEP_FACES.len()]())
            .rounding(Rounding::None)
            .at(Location::new().xs(
                center
                    .pct()
                    .as_center_x()
                    .with(POLY_STEP_MARK_W.px().as_width()),
                (POLY_STEP + space::SM + POLY_STEP_LABEL_H + space::XS)
                    .px()
                    .as_top()
                    .with(POLY_STEP_MARK_H.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .pass_through()
            .opacity(0.0),
    );
    // Faded in with the rest of the row even when it is hidden. `visible` is the switch and it
    // is independent of the tween, so a mark that is turned on later is already at full opacity
    // rather than stuck at the zero it spawned with.
    fade_in(g.canopy, mark, seq, start);
    StepControl {
        button,
        mark,
        label,
    }
}

/// Holds an element out of the hit-test until `at` ms into the page's entrance -- the moment
/// its own fade finishes and it is actually on screen.
///
/// Entrances animate opacity up from zero, and an element at zero opacity still draws and
/// still competes for input. So every control on this site was live for the length of its own
/// entrance delay while invisible: on the hero that is about 1.4s of a chevron you cannot see
/// but can navigate with, and the poly buttons carry links, which makes it an invisible link
/// to another site.
///
/// The timer's report is answered in [`crate::entry::Site::respond`]. Leaving the route
/// mid-entrance needs no guard of its own: the page is pruned, and a command naming a withered
/// `Leaf` is dropped.
pub(crate) fn arm_at(g: &mut Grow, leaf: Leaf, at: u64) {
    g.canopy.disable(leaf);
    let timer = g.canopy.timer(at);
    g.page.arming.push((timer, leaf));
}

/// Fades an element in without a shape change, for text and panels.
pub(crate) fn fade_in(canopy: &mut Canopy, leaf: Leaf, seq: Leaf, start: u64) {
    canopy.animate_during(
        leaf,
        Motion::Opacity(1.0),
        timing(start, motion::FADE, Ease::Linear),
        seq,
    );
}

/// A vertical stack of content written straight into the page's scroll container.
///
/// Each element anchors to the bottom of the one before it rather than sitting at a
/// computed offset, so prose that wraps to three lines on a phone pushes everything below
/// it down instead of being overlapped. Nothing here needs to know the viewport width.
///
/// Elements carry the measure themselves ([`shell::measure`]) rather than sitting inside a
/// measured box. The box was the obvious shape and the wrong one: it needed a `Grid`, which
/// brings a view, which made the page's middle and its side gutters two different scroll
/// targets -- the gutters resolving to a container with no extent, so the wheel did nothing
/// there. One container, one view, and the inset travels with each element.
pub(crate) struct Column {
    parent: Leaf,
    last: Option<Leaf>,
    /// Runs the entrance animations, so a whole page arrives as one staggered gesture.
    seq: Leaf,
    step: u64,
}

impl Column {
    pub(crate) fn new(canopy: &mut Canopy, parent: Leaf) -> Self {
        let seq = canopy.sequence();
        Self {
            parent,
            last: None,
            seq,
            step: 0,
        }
    }
    pub(crate) fn sequence(&self) -> Leaf {
        self.seq
    }
    /// The next element's vertical placement: below the previous element, or at the top of
    /// the column for the first one.
    ///
    /// `height` is a value rather than a number of pixels because the two kinds of element in
    /// a column want different answers. A run of text takes [`text_content()`], measuring
    /// itself -- which is the whole reason the stack anchors instead of computing offsets.
    /// Everything else (a plate, a card grid) states the px height it was built to.
    fn below(&self, gap: i32, height: LocationValue) -> ConfigurationDescriptor {
        if self.last.is_some() {
            anchor()
                .bottom()
                .as_top()
                .adjust(gap)
                .with(height.as_height())
        } else {
            gap.px().as_top().with(height.as_height())
        }
    }
    fn anchor_to_last(&self) -> Leaf {
        self.last.unwrap_or(self.parent)
    }
    /// This element's box, measured horizontally and stacked vertically.
    ///
    /// `md` carries the `lg`/`xl` steps too by falling through, so an element that does not
    /// reflow only spells out two.
    fn placed(&self, gap: i32, height: LocationValue) -> Location {
        let (xs, md) = shell::measure();
        Location::new()
            .xs(xs, self.below(gap, height))
            .md(md, self.below(gap, height))
    }
    fn stagger(&mut self) -> u64 {
        let at = self.step;
        self.step += motion::STAGGER;
        at
    }
    fn text(
        &mut self,
        canopy: &mut Canopy,
        value: &str,
        size: u32,
        tone: Color,
        gap: i32,
        font: FontId,
    ) -> Leaf {
        let start = self.stagger();
        let leaf = canopy.branch(
            self.parent,
            Text::new(value)
                .size(FontSize::new(size))
                .color(tone)
                .at(self.placed(gap, text_content()))
                .elevate(Elevation::up(2))
                .align(HorizontalAlignment::Left, VerticalAlignment::Top)
                .anchored(self.anchor_to_last())
                .opacity(0.0)
                .font(font),
        );
        fade_in(canopy, leaf, self.seq, start);
        self.last = Some(leaf);
        leaf
    }
    /// The page's own title. One per page.
    ///
    /// Upper-cased here rather than at the call sites, so a page cannot spell its title in a
    /// case the rest of the site does not use. Same for [`heading`](Self::heading) -- caps are
    /// the site's structural voice, and the prose beneath them is the only thing in sentence
    /// case, which is what keeps the two from blurring.
    ///
    /// Opens at [`PAGE_TOP`] rather than the usual [`space::XL`], which is what keeps it out
    /// from under the drawer's menu button.
    pub(crate) fn display(&mut self, canopy: &mut Canopy, value: &str) -> Leaf {
        self.text(
            canopy,
            &value.to_uppercase(),
            type_scale::DISPLAY,
            role::on_surface_title(),
            PAGE_TOP,
            FontId::default(),
        )
    }
    /// A section heading within the page.
    pub(crate) fn heading(&mut self, canopy: &mut Canopy, value: &str) -> Leaf {
        self.text(
            canopy,
            &value.to_uppercase(),
            type_scale::HEADLINE,
            role::on_surface_heading(),
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
    pub(crate) fn prose(&mut self, canopy: &mut Canopy, value: &str) -> Leaf {
        self.text(
            canopy,
            value,
            type_scale::BODY,
            role::on_surface_variant(),
            space::MD,
            italic(),
        )
    }
    /// A page's opening paragraph, marked with an accent rule down its left edge.
    ///
    /// Deliberately not a filled surface. The slabs on these pages are the card grids, and an
    /// opener built as one more slab reads as the first card rather than as the lead -- so the
    /// emphasis is a rule, which is the one thing a slab is not.
    ///
    /// On the measure like every other paragraph, with the rule sitting out in the gutter
    /// beside it. It used to be indented instead, which cost it alignment with the prose under
    /// it; now that the container is full-bleed there is somewhere for the rule to go.
    pub(crate) fn lead(&mut self, canopy: &mut Canopy, value: &str) -> Leaf {
        let start = self.stagger();
        let leaf = canopy.branch(
            self.parent,
            Text::new(value)
                .size(FontSize::new(type_scale::LEAD))
                .color(role::on_surface())
                .at(self.placed(space::MD, text_content()))
                .elevate(Elevation::up(2))
                .align(HorizontalAlignment::Left, VerticalAlignment::Top)
                .anchored(self.anchor_to_last())
                .opacity(0.0)
                .font(italic()),
        );
        fade_in(canopy, leaf, self.seq, start);
        // Anchored to the paragraph rather than given a height: the wrap count changes with
        // every breakpoint, and a rule that stops short of the text it marks looks like a
        // mistake at exactly the width nobody tested.
        let rule = canopy.branch(
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
                .anchored(leaf)
                .opacity(0.0),
        );
        fade_in(canopy, rule, self.seq, start);
        self.last = Some(leaf);
        leaf
    }
    /// A hairline across the column. Separates one kind of content from another without
    /// spending a heading on it.
    pub(crate) fn rule(&mut self, canopy: &mut Canopy) -> Leaf {
        let start = self.stagger();
        let leaf = canopy.branch(
            self.parent,
            Panel::new()
                .color(role::outline())
                .rounding(Rounding::None)
                .at(self.placed(space::XL, 1.px()))
                .elevate(Elevation::up(1))
                .anchored(self.anchor_to_last())
                .opacity(0.0),
        );
        fade_in(canopy, leaf, self.seq, start);
        self.last = Some(leaf);
        leaf
    }
    /// A bare region in the stack -- no surface of its own, just space the caller owns, whose
    /// height differs by breakpoint for content that reflows into fewer rows as the column
    /// widens, like a card grid going one-up to two-up.
    ///
    /// All three steps are spelled out because the two callers reflow at different widths: a
    /// plate just gets taller at `md`, while the card grid stays one-up until `lg`.
    pub(crate) fn region(
        &mut self,
        canopy: &mut Canopy,
        heights: (i32, i32, i32),
        gap: i32,
    ) -> Leaf {
        self.region_on(canopy, shell::measure(), heights, gap)
    }
    /// A region on the figure measure: uncapped, so a drawing widens with the window while the
    /// prose above and below it stays at a readable line length.
    ///
    /// Give it a taller `lg` than `md`. A figure that only gets wider goes letterboxed on a
    /// big screen, and the whole point of the extra room is that the drawing gets to use it.
    pub(crate) fn figure(
        &mut self,
        canopy: &mut Canopy,
        heights: (i32, i32, i32),
        gap: i32,
    ) -> Leaf {
        self.region_on(canopy, shell::figure_measure(), heights, gap)
    }
    /// A region whose height is stated in characters, plus a fixed allowance for whatever sits
    /// between them that is not text -- rules, gaps, padding.
    ///
    /// The `Bare` is given a [`FontSize`] for the same reason its contents have one:
    /// `.letters()` measures against the entity's *own* cell, and a container that holds text
    /// without being text has no cell until it is handed one. Stated this way the box tracks
    /// the type scale, instead of being a pixel count that silently stops fitting the first
    /// time the scale moves.
    ///
    /// `letters` is (xs, sm, md, lg) -- one more step than [`region`](Self::region) spells,
    /// because what goes inside a card *can* reflow at `sm` and a container that could not say
    /// so would be sized for a different number of lines than it holds.
    pub(crate) fn region_letters(
        &mut self,
        canopy: &mut Canopy,
        letters: (i32, i32, i32, i32),
        extra: i32,
        size: u32,
        gap: i32,
    ) -> Leaf {
        // For the side effect alone: nothing here fades, but the slot has to be spent or every
        // element after this one moves up a step in the entrance.
        self.stagger();
        let (xs, wide) = shell::measure();
        let anchored = self.last.is_some();
        let below = |lines: i32| {
            let height = lines.letters().as_height().adjust(extra);
            if anchored {
                anchor().bottom().as_top().adjust(gap).with(height)
            } else {
                gap.px().as_top().with(height)
            }
        };
        let leaf = canopy.branch(
            self.parent,
            Bare::new()
                .at(Location::new()
                    .xs(xs, below(letters.0))
                    .sm(xs, below(letters.1))
                    .md(wide, below(letters.2))
                    .lg(wide, below(letters.3)))
                .elevate(Elevation::up(1))
                .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
                .anchored(self.anchor_to_last())
                .size(FontSize::new(size)),
        );
        self.last = Some(leaf);
        leaf
    }
    fn region_on(
        &mut self,
        canopy: &mut Canopy,
        measure: (ConfigurationDescriptor, ConfigurationDescriptor),
        heights: (i32, i32, i32),
        gap: i32,
    ) -> Leaf {
        let start = self.stagger();
        let (xs, wide) = measure;
        let leaf = canopy.branch(
            self.parent,
            Bare::new()
                .at(Location::new()
                    .xs(xs, self.below(gap, heights.0.px()))
                    .md(wide, self.below(gap, heights.1.px()))
                    .lg(wide, self.below(gap, heights.2.px())))
                .elevate(Elevation::up(1))
                .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
                .anchored(self.anchor_to_last()),
        );
        let _ = start;
        self.last = Some(leaf);
        leaf
    }
    pub(crate) fn surface_plain(&mut self, canopy: &mut Canopy, height: i32, gap: i32) -> Leaf {
        let start = self.stagger();
        let leaf = canopy.branch(
            self.parent,
            Panel::new()
                .color(background())
                .rounding(Rounding::Xs)
                .at(self.placed(gap, height.px()))
                .elevate(Elevation::up(1))
                .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
                .anchored(self.anchor_to_last())
                .opacity(0.0),
        );
        fade_in(canopy, leaf, self.seq, start);
        self.last = Some(leaf);
        leaf
    }
    /// Empty space after the last element, so a page can be scrolled clear of its own bottom
    /// edge.
    ///
    /// The scroll extent grows to cover the content and no further, so the last thing on a
    /// page ends flush with the viewport -- there is no overscroll to give it breathing room.
    /// This is that room, made of the only thing the extent understands: more content.
    ///
    /// Interaction passes through, or a full-width invisible box below the buttons would eat
    /// clicks aimed at whatever the layout puts near it.
    pub(crate) fn tail(&mut self, canopy: &mut Canopy, height: i32) -> Leaf {
        let leaf = canopy.branch(
            self.parent,
            Bare::new()
                .at(self.placed(0, height.px()))
                .elevate(Elevation::up(1))
                .anchored(self.anchor_to_last())
                .pass_through(),
        );
        self.last = Some(leaf);
        leaf
    }
    /// A caption under a figure.
    ///
    /// Still unused: both figures on the site are plates, and a plate captions itself in its
    /// own strip below the field. This is for a figure that is not one.
    #[allow(dead_code)]
    pub(crate) fn caption(&mut self, canopy: &mut Canopy, value: &str) -> Leaf {
        self.text(
            canopy,
            value,
            type_scale::LABEL,
            role::on_surface_variant(),
            space::SM,
            italic(),
        )
    }
}
