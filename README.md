# foliage

This library attempts to build a cross-platform ui with wgpu.rs and winit.

### Android

#### Prerequisites

Add desired android targets

`rustup target add aarch64-linux-android x86_64-linux-android`

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

    sdkmanager "platform-tools" "platforms;android-<api-version>" 
    "build-tools;<version>" "ndk;<ndk-version>"

To run on android,

    run --package build_android -- <path-to-android.toml>

which points to a `.toml` file that describes the environment for compiling to android.

    package = "<package-name>"
    arch = "<arch>"
    ndk_home = "/path/to/android_sdk/ndk/<ndk-version>"
    sdk_home = "/path/to/android_sdk/"
    min_sdk = <min-api-level>
    target_sdk = <target-api-level>
    compile_sdk = <compile-api-level>
    android_application_version = "<version>"
    gradle_distribution_url = <distribution-url-id> e.g. "8.0-all"
    ndk_version = "<ndk-version>"
    androidx_version = "<version>"
    androidx_constraintlayout_version = "<version>"
    androidx_games_activity_version = "<version>"
    androidx_fragment_version = "<version>"
    oboe_version = "<version>"

`<arch>` can be `arm64-v8a` / `aarch64-linux-android`
for ARM support or
`x86_64-linux-android` for
x86 support.

You will need to configure your entry point with `foliage::AndroidApp`
as a parameter and `#[no_mangle]`
attribute.

```rust 
// android app hook
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: foliage::AndroidApp) {
    // call your regular main here
}
```

This will need to be compiled into a `cdylib` to generate a `.so` file for use
with the `jni` interface.
`crate-type = ["cdylib"]`

### Desktop

To run on desktop, `cargo run --package entry`

### Wasm

To run on web, `trunk serve` in the `entry` directory