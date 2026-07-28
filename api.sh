rm -rf target/doc
cargo doc --no-deps -p foliage
rm -rf docs/api
mkdir -p docs/api
cp -r target/doc/* docs/api
# foliage_proper re-exports bevy_ecs wholesale (`pub use bevy_ecs::{self, prelude::*}`),
# and --no-deps leaves rustdoc no external docs to link to, so it inlines the entire crate
# -- 74M of the ~98M output. Dropped: links to bevy's own types 404, but every foliage item
# documents fine and docs/ stays a sane size to keep in git.
rm -rf docs/api/foliage/bevy_ecs
# another ~8M of index nobody asked for: search.index powers the search box (which stops
# working without it) and type.impl the "Implementations on Foreign Types" sections.
rm -rf docs/api/search.index docs/api/type.impl
