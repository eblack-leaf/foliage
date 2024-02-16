use crate::elm::leaf::Leaflet;
use crate::elm::Elm;
use bevy_ecs::prelude::{apply_deferred, IntoSystemConfigs, SystemSet};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum ExternalSet {
    Process,
    ViewBindings,
    Spawn,
    Configure,
    InteractionTriggers,
}
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum CoreSet {
    ExternalEvent,
    Interaction,
    // InteractionTriggers,
    // Process,
    ProcessEvent,
    TransitionView,
    // ViewBindings,
    // Spawn,
    Compositor,
    Coordinate,
    // Configure,
    CoordinateFinalize,
    Visibility,
    Differential,
    RenderPacket,
}
pub struct ElmConfiguration<'a>(&'a mut Elm);
impl<'a> ElmConfiguration<'a> {
    pub fn configure_hook<L: SystemSet + Hash + Eq + PartialEq + Debug + Copy + Clone>(
        &mut self,
        external_set: ExternalSet,
        hook: L,
    ) {
        use bevy_ecs::prelude::IntoSystemSetConfigs;
        self.0.job.main().configure_sets(hook.in_set(external_set));
    }
    pub(crate) fn configure_elm(elm: &'a mut Elm, leaflets: &[Leaflet]) {
        use bevy_ecs::prelude::IntoSystemSetConfigs;
        elm.main().configure_sets(
            (
                CoreSet::ExternalEvent,
                CoreSet::Interaction,
                ExternalSet::InteractionTriggers,
                ExternalSet::Process,
                CoreSet::ProcessEvent,
                CoreSet::TransitionView,
                ExternalSet::Spawn,
                ExternalSet::ViewBindings,
                CoreSet::Compositor,
                CoreSet::Coordinate,
                ExternalSet::Configure,
                CoreSet::CoordinateFinalize,
                CoreSet::Visibility,
                CoreSet::Differential,
                CoreSet::RenderPacket,
            )
                .chain(),
        );
        elm.main().add_systems((
            crate::scene::despawn_bindings.in_set(ExternalSet::ViewBindings),
            (
                crate::scene::resolve_anchor,
                crate::scene::update_from_anchor,
            )
                .chain()
                .in_set(CoreSet::Coordinate),
            crate::coordinate::position_set.in_set(CoreSet::CoordinateFinalize),
            crate::coordinate::area_set.in_set(CoreSet::CoordinateFinalize),
            crate::differential::send_render_packet.in_set(CoreSet::RenderPacket),
            crate::differential::despawn
                .in_set(CoreSet::RenderPacket)
                .after(crate::differential::send_render_packet),
        ));
        elm.main().add_systems((
            apply_deferred
                .after(CoreSet::ExternalEvent)
                .before(CoreSet::Interaction),
            apply_deferred
                .after(CoreSet::Interaction)
                .before(ExternalSet::InteractionTriggers),
            apply_deferred
                .after(ExternalSet::InteractionTriggers)
                .before(ExternalSet::Process),
            apply_deferred
                .after(ExternalSet::Process)
                .before(CoreSet::ProcessEvent),
            apply_deferred
                .after(CoreSet::ProcessEvent)
                .before(CoreSet::TransitionView),
            apply_deferred
                .after(CoreSet::TransitionView)
                .before(ExternalSet::Spawn),
            apply_deferred
                .after(ExternalSet::Spawn)
                .before(ExternalSet::ViewBindings),
            apply_deferred
                .after(ExternalSet::ViewBindings)
                .before(CoreSet::Compositor),
            apply_deferred
                .after(CoreSet::Compositor)
                .before(CoreSet::Coordinate),
            apply_deferred
                .after(CoreSet::Coordinate)
                .before(ExternalSet::Configure),
            apply_deferred
                .after(ExternalSet::Configure)
                .before(CoreSet::CoordinateFinalize),
        ));
        let mut config = Self(elm);
        for leaf in leaflets.iter() {
            leaf.0(&mut config);
        }
    }
}
