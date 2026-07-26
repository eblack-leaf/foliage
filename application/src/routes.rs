/// One name per route index, in the exact order `entry.rs` registers them in
/// `RouterRoutes::new([..])` -- the single source of truth for "what is route N called",
/// used by both `navigator.rs` (forward/back labels) and `toc.rs` (card titles) so the
/// two can't independently drift out of sync with the actual route list the way
/// `navigator.rs`'s own local `PAGE_NAMES` just did when `next`/`third` were replaced by
/// the nine `chapters` routes (stale 4-entry array against an 11-route list -- both an
/// index-out-of-bounds panic past index 4 and wrong labels below that).
pub const ROUTE_NAMES: [&str; 10] = [
    "home",
    "contents",
    "location",
    "elevate",
    "relative",
    "grid",
    "anchor",
    "animate",
    "sequence",
    "interact",
];
