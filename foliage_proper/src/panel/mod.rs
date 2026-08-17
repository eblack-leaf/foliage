use crate::AsTree;
use crate::Trigger;
use crate::anim::interpolation::Interpolations;
use crate::ash::clip::ClipContext;
use crate::ginkgo::ScaleFactor;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::rounding::CornerRadii;
use crate::{
    Animate, Attachment, Color, Component, Differential, Foliage, Logical, Resolve, Resolved,
    ResolvedElevation, Rounding, Section, Side, Tree, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::system::{Query, Res};
use bevy_ecs::world::DeferredWorld;

mod pipeline;
mod vertex;

#[derive(Component, Copy, Clone, Default, PartialEq)]
#[require(Rounding, Side, Color, Outline)]
#[require(Differential<Self, ResolvedElevation>)]
#[require(Differential<Self, Color>)]
#[require(Differential<Self, Panel>)]
#[require(Differential<Self, Outline>)]
#[require(Differential<Self, Section<Logical>>)]
#[require(Differential<Self, BlendedOpacity>)]
#[require(Differential<Self, ClipContext>)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
/// A filled rectangle with optionally rounded corners and an optional outline -- the
/// background primitive nearly every composite is built on.
///
/// Spawned through [`Panel::new`]. Shape comes from three independent components:
/// [`Rounding`] (how much), [`Side`] (which corners), and [`Outline`] (ring instead of
/// fill). The radii the shader consumes are resolved from those into [`CornerRadii`],
/// never authored -- see `Panel::update`.
///
/// [`Rounding::Full`] also switches the entity's hit test to
/// [`InteractionShape::Circle`](crate::InteractionShape), so a pill or dot only responds
/// where it is actually drawn.
pub struct Panel {
    pub(crate) radii: CornerRadii,
}
impl Panel {
    /// Starts a [`Panel`] entity:
    /// `canopy.branch(parent, Panel::new().color(c).rounding(Rounding::Md).at(loc))`.
    pub fn new() -> PanelSprout {
        PanelSprout::default()
    }
    pub(crate) fn new_marker() -> Panel {
        Panel {
            radii: CornerRadii::default(),
        }
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let mut tree = world.tree();
        tree.subscribe(this, Self::update_from_section);
        tree.subscribe(this, Remove::push_remove_packet::<Self>);
        tree.subscribe(this, Visibility::push_remove_packet::<Self>);
    }
    fn update_from_section(trigger: Trigger<Resolved<Section<Logical>>>, mut tree: Tree) {
        tree.send_to(Resolve::<Panel>::new(), trigger.event_target());
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.tree().send_to(Resolve::<Panel>::new(), this);
    }
    /// [`Outline`] is deliberately absent: the ring is carved out of the same distance
    /// field in `panel.wgsl` from the weight that already ships with the elevation, so
    /// changing (or animating) an outline no longer re-resolves any geometry here.
    fn update(
        trigger: Trigger<Resolve<Panel>>,
        mut panels: Query<&mut Panel>,
        roundings: Query<&Rounding>,
        sides: Query<&Side>,
        sections: Query<&Section<Logical>>,
        scale_factor: Res<ScaleFactor>,
    ) {
        let this = trigger.event_target();
        let (Ok(section), Ok(rounding)) = (sections.get(this), roundings.get(this)) else {
            return;
        };
        let side = sides.get(this).copied().unwrap_or_default();
        let section = section.to_physical(scale_factor.value()).rounded();
        // Written unconditionally, never guarded on inequality: `Differential::different`
        // is what deduplicates downstream, and a guard here would leave `Changed<Panel>`
        // unset for every square panel -- `Rounding::None` resolves to the same all-zero
        // radii `new_marker` already put there.
        if let Ok(mut panel) = panels.get_mut(this) {
            panel.radii = CornerRadii::resolve(section, *rounding, side, scale_factor.value());
        }
    }
}
impl Attachment for Panel {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Panel::update);
        foliage.remove_queue::<Self>();
        foliage.differential::<Self, Section<Logical>>();
        foliage.differential::<Self, BlendedOpacity>();
        foliage.differential::<Self, Panel>();
        foliage.differential::<Self, Color>();
        foliage.differential::<Self, Outline>();
        foliage.differential::<Self, ResolvedElevation>();
        foliage.differential::<Self, ClipContext>();
        foliage.enable_animation::<Outline>();
    }
}
/// Builder for a [`Panel`] entity -- see [`Panel::new`].
#[derive(Default)]
pub struct PanelSprout {
    leaf: crate::LeafSprout,
    color: Option<Color>,
    rounding: Option<Rounding>,
    side: Option<Side>,
    outline: Option<i32>,
}
impl crate::Author for PanelSprout {
    fn seed(&mut self) -> &mut crate::LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Panel::new_marker(),
            self.color.unwrap_or_default(),
            self.rounding.unwrap_or_default(),
            self.side.unwrap_or_default(),
            self.outline.map(Outline::new).unwrap_or_default(),
        )
    }
}
impl PanelSprout {
    /// Fill color, or the ring's color when an [`Outline`] is set.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    /// Corner radius bracket. Applies to the corners [`Side`] names; defaults to square.
    pub fn rounding(mut self, r: Rounding) -> Self {
        self.rounding = Some(r);
        self
    }
    /// Restricts [`Rounding`] to particular corners. Defaults to all four.
    pub fn side(mut self, s: Side) -> Self {
        self.side = Some(s);
        self
    }
    /// Draws a ring of this width in logical pixels instead of a solid fill. Squared
    /// corners fill solid rather than mitering -- see [`Outline`].
    pub fn outline(mut self, w: i32) -> Self {
        self.outline = Some(w);
        self
    }
}
/// Draws the panel as a ring of this width in logical pixels instead of a solid fill.
/// Animatable, so a border can be drawn on.
///
/// Negative is the solid fill; `0` is the thinnest ring the shape can show, one physical
/// pixel. The ring is the panel's own outline inset by this width, so it follows squared
/// and rounded corners alike.
// No insert hook and no animation hook: the weight reaches the shader on its own
// differential, and nothing about the panel's shape is derived from it any more.
#[derive(Component, Copy, Clone, PartialEq)]
pub struct Outline {
    pub value: i32,
}
impl Outline {
    /// A ring `value` logical pixels wide; negative is a solid fill.
    pub fn new(value: i32) -> Outline {
        Outline { value }
    }
}
impl Default for Outline {
    fn default() -> Self {
        Outline { value: -1 }
    }
}
impl Animate for Outline {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new().with(start.value as f32, end.value as f32)
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(o) = interpolations.read(0) {
            self.value = o as i32;
        }
    }
}
