//! The files `init` writes.
//!
//! Every Gradle and Java file here is a parameterised copy of `rust-mobile/android-activity`'s own
//! `agdk-mainloop` example -- the crate that implements winit's `android-game-activity` backend --
//! verified against `androidx.games:games-activity` rather than invented. Only the values that have
//! to vary per app are placeholders; everything else is copied as it stands.
//!
//! Placeholders are `{{NAME}}` and are substituted by [`fill`], rather than the text being built up
//! by `format!`. A Gradle build file is mostly braces, and the escaping that would take is exactly
//! the noise that makes a template hard to check against its upstream.

/// Substitutes `{{NAME}}` placeholders.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in values {
        out = out.replace(&format!("{{{{{name}}}}}"), value);
    }
    out
}

pub fn root_build_gradle(agp: &str) -> String {
    fill(ROOT_BUILD_GRADLE, &[("AGP", agp)])
}

pub fn settings_gradle() -> String {
    SETTINGS_GRADLE.to_string()
}

pub fn gradle_properties() -> String {
    GRADLE_PROPERTIES.to_string()
}

pub fn gradle_wrapper_properties(gradle: &str) -> String {
    fill(GRADLE_WRAPPER_PROPERTIES, &[("GRADLE", gradle)])
}

pub fn app_build_gradle(
    app_id: &str,
    min_sdk: u32,
    compile_sdk: u32,
    target_sdk: u32,
    games_activity: &str,
) -> String {
    fill(
        APP_BUILD_GRADLE,
        &[
            ("APP_ID", app_id),
            ("MIN_SDK", &min_sdk.to_string()),
            ("COMPILE_SDK", &compile_sdk.to_string()),
            ("TARGET_SDK", &target_sdk.to_string()),
            ("GAMES_ACTIVITY", games_activity),
        ],
    )
}

pub fn android_manifest(app_name: &str, lib_name: &str, permissions: &[String]) -> String {
    let permissions: String = permissions
        .iter()
        .map(|name| format!("    <uses-permission android:name=\"android.permission.{name}\" />\n"))
        .collect();
    fill(
        ANDROID_MANIFEST,
        &[
            ("APP_NAME", app_name),
            ("LIB_NAME", lib_name),
            ("PERMISSIONS", &permissions),
        ],
    )
}

pub fn themes_xml() -> String {
    THEMES_XML.to_string()
}

/// Whether an app asking for `permissions` is told where the device is: it asks for either
/// location permission.
pub fn locates(permissions: &[String]) -> bool {
    permissions
        .iter()
        .any(|name| name == "ACCESS_FINE_LOCATION" || name == "ACCESS_COARSE_LOCATION")
}

/// The activity. One that [`locates`] starts a [`LOCATOR_JAVA`] when it comes to the front and
/// stops it when it goes; one that does not is as it always was.
pub fn main_activity_java(app_id: &str, lib_name: &str, locates: bool) -> String {
    let (create, resume, members) = match locates {
        true => (LOCATE_CREATE, LOCATE_RESUME, LOCATE_MEMBERS),
        false => ("", "", ""),
    };
    fill(
        MAIN_ACTIVITY_JAVA,
        &[
            ("APP_ID", app_id),
            ("LIB_NAME", lib_name),
            ("LOCATE_CREATE", create),
            ("LOCATE_RESUME", resume),
            ("LOCATE_MEMBERS", members),
        ],
    )
}

/// The listener that hands each fix to foliage, for an app that [`locates`].
pub fn locator_java() -> String {
    LOCATOR_JAVA.to_string()
}

const LOCATE_CREATE: &str = "
        locator = new dev.foliage.Locator(this);";

const LOCATE_RESUME: &str = "
        locator.start();";

