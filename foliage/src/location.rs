//! Where the device is: the latest fix the platform gave, read by an app whenever it likes.
//!
//! On Android the activity `foliage-android init` writes for an app that asks for a location
//! permission listens to every provider the device has on, and hands each fix here through
//! [`Java_dev_foliage_Locator_located`], saying which provider it came from. Each one that should
//! stand over the one held -- see [`better`] -- is kept, and wakes the loop, so a frame runs to
//! read it. Everywhere else nothing is ever handed in, and [`location`] says there is none.

use std::sync::Mutex;
#[cfg(not(target_family = "wasm"))]
use std::sync::OnceLock;

use crate::queue::Wake;

/// Where the device was, as one provider said.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Fix {
    /// North of the equator, in degrees.
    pub lat: f64,
    /// East of Greenwich, in degrees.
    pub lon: f64,
    /// How far off it may be, in metres, as the provider reckons it: the radius it is two thirds
    /// likely to be within. None where the provider said nothing.
    pub accuracy: Option<f32>,
    /// Which way it was moving, in degrees clockwise from north. None where it was not.
    pub bearing: Option<f32>,
    /// When it was taken, in milliseconds since the Unix epoch.
    pub time: i64,
}

/// How much newer a fix from another provider has to be to stand over a more accurate one: a GPS
/// fix is not given up for a network one every second, but one from a minute ago is not where the
/// device is now.
const STALE_MS: i64 = 10_000;

/// Which provider a fix came from, as the activity numbers them: 1 GPS, 2 network, 3 fused, 0 any
/// other. Kept beside the fix rather than on it, since it says how to weigh the next fix and
/// nothing about where the device is.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Source(pub(crate) i32);

/// The fix held and where it came from, and the wake that says a new one came.
static HELD: Mutex<Option<(Fix, Source)>> = Mutex::new(None);
#[cfg(not(target_family = "wasm"))]
static WAKE: OnceLock<Wake> = OnceLock::new();
// On the web a wake is not `Sync` -- see `queue::Rouse` -- so it can't be a static. There is one
// thread there, so that thread's own is the same thing.
#[cfg(target_family = "wasm")]
thread_local! {
    static WAKE: std::cell::OnceCell<Wake> = const { std::cell::OnceCell::new() };
}

/// The latest fix the platform gave, or none where it has given none -- every desktop, and an
/// Android app that asks for no location permission or was refused it.
pub fn location() -> Option<Fix> {
    HELD.lock()
        .unwrap_or_else(|e| e.into_inner())
        .map(|(fix, _)| fix)
}

/// Hands the loop's wake over, so a fix arriving off the frame is answered by one.
pub(crate) fn attach(wake: &Wake) {
    #[cfg(not(target_family = "wasm"))]
    let _ = WAKE.set(wake.clone());
    #[cfg(target_family = "wasm")]
    WAKE.with(|held| {
        let _ = held.set(wake.clone());
    });
}

/// Keeps `fix`, from `source`, if it should stand over what is held, and wakes the loop for it.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) fn hand(fix: Fix, source: Source) {
    let mut held = HELD.lock().unwrap_or_else(|e| e.into_inner());
    if held.is_some_and(|held| !better((fix, source), held)) {
        return;
    }
    *held = Some((fix, source));
    drop(held);
    #[cfg(not(target_family = "wasm"))]
    let wake = WAKE.get().cloned();
    #[cfg(target_family = "wasm")]
    let wake = WAKE.with(|held| held.get().cloned());
    if let Some(wake) = wake {
        wake.rouse();
    }
}

/// Whether `new` should stand over `held`. Never when it is older. From the same provider, always:
/// a provider's newest fix is its best word on where the device is, and the accuracy it reckons
/// wanders a little from one fix to the next, so holding out for one as good as the last would
/// keep the device where it was -- by as much as [`STALE_MS`] at a time, slow enough to show at
/// a walk or in town. From another, when it is at least as accurate, or `held` is old enough not
/// to be where the device is now. A fix that says no accuracy is taken as the worst.
fn better((new, from): (Fix, Source), (held, had): (Fix, Source)) -> bool {
    let off = |fix: Fix| fix.accuracy.unwrap_or(f32::INFINITY);
    new.time >= held.time
        && (from == had || off(new) <= off(held) || new.time - held.time >= STALE_MS)
}

/// How fast the device has to be going, in metres a second, for the way it is going to be one: a
/// slow walk. Below it a provider still says a bearing, but a fix wandering a few metres about a
/// device at rest is all it is measured from, and it turns every second.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
const MOVING: f32 = 1.0;

/// Which way the device is going, from the bearing and speed a provider said: none where it said no
/// bearing, and none where it said too little speed -- or none -- for the bearing to be a way.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn heading(bearing: f32, speed: f32) -> Option<f32> {
    (!bearing.is_nan() && speed >= MOVING).then_some(bearing)
}

/// The activity's native method: one fix, as Android's `Location` said it, and which provider said
/// it. An accuracy or a speed below zero is none, and a bearing that is not a number is none.
///
/// Named apart from the `fix` an activity written before the provider was passed declares, so one
/// of those fails to link -- and says so -- rather than calling this with arguments short.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub extern "system" fn Java_dev_foliage_Locator_located(
    _env: *mut core::ffi::c_void,
    _class: *mut core::ffi::c_void,
    lat: f64,
    lon: f64,
    accuracy: f32,
    bearing: f32,
    speed: f32,
    time: i64,
    source: i32,
) {
    hand(
        Fix {
            lat,
            lon,
            accuracy: (accuracy >= 0.0).then_some(accuracy),
            bearing: heading(bearing, speed),
            time,
        },
        Source(source),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const GPS: Source = Source(1);
    const NETWORK: Source = Source(2);

    fn fix(accuracy: Option<f32>, time: i64) -> Fix {
        Fix {
            lat: 49.61,
            lon: 6.13,
            accuracy,
            bearing: None,
            time,
        }
    }

    #[test]
    fn a_worse_fix_from_elsewhere_does_not_stand_over_a_better_one_until_that_one_is_stale() {
        let gps = (fix(Some(5.0), 1_000), GPS);
        assert!(!better((fix(Some(900.0), 2_000), NETWORK), gps));
        assert!(better((fix(Some(4.0), 2_000), NETWORK), gps));
        assert!(better((fix(Some(900.0), 1_000 + STALE_MS), NETWORK), gps));
        assert!(!better((fix(None, 2_000), NETWORK), gps));
    }

    #[test]
    fn the_next_fix_from_the_same_provider_stands_however_it_reckons_itself() {
        let gps = (fix(Some(3.8), 1_000), GPS);
        assert!(better((fix(Some(4.2), 2_000), GPS), gps));
        assert!(better((fix(None, 2_000), GPS), gps));
    }

    #[test]
    fn a_bearing_is_a_heading_only_while_the_device_is_moving() {
        assert_eq!(heading(270.0, 13.4), Some(270.0));
        assert_eq!(heading(270.0, MOVING), Some(270.0));
        assert_eq!(heading(270.0, 0.3), None, "at rest, a bearing is noise");
        assert_eq!(heading(270.0, -1.0), None, "no speed said");
        assert_eq!(heading(f32::NAN, 13.4), None, "no bearing said");
    }

    #[test]
    fn an_older_fix_never_stands_over_a_newer() {
        let gps = (fix(Some(5.0), 1_000), GPS);
        assert!(!better((fix(Some(1.0), 500), GPS), gps));
        assert!(!better((fix(Some(1.0), 500), NETWORK), gps));
    }
}
