#!/bin/bash
set -xeo pipefail
source scripts/build-helpers.sh

# `x86_64-osx` is also available, requires osxcross to be installed (see bwt/builder-osx.Dockerfile)
TARGETS=${TARGETS:-x86_64-linux,x86_64-win,arm32v7-linux,arm64v8-linux}

# Build library variants for the specified computer (non-android) platform
build() {
  if [[ $TARGETS != *"$1"* ]]; then return; fi

  # enable features for STDERR logging and automatic detection of bitcoind's data dir location
  build_platform $1 $2 pretty_env_logger,dirs
}

build x86_64-linux   x86_64-unknown-linux-gnu
build x86_64-osx     x86_64-apple-darwin
build x86_64-windows x86_64-pc-windows-gnu
build arm32v7-linux  armv7-unknown-linux-gnueabihf
build arm64v8-linux  aarch64-unknown-linux-gnu

# Android targets are built separately, see build-android.sh