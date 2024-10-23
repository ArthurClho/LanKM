#!/bin/bash
# Simple helper script to build the Rust backend and the Vala+GTK gui

# TODO: Add "build", "run" and "install" actions

# Quit if anything fails
set -e

# Cd into this scripts directory
cd $(dirname $0)

LOCAL_DIR=".local"

mkdir -p $LOCAL_DIR

pushd lankm
    cargo build
    RUST_OUTPUT=$(realpath "./target/debug/lankm-headless")
popd

# Test for the presence of a vcvars64.bat variable as an indication
# we're on Windows
if [ ! -z "$VSCMD_VER" ]; then
    RUST_OUTPUT="$RUST_OUTPUT.exe"

    # We use 'publish' instead of build here so that we get a single
    # executable instead of an exe and a dll
    dotnet publish gui/Windows

    GUI_OUTPUT=$(realpath "gui/Windows/bin/Release/net8.0-windows/win-x64/publish/LanKM.exe")
else
    pushd gui/gnu
        if [[ ! -e "./build/build.ninja" ]]; then
            meson setup "./build" .
        fi

        ninja -C ./build

        GUI_OUTPUT=$(realpath "./build/src/lankm-gui")
    popd
fi

# The gui will look for the backend on the same folder it's on
# so we copy both of them to a common location
cp -v $RUST_OUTPUT $LOCAL_DIR/
cp -v $GUI_OUTPUT $LOCAL_DIR/
