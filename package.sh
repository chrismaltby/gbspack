#!/bin/sh
VERSION=`awk -F ' = ' '$1 ~ /version/ { gsub(/[\"]/, "", $2); printf("%s",$2) }' Cargo.toml`

mkdir -p releases/$VERSION/mac
mkdir -p releases/$VERSION/linux
mkdir -p releases/$VERSION/windows

cp README.md releases/$VERSION/mac
cp LICENSE releases/$VERSION/mac

cp README.md releases/$VERSION/linux
cp LICENSE releases/$VERSION/linux

cp README.md releases/$VERSION/windows
cp LICENSE releases/$VERSION/windows

cargo build --release
mv target/release/gbspack releases/$VERSION/mac

cross build --target x86_64-unknown-linux-gnu --release
mv target/x86_64-unknown-linux-gnu/release/gbspack releases/$VERSION/linux

cross build --target i686-pc-windows-gnu --release
mv target/i686-pc-windows-gnu/release/gbspack.exe releases/$VERSION/windows

cd releases/$VERSION
cd mac
upx gbspack
zip gbspack_${VERSION}_mac.zip gbspack LICENSE README.md
cd ..
cd linux
zip gbspack_${VERSION}_linux.zip gbspack LICENSE README.md
cd ..
cd windows
upx gbspack.exe
zip gbspack_${VERSION}_windows.zip gbspack.exe LICENSE README.md
cd ..
mv mac/gbspack_${VERSION}_mac.zip .
mv linux/gbspack_${VERSION}_linux.zip .
mv windows/gbspack_${VERSION}_windows.zip .
