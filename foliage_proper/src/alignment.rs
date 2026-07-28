//! Shared content-in-box alignment vocabulary.
//!
//! Two primitives render fixed-metric content inside a `Location` box they don't stretch to
//! fill: `Text` (glyph runs) and `Icon` (ahead-of-time rasterized footprints). Both resolve
//! the mismatch the same way -- the box is an ALIGNMENT REGION, and these components say
//! where the true-size content sits within it. Defaults differ by primitive on purpose:
//! text reads Left/Top (prose; the enums' `Default`), icons center (UI glyphs; absent
//! components read as Center/Middle inside `Icon::align_render_size`).

use crate::{Component, EcsExtension, Resolve};
use bevy_ecs::component::ComponentId;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::world::DeferredWorld;

#[derive(Component, Copy, Clone, Default, PartialEq, Debug)]
#[component(on_insert = on_alignment_insert)]
/// How a text run sits within its box horizontally. Shared with [`Icon`](crate::Icon),
/// which uses it to place its artwork in an oversized box.
pub enum HorizontalAlignment {
    #[default]
    Left,
    Center,
    Right,
}
#[derive(Component, Copy, Clone, Default, PartialEq, Debug)]
#[component(on_insert = on_alignment_insert)]
/// How a text run sits within its box vertically. Shared with [`Icon`](crate::Icon).
pub enum VerticalAlignment {
    #[default]
    Top,
    Middle,
    Bottom,
}

/// Alignment landed -- tell whichever content this entity actually carries to re-place
/// itself. Text relayouts (fontdue consumes the alignment); an icon re-resolves its
/// `Location` so the placement pass (`Icon::align_render_size`) runs against a fresh box.
/// Unconditional `Resolve::<Text>` here was fine when only Text used these; on an icon it
/// would reach `Text::update`'s `texts.get(..).unwrap()`.
fn on_alignment_insert(mut world: DeferredWorld, ctx: HookContext) {
    let this = ctx.entity;
    if world.get::<crate::Text>(this).is_some() {
        world.trigger_targets(Resolve::<crate::Text>::new(), this);
    } else if world.get::<crate::Icon>(this).is_some() {
        if let Some(location) = world.get::<crate::Location>(this).cloned() {
            world.commands().entity(this).insert(location);
        }
    }
}