const LOCATE_MEMBERS: &str = r#"

    /** Where the device is, handed to foliage while the activity is in front. */
    private dev.foliage.Locator locator;

    @Override
    protected void onPause() {
        locator.stop();
        super.onPause();
    }

    // The permission is asked for by the locator the first time it starts; once it is given, the
    // locator starts again with it.
    @Override
    public void onRequestPermissionsResult(int request, String[] permissions, int[] results) {
        super.onRequestPermissionsResult(request, permissions, results);
        if (request == dev.foliage.Locator.ASKED) {
            locator.start();
        }
    }"#;

const LOCATOR_JAVA: &str = r#"package dev.foliage;

import android.Manifest;
import android.app.Activity;
import android.content.Context;
import android.content.pm.PackageManager;
import android.location.LocationListener;
import android.location.LocationManager;
import android.os.Bundle;
import android.os.Looper;

/**
 * Where the device is, handed to foliage as each fix arrives: from every provider the device has
 * on, so a tablet with no GPS is still placed by its network. Which of two fixes is the better is
 * foliage's to say.
 *
 * Written by `foliage-android init` for an app that asks for a location permission.
 */
public final class Locator implements LocationListener {

    /** The request the permission is asked for under. */
    public static final int ASKED = 0x10ca;

    /** How often a provider is asked for a fix, in milliseconds. */
    private static final long EVERY_MS = 1000;

    /**
     * foliage's: one fix, and which provider it came from -- 1 GPS, 2 network, 3 fused, 0 any
     * other. An accuracy or a speed below zero is none, and a bearing that is NaN is none.
     */
    private static native void located(double lat, double lon, float accuracy, float bearing,
            float speed, long time, int source);

    /** The number foliage knows a provider by. */
    private static int source(String provider) {
        if (LocationManager.GPS_PROVIDER.equals(provider)) {
            return 1;
        }
        if (LocationManager.NETWORK_PROVIDER.equals(provider)) {
            return 2;
        }
        if ("fused".equals(provider)) {
            return 3;
        }
        return 0;
    }

    private final Activity activity;
    private final LocationManager manager;
    private boolean asked = false;
    private boolean listening = false;

    public Locator(Activity activity) {
        this.activity = activity;
        this.manager = (LocationManager) activity.getSystemService(Context.LOCATION_SERVICE);
    }

    private boolean allowed(String permission) {
        return activity.checkSelfPermission(permission) == PackageManager.PERMISSION_GRANTED;
    }

    /** Listens, or asks for the permission to -- once a run, so a refusal is not asked again. */
    public void start() {
        if (manager == null || listening) {
            return;
        }
        if (!allowed(Manifest.permission.ACCESS_FINE_LOCATION)
                && !allowed(Manifest.permission.ACCESS_COARSE_LOCATION)) {
            if (!asked) {
                asked = true;
                activity.requestPermissions(new String[] {
                        Manifest.permission.ACCESS_FINE_LOCATION,
                        Manifest.permission.ACCESS_COARSE_LOCATION,
                }, ASKED);
            }
            return;
        }
        for (String provider : manager.getProviders(true)) {
            if (LocationManager.PASSIVE_PROVIDER.equals(provider)) {
                continue;
            }
            try {
                manager.requestLocationUpdates(provider, EVERY_MS, 0f, this,
                        Looper.getMainLooper());
                listening = true;
                android.location.Location last = manager.getLastKnownLocation(provider);
                if (last != null) {
                    onLocationChanged(last);
                }
            } catch (SecurityException | IllegalArgumentException e) {
                // Taken away between the check and the request, or gone: nothing to listen to.
            }
        }
    }

    public void stop() {
        if (manager != null && listening) {
            manager.removeUpdates(this);
            listening = false;
        }
    }

    @Override
    public void onLocationChanged(android.location.Location at) {
        located(at.getLatitude(), at.getLongitude(),
                at.hasAccuracy() ? at.getAccuracy() : -1f,
                at.hasBearing() ? at.getBearing() : Float.NaN,
                at.hasSpeed() ? at.getSpeed() : -1f,
                at.getTime(),
                source(at.getProvider()));
    }

