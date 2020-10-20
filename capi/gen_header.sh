#!/usr/bin/env bash

CAPI_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

WORK_DIR=`mktemp -d`


# check if tmp dir was created
if [[ ! "$WORK_DIR" || ! -d "$WORK_DIR" ]]; then
    echo "Could not create temp dir"
    exit 1
fi

function cleanup {
    #echo "$WORK_DIR"
    rm -rf "$WORK_DIR"
}

trap cleanup EXIT

mkdir "$WORK_DIR/src"

# Fake a library
cat > "$WORK_DIR/src/lib.rs" << EOF
#[path = "$CAPI_DIR/../src/ffi/mod.rs"]
pub mod ffi;
EOF

# And its Cargo.toml
cat > "$WORK_DIR/Cargo.toml" << EOF
[package]
name = "hyper"
version = "0.0.0"
edition = "2018"
publish = false

[dependencies]
EOF

#cargo metadata --no-default-features --features ffi --format-version 1 > "$WORK_DIR/metadata.json"

cd $WORK_DIR

# Expand just the ffi module
cargo rustc -- -Z unstable-options --pretty=expanded > expanded.rs 2>/dev/null

# Replace the previous copy with the single expanded file
rm -rf ./src
mkdir src
mv expanded.rs src/lib.rs


# Bindgen!
cbindgen\
    -c "$CAPI_DIR/cbindgen.toml"\
    --lockfile "$CAPI_DIR/../Cargo.lock"\
    -o "$CAPI_DIR/include/hyper.h"\
    $1
