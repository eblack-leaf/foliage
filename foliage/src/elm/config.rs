use crate::elm::leaf::{Leaf, Leaflet};
use crate::elm::Elm;
use bevy_ecs::prelude::{apply_deferred, IntoSystemConfigs, SystemSet};

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum ExternalSet {
    Process,
    CompositorBind,
    CompositorExtension,
    Spawn,
    SceneBind,
    Configure,
    Resolve,
}
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum CoreSet {
    ExternalEvent,
    // Process,
    ProcessEvent,
    CompositorSetup,
    // CompositorBind,
    // CompositorExtension,
    CompositorTeardown,
    // Spawn,
    SceneSetup,
    // SceneBind,
    SceneTeardown,
    SceneExtension,
    // Configure,
    SceneResolve,
    // Resolve,
    SceneFinalize,
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
                ExternalSet::Process,
                CoreSet::ProcessEvent,
                CoreSet::CompositorSetup,
                ExternalSet::CompositorBind,
                ExternalSet::CompositorExtension,
                CoreSet::CompositorTeardown,
                ExternalSet::Spawn,
                CoreSet::SceneSetup,
                ExternalSet::SceneBind,
                CoreSet::SceneTeardown,
                CoreSet::SceneExtension,
                ExternalSet::Configure,
                CoreSet::SceneResolve,
                ExternalSet::Resolve,
                CoreSet::SceneFinalize,
                CoreSet::CoordinateFinalize,
                CoreSet::Visibility,
                CoreSet::Differential,
                CoreSet::RenderPacket,
            )
                .chain(),
        );
        elm.main().add_systems((
            // crate::scene::hook_to_anchor.in_set(CoreSet::SceneTeardown),
            // crate::scene::scene_register
            //     .in_set(CoreSet::SceneResolve)
            //     .before(crate::scene::align::calc_alignments),
            // crate::scene::resolve_anchor
            //     .in_set(CoreSet::SceneResolve)
            //     .before(crate::scene::align::calc_alignments)
            //     .after(crate::scene::scene_register),
            // apply_deferred
            //     .in_set(CoreSet::SceneResolve)
            //     .before(crate::scene::align::calc_alignments)
            //     .after(crate::scene::resolve_anchor),
            // crate::scene::align::calc_alignments.in_set(CoreSet::SceneResolve),
            // crate::scene::hook_to_anchor
            //     .in_set(CoreSet::SceneFinalize)
            //     .before(crate::scene::scene_register_two),
            crate::scene::scene_register_two
                .in_set(CoreSet::SceneFinalize)
                .before(crate::scene::align::calc_alignments_two),
            crate::scene::resolve_anchor_two
                .in_set(CoreSet::SceneFinalize)
                .before(crate::scene::align::calc_alignments_two)
                .after(crate::scene::scene_register_two),
            apply_deferred
                .in_set(CoreSet::SceneFinalize)
                .before(crate::scene::align::calc_alignments_two)
                .after(crate::scene::resolve_anchor_two),
            crate::scene::align::calc_alignments_two.in_set(CoreSet::SceneFinalize),
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
                .before(ExternalSet::Process),
            apply_deferred
                .after(ExternalSet::Process)
                .before(CoreSet::ProcessEvent),
            apply_deferred
                .after(CoreSet::ProcessEvent)
                .before(CoreSet::CompositorSetup),
            apply_deferred
                .after(CoreSet::CompositorSetup)
                .before(ExternalSet::CompositorBind),
            apply_deferred
                .after(ExternalSet::CompositorBind)
                .before(ExternalSet::CompositorExtension),
            apply_deferred
                .after(ExternalSet::CompositorExtension)
                .before(CoreSet::CompositorTeardown),
            apply_deferred
                .after(CoreSet::CompositorTeardown)
                .before(ExternalSet::Spawn),
            apply_deferred
                .after(ExternalSet::Spawn)
                .before(CoreSet::SceneSetup),
            apply_deferred
                .after(CoreSet::SceneSetup)
                .before(ExternalSet::SceneBind),
            apply_deferred
                .after(ExternalSet::SceneBind)
                .before(CoreSet::SceneTeardown),
            apply_deferred
                .after(CoreSet::SceneTeardown)
                .before(CoreSet::SceneExtension),
            apply_deferred
                .after(CoreSet::SceneExtension)
                .before(ExternalSet::Configure),
            apply_deferred
                .after(ExternalSet::Configure)
                .before(CoreSet::SceneResolve),
            apply_deferred
                .after(CoreSet::SceneResolve)
                .before(CoreSet::CoordinateFinalize),
        ));
        let mut config = Self(elm);
        for leaf in leaflets.iter() {
            leaf.0(&mut config);
        }
    }
}