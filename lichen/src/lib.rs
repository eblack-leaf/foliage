//! One taste applied to `foliage`, extracted from the repo's own site once the site has
//! earned it -- deliberately *not* part of `foliage_proper::composite`, which stays
//! unopinionated (a `Slider` has no house style; anything in here has exactly one). Depend on
//! this only if you want that specific look; `foliage` alone is the framework.
//!
//! Currently empty, and intentionally so. The previous contents were distilled from an
//! earlier iteration of the app that has since been rebuilt from scratch, which left them
//! describing a look nothing uses -- a house style for a house that was torn down.
//!
//! The refill comes from `application/src/site`, where the patterns are being written against
//! a real page first: the card, the blueprint plate, the cutout badge, the morph entrance.
//! Each of those already takes its placement as a `Location` rather than deciding where it
//! lives, which is the property that makes one movable here at all. They stay in the app
//! until the polish settles, so that what lands here is a shape proven by use rather than one
//! guessed at ahead of the second call site.
