cd application || exit
trunk build --release
rm -rf ../docs/*
mv dist/* ../docs
