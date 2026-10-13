//! Where the device is: the latest fix the platform gave, read by an app whenever it likes.
//!
//! On Android the activity `foliage-android init` writes for an app that asks for a location
//! permission listens to every provider the device has on, and hands each fix here through
//! [`Java_dev_foliage_Locator_fix`]. Each one that is no worse than the one held -- see
//! [`better`] -- is kept, and wakes the loop, so a frame runs to read it. Everywhere else nothing
//! is ever handed in, and [`location`] says there is none.

use std::sync::{Mutex, OnceLock};

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

/// How much newer a fix has to be to stand over a more accurate one: a GPS fix is not given up
/// for a network one every second, but one from a minute ago is not where the device is now.
const STALE_MS: i64 = 10_000;

/// The fix held, and the wake that says a new one came.
static HELD: Mutex<Option<Fix>> = Mutex::new(None);
static WAKE: OnceLock<Wake> = OnceLock::new();

/// The latest fix the platform gave, or none where it has given none -- every desktop, and an
/// Android app that asks for no location permission or was refused it.
pub fn location() -> Option<Fix> {
    *HELD.lock().unwrap_or_else(|e| e.into_inner())
}

/// Hands the loop's wake over, so a fix arriving off the frame is answered by one.
pub(crate) fn attach(wake: &Wake) {
    let _ = WAKE.set(wake.clone());
}

/// Keeps `fix` if it is no worse than what is held, and wakes the loop for it.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) fn hand(fix: Fix) {
    let mut held = HELD.lock().unwrap_or_else(|e| e.into_inner());
    if held.is_some_and(|held| !better(fix, held)) {
        return;
    }
    *held = Some(fix);
    drop(held);
    if let Some(wake) = WAKE.get() {
        wake.rouse();
    }
}

/// Whether `new` should stand over `held`: it is at least as accurate, or `held` is old enough not
/// to be where the device is now. A fix that says no accuracy is taken as the worst.
fn better(new: Fix, held: Fix) -> bool {
    let off = |fix: Fix| fix.accuracy.unwrap_or(f32::INFINITY);
    new.time >= held.time && (off(new) <= off(held) || new.time - held.time >= STALE_MS)
}

/// The activity's native method: one fix, as Android's `Location` said it. An accuracy below zero
/// is none, and a bearing that is not a number is none.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_foliage_Locator_fix(
    _env: *mut core::ffi::c_void,
    _class: *mut core::ffi::c_void,
    lat: f64,
    lon: f64,
    accuracy: f32,
    bearing: f32,
    time: i64,
) {
    hand(Fix {
        lat,
        lon,
        accuracy: (accuracy >= 0.0).then_some(accuracy),
        bearing: (!bearing.is_nan()).then_some(bearing),
        time,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_worse_fix_does_not_stand_over_a_better_one_until_that_one_is_stale() {
        let gps = fix(Some(5.0), 1_000);
        assert!(!better(fix(Some(900.0), 2_000), gps));
        assert!(better(fix(Some(4.0), 2_000), gps));
        assert!(better(fix(Some(900.0), 1_000 + STALE_MS), gps));
        assert!(!better(fix(None, 2_000), gps));
        assert!(
            !better(fix(Some(1.0), 500), gps),
            "an older fix never stands over a newer"
        );
    }
}