    // Abstract before API 30, so a device older than that calls them and they have to be here.
    @Override
    public void onStatusChanged(String provider, int status, Bundle extras) {}

    @Override
    public void onProviderEnabled(String provider) {}

    @Override
    public void onProviderDisabled(String provider) {}
}
"#;

/// `Cargo.toml` for the cdylib the Gradle project loads.
///
/// Two dependencies and nothing else: the crate that owns the real entry function, and `foliage`
/// for the [`AndroidApp`] handle and `Foliage::android`.
///
/// [`AndroidApp`]: https://docs.rs/android-activity/latest/android_activity/struct.AndroidApp.html
pub fn entry_crate_manifest(
    entry_crate: &str,
    app_crate: &str,
    app_path: &str,
    foliage: &str,
) -> String {
    fill(
        ENTRY_CRATE_MANIFEST,
        &[
            ("ENTRY_CRATE", entry_crate),
            ("APP_CRATE", app_crate),
            ("APP_PATH", app_path),
            ("FOLIAGE", foliage),
        ],
    )
}

/// The JNI entry point.
///
/// A separate crate from the app on purpose: a `cdylib` target also emits a
/// `wasm32-unknown-unknown` artifact sharing the app crate's binary name, which breaks trunk's
/// artifact selection for a real wasm build of the same app.
pub fn entry_crate_lib(app_crate: &str, app_entry: &str) -> String {
    fill(
        ENTRY_CRATE_LIB,
        &[
            ("APP_CRATE", &app_crate.replace('-', "_")),
            ("APP_ENTRY", app_entry),
        ],
    )
}

pub fn readme(app_name: &str, lib_name: &str, project: &str, compile_sdk: u32) -> String {
    fill(
        README,
        &[
            ("APP_NAME", app_name),
            ("LIB_NAME", lib_name),
            ("PROJECT", project),
            ("COMPILE_SDK", &compile_sdk.to_string()),
            ("EMULATOR_ABI", crate::environment::host_abi()),
        ],
    )
}

const ROOT_BUILD_GRADLE: &str = r#"// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    id 'com.android.application' version '{{AGP}}' apply false
    id 'com.android.library' version '{{AGP}}' apply false
}
"#;

const SETTINGS_GRADLE: &str = r#"pluginManagement {
    repositories {
        gradlePluginPortal()
        google()
        mavenCentral()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}
include ':app'
"#;

const GRADLE_PROPERTIES: &str = r#"# Enable Gradle Daemon
org.gradle.daemon=true
# JVM arguments
org.gradle.jvmargs=-Xmx4g -XX:+HeapDumpOnOutOfMemoryError -Dfile.encoding=UTF-8
# Enable AndroidX
android.useAndroidX=true
# Build caching and parallel execution
org.gradle.caching=true
org.gradle.parallel=true
# File system watching for faster builds
org.gradle.unsafe.watch-fs=true
"#;

const GRADLE_WRAPPER_PROPERTIES: &str = r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-{{GRADLE}}-bin.zip
networkTimeout=10000
validateDistributionUrl=true
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
"#;

const APP_BUILD_GRADLE: &str = r#"plugins {
    id 'com.android.application'
}

// Release signing: optional, and absent by default -- `assembleRelease` still works without it
// (producing an unsigned artifact no device will install). Create keystore.properties next to this
// file, as README.md's "Release signing" section describes, and it is picked up automatically.
def keystorePropertiesFile = rootProject.file("keystore.properties")
def keystoreProperties = new Properties()
if (keystorePropertiesFile.exists()) {
    keystoreProperties.load(new FileInputStream(keystorePropertiesFile))
}

