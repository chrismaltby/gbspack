#!/bin/sh
VERSION=`awk -F ' = ' '$1 ~ /version/ { gsub(/[\"]/, "", $2); printf("%s",$2) }' Cargo.toml`

mkdir -p releases/$VERSION/mac-arm64

cp README.md releases/$VERSION/mac-arm64
cp LICENSE releases/$VERSION/mac-arm64

cargo build --release
mv target/release/gbspack releases/$VERSION/mac-arm64

cd releases/$VERSION
cd mac-arm64
upx gbspack
zip gbspack_${VERSION}_mac-arm64.zip gbspack LICENSE README.md
cd ..
mv mac-arm64/gbspack_${VERSION}_mac-arm64.zip .
cd ../..
rm gbspack
cp releases/$VERSION/mac-arm64/gbspack .
