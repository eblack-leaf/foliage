use crate::elm::leaf::{Leaf, Leaflet};
use crate::elm::Elm;
use bevy_ecs::prelude::{apply_deferred, IntoSystemConfigs, SystemSet};

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum ExternalSet {
    Process,
    Spawn,
    Configure,
}
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum CoreSet {
    ExternalEvent,
    Interaction,
    // Process,
    ProcessEvent,
    ViewTransition,
    // Spawn,
    Compositor,
    // Configure,
    CoordinateFinalize,
    Visibility,
    Differential,
    RenderPacket,
}
pub struct ElmConfiguration<'a>(&'a mut Elm);
impl<'a> ElmConfiguration<'a> {
    pub fn configure_hook<L: Leaf>(
        &mut self,
        external_set: ExternalSet,
        hook: <L as Leaf>::SetDescriptor,
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
                ExternalSet::Process,
                CoreSet::ProcessEvent,
                CoreSet::ViewTransition,
                ExternalSet::Spawn,
                CoreSet::Compositor,
                ExternalSet::Configure,
                CoreSet::CoordinateFinalize,
                CoreSet::Visibility,
                CoreSet::Differential,
                CoreSet::RenderPacket,
            )
                .chain(),
        );
        elm.main().add_systems((
            crate::scene::despawn_scenes.in_set(ExternalSet::Spawn),
            crate::scene::place_scenes
                .in_set(CoreSet::CoordinateFinalize)
                .before(crate::coordinate::position_set)
                .before(crate::coordinate::area_set),
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
                .before(ExternalSet::Process),
            apply_deferred
                .after(ExternalSet::Process)
                .before(CoreSet::ProcessEvent),
            apply_deferred
                .after(CoreSet::ProcessEvent)
                .before(CoreSet::ViewTransition),
            apply_deferred
                .after(CoreSet::ViewTransition)
                .before(ExternalSet::Spawn),
            apply_deferred
                .after(ExternalSet::Spawn)
                .before(CoreSet::Compositor),
            apply_deferred
                .after(CoreSet::Compositor)
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