android {
    compileSdk = {{COMPILE_SDK}}

    // `foliage-android build` sets this, because it is what resolved the NDK in the first place.
    // Without it AGP cannot find the strip tool and packages debug libraries whole -- unstripped
    // Rust debug info is most of a debug APK, and the only sign is one line of build output.
    def ndk = System.getenv("ANDROID_NDK_HOME")
    if (ndk != null && !ndk.isEmpty()) {
        ndkPath = ndk
        // An SDK-installed NDK sits in a directory named for its version. AGP defaults ndkVersion
        // to whatever it was built against and reports CXX1100 when the two disagree, so the path
        // is not enough on its own. A standalone NDK elsewhere keeps AGP's default.
        def version = new File(ndk).getName()
        if (version ==~ /\d+\.\d+\.\d+/) {
            ndkVersion = version
        }
    }

    defaultConfig {
        applicationId = "{{APP_ID}}"
        minSdk = {{MIN_SDK}}
        targetSdk = {{TARGET_SDK}}
        versionCode = 1
        versionName = "1.0"
    }

    signingConfigs {
        release {
            if (keystorePropertiesFile.exists()) {
                storeFile rootProject.file(keystoreProperties['storeFile'])
                storePassword keystoreProperties['storePassword']
                keyAlias keystoreProperties['keyAlias']
                keyPassword keystoreProperties['keyPassword']
            }
        }
    }

    buildTypes {
        release {
            minifyEnabled = false
            if (keystorePropertiesFile.exists()) {
                signingConfig signingConfigs.release
            }
        }
        debug {
            minifyEnabled = false
        }
    }
    compileOptions {
        sourceCompatibility JavaVersion.VERSION_17
        targetCompatibility JavaVersion.VERSION_17
    }
    namespace = '{{APP_ID}}'
}

dependencies {
    implementation 'androidx.appcompat:appcompat:1.7.0'

    // To use the Games Activity library
    implementation "androidx.games:games-activity:{{GAMES_ACTIVITY}}"
    // Note: don't include game-text-input separately, since it's integrated into game-activity
}
"#;

const ANDROID_MANIFEST: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">

{{PERMISSIONS}}    <application
        android:icon="@android:drawable/sym_def_app_icon"
        android:label="{{APP_NAME}}"
        android:supportsRtl="true"
        android:theme="@style/ActivityTheme">
        <activity
            android:name=".MainActivity"
            android:configChanges="orientation|screenSize|screenLayout|keyboardHidden"
            android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>

            <meta-data android:name="android.app.lib_name" android:value="{{LIB_NAME}}" />
        </activity>
    </application>

</manifest>
"#;

const THEMES_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <style name="ActivityTheme" parent="Theme.AppCompat.Light.NoActionBar">
        <!-- For full-screen layout, cutout support -->
        <item name="android:windowLayoutInDisplayCutoutMode">shortEdges</item>
        <item name="android:windowFullscreen">true</item>
    </style>
</resources>
"#;

const MAIN_ACTIVITY_JAVA: &str = r#"package {{APP_ID}};

import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

import com.google.androidgamesdk.GameActivity;

import android.graphics.Rect;
import android.os.Bundle;
import android.os.SystemClock;
import android.view.KeyEvent;
import android.view.View;
import android.view.WindowManager;

public class MainActivity extends GameActivity {

    static {
        // Must match both the entry crate's cdylib output name and the `android.app.lib_name`
        // meta-data value in AndroidManifest.xml -- all three have to agree, or the JVM cannot
        // find `android_main` to call into.
        System.loadLibrary("{{LIB_NAME}}");
    }

    /** How long after a bar is found covering the window it is hidden again. */
    private static final long REHIDE_MS = 300;

    private final Runnable rehide = this::hideSystemUI;

