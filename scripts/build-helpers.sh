#!/bin/bash
set -xeo pipefail

[ -f bwt/Cargo.toml ] || (echo >&2 "Missing bwt submodule, run 'git submodule update --init'" && exit 1)

version=$(grep -E '^version =' Cargo.toml | cut -d'"' -f2)

if [[ -n "$SCCACHE_DIR" && -d "$SCCACHE_DIR" ]]; then
  export RUSTC_WRAPPER=$(which sccache)
fi

# Build library for a single platform/variant with the specified feature set
build_platform_variant() {
  local platform_nick=$1
  local platform_rust=$2
  local features=$3
  local variant=$4
  local name=libbwt-jni-$version$variant-$platform_nick
  local filename=$(lib_filename $platform_rust)

  cargo build --release --target $platform_rust --no-default-features --features $features

  mkdir -p dist/$name
  mv target/$platform_rust/release/$filename dist/$name/
  strip_symbols $platform_rust dist/$name/$filename || true
  pack $name
}

# Build variants (full/electrum_only) for the specified platform
build_platform() {
  [ -n "$ELECTRUM_ONLY_ONLY" ] || build_platform_variant $1 $2 $3,http,electrum  ''
  build_platform_variant $1 $2 $3,electrum '-electrum_only'
}

lib_filename() {
  # Windows dll files ar built as `bwt_jni.dll`, without the `lib` prefix
  local pre=$([[ $1 == *"-windows-"* ]] || echo lib)
  local ext=$([[ $1 == *"-windows-"* ]] && echo .dll || ([[ $1 == *"-apple-"* ]] && echo .dylib || echo .so))
  echo -n ${pre}bwt_jni${ext}
}

strip_symbols() {
  case $1 in
    "x86_64-unknown-linux-gnu") x86_64-linux-gnu-strip $2 ;;
    "x86_64-pc-windows-gnu") x86_64-w64-mingw32-strip $2 ;;
    "x86_64-apple-darwin") x86_64-apple-darwin15-strip $2 ;;
    "armv7-unknown-linux-gnueabihf") arm-linux-gnueabihf-strip $2 ;;
    "aarch64-unknown-linux-gnu") aarch64-linux-gnu-strip $2 ;;
    "i686-linux-android") i686-linux-android-strip $2 ;;
    "x86_64-linux-android") x86_64-linux-android-strip $2 ;;
    "armv7-linux-androideabi") arm-linux-androideabi-strip $2 ;;
    "aarch64-linux-android") aarch64-linux-android-strip $2 ;;
    *) echo >&2 Platform not found: $1; strip $2 ;;
  esac
}

# pack tar.gz with static/removed metadata attrs and deterministic file order for reproducibility
pack() {
  local name=$1
  cp {LICENSE,README.md} dist/$name/
  touch -t 1711081658 dist/$name dist/$name/*
  pushd dist
  TZ=UTC tar --mtime='2017-11-08 16:58:00' --owner=0 --sort=name -I 'gzip --no-name' -chf $name.tar.gz $name
  popd
}
