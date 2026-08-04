/// The one macro an app needs, and the only one that was ever reachable from out here.
///
/// The five bevy-derive wrappers this used to re-export -- `component`, `resource`,
/// `query_data`, `system_set`, `event` -- date from before the boundary, when writing an app
/// meant declaring your own components and systems. Nothing an app touches is a bevy type
/// now, so there was nothing left to derive; they had no callers in the workspace and could
/// not have had useful ones outside it. `targeted_event` went with them for a harder reason:
/// it emits `impl foliage::TargetedEvent`, and that trait is `pub(crate)` in
/// `foliage_proper`, so it never compiled anywhere but inside the engine -- which invokes it
/// as `#[foliage_macros::targeted_event]` and is unaffected.
pub use foliage_macros::icon_handle;
pub use foliage_proper::*;