    private void hideSystemUI() {
        getWindow().getAttributes().layoutInDisplayCutoutMode
                = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_ALWAYS;
        View decorView = getWindow().getDecorView();
        WindowInsetsControllerCompat controller = new WindowInsetsControllerCompat(getWindow(),
                decorView);
        controller.hide(WindowInsetsCompat.Type.systemBars());
        controller.hide(WindowInsetsCompat.Type.displayCutout());
        controller.setSystemBarsBehavior(
                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        WindowCompat.setDecorFitsSystemWindows(getWindow(), false);
        hideSystemUI();
        super.onCreate(savedInstanceState);{{LOCATE_CREATE}}
    }

    protected void onResume() {
        super.onResume();
        hideSystemUI();{{LOCATE_RESUME}}
    }{{LOCATE_MEMBERS}}

    // The soft keyboard brings the navigation bar up with it and leaves it there when it goes,
    // over the foot of the surface, until something hides it again. Nothing about insets is said
    // when it goes (measured on Android 10: no insets are applied, and GameActivity still thinks
    // the keyboard is up), but the window is laid out again. So on every layout: if something
    // covers the foot of the window, and less than a keyboard would, it is a bar, and the bars are
    // hidden again -- a moment later, once the keyboard has finished going: asked while it is still
    // closing, the bars stay.
    @Override
    public void onGlobalLayout() {
        super.onGlobalLayout();
        View decor = getWindow().getDecorView();
        Rect visible = new Rect();
        decor.getWindowVisibleDisplayFrame(visible);
        int covered = decor.getHeight() - visible.bottom;
        if (covered > 0 && covered < decor.getHeight() / 4) {
            decor.removeCallbacks(rehide);
            decor.postDelayed(rehide, REHIDE_MS);
        }
    }

    // And whenever the window has focus back: a dialog or the notification shade shows the bars
    // the same way.
    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            hideSystemUI();
        }
    }

    /** How soon after one editor action another is the same press of the key. */
    private static final long ENTER_MS = 100;

    private long entered = Long.MIN_VALUE / 2;

    // The soft keyboard's enter key is turned into an editor action by the text input's own
    // connection, and the key itself goes no further -- and winit reads the action and drops it. So
    // the action is handed on as the key it was, which reaches the app like a hardware enter does.
    // Once a press: the connection makes an action of the key going down and of it coming up, a
    // few milliseconds apart.
    @Override
    public void onEditorAction(int action) {
        super.onEditorAction(action);
        long now = SystemClock.uptimeMillis();
        if (now - entered < ENTER_MS) {
            return;
        }
        entered = now;
        onKeyDown(KeyEvent.KEYCODE_ENTER,
                new KeyEvent(now, now, KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_ENTER, 0));
        onKeyUp(KeyEvent.KEYCODE_ENTER,
                new KeyEvent(now, now, KeyEvent.ACTION_UP, KeyEvent.KEYCODE_ENTER, 0));
    }

    // A soft keyboard's control key is never sent as a key of its own: it rides on the letter, as
    // meta state, and winit reads no meta state. So a key that carries control without being it is
    // given a control key going down before it and coming up after, which is what a hardware
    // keyboard would have sent -- and Ctrl+A selects rather than typing an "a".
    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (!chorded(event)) {
            return super.onKeyDown(keyCode, event);
        }
        super.onKeyDown(KeyEvent.KEYCODE_CTRL_LEFT, control(event, KeyEvent.ACTION_DOWN));
        boolean handled = super.onKeyDown(keyCode, event);
        super.onKeyUp(KeyEvent.KEYCODE_CTRL_LEFT, control(event, KeyEvent.ACTION_UP));
        return handled;
    }

    private static boolean chorded(KeyEvent event) {
        int code = event.getKeyCode();
        return event.isCtrlPressed()
                && code != KeyEvent.KEYCODE_CTRL_LEFT
                && code != KeyEvent.KEYCODE_CTRL_RIGHT;
    }

    private static KeyEvent control(KeyEvent event, int action) {
        return new KeyEvent(event.getDownTime(), event.getEventTime(), action,
                KeyEvent.KEYCODE_CTRL_LEFT, 0, event.getMetaState(), event.getDeviceId(), 0,
                event.getFlags(), event.getSource());
    }
}
"#;

const ENTRY_CRATE_MANIFEST: &str = r#"[package]
name = "{{ENTRY_CRATE}}"
version = "0.1.0"
edition = "2024"
publish = false
description = "Android's entry point. Generated by `foliage-android init`."

