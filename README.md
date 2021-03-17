# Bitcoin Wallet Tracker - JNI bindings

[![Build Status](https://travis-ci.org/bwt-dev/libbwt-jni.svg?branch=master)](https://travis-ci.org/bwt-dev/libbwt-jni)
[![Latest release](https://img.shields.io/github/v/release/bwt-dev/libbwt-jni?color=orange)](https://github.com/bwt-dev/libbwt-jni/releases/tag/v0.2.3)
[![Downloads](https://img.shields.io/github/downloads/bwt-dev/libbwt-jni/total.svg?color=blueviolet)](https://github.com/bwt-dev/libbwt-jni/releases)
[![MIT license](https://img.shields.io/github/license/bwt-dev/libbwt-jni.svg?color=yellow)](https://github.com/bwt-dev/libbwt-jni/blob/master/LICENSE)
[![Pull Requests Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/bwt-dev/bwt#developing)

Java Native Interface bindings for [Bitcoin Wallet Tracker](https://github.com/bwt-dev/bwt), a lightweight personal indexer for bitcoin wallets.

`libbwt-jni` allows to programmatically manage bwt's Electrum RPC and HTTP API servers.
It can be used as a compatibility layer for easily upgrading Electrum-backed wallets to support a
Bitcoin Core full node backend (by running the Electrum server *in* the wallet),
or for shipping software that integrates bwt's [HTTP API](https://github.com/bwt-dev/bwt#http-api)
as an all-in-one package.

Support development: [⛓️ on-chain or ⚡ lightning via BTCPay](https://btcpay.shesek.info/)

- [Usage](#usage)
- [Installation](#installation)
  - [AAR](#aar-for-androidkotlin) (for Android/Kotlin)
  - [Manual](#manual-installation-non-android-or-without-kotlin) (non-Android or without Kotlin)
  - [Electrum only](#electrum-only-variant)
  - [Verifying the signature](#verifying-the-signature)
- [Building from source](#building-from-source)
- [Reproducible builds](#reproducible-builds)
- [License](#license)

> Also see: [bwt](https://github.com/bwt-dev/bwt), [libbwt](https://github.com/bwt-dev/libbwt) and [libbwt-nodejs](https://github.com/bwt-dev/libbwt-nodejs).

## Usage

With the higher-level `BwtDaemon` implemented in Kotlin:

```kotlin
import dev.bwt.libbwt.daemon.*
import dev.bwt.libbwt.BwtException

val bwt = BwtDaemon(BwtConfig(
    bitcoindUrl = "http://127.0.0.1:8332",
    bitcoindAuth = "satoshi:otomakan",
    descriptors = arrayOf("wpkh(xpub66../0/*)", "wpkh(xpub66../1/*)"),
    xpubs = arrayOf("xpub11...", "ypub22...", "zpub33..."),
    rescanSince = 1510160280, // unix timestamp
    electrumAddr = "127.0.0.1:0", // bind on any available port
))

try {
    // start() will block the current thread and won't return until the daemon is stopped.
    bwt.start(object : ProgressNotifier {
        override fun onBooting() {
            // Called after the configuration is validated, right before starting up
            println("Daemon booting up...")
        }
        override fun onSyncProgress(progress: Float, tip: Date) {
            println("Initial block download in progress... ($progress done, synced up to $tip)")
        }
        override fun onScanProgress(progress: Float, eta: Int) {
            println("Wallet rescanning in progress... ($progress done, ETA $eta seconds)")
        }
        override fun onReady() {
            println("Daemon ready, Electrum server bound on ${bwt.electrumAddr}")
            bwt.shutdown()
        }
    })
} catch (e: BwtException) {
    println("Daemon startup failed: $e")
}
```

Alternatively, you may use the lower-level [`NativeBwtDaemon`](library/src/main/java/dev/bwt/libbwt/daemon/NativeBwtDaemon.java)
directly, which does not require Kotlin. Refer to the [`BwtDaemon`](library/src/main/java/dev/bwt/libbwt/daemon/BwtDaemon.kt)
implementation for example usage.

For the full list of available configuration options, refer to the
[`libbwt` C FFI docs](https://github.com/bwt-dev/libbwt#config-options).

The API servers are unauthenticated by default, but
[authentication can be enabled](https://github.com/bwt-dev/bwt/blob/master/doc/auth.md).

Note that if you call `shutdown()` while bitcoind is importing/rescanning addresses, the daemon will
not stop immediately but will be marked for later termination.

## Installation

Pre-built [signed](#verifying-the-signature) & [deterministic](#reproducible-builds) library
files are available for Linux, Mac, Windows, ARMv7/8 and Android.

> ⚠️ The pre-built libraries are meant to make it easier to get started. If you're integrating bwt
> into real-world software, [building from source](#building-from-source) is *highly* recommended.

#### AAR (for Android/Kotlin)

The AAR library is available for download from the [releases page](https://github.com/bwt-dev/libbwt-jni/releases).

Installation instructions for AAR [are available here](https://developer.android.com/studio/projects/android-library#AddDependency).

#### Manual installation (non-Android or without Kotlin)

If you're not developing for Android, or if you are but would like to avoid using the AAR or Kotlin,
you can use the JNI libraries directly.

1. Download the JNI library for you platform(s) from the [releases page](https://github.com/bwt-dev/libbwt-jni/releases)
  and copy it into your `jniLibs` directory.

2. Copy the [`dev.bwt.libbwt` source code](https://github.com/bwt-dev/libbwt-jni/tree/master/library/src/main/java/dev/bwt/libbwt)
   into your project. You may omit `BwtDaemon.kt` and use `NativeBwtDaemon` directly instead.

3. If you choose to use `BwtDaemon`, you'll need to have [`com.google.gson`](https://github.com/google/gson) installed.

#### Electrum-only variant

The pre-built libraries are also available for download as an `electrum_only` variant,
which doesn't include the HTTP API server. It is roughly 33% smaller and comes with less dependencies.

#### Verifying the signature

The releases are signed by Nadav Ivgi (@shesek).
The public key can be verified on
the [PGP WoT](http://keys.gnupg.net/pks/lookup?op=vindex&fingerprint=on&search=0x81F6104CD0F150FC),
[github](https://api.github.com/users/shesek/gpg_keys),
[twitter](https://twitter.com/shesek),
[keybase](https://keybase.io/nadav),
[hacker news](https://news.ycombinator.com/user?id=nadaviv)
and [this video presentation](https://youtu.be/SXJaN2T3M10?t=4) (bottom of slide).

```bash
# Download (change x86_64-linux to your platform)
$ wget https://github.com/bwt-dev/libbwt-jni/releases/download/v0.2.3/libbwt-jni-0.2.3-x86_64-linux.tar.gz

# Fetch public key
$ gpg --keyserver keyserver.ubuntu.com --recv-keys FCF19B67866562F08A43AAD681F6104CD0F150FC

# Verify signature
$ wget -qO - https://github.com/bwt-dev/libbwt-jni/releases/download/v0.2.3/SHA256SUMS.asc \
  | gpg --decrypt - | grep x86_64-linux.tar.gz | sha256sum -c -
```

The signature verification should show `Good signature from "Nadav Ivgi <nadav@shesek.info>" ... Primary key fingerprint: FCF1 9B67 ...` and `libbwt-jni-0.2.3-x86_64-linux.tar.gz: OK`.

## Building from source

Manually build the JNI library for a single platform (requires Rust):

```bash
$ git clone https://github.com/bwt-dev/libbwt-jni && cd libbwt-jni
$ git checkout <tag>
$ git verify-commit HEAD
$ git submodule update --init

$ cargo build --release --target <platform>
```

The library file will be available in `target/<platform>/release`, named
`libbwt_jni.so` for Linux/Android/ARM, `libbwt_jni.dylib` for OSX, or `bwt_jni.dll` for Windows.

To build the `electrum_only` variant, set `--no-default-features --features electrum`.

Building for Android targets requires NDK to be installed in your `PATH`.

To build the AAR, first build the JNI libraries for all android platforms, place them under
`library/src/main/jniLibs`, then run `./gradlew build` (or use the Docker builder below).

## Reproducible builds

The JNI builds for all supported platforms and the AAR library for Android can be reproduced
in a Docker container environment as follows:

```bash
$ git clone https://github.com/bwt-dev/libbwt-jni && cd libbwt-jni
$ git checkout <tag>
$ git verify-commit HEAD
$ git submodule update --init

# JNI libraries for Linux, Windows, ARMv7 and ARMv8
$ docker build -t bwt-builder - < bwt/scripts/builder.Dockerfile
$ docker run -it --rm -u `id -u` -v `pwd`:/usr/src/libbwt-jni -w /usr/src/libbwt-jni \
  --entrypoint scripts/build.sh bwt-builder

# JNI libraries for Mac OSX (cross-compiled via osxcross)
$ docker build -t bwt-builder-osx - < bwt/scripts/builder-osx.Dockerfile
$ docker run -it --rm -u `id -u` -v `pwd`:/usr/src/libbwt-jni -w /usr/src/libbwt-jni \
  --entrypoint scripts/build.sh bwt-builder-osx

# JNI libraries + AAR for Android platforms (x86, x86_64, arm32v7, arm64v8)
$ docker build -t bwt-builder-android - < bwt/scripts/builder-android.Dockerfile
$ docker run -it --rm -u `id -u` -v `pwd`:/usr/src/libbwt-jni -w /usr/src/libbwt-jni \
  --entrypoint scripts/build-android.sh bwt-builder-android

$ sha256sum dist/*.tar.gz
```

You may set `-e TARGETS=...` to a comma separated list of the platforms to build. 
The available platforms are: `x86_64-linux`, `x86_64-osx`, `x86_64-windows`, `arm32v7-linux`, `arm64v8-linux`,
`arm32v7-android`, `arm64v8-android`, `i686-android` and `x86_64-android`.

Both variants will be built by default. To build the `electrum_only` variant only, set `-e ELECTRUM_ONLY_ONLY=1`.

The builds are [reproduced on Travis CI](https://travis-ci.org/github/bwt-dev/libbwt-jni/branches) using the code from GitHub.
The SHA256 checksums are available under the "Reproducible builds" stage.

## License

MIT
