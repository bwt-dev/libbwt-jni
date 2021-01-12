#!/bin/bash
set -xeo pipefail

source scripts/build-helpers.sh

TARGETS=${TARGETS:-arm32v7-android,arm64v8-android,i686-android,x86_64-android}

[ -z "$ANDROID_NDK_HOME" ] && export ANDROID_NDK_HOME=${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk-bundle}
export PATH=$PATH:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname | tr '[:upper:]' '[:lower:]')-x86_64/bin

# Build library variants for the specified android platform
build() {
  local platform_nick=$1; local platform_rust=$2; local platform_jni=$3
  if [[ $TARGETS != *"$platform_nick"* ]]; then return; fi

  build_platform $platform_nick $platform_rust android

  # Prepare 'all_android' variant directories with all android platform builds
  for variant in '' '-electrum_only'; do
    local src=dist/libbwt-jni-$version$variant-$platform_nick
    local dest=dist/libbwt-jni-$version$variant-all_android/jniLibs/$platform_jni
    if [ -d $src ]; then
      mkdir -p $dest
      cp $src/*bwt_jni* $dest/
    fi
  done
}

build arm32v7-android armv7-linux-androideabi armeabi-v7a
build arm64v8-android aarch64-linux-android   arm64-v8a
build i686-android    i686-linux-android      x86
build x86_64-android  x86_64-linux-android    x86_64

[ -n "$ELECTRUM_ONLY_ONLY" ] || pack libbwt-jni-$version-all_android
pack libbwt-jni-$version-electrum_only-all_android

# Build Android Archive Library
build_aar() {
  local variant=$1
  local jni_dir=dist/libbwt-jni-$version$variant-all_android/jniLibs
  if [ ! -d $jni_dir ]; then return; fi

  rm -rf library/src/main/jniLibs
  cp -r $jni_dir library/src/main/jniLibs
  ./gradlew build

  local name=libbwt-aar-$version$variant
  mkdir -p dist/$name
  mv library/build/outputs/aar/library-release.aar dist/$name/libbwt.aar
  pack $name
}

if [ -z "$SKIP_AAR" ]; then
  build_aar ''
  build_aar '-electrum_only'
fi
