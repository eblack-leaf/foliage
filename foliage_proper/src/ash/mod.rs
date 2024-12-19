use crate::ash::clip::prepare_clip_section;
use crate::{Attachment, Component, DiffMarkers, Foliage, Resource};
use bevy_ecs::prelude::IntoSystemConfigs;

pub(crate) mod clip;
pub(crate) mod differential;
pub(crate) mod queue;

pub struct Ash {}
impl Attachment for Ash {
    fn attach(foliage: &mut Foliage) {
        foliage
            .diff
            .add_systems(prepare_clip_section.in_set(DiffMarkers::Prepare));
    }
}
