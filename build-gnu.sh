#!/bin/bash
# Simple helper script to build the Rust backend and the Vala+GTK gui

# TODO: Add "build", "run" and "install" actions

# Quit if anything fails
set -e

# Cd into this scripts directory
cd $(dirname $0)

BUILD_DIR=$(realpath "build")
RUST_TARGET_DIR="$BUILD_DIR/target"
GUI_BUILD_DIR="$BUILD_DIR/gui"
LOCAL_DIR="$BUILD_DIR/local"

mkdir -p $BUILD_DIR
mkdir -p $LOCAL_DIR

pushd lankm
    cargo build --target-dir "$RUST_TARGET_DIR"
popd

pushd gui/gnu
    if [[ ! -e "$GUI_BUILD_DIR/build.ninja" ]]; then
        meson setup "$GUI_BUILD_DIR" .
    fi

    ninja -C "$GUI_BUILD_DIR"
popd

# The gui will look for the backend on the same folder it's on
# so we copy both of them to a common location
cp -v $BUILD_DIR/gui/src/lankm-gui $LOCAL_DIR/
