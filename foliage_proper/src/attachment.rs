use crate::foliage::Foliage;

/// A unit of engine setup: the components, systems, resources and observers one feature
/// needs, installed together.
///
/// Engine-internal: every built-in primitive implements this, and [`Foliage::new`](crate::Foliage::new)
/// calls each one's `attach` in turn to build the instance. `pub(crate)` on purpose -- an
/// app has no engine-level setup to register, only the app-level calls `Foliage` exposes
/// directly (`desktop_size`, `font`, `icon`, `tune`, ...).
pub(crate) trait Attachment {
    fn attach(foliage: &mut Foliage);
}

/// A setting an app may install before the loop starts.
///
/// Deliberately a closed set rather than "any resource": these are the knobs the engine reads
/// and never writes, so handing one over cannot desynchronise anything. Sealed by
/// `Attachment` being `pub(crate)` -- nothing outside can add a member.
#[allow(private_bounds)]
pub trait Tuning: Sealed {
    #[doc(hidden)]
    fn install(self, foliage: &mut Foliage);
}

pub(crate) trait Sealed {}

macro_rules! tuning {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Sealed for $t {}
            impl Tuning for $t {
                fn install(self, foliage: &mut Foliage) {
                    foliage.world.insert_resource(self);
                }
            }
        )+
    };
}
tuning!(crate::ScrollMomentum, crate::KeyBindings);
