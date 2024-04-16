use std::fmt::Debug;
use std::hash::Hash;

use bevy_ecs::prelude::{apply_deferred, IntoSystemConfigs, SystemSet};

use crate::elm::Elm;
use crate::elm::leaf::Leaflet;

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum ExternalSet {
    InteractionTriggers,
    Animation,
    Process,
    ConditionalBind,
    ConditionalExt,
    Spawn,
    Configure,
}
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum CoreSet {
    ExternalEvent,
    Interaction,
    // InteractionTriggers,
    // Animation,
    // Process,
    ProcessEvent,
    Navigation,
    ConditionPrepare,
    // ConditionalBind,
    // ConditionalExt,
    // Spawn,
    Compositor,
    SceneCoordinate,
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
                ExternalSet::Animation,
                ExternalSet::Process,
                CoreSet::ProcessEvent,
                CoreSet::Navigation,
                CoreSet::ConditionPrepare,
                ExternalSet::ConditionalBind,
                ExternalSet::ConditionalExt,
                ExternalSet::Spawn,
                CoreSet::Compositor,
                CoreSet::SceneCoordinate,
                ExternalSet::Configure,
                CoreSet::CoordinateFinalize,
                CoreSet::Visibility,
                CoreSet::Differential,
                CoreSet::RenderPacket,
            )
                .chain(),
        );
        elm.main().add_systems((
            apply_deferred
                .after(CoreSet::ExternalEvent)
                .before(CoreSet::Interaction),
            apply_deferred
                .after(CoreSet::Interaction)
                .before(ExternalSet::InteractionTriggers),
            apply_deferred
                .after(ExternalSet::InteractionTriggers)
                .before(ExternalSet::Animation),
            apply_deferred
                .after(ExternalSet::Animation)
                .before(ExternalSet::Process),
            apply_deferred
                .after(ExternalSet::Process)
                .before(CoreSet::ProcessEvent),
            apply_deferred
                .after(CoreSet::ProcessEvent)
                .before(CoreSet::Navigation),
            apply_deferred
                .after(CoreSet::Navigation)
                .before(CoreSet::ConditionPrepare),
            apply_deferred
                .after(CoreSet::ConditionPrepare)
                .before(ExternalSet::ConditionalBind),
            apply_deferred
                .after(ExternalSet::ConditionalBind)
                .before(ExternalSet::ConditionalExt),
            apply_deferred
                .after(ExternalSet::ConditionalExt)
                .before(ExternalSet::Spawn),
            apply_deferred
                .after(ExternalSet::Spawn)
                .before(CoreSet::Compositor),
            apply_deferred
                .after(CoreSet::Compositor)
                .before(CoreSet::SceneCoordinate),
            apply_deferred
                .after(CoreSet::SceneCoordinate)
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