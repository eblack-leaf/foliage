//! Every text file here is a parameterized copy of `rust-mobile/android-activity`'s own
//! `agdk-mainloop` example project (the crate that implements winit's `android-game-activity`
//! backend) -- verified working against `androidx.games:games-activity`, not invented from
//! scratch. Only the values that have to vary per-app (application id, cdylib name, SDK
//! levels) are parameters; everything else is copied as-is.

pub fn root_build_gradle(agp_version: &str) -> String {
    let mut out = String::new();
    out.push_str("// Top-level build file where you can add configuration options common to all sub-projects/modules.\n");
    out.push_str("plugins {\n");
    out.push_str(&format!(
        "    id 'com.android.application' version '{agp_version}' apply false\n"
    ));
    out.push_str(&format!(
        "    id 'com.android.library' version '{agp_version}' apply false\n"
    ));
    out.push_str("}\n");
    out
}

pub fn settings_gradle() -> String {
    let mut out = String::new();
    out.push_str("pluginManagement {\n");
    out.push_str("    repositories {\n");
    out.push_str("        gradlePluginPortal()\n");
    out.push_str("        google()\n");
    out.push_str("        mavenCentral()\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out.push_str("dependencyResolutionManagement {\n");
    out.push_str("    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)\n");
    out.push_str("    repositories {\n");
    out.push_str("        google()\n");
    out.push_str("        mavenCentral()\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out.push_str("include ':app'\n");
    out
}

pub fn gradle_properties() -> String {
    let mut out = String::new();
    out.push_str("# Enable Gradle Daemon\n");
    out.push_str("org.gradle.daemon=true\n");
    out.push_str("# JVM arguments\n");
    out.push_str(
        "org.gradle.jvmargs=-Xmx4g -XX:+HeapDumpOnOutOfMemoryError -Dfile.encoding=UTF-8\n",
    );
    out.push_str("# Enable AndroidX\n");
    out.push_str("android.useAndroidX=true\n");
    out.push_str("# Build caching and parallel execution\n");
    out.push_str("org.gradle.caching=true\n");
    out.push_str("org.gradle.parallel=true\n");
    out.push_str("# File system watching for faster builds\n");
    out.push_str("org.gradle.unsafe.watch-fs=true\n");
    out
}

pub fn gradle_wrapper_properties(gradle_version: &str) -> String {
    let mut out = String::new();
    out.push_str("distributionBase=GRADLE_USER_HOME\n");
    out.push_str("distributionPath=wrapper/dists\n");
    out.push_str(&format!(
        "distributionUrl=https\\://services.gradle.org/distributions/gradle-{gradle_version}-bin.zip\n"
    ));
    out.push_str("networkTimeout=10000\n");
    out.push_str("validateDistributionUrl=true\n");
    out.push_str("zipStoreBase=GRADLE_USER_HOME\n");
    out.push_str("zipStorePath=wrapper/dists\n");
    out
}

pub fn app_build_gradle(
    app_id: &str,
    min_sdk: u32,
    compile_sdk: u32,
    target_sdk: u32,
    games_activity_version: &str,
) -> String {
    let mut out = String::new();
    out.push_str("plugins {\n");
    out.push_str("    id 'com.android.application'\n");
    out.push_str("}\n\n");
    out.push_str(
        "// Release signing: optional, and absent by default -- `assembleRelease` still\n",
    );
    out.push_str(
        "// works without it (produces an unsigned artifact you can't install directly).\n",
    );
    out.push_str(
        "// Create keystore.properties next to this file (see android/README.md's \"Release\n",
    );
    out.push_str(
        "// signing\" section) and it's picked up automatically -- nothing else to edit here.\n",
    );
    out.push_str("def keystorePropertiesFile = rootProject.file(\"keystore.properties\")\n");
    out.push_str("def keystoreProperties = new Properties()\n");
    out.push_str("if (keystorePropertiesFile.exists()) {\n");
    out.push_str("    keystoreProperties.load(new FileInputStream(keystorePropertiesFile))\n");
    out.push_str("}\n\n");
    out.push_str("android {\n");
    out.push_str(&format!("    compileSdk = {compile_sdk}\n\n"));
    out.push_str("    defaultConfig {\n");
    out.push_str(&format!("        applicationId = \"{app_id}\"\n"));
    out.push_str(&format!("        minSdk = {min_sdk}\n"));
    out.push_str(&format!("        targetSdk = {target_sdk}\n"));
    out.push_str("        versionCode = 1\n");
    out.push_str("        versionName = \"1.0\"\n");
    out.push_str("    }\n\n");
    out.push_str("    signingConfigs {\n");
    out.push_str("        release {\n");
    out.push_str("            if (keystorePropertiesFile.exists()) {\n");
    out.push_str("                storeFile rootProject.file(keystoreProperties['storeFile'])\n");
    out.push_str("                storePassword keystoreProperties['storePassword']\n");
    out.push_str("                keyAlias keystoreProperties['keyAlias']\n");
    out.push_str("                keyPassword keystoreProperties['keyPassword']\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");
    out.push_str("    buildTypes {\n");
    out.push_str("        release {\n");
    out.push_str("            minifyEnabled = false\n");
    out.push_str("            if (keystorePropertiesFile.exists()) {\n");
    out.push_str("                signingConfig signingConfigs.release\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("        debug {\n");
    out.push_str("            minifyEnabled = false\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("    compileOptions {\n");
    out.push_str("        sourceCompatibility JavaVersion.VERSION_17\n");
    out.push_str("        targetCompatibility JavaVersion.VERSION_17\n");
    out.push_str("    }\n");
    out.push_str(&format!("    namespace = '{app_id}'\n"));
    out.push_str("}\n\n");
    out.push_str("dependencies {\n");
    out.push_str("    implementation 'androidx.appcompat:appcompat:1.7.0'\n\n");
    out.push_str("    // To use the Games Activity library\n");
    out.push_str(&format!(
        "    implementation \"androidx.games:games-activity:{games_activity_version}\"\n"
    ));
    out.push_str(
        "    // Note: don't include game-text-input separately, since it's integrated into game-activity\n",
    );
    out.push_str("}\n");
    out
}

pub fn android_manifest(app_name: &str, lib_name: &str) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<manifest xmlns:android=\"http://schemas.android.com/apk/res/android\">\n\n");
    out.push_str("    <application\n");
    out.push_str("        android:icon=\"@android:drawable/sym_def_app_icon\"\n");
    out.push_str(&format!("        android:label=\"{app_name}\"\n"));
    out.push_str("        android:supportsRtl=\"true\"\n");
    out.push_str("        android:theme=\"@style/ActivityTheme\">\n");
    out.push_str("        <activity\n");
    out.push_str("            android:name=\".MainActivity\"\n");
    out.push_str(
        "            android:configChanges=\"orientation|screenSize|screenLayout|keyboardHidden\"\n",
    );
    out.push_str("            android:exported=\"true\">\n");
    out.push_str("            <intent-filter>\n");
    out.push_str("                <action android:name=\"android.intent.action.MAIN\" />\n");
    out.push_str(
        "                <category android:name=\"android.intent.category.LAUNCHER\" />\n",
    );
    out.push_str("            </intent-filter>\n\n");
    out.push_str(&format!(
        "            <meta-data android:name=\"android.app.lib_name\" android:value=\"{lib_name}\" />\n"
    ));
    out.push_str("        </activity>\n");
    out.push_str("    </application>\n\n");
    out.push_str("</manifest>\n");
    out
}

pub fn themes_xml() -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<resources>\n");
    out.push_str(
        "    <style name=\"ActivityTheme\" parent=\"Theme.AppCompat.Light.NoActionBar\">\n",
    );
    out.push_str("        <!-- For full-screen layout, cutout support -->\n");
    out.push_str(
        "        <item name=\"android:windowLayoutInDisplayCutoutMode\">shortEdges</item>\n",
    );
    out.push_str("        <item name=\"android:windowFullscreen\">true</item>\n");
    out.push_str("    </style>\n");
    out.push_str("</resources>\n");
    out
}

pub fn main_activity_java(app_id: &str, lib_name: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("package {app_id};\n\n"));
    out.push_str("import androidx.core.view.WindowCompat;\n");
    out.push_str("import androidx.core.view.WindowInsetsCompat;\n");
    out.push_str("import androidx.core.view.WindowInsetsControllerCompat;\n\n");
    out.push_str("import com.google.androidgamesdk.GameActivity;\n\n");
    out.push_str("import android.os.Bundle;\n");
    out.push_str("import android.view.View;\n");
    out.push_str("import android.view.WindowManager;\n\n");
    out.push_str("public class MainActivity extends GameActivity {\n\n");
    out.push_str("    static {\n");
    out.push_str(
        "        // Must match both the foliage app's `[lib] name` (its cdylib output) and\n",
    );
    out.push_str(
        "        // the `android.app.lib_name` meta-data value in AndroidManifest.xml -- all\n",
    );
    out.push_str(
        "        // three have to agree, or the JVM can't find `android_main` to call into.\n",
    );
    out.push_str(&format!("        System.loadLibrary(\"{lib_name}\");\n"));
    out.push_str("    }\n\n");
    out.push_str("    private void hideSystemUI() {\n");
    out.push_str("        getWindow().getAttributes().layoutInDisplayCutoutMode\n");
    out.push_str(
        "                = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_ALWAYS;\n",
    );
    out.push_str("        View decorView = getWindow().getDecorView();\n");
    out.push_str(
        "        WindowInsetsControllerCompat controller = new WindowInsetsControllerCompat(getWindow(),\n",
    );
    out.push_str("                decorView);\n");
    out.push_str("        controller.hide(WindowInsetsCompat.Type.systemBars());\n");
    out.push_str("        controller.hide(WindowInsetsCompat.Type.displayCutout());\n");
    out.push_str("        controller.setSystemBarsBehavior(\n");
    out.push_str(
        "                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);\n",
    );
    out.push_str("    }\n\n");
    out.push_str("    @Override\n");
    out.push_str("    protected void onCreate(Bundle savedInstanceState) {\n");
    out.push_str("        WindowCompat.setDecorFitsSystemWindows(getWindow(), false);\n");
    out.push_str("        hideSystemUI();\n");
    out.push_str("        super.onCreate(savedInstanceState);\n");
    out.push_str("    }\n\n");
    out.push_str("    protected void onResume() {\n");
    out.push_str("        super.onResume();\n");
    out.push_str("        hideSystemUI();\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out
}

/// Setup and build instructions, written next to the project they describe. A generated Gradle
/// directory is not self-explanatory, and none of the SDK-side setup it needs is discoverable
/// from the files themselves.
///
/// Unlike the other templates this has no upstream in `agdk-mainloop` -- the `android` CLI and
/// NDK specifics are what this repo verified against its own build.
pub fn readme(
    app_name: &str,
    lib_name: &str,
    out_dir: &str,
    min_sdk: u32,
    compile_sdk: u32,
    target_sdk: u32,
    ndk_version: &str,
    build_tools_version: &str,
    host: &str,
) -> String {
    // An emulator runs the host's architecture, so the system image, the `cargo ndk -t` value
    // and the rust target for emulator builds all follow from `--host` rather than being
    // separate choices.
    let host_is_arm = host.ends_with("arm64");
    let emulator_abi = if host_is_arm { "arm64-v8a" } else { "x86_64" };
    let emulator_rust_target = if host_is_arm {
        "aarch64-linux-android"
    } else {
        "x86_64-linux-android"
    };
    format!(
        r#"# {app_name} -- Android

Generated by `foliage_android gen`, **including this README** -- a rerun overwrites everything
in this directory, so keep notes of your own somewhere else.

The Gradle project exists because `androidx.games:games-activity` -- Google's `GameActivity`,
which winit's `android-game-activity` backend targets -- is a Java/Kotlin AAR, and there's no
way to link one without Gradle. Unavoidable rather than a workaround. Everything here is a
parameterized copy of `rust-mobile/android-activity`'s own `agdk-mainloop` example.

## One-time environment setup

Normally this is Android Studio's setup wizard, hand-picking SDK components and clicking
through license dialogs. It's all doable from a terminal instead, no Studio required.

Two separate things get installed: the `android` CLI (a ~5MB binary) and the SDK packages it
downloads (multiple GB, mostly NDK). Put both **outside this directory**, since `gen`
overwrites this one -- your repo root works. These instructions use `.android-sdk/` there;
gitignore it. Removing it is `rm -rf .android-sdk`, though any AVDs you create live in
`~/.android/` and outlive it.

1. **A JDK** -- Gradle needs one, it's rarely preinstalled, and nothing below installs it for
   you. `sudo apt install default-jdk` on Debian/Ubuntu, your distro's equivalent elsewhere,
   or https://adoptium.net. Take whatever version your package manager currently offers --
   it'll likely be well ahead of the minimum, which is fine. AGP enforces a floor, not an
   exact version, and a too-old JDK fails with an error naming what it wants.

2. **The `android` CLI.** It's a single self-updating binary, so you can just drop it in
   place. From your repo root:

   ```sh
   export ANDROID_HOME="$PWD/.android-sdk"
   mkdir -p "$ANDROID_HOME/bin"
   curl -fsSL -o "$ANDROID_HOME/bin/android" \
     https://dl.google.com/android/cli/latest/{host}/android
   chmod +x "$ANDROID_HOME/bin/android"
   export PATH="$ANDROID_HOME/bin:$ANDROID_HOME/platform-tools:$PATH"
   ```

   That URL targets `{host}`; swap it for `linux_x86_64`, `darwin_arm64`, `darwin_x86_64`,
   as needed. Google also offers `apt-get install android-cli` (adds a Google apt repo, needs
   sudo), `brew install android-cli`, and an `install.sh` -- all fine, but they install
   system-wide or into `~/.local/bin`, and `install.sh` appends a `PATH` line to your shell
   rc file. The download above keeps everything in one deletable directory instead. See
   https://developer.android.com/tools/agents/android-cli/download for the alternatives.

   Export `ANDROID_HOME` **before** running `android` the first time. Unset, it falls back to
   `~/Android/Sdk` and puts gigabytes there instead.

   Older guides tell you to unzip "command line tools only" and rename a folder to `latest`.
   That still ships the same binary, but it's the legacy path -- skip it.

3. **Install the SDK packages this project needs:**

   ```sh
   android update   # the binary is a small launcher; get it current first
   android sdk install platform-tools platforms/android-{compile_sdk} \
     build-tools/{build_tools_version} ndk/{ndk_version}
   export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/{ndk_version}"
   ```

   Only `platforms/android-{compile_sdk}` is fixed -- it has to match `compileSdk`. The other
   versions are examples of the shape, not requirements: run `android sdk list --all` and take
   whatever's current. Build-tools just needs to be recent (its major tracks the platform, but
   the trailing numbers move); cargo-ndk works against any reasonably recent NDK and doesn't
   care which. Whichever NDK you install, `ANDROID_NDK_HOME` has to name that same directory.

   `android` deprecates the old `sdkmanager`, and package coordinates changed with it -- `/`
   instead of `;`, so a tutorial's `"platforms;android-{compile_sdk}"` is
   `platforms/android-{compile_sdk}` here.

4. **Rust side:**

   ```sh
   rustup target add aarch64-linux-android   # add other ABIs only if you need them
   cargo install cargo-ndk
   ```

   These go to `~/.rustup` and `~/.cargo` like any other toolchain component; there's no
   per-project equivalent, and they're shared with every Rust project you build.

5. **Persisting the environment.** Those exports are per-shell, and a shell rc file is the
   wrong place for them when the SDK lives beside one specific repo. Keep them in a gitignored
   `android-env.sh` at your repo root instead, using an absolute path:

   ```sh
   export ANDROID_HOME="/absolute/path/to/your/repo/.android-sdk"
   export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/{ndk_version}"
   export PATH="$ANDROID_HOME/bin:$ANDROID_HOME/platform-tools:$PATH"
   ```

   Then `source ./android-env.sh` in any shell you build from.

## SDK levels

Three numbers do three different jobs, and only one gates hardware:

- **`compileSdk` ({compile_sdk})** -- which API the compiler sees. A build-time download
  (`platforms/android-{compile_sdk}`), no bearing on what the APK runs on.
- **`minSdk` ({min_sdk})** -- the device gate. Anything below refuses to install the APK.
- **`targetSdk` ({target_sdk})** -- which platform behaviors the app opts into. Affects
  runtime behavior on newer devices; not an install gate.

Lowering `compileSdk` doesn't widen device support -- `minSdk` is the knob for that. All three
are `gen` flags (`--min-sdk`, `--compile-sdk`, `--target-sdk`), so change them there and
regenerate rather than editing `app/build.gradle`, which a rerun overwrites. Whatever
`--compile-sdk` you pick, install the matching `platforms/android-<n>` or gradle won't find
it; AGP also enforces its own floor, and going below it fails with an error naming the
minimum it accepts.

## Build

```sh
cargo ndk -t arm64-v8a -o {out_dir}/app/src/main/jniLibs/ \
  build --release -p {lib_name}
cd {out_dir} && ./gradlew assembleDebug
```

The APK lands at `app/build/outputs/apk/debug/app-debug.apk`.

**Getting it onto your phone:** transfer the file however you like (USB copy, email,
syncthing) -- no `adb`/USB-debugging link required. Your phone will refuse to install it until
you allow it: whatever app opens the APK (Files, a browser download), Android prompts "Install
unknown apps" the first time -- allow it for that one app, then tap the APK again. (With
`adb`: `./gradlew installDebug`, or `adb install app/build/outputs/apk/debug/app-debug.apk`,
does transfer+install in one step. Entirely optional.)

**Run `cargo ndk` from the repo root, not from here.** `-o` is relative to your working
directory, and the block above ends by `cd`-ing into `{out_dir}` for Gradle -- so re-running
the cargo line from that shell writes to `{out_dir}/{out_dir}/app/src/main/jniLibs/`, a
mirrored path nothing reads. Cargo finds the workspace either way and reports success, so
there is no error to notice.

That failure is silent twice over, because `jniLibs/` is an *input* directory to Gradle, not
an output it tracks. Whatever `.so` is sitting there gets packaged; if the Rust build went
somewhere else, or was skipped, or failed, `assembleDebug` still succeeds and ships the
*previous* library. The luckier outcome is a hard `RegisterNatives`/`UnsatisfiedLinkError`
abort at launch; the unluckier one is old code running quietly.

So after changing Rust code or dependencies, run the steps separately and confirm the
timestamp actually moved before packaging:

```sh
ls -la --time-style=full-iso {out_dir}/app/src/main/jniLibs/*/*.so
```

`cargo clean -p {lib_name}` forces a rebuild if cargo believes there's nothing to do. Each
ABI is built separately, so rebuilding x86_64 leaves `arm64-v8a` stale -- which looks fine on
an emulator and crashes on a phone.

**Picking `-t <abi>`:** `arm64-v8a` covers essentially every phone made since ~2017. Settings
-> About phone usually shows the chipset, or `adb shell getprop ro.product.cpu.abi` reports it
directly once you have any USB connection.

## Emulator

Useful for exercising API levels you don't own hardware for. `android` manages AVDs directly:

```sh
android sdk install emulator system-images/android-{compile_sdk}/google_apis/{emulator_abi}
android emulator create --list-profiles      # what you can pick from
android emulator create medium_phone         # profile is positional, and required
android emulator list                        # names you can start
android emulator start medium_phone
android emulator stop emulator-5554          # takes the serial, not the name
```

`android emulator` is currently disabled on Windows. On Linux the emulator needs hardware
acceleration through `/dev/kvm`, and lacking access is the usual reason a freshly created AVD
won't start at all -- `test -r /dev/kvm && test -w /dev/kvm && echo ok` settles it. Group
membership alone isn't the whole story: some distros grant access via an ACL on the device
node instead, so test rather than reading `groups`. If you do lack it, `sudo usermod -aG kvm
$USER` plus a re-login.

**Force the GPU, or everything is software-rendered.** New AVDs default to
`hw.gpu.mode=auto`, and `auto` readily decides against a perfectly capable host GPU and falls
back to SwiftShader -- CPU rendering, sluggish enough that the launcher and Settings feel
broken, not just your app. Fix it once per AVD in
`~/.android/avd/<name>.avd/config.ini`:

```ini
hw.gpu.mode=host
```

Then `android emulator start <name>` is enough. To override without editing config, go
around the CLI to the binary it wraps: `$ANDROID_HOME/emulator/emulator -avd <name> -gpu
host`. To check which you actually got, `adb shell dumpsys SurfaceFlinger | grep -i gles`
names either your real GPU or `SwiftShader`.

**Build the emulator's native ABI.** Current x86_64 system images ship ARM binary
translation, so an `arm64-v8a`-only APK does install and run on an x86_64 emulator -- it just
routes every instruction through `libndk_translation.so`, which is slow enough to make frame
timings meaningless and to trip ANR dialogs. Build the host's ABI too:

```sh
rustup target add {emulator_rust_target}
cargo ndk -t {emulator_abi} -o {out_dir}/app/src/main/jniLibs/ \
  build --release -p {lib_name}
```

Both ABIs coexist in `jniLibs/` -- they land in per-ABI subdirectories, and Android prefers a
native ABI over a translated one, so one APK serves emulator and phone. (On an arm64 host,
invert this: the phone build is already native.) A crash dump tells you which you ran --
`ABI: 'x86_64'` beside `Guest architecture: 'arm64'`, with `berberis::` frames, means
translated.

Match the system image's API level to what you want to test -- the one place installing a
platform other than `compileSdk`'s genuinely makes sense.

AVDs live in `~/.android/avd/`, **not** in `$ANDROID_HOME` -- deleting the SDK directory
leaves them behind, and they are where `hw.ramSize` and `hw.cpu.ncore` live too.

## Release signing

`assembleDebug` needs nothing extra -- AGP auto-generates and uses a debug keystore.
`assembleRelease` also runs without one, but produces an **unsigned** APK no device will
install. For a real signed release:

1. Generate a keystore once (the JDK's `keytool` has everything needed). Keep the file and
   passwords somewhere safe outside the repo -- losing this keystore means you can never ship
   an update to the same `applicationId` under the same identity again.

   ```sh
   keytool -genkeypair -v -keystore ~/keys/{lib_name}-release.keystore \
     -alias {lib_name} -keyalg RSA -keysize 2048 -validity 10000
   ```

2. Create `{out_dir}/keystore.properties` -- gitignore it, never commit it:

   ```properties
   storeFile=/home/you/keys/{lib_name}-release.keystore
   storePassword=<the password you set above>
   keyAlias={lib_name}
   keyPassword=<same password, unless you set a separate key password>
   ```

3. That's it -- `app/build.gradle`'s `signingConfigs.release` reads that file automatically
   whenever it's present:

   ```sh
   cd {out_dir} && ./gradlew assembleRelease
   # -> app/build/outputs/apk/release/app-release.apk, already signed
   ```

   No `keystore.properties`? `assembleRelease` still succeeds, just unsigned at the same path
   -- fine for a build check, not for something you install.

## Troubleshooting

- **Gradle disagrees with you about where the SDK is**: check `local.properties`, which gradle
  writes with the SDK path it actually resolved, against `$ANDROID_HOME`.
- **Packages install to `~/Android/Sdk` instead of your SDK directory**: `ANDROID_HOME` wasn't
  exported before `android` ran. Remove the stray tree, export it, and pass it explicitly if
  it still misbehaves: `android --sdk="$ANDROID_HOME" sdk install ...`.
- **`android: command not found`**: `PATH` isn't exported in this shell -- `source
  ./android-env.sh`.
- **`cargo ndk` can't find the NDK**: `ANDROID_NDK_HOME` isn't exported, or points at the wrong
  directory -- it should be `$ANDROID_HOME/ndk/<version>`, not `$ANDROID_HOME/ndk` itself.
- **App installs but crashes immediately**: an emulator gives you `adb` with no USB; on a
  phone a temporary cable is enough. Clear, launch, and read the abort message:

  ```sh
  adb logcat -c
  adb shell am start -n <app-id>/.MainActivity
  adb logcat -d | grep -B5 -A40 "Fatal signal"
  ```

  The tombstone's `Abort message:` line names the cause directly. Common ones:
  `System.loadLibrary` failing means `--lib-name` doesn't match the cdylib's output name, or
  `jniLibs/` has nothing for the device's ABI. `RegisterNatives failed for
  'com/google/androidgamesdk/GameActivity'` means the AAR and the native GameActivity
  disagree -- see below.
- **`RegisterNatives failed` / `NoSuchMethodError` on a GameActivity method**: the
  `--games-activity-version` AAR and the GameActivity C++ compiled into your library are
  different versions. The `NoSuchMethodError` names the method and the signature the *native*
  side expects, which tells you which half is stale. Match the AAR to whatever
  `android-activity` your `Cargo.lock` resolved -- grep `GAMEACTIVITY_MAJOR_VERSION` from the
  `GameActivity.h` it vendors. Prefer updating the lock over downgrading the AAR; an old
  `android-activity` pins you to a GameActivity years behind current Android. Note this
  aborts before any Rust runs, so it is not a bug in your app.

## Regenerating

Everything here is disposable -- delete and rerun `gen` any time (e.g. after bumping SDK
levels, `--games-activity-version`, or `--gradle-version`). `app/src/main/jniLibs/` and
`app/build/` are build output, not part of what `gen` writes.
"#
    )
}