# The whole point of this crate: Android loads it as a `.so` and calls `android_main` through JNI.
# The name below is what `System.loadLibrary` and the manifest's `android.app.lib_name` resolve.
[lib]
crate-type = ["cdylib"]

[dependencies]
{{APP_CRATE}} = { path = "{{APP_PATH}}" }
# The same spec the app crate declares. Both halves of the program have to link one engine, so this
# is copied rather than chosen -- change it there and rerun `foliage-android init`.
foliage = {{FOLIAGE}}
"#;

const ENTRY_CRATE_LIB: &str = r#"//! Android's process entry point, kept in its own `crate-type = ["cdylib"]` crate rather than in
//! the app crate itself -- a cdylib target also produces a competing `wasm32-unknown-unknown`
//! artifact sharing the app's own binary name, which breaks trunk's artifact selection for the real
//! wasm build.
//!
//! Built through `foliage-android build`, which is `cargo ndk ... -p` this crate. Nothing else
//! should depend on it.

/// The Java `GameActivity` shim loads this crate as a `.so` and calls this through JNI, handing
/// over the one thing [`Foliage::android`] needs and no other platform has: a live activity.
///
/// [`Foliage::android`]: foliage::Foliage::android
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(activity: foliage::AndroidApp) {
    {{APP_CRATE}}::{{APP_ENTRY}}(foliage::Foliage::android(activity));
}
"#;

const README: &str = r#"# {{APP_NAME}} -- Android

Generated by `foliage-android init`, **including this file**. A rerun overwrites everything in this
directory, so keep notes of your own somewhere else. Settings live in `foliage-android.toml` at the
repo root -- edit there and rerun `init`, rather than editing these files.

This Gradle project exists because `androidx.games:games-activity` -- Google's `GameActivity`, which
winit's `android-game-activity` backend targets -- is a Java/Kotlin AAR, and there is no way to link
one without Gradle. Unavoidable rather than a workaround.

## Building

From the repo root:

```sh
foliage-android setup    # the SDK, NDK and rust targets, into the `sdk` directory
foliage-android doctor   # what is installed, and what is not
foliage-android build    # every configured ABI, then the APK
foliage-android run      # the above, installed and launched on a connected device
```

`build` compiles each ABI into `app/src/main/jniLibs/<abi>/` and then assembles. Doing it by hand is
what the tool exists to avoid: `jniLibs/` is an *input* directory to Gradle rather than an output it
tracks, so whatever `.so` is sitting there gets packaged. A Rust build that failed, was skipped, or
wrote somewhere else still assembles successfully -- shipping the *previous* library. The luckier
outcome is an `UnsatisfiedLinkError` at launch; the unluckier one is old code running quietly.

## SDK levels

Three numbers do three different jobs, and only one gates hardware:

- **`compile-sdk`** -- which API the compiler sees. A build-time download
  (`platforms/android-{{COMPILE_SDK}}`), with no bearing on what the APK runs on.
- **`min-sdk`** -- the device gate. Anything below refuses to install the APK.
- **`target-sdk`** -- which platform behaviours the app opts into. Affects runtime behaviour on
  newer devices; not an install gate.

Lowering `compile-sdk` does not widen device support -- `min-sdk` is the knob for that. Whichever
`compile-sdk` is set, install the matching `platforms/android-<n>` or Gradle will not find it.

## Emulator

Useful for exercising API levels you do not own hardware for.

```sh
android sdk install emulator system-images/android-{{COMPILE_SDK}}/google_apis/{{EMULATOR_ABI}}
android emulator create --list-profiles      # what you can pick from
android emulator create medium_phone         # profile is positional, and required
android emulator start medium_phone
```

On Linux the emulator needs `/dev/kvm`; `test -r /dev/kvm && test -w /dev/kvm && echo ok` settles
whether you have it, and `sudo usermod -aG kvm $USER` plus a re-login is the fix. Group membership
is not the whole story -- some distributions grant access through an ACL -- so test rather than
reading `groups`.

