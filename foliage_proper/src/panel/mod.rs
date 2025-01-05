use crate::ash::clip::ClipSection;
use crate::ginkgo::ScaleFactor;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::{
    Attachment, ClipContext, Color, Component, CoordinateUnit, Coordinates, Differential, Foliage,
    Logical, Position, ResolvedElevation, Section, Tree, Update, Visibility, Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Trigger;
use bevy_ecs::system::{Query, Res};
use bevy_ecs::world::DeferredWorld;
use bytemuck::{Pod, Zeroable};

mod pipeline;
mod vertex;

#[derive(Component, Copy, Clone, Default, PartialEq)]
#[require(Rounding, Color, Outline, ClipContext)]
#[require(Differential<Self, ResolvedElevation>)]
#[require(Differential<Self, Color>)]
#[require(Differential<Self, Panel>)]
#[require(Differential<Self, Outline>)]
#[require(Differential<Self, Section<Logical>>)]
#[require(Differential<Self, BlendedOpacity>)]
#[require(Differential<Self, ClipSection>)]
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
        if let Ok(section) = sections.get(this) {
            if let Ok(rounding) = roundings.get(this) {
                let min = section.width().min(section.height());
                let depth = match rounding {
                    Rounding::None => 0.0,
                    Rounding::Xs => 0.2 * min,
                    Rounding::Sm => 0.4 * min,
                    Rounding::Md => 0.6 * min,
                    Rounding::Lg => 0.8 * min,
                    Rounding::Xl => 1.0 * min,
                };
                let weight = if let Ok(outline) = outlines.get(this) {
                    if outline.value.is_negative() {
                        None
                    } else {
                        Some(outline.value)
                    }
                } else {
                    None
                };
                if let Ok(mut panel) = panels.get_mut(this) {
                    let near = if let Some(w) = weight {
                        depth - w as f32
                    } else {
                        0.0
                    } * scale_factor.value();
                    panel.corner_i = {
                        let other = Position::logical((0, 0)).to_physical(scale_factor.value());
                        let c = Position::logical((depth, depth)).to_physical(scale_factor.value());
                        Corner::new(c.coordinates, c.distance(other), near)
                    };
                    panel.corner_ii = {
                        let other = Position::logical((section.right(), 0.0))
                            .to_physical(scale_factor.value());
                        let c = Position::logical((section.right() - depth, depth))
                            .to_physical(scale_factor.value());
                        Corner::new(c.coordinates, c.distance(other), near)
                    };
                    panel.corner_iii = {
                        let other = Position::logical((0.0, section.bottom()))
                            .to_physical(scale_factor.value());
                        let c = Position::logical((depth, section.bottom() - depth))
                            .to_physical(scale_factor.value());
                        Corner::new(c.coordinates, c.distance(other), near)
                    };
                    panel.corner_iv = {
                        let other = Position::logical((section.right(), section.bottom()))
                            .to_physical(scale_factor.value());
                        let c =
                            Position::logical((section.right() - depth, section.bottom() - depth))
                                .to_physical(scale_factor.value());
                        Corner::new(c.coordinates, c.distance(other), near)
                    };
                }
            }
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
        foliage.differential::<Self, ClipSection>();
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
    Xl,
}
impl Rounding {
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Self>::new(), this);
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
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Panel>::new(), this);
    }
}
impl Default for Outline {
    fn default() -> Self {
        Outline { value: -1 }
    }
}
