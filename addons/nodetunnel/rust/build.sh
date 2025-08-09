#!/bin/bash
set -e

echo "Building NodeTunnel Rust extension..."
cargo build --release

echo "Copying library to bin folder..."
mkdir -p ../bin

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cp target/release/libnodetunnel.so ../bin/
    echo "Copied libnodetunnel.so"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    cp target/release/libnodetunnel.dylib ../bin/
    echo "Copied libnodetunnel.dylib"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    cp target/release/nodetunnel.dll ../bin/
    echo "Copied nodetunnel.dll"
fi

echo "Build complete!"