**Force the GPU, or everything is software-rendered.** New AVDs default to `hw.gpu.mode=auto`, and
`auto` readily decides against a capable host GPU and falls back to SwiftShader -- CPU rendering,
sluggish enough that the launcher feels broken, not just your app. Set `hw.gpu.mode=host` in
`~/.android/avd/<name>.avd/config.ini`. To check which you got:
`adb shell dumpsys SurfaceFlinger | grep -i gles` names either your real GPU or `SwiftShader`.

**Build the emulator's own ABI.** Current x86_64 system images translate ARM binaries, so an
`arm64-v8a`-only APK does run on an x86_64 emulator -- routed through `libndk_translation.so`, slow
enough to make frame timings meaningless and to trip ANR dialogs. `init` adds this machine's ABI to
`foliage-android.toml` for that reason; both live in `jniLibs/` at once and Android prefers a native
ABI over a translated one, so one APK serves emulator and phone.

AVDs live in `~/.android/avd/`, **not** in the SDK directory -- deleting the SDK leaves them behind.

## Release signing

`foliage-android build` needs nothing extra: AGP generates and uses a debug keystore.
`--release` also runs without one, but produces an **unsigned** APK no device will install. For a
real signed release:

1. Generate a keystore once. Keep the file and passwords somewhere safe outside the repo -- losing
   this keystore means never shipping an update to the same `applicationId` under the same identity
   again.

   ```sh
   keytool -genkeypair -v -keystore ~/keys/{{LIB_NAME}}-release.keystore \
     -alias {{LIB_NAME}} -keyalg RSA -keysize 2048 -validity 10000
   ```

2. Create `{{PROJECT}}/keystore.properties`, and gitignore it:

   ```properties
   storeFile=/home/you/keys/{{LIB_NAME}}-release.keystore
   storePassword=<the password you set above>
   keyAlias={{LIB_NAME}}
   keyPassword=<same password, unless you set a separate key password>
   ```

3. That is all -- `app/build.gradle` reads that file whenever it is present.

## Troubleshooting

- **App installs but crashes immediately.** Clear the log, launch, and read the abort message:

  ```sh
  adb logcat -c
  adb shell am start -n {{APP_NAME}}/.MainActivity
  adb logcat -d | grep -B5 -A40 "Fatal signal"
  ```

  The tombstone's `Abort message:` line names the cause. `System.loadLibrary` failing means
  `jniLibs/` has nothing for the device's ABI -- add it to `abis` in `foliage-android.toml`.
- **`RegisterNatives failed` / `NoSuchMethodError` on a GameActivity method.** The AAR and the
  GameActivity C++ compiled into the library are different versions. `foliage-android doctor`
  compares the two and names the mismatch. This aborts before any Rust runs, so it is not a bug in
  your app. Prefer `cargo update -p android-activity` over pinning an older AAR: matching them at
  the older version also stops the crash, but a GameActivity years behind the running platform can
  deliver no touch input at all -- the app launches, draws correctly, and ignores every tap, with
  nothing in the log.
- **Gradle disagrees about where the SDK is.** Check `local.properties`, which Gradle writes with
  the path it resolved, against what `foliage-android doctor` reports. Nothing in this toolchain
  reads `ANDROID_HOME`: the SDK is the `sdk` path in `foliage-android.toml` and nowhere else, and
  the variables Gradle and `cargo ndk` need are set on those processes rather than in your shell.
- **A download died partway.** `setup` installs one package at a time and retries each, so what
  already arrived is kept -- rerun it and it resumes rather than starting over.

## Getting the APK onto a phone

`foliage-android run` uses `adb`, which needs USB debugging. Without it, transfer the APK however
you like -- USB copy, email, syncthing. The phone will refuse to install it until you allow it:
whatever app opens the APK prompts "Install unknown apps" the first time.
"#;
