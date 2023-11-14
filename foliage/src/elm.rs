use crate::job::Job;
use crate::Leaflet;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, SystemSet};

pub struct Elm {
    initialized: bool,
    pub job: Job,
}
impl Elm {
    pub(crate) fn new() -> Self {
        Self {
            initialized: false,
            job: Job::new(),
        }
    }
    pub(crate) fn initialized(&self) -> bool {
        self.initialized
    }
    pub(crate) fn attach_leafs(&mut self, leaflets: Vec<Leaflet>) {
        for leaf in leaflets {
            leaf.0(self)
        }
        self.initialized = true;
    }
    pub fn enable_differential<T: Component + Clone + PartialEq>(&mut self) {
        self.job.main().add_systems((
            crate::differential::differential::<T>.in_set(SystemSets::Differential),
        ));
    }
    pub(crate) fn finish_initialization(&mut self) {
        self.job.resume();
        self.initialized = true;
    }
}
#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum SystemSets {
    Differential,
}
