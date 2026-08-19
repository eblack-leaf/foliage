//! The seam between an app and the engine.
//!
//! Everything crossing it is plain data: commands in, emissions out, and read-only samples
//! taken at the app's own callsite. No engine type is handed out, nothing an app holds
//! borrows from the world, and there is no way to reach the ECS from the far side -- which is
//! what lets an app run its own `bevy_ecs`, at whatever version it likes, without the two
//! meeting.
//!
//! - [`Leaf`](crate::Leaf) names an element. [`Canopy`](crate::Canopy) is the per-frame
//!   surface, and [`Root`](crate::Root) is the app it is handed to.
//! - [`Sprig`](crate::Sprig) is the same command set, `Send`, for another thread.
//! - [`Bloom`](crate::Bloom) is what comes back out; [`Sap`](crate::Sap)/
//!   [`Sample`](crate::Sample) is what may be looked at.

pub(crate) mod bloom;
pub(crate) mod canopy;
pub(crate) mod funnel;
pub(crate) mod leaf;
pub(crate) mod op;
pub(crate) mod root;
pub(crate) mod sprig;
pub(crate) mod tween;
pub(crate) mod verbs;
pub(crate) mod watch;

use crate::boundary::bloom::Emissions;
use crate::boundary::op::Spec;
use crate::foliage::{Foliage, MainMarkers};
use bevy_ecs::prelude::IntoScheduleConfigs;

/// The seam installs itself the same way every other part of the engine does.
pub(crate) struct Boundary;

impl crate::Attachment for Boundary {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(Emissions::default());
        foliage.world.insert_resource(watch::Watches::default());
        // In `diff` rather than `main`: this runs after the frame's own commands have landed,
        // so a watch reports the value the element settled on this frame rather than the one
        // it held before the frame acted on it.
        foliage
            .diff
            .add_systems(watch::report.in_set(crate::foliage::DiffMarkers::Prepare));
        // Alongside the component animations rather than after them: a tween's numbers and an
        // animated element's values belong to the same instant.
        foliage
            .main
            .add_systems(tween::drive_tweens.in_set(MainMarkers::Animation));
        funnel::Funnel::attach(foliage);
    }
}

/// `canopy.branch(under, Panel::new()...)` rather than
/// `canopy.branch(under, Spec::Panel(Panel::new()...))` -- the enum is the queued form, not
/// something a call site should have to name.
macro_rules! spec_from {
    ($($sprout:ident => $variant:ident),+ $(,)?) => {
        $(
            impl From<crate::$sprout> for Spec {
                fn from(sprout: crate::$sprout) -> Self {
                    Spec::$variant(sprout)
                }
            }
        )+
    };
}
spec_from!(
    LeafSprout => Bare,
    PanelSprout => Panel,
    TextSprout => Text,
    IconSprout => Icon,
    ImageSprout => Image,
    LineSprout => Line,
    PolygonSprout => Polygon,
    PolylineSprout => Polyline,
    TextInputSprout => TextInput,
);

// `WithExtras` deliberately has no conversion: it is the engine's own escape hatch for
// attaching components an app cannot name, and letting it through here would put an open
// generic back inside a closed set.
