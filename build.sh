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
    RUST_OUTPUT="$RUST_TARGET_DIR/debug/lankm-headless"
    cargo build --target-dir "$RUST_TARGET_DIR"
popd

# Test for the presence of a vcvars64.bat variable as an indication
# we're on Windows
if [ ! -z "$VSCMD_VER" ]; then
    RUST_OUTPUT="$RUST_OUTPUT.exe"
    GUI_OUTPUT="$GUI_BUILD_DIR/publish/LanKM/release_win-x64/LanKM.exe"

    # We use 'publish' instead of build here so that we get a single
    # executable instead of an exe and a dll
    dotnet publish gui/Windows --artifacts-path "$GUI_BUILD_DIR"
else
    pushd gui/gnu
        if [[ ! -e "$GUI_BUILD_DIR/build.ninja" ]]; then
            meson setup "$GUI_BUILD_DIR" .
        fi

        ninja -C "$GUI_BUILD_DIR"
    popd
fi

# The gui will look for the backend on the same folder it's on
# so we copy both of them to a common location
cp -v $RUST_OUTPUT $LOCAL_DIR/
cp -v $GUI_OUTPUT $LOCAL_DIR/
