cd application || exit
trunk build --release
rm -rf ../docs/*
mv dist/* ../docs
cd ..

mdbook build foliage/book
mkdir -p docs/book
cp -r foliage/book/dist/* docs/book
