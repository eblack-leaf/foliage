use crate::elm::leaf::{Leaf, Leaflet};
use crate::elm::Elm;
use crate::scene::Scene;
use bevy_ecs::prelude::{apply_deferred, IntoSystemConfigs, SystemSet};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum ExternalSet {
    Process,
    ViewBindings,
    Spawn,
    Configure,
}
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum CoreSet {
    ExternalEvent,
    Interaction,
    // Process,
    ProcessEvent,
    TransitionView,
    // ViewBindings,
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
    pub fn configure_hook<L: SystemSet + Hash + Eq + PartialEq + Debug + Copy + Clone>(
        &mut self,
        external_set: ExternalSet,
        hook: L,
    ) {
        use bevy_ecs::prelude::IntoSystemSetConfigs;
        self.0.job.main().configure_sets(hook.in_set(external_set));
    }
    pub fn establish_scene_config<
        S: Scene,
        SET: SystemSet + Hash + Eq + PartialEq + Debug + Copy + Clone,
    >(
        &mut self,
        set: SET,
    ) {
        // self.0.main().add_systems(r_scene::config<S>.in_set(set));
    }
    pub(crate) fn configure_elm(elm: &'a mut Elm, leaflets: &[Leaflet]) {
        use bevy_ecs::prelude::IntoSystemSetConfigs;
        elm.main().configure_sets(
            (
                CoreSet::ExternalEvent,
                CoreSet::Interaction,
                ExternalSet::Process,
                CoreSet::ProcessEvent,
                CoreSet::TransitionView,
                ExternalSet::Spawn,
                ExternalSet::ViewBindings,
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
            crate::scene::despawn_scenes.in_set(ExternalSet::ViewBindings),
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
