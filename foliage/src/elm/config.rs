use crate::elm::leaf::{Leaf, Leaflet};
use crate::elm::Elm;
use bevy_ecs::prelude::{IntoSystemConfigs, SystemSet};


#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum ExternalSet {
    Process,
    Resolve,
}
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum CoreSet {
    Event,
    // Process,
    Spawn,
    // Resolve,
    Coordinate,
    Visibility,
    Differential,
    RenderPacket,
}
pub struct ElmConfiguration<'a>(&'a mut Elm);
impl<'a> ElmConfiguration<'a> {
    pub fn configure_hook<L: Leaf>(&mut self, external_set: ExternalSet, hook: <L as Leaf>::SetDescriptor) {
        use bevy_ecs::prelude::IntoSystemSetConfigs;
        self.0.job.main().configure_sets(hook.in_set(external_set));
    }
    pub(crate) fn configure_elm(elm: &'a mut Elm, leaflets: &[Leaflet]) {
        use bevy_ecs::prelude::IntoSystemSetConfigs;
        elm.main().configure_sets(
            (
                CoreSet::Event,
                ExternalSet::Process,
                CoreSet::Spawn,
                ExternalSet::Resolve,
                CoreSet::Coordinate,
                CoreSet::Visibility,
                CoreSet::Differential,
                CoreSet::RenderPacket,
            )
                .chain(),
        );
        elm.main().add_systems((
            crate::scene::register_root
                .in_set(CoreSet::Coordinate)
                .before(crate::scene::align::calc_alignments),
            crate::scene::resolve_anchor
                .in_set(CoreSet::Coordinate)
                .before(crate::scene::align::calc_alignments)
                .after(crate::scene::register_root),
            crate::scene::align::calc_alignments.in_set(CoreSet::Coordinate),
            crate::coordinate::position_set
                .in_set(CoreSet::Coordinate)
                .after(crate::scene::align::calc_alignments),
            crate::coordinate::area_set
                .in_set(CoreSet::Coordinate)
                .after(crate::scene::align::calc_alignments),
            crate::differential::send_render_packet.in_set(CoreSet::RenderPacket),
            crate::differential::despawn
                .in_set(CoreSet::RenderPacket)
                .after(crate::differential::send_render_packet),
        ));
        let mut config = Self(elm);
        for leaf in leaflets.iter() {
            leaf.0(&mut config);
        }
    }
}