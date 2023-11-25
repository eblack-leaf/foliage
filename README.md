# foliage

This library attempts to build a cross-platform ui with wgpu.rs and winit.

### Android

#### Prerequisites

Must be able to compile Java on your system.

Android SDK must be installed and some tools downloaded
using the `sdkmanager`.
Get the command line tools from
[android](https://developer.android.com/studio).
Unzip the tools to your sdk root (can be wherever you want).
It refers to the `android_sdk` in the
examples below. The `sdkmanager` needs to be set up before
installing packages, here are the
[instructions](https://developer.android.com/tools/sdkmanager).
Follow steps 1 - 4.

You will need to install these tools to your SDK.
`sdkmanager "platform-tools" "platforms;android-<api-version>"
"build-tools;<version>" "ndk;<ndk-version>"`.

To run on android, `run --package build_android -- entry <arch>
<ndk-path> <sdk-path>`
where `<arch>` can be `arm64-v8a` / `aarch64-linux-android` for ARM support or
`x86_64-linux-android` for
x86 support. Example `<ndk-path>` could be
`/home/<user>/android_sdk/ndk/<version-num>` and the `<sdk-path>`
would point to the sdk root directory `/home/<user>/android_sdk/`.

### Desktop

To run on desktop, `cargo run --package entry`

### Wasm

To run on web, `trunk serve` in the `entry` directory