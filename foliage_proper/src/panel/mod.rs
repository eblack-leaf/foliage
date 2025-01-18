use crate::anim::interpolation::Interpolations;
use crate::ginkgo::ScaleFactor;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::{
    Animate, Animation, Attachment, Color, Component, CoordinateUnit, Coordinates, Differential,
    Foliage, Logical, Position, ResolvedElevation, Section, Stem, Tree, Update, Visibility, Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::{Query, Res};
use bevy_ecs::world::DeferredWorld;

mod pipeline;
mod vertex;

#[derive(Component, Copy, Clone, Default, PartialEq)]
#[require(Rounding, Color, Outline)]
#[require(Differential<Self, ResolvedElevation>)]
#[require(Differential<Self, Color>)]
#[require(Differential<Self, Panel>)]
#[require(Differential<Self, Outline>)]
#[require(Differential<Self, Section<Logical>>)]
#[require(Differential<Self, BlendedOpacity>)]
#[require(Differential<Self, Stem>)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
pub struct Panel {
    pub(crate) corner_i: Corner,
    pub(crate) corner_ii: Corner,
    pub(crate) corner_iii: Corner,
    pub(crate) corner_iv: Corner,
}
impl Panel {
    pub fn new() -> Panel {
        Panel {
            corner_i: Default::default(),
            corner_ii: Default::default(),
            corner_iii: Default::default(),
            corner_iv: Default::default(),
        }
    }
    fn on_add(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world
            .commands()
            .entity(this)
            .observe(Self::update_from_section)
            .observe(Remove::push_remove_packet::<Self>)
            .observe(Visibility::push_remove_packet::<Self>);
    }
    fn update_from_section(trigger: Trigger<Write<Section<Logical>>>, mut tree: Tree) {
        tree.trigger_targets(Update::<Panel>::new(), trigger.entity());
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Panel>::new(), this);
    }
    fn update(
        trigger: Trigger<Update<Panel>>,
        mut panels: Query<&mut Panel>,
        roundings: Query<&Rounding>,
        outlines: Query<&Outline>,
        sections: Query<&Section<Logical>>,
        scale_factor: Res<ScaleFactor>,
    ) {
        let this = trigger.entity();
        if panels.get(this).ok().is_none() {
            return;
        }
        if let Ok(section) = sections.get(this) {
            if let Ok(rounding) = roundings.get(this) {
                let section = section.to_physical(scale_factor.value()).rounded();
                let min = section.width().min(section.height()) * 0.5;
                let depth = match rounding {
                    Rounding::None => 0.0,
                    Rounding::Xs => 0.1 * min,
                    Rounding::Sm => 0.3 * min,
                    Rounding::Md => 0.5 * min,
                    Rounding::Lg => 0.7 * min,
                    Rounding::Full => 1.0 * min,
                };
                let weight = if let Ok(outline) = outlines.get(this) {
                    if outline.value.is_negative() {
                        None
                    } else {
                        Some(outline.value as f32 * scale_factor.value())
                    }
                } else {
                    None
                };
                if let Ok(mut panel) = panels.get_mut(this) {
                    let edge_adjust = 0.15;
                    let near = if let Some(w) = weight {
                        depth - w.max(1.0)
                    } else {
                        0.0
                    } - edge_adjust;
                    let adjusted_depth = depth + edge_adjust;
                    panel.corner_i = {
                        let c = Position::physical((depth, depth));
                        Corner::new(c.coordinates, adjusted_depth, near)
                    };
                    panel.corner_ii = {
                        let c = Position::physical((section.width() - depth, depth));
                        Corner::new(c.coordinates, adjusted_depth, near)
                    };
                    panel.corner_iii = {
                        let c = Position::physical((depth, section.height() - depth));
                        Corner::new(c.coordinates, adjusted_depth, near)
                    };
                    panel.corner_iv = {
                        let c =
                            Position::physical((section.width() - depth, section.height() - depth));
                        Corner::new(c.coordinates, adjusted_depth, near)
                    };
                }
            }
        }
    }
}
impl Attachment for Panel {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Panel::update);
        foliage.define(Outline::update_anim);
        foliage.remove_queue::<Self>();
        foliage.differential::<Self, Section<Logical>>();
        foliage.differential::<Self, BlendedOpacity>();
        foliage.differential::<Self, Panel>();
        foliage.differential::<Self, Color>();
        foliage.differential::<Self, Outline>();
        foliage.differential::<Self, ResolvedElevation>();
        foliage.differential::<Self, Stem>();
        foliage.enable_animation::<Outline>();
    }
}
#[derive(Component, Copy, Clone, Default)]
#[component(on_insert = Self::on_insert)]
pub enum Rounding {
    #[default]
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Full,
}
impl Rounding {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        if world.get::<Panel>(this).is_some() {
            world.trigger_targets(Update::<Self>::new(), this);
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub(crate) struct Corner {
    pub(crate) coordinates: Coordinates,
    pub(crate) far: CoordinateUnit,
    pub(crate) near: CoordinateUnit,
}
impl Corner {
    pub(crate) fn new<C: Into<Coordinates>>(
        c: C,
        far: CoordinateUnit,
        near: CoordinateUnit,
    ) -> Self {
        Self {
            coordinates: c.into(),
            far,
            near,
        }
    }
}
#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_insert = Self::on_insert)]
pub struct Outline {
    pub value: i32,
}
impl Outline {
    pub fn new(value: i32) -> Outline {
        Outline { value }
    }
    fn update_anim(trigger: Trigger<Update<Animation<Self>>>, mut tree: Tree) {
        tree.trigger_targets(Update::<Panel>::new(), trigger.entity());
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        if world.get::<Panel>(this).is_some() {
            world.trigger_targets(Update::<Panel>::new(), this);
        }
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
