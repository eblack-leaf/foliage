cd application || exit
trunk build --release
mv dist/* ../docs
