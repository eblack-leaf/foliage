use crate::boundary::moss::{Moss, Emissions};
use crate::boundary::forest::{Reads, sample};
use crate::boundary::leaf::Leaf;
use crate::{Sample, Sap};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::ResMut;

/// One standing request to be told what a property is.
struct Watch {
    leaf: Leaf,
    sap: Sap,
    /// What was last reported. `None` until the first report, which is why a fresh watch
    /// answers immediately rather than waiting for the value to move.
    last: Option<Sample<'static>>,
}

/// Every standing watch, and the last value each one reported.
///
/// A resource rather than components on the watched elements: a watch is not a property of
/// the element, it is a standing question asked from outside, and an element knows nothing
/// about who is asking.
#[derive(Resource, Default)]
pub(crate) struct Watches(Vec<Watch>);

impl Watches {
    /// Idempotent -- asking twice for the same reading is one watch, not two, so a worker can
    /// re-ask without arranging not to.
    pub(crate) fn add(&mut self, leaf: Leaf, sap: Sap) {
        if self
            .0
            .iter()
            .any(|watch| watch.leaf == leaf && watch.sap == sap)
        {
            return;
        }
        self.0.push(Watch {
            leaf,
            sap,
            last: None,
        });
    }
    pub(crate) fn remove(&mut self, leaf: Leaf, sap: Sap) {
        self.0
            .retain(|watch| !(watch.leaf == leaf && watch.sap == sap));
    }
}

/// Reports what changed, and drops what no longer exists.
///
/// Runs after the frame's commands have landed, so a value reported here is the one the
/// element actually settled on this frame rather than the one it held before the frame acted.
///
/// A watch whose element has withered stops here rather than reporting forever: the sample
/// comes back `None`, which is indistinguishable from a property the element never carried,
/// and neither is worth holding a watch open for.
pub(crate) fn report(mut watches: ResMut<Watches>, reads: Reads, mut emissions: ResMut<Emissions>) {
    watches.0.retain_mut(|watch| {
        let Some(current) = sample(&reads, watch.leaf, watch.sap) else {
            return false;
        };
        if watch.last.as_ref() == Some(&current) {
            return true;
        }
        let current = current.into_owned();
        emissions.push(Moss::Reading {
            leaf: watch.leaf,
            sap: watch.sap,
            value: current.clone(),
        });
        watch.last = Some(current);
        true
    });
}
