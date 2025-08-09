#!/bin/bash
set -e

echo "Building for all platforms..."

# Build for each target
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin

echo "Copying all libraries..."
mkdir -p ../bin

cp target/x86_64-unknown-linux-gnu/release/libnodetunnel.so ../bin/
cp target/x86_64-pc-windows-gnu/release/nodetunnel.dll ../bin/
cp target/x86_64-apple-darwin/release/libnodetunnel.dylib ../bin/

echo "All platforms built!"