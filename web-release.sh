cd application || exit
trunk build --release
rm -rf ../docs/*
mv dist/* ../docs
cd ..
mdbook build foliage/book
mkdir -p docs/book
cp -r foliage/book/dist/* docs/book
rm -rf target/doc
cargo doc --no-deps -p foliage
rm -rf docs/api
mkdir -p docs/api
cp -r target/doc/* docs/api
rm -rf docs/api/foliage/bevy_ecs
rm -rf docs/api/search.index docs/api/type.impl
