mod templates;

use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The `agdk-mainloop` example inside `rust-mobile/android-activity` -- the crate that
/// actually implements winit's `android-game-activity` backend -- is the source of truth for
/// every template in `templates.rs`, and for the Gradle wrapper files fetched at generation
/// time (a wrapper jar is a binary, not something to hand-transcribe).
const WRAPPER_SOURCE: &str =
    "https://raw.githubusercontent.com/rust-mobile/android-activity/main/examples/agdk-mainloop";

/// Scaffolds a minimal GameActivity + Gradle Android project around a foliage app's compiled
/// cdylib -- the parts that are unavoidable (Android only links a GameActivity AAR in through
/// Gradle, there's no way around that), generated instead of hand-written.
#[derive(Parser)]
#[command(name = "foliage_android")]
enum Cli {
    /// Write the Gradle project (build files, manifest, `MainActivity`) into `--out`, and
    /// fetch the Gradle wrapper unless `--skip-wrapper` is set.
    Gen {
        /// Reverse-domain application id, e.g. `io.github.yourname.yourapp`.
        #[arg(long)]
        app_id: String,
        /// The compiled cdylib's crate name (the `[lib] name` in the app's Cargo.toml) --
        /// what `System.loadLibrary` and the manifest's `android.app.lib_name` load by.
        #[arg(long)]
        lib_name: String,
        /// Human-readable label shown under the launcher icon. Defaults to `--lib-name`.
        #[arg(long)]
        app_name: Option<String>,
        /// Output directory for the generated Gradle project.
        #[arg(long, default_value = "android")]
        out: PathBuf,
        #[arg(long, default_value_t = 26)]
        min_sdk: u32,
        #[arg(long, default_value_t = 35)]
        compile_sdk: u32,
        #[arg(long, default_value_t = 35)]
        target_sdk: u32,
        /// `androidx.games:games-activity` version. Must match the GameActivity C++ that
        /// `android-activity` bundles -- grep `GAMEACTIVITY_{MAJOR,MINOR,BUGFIX}_VERSION` out
        /// of its vendored `GameActivity.h` to check. `GameActivity_register` registers its
        /// whole JNI table in one call, so one changed signature fails `RegisterNatives` and
        /// aborts the process at launch, before any Rust runs.
        ///
        /// 4.4.0 matches android-activity 0.6.1. Note 0.6.0 bundles 2.0.2 instead, so a stale
        /// `Cargo.lock` resolving to 0.6.0 is what makes this default look wrong -- update the
        /// lock rather than downgrading this, since 2.0.2 predates current Android by years.
        #[arg(long, default_value = "4.4.0")]
        games_activity_version: String,
        /// NDK the README tells people to install. Any reasonably recent one works --
        /// cargo-ndk doesn't care which -- so this is a known-good default, not a pin.
        #[arg(long, default_value = "27.3.13750724")]
        ndk_version: String,
        /// Build-tools package for the README's install line. Defaults to
        /// `<compile-sdk>.0.0`, which is the usual shape but not a rule -- point releases
        /// exist (35.0.1, 36.1.0). `android sdk list --all` shows what's real.
        #[arg(long)]
        build_tools_version: Option<String>,
        /// Host the README's `android` CLI download URL targets, as `<os>_<arch>`:
        /// `linux_x86_64`, `darwin_arm64`, `darwin_x86_64`, `windows_x86_64`. Defaults to
        /// the machine running `gen`. Also picks the emulator system-image ABI, since an
        /// emulator runs the host's architecture.
        #[arg(long)]
        host: Option<String>,
        /// Android Gradle Plugin version.
        #[arg(long, default_value = "9.1.0")]
        agp_version: String,
        #[arg(long, default_value = "9.3.1")]
        gradle_version: String,
        /// Also scaffold the cdylib entry crate at this path -- the `android_main` shim the
        /// Gradle project loads. Off unless given, and unlike `--out` this writes *outside*
        /// the generated project, so it refuses to touch a directory that already holds a
        /// `Cargo.toml` unless `--overwrite-entry-crate` is set.
        #[arg(long)]
        entry_crate: Option<PathBuf>,
        /// Crate holding the real entry function. Defaults to `--lib-name` minus a trailing
        /// `_android`, which is the usual pairing.
        #[arg(long)]
        app_crate: Option<String>,
        /// Function in `--app-crate` taking a `Foliage`, called from `android_main`.
        #[arg(long, default_value = "run")]
        app_entry: String,
        /// Cargo dependency spec for `--app-crate`, as it should appear after the `=`.
        /// Defaults to a path dep pointing at a sibling directory.
        #[arg(long)]
        app_crate_dep: Option<String>,
        /// Cargo dependency spec for `foliage`, as it should appear after the `=`. Spliced in
        /// verbatim and never parsed, so anything cargo accepts works -- add `tag = "v1.0.0"`
        /// to pin a release, or pass `{{ path = "../foliage" }}` when generating inside the
        /// foliage repo itself. Defaults to a git dependency because foliage isn't published.
        #[arg(
            long,
            default_value = "{ git = \"https://github.com/eblack-leaf/foliage\" }"
        )]
        foliage_dep: String,
        /// Let `--entry-crate` overwrite an existing crate. Off by default: that directory is
        /// yours, not generated output, and may hold edits worth keeping.
        #[arg(long)]
        overwrite_entry_crate: bool,
        /// Don't fetch gradlew/gradlew.bat/gradle-wrapper.jar -- e.g. if you already have a
        /// wrapper set up elsewhere, or have no network access right now.
        #[arg(long)]
        skip_wrapper: bool,
    },
}

fn main() -> Result<(), String> {
    let Cli::Gen {
        app_id,
        lib_name,
        app_name,
        out,
        min_sdk,
        compile_sdk,
        target_sdk,
        games_activity_version,
        ndk_version,
        build_tools_version,
        host,
        entry_crate,
        app_crate,
        app_entry,
        app_crate_dep,
        foliage_dep,
        overwrite_entry_crate,
        agp_version,
        gradle_version,
        skip_wrapper,
    } = Cli::parse();
    let app_name = app_name.unwrap_or_else(|| lib_name.clone());
    let build_tools_version = build_tools_version.unwrap_or_else(|| format!("{compile_sdk}.0.0"));
    let host = host.unwrap_or_else(default_host);
    let package_path = app_id.replace('.', "/");

    write(
        &out.join("build.gradle"),
        &templates::root_build_gradle(&agp_version),
    )?;
    write(&out.join("settings.gradle"), &templates::settings_gradle())?;
    write(
        &out.join("gradle.properties"),
        &templates::gradle_properties(),
    )?;
    write(
        &out.join("gradle/wrapper/gradle-wrapper.properties"),
        &templates::gradle_wrapper_properties(&gradle_version),
    )?;
    write(
        &out.join("app/build.gradle"),
        &templates::app_build_gradle(
            &app_id,
            min_sdk,
            compile_sdk,
            target_sdk,
            &games_activity_version,
        ),
    )?;
    write(
        &out.join("app/src/main/AndroidManifest.xml"),
        &templates::android_manifest(&app_name, &lib_name),
    )?;
    write(
        &out.join("app/src/main/res/values/themes.xml"),
        &templates::themes_xml(),
    )?;
    write(
        &out.join(format!(
            "app/src/main/java/{package_path}/MainActivity.java"
        )),
        &templates::main_activity_java(&app_id, &lib_name),
    )?;
    write(
        &out.join("README.md"),
        &templates::readme(
            &app_name,
            &lib_name,
            &out.display().to_string(),
            min_sdk,
            compile_sdk,
            target_sdk,
            &ndk_version,
            &build_tools_version,
            &host,
        ),
    )?;

    if let Some(entry_crate) = entry_crate.as_ref() {
        // `--out` is generated output and safe to clobber; this isn't. Refuse rather than
        // silently replacing a crate someone has edited.
        let manifest = entry_crate.join("Cargo.toml");
        if manifest.exists() && !overwrite_entry_crate {
            return Err(format!(
                "{} already exists -- refusing to overwrite an entry crate that may hold your \
                 edits. Pass --overwrite-entry-crate to replace it, or point --entry-crate \
                 somewhere else.",
                manifest.display()
            ));
        }
        let app_crate = app_crate.unwrap_or_else(|| {
            lib_name
                .strip_suffix("_android")
                .unwrap_or(&lib_name)
                .to_string()
        });
        let app_crate_dep =
            app_crate_dep.unwrap_or_else(|| format!("{{ path = \"../{app_crate}\" }}"));
        write(
            &manifest,
            &templates::entry_crate_cargo_toml(&lib_name, &app_crate, &app_crate_dep, &foliage_dep),
        )?;
        write(
            &entry_crate.join("src/lib.rs"),
            &templates::entry_crate_lib_rs(&app_crate, &app_entry),
        )?;
    }

    if skip_wrapper {
        println!(
            "skipped wrapper download -- run `gradle wrapper --gradle-version {gradle_version}` \
             inside {} yourself before building",
            out.display()
        );
    } else {
        download(&format!("{WRAPPER_SOURCE}/gradlew"), &out.join("gradlew"))?;
        download(
            &format!("{WRAPPER_SOURCE}/gradlew.bat"),
            &out.join("gradlew.bat"),
        )?;
        download(
            &format!("{WRAPPER_SOURCE}/gradle/wrapper/gradle-wrapper.jar"),
            &out.join("gradle/wrapper/gradle-wrapper.jar"),
        )?;
    }

    println!();
    if let Some(entry_crate) = entry_crate.as_ref() {
        println!(
            "note: add \"{}\" to your workspace's `members` if it isn't covered already -- \
             cargo won't build it otherwise.",
            entry_crate.display()
        );
        println!();
    }
    println!(
        "next: compile your app's cdylib per-ABI into {}/app/src/main/jniLibs/<abi>/, e.g.:",
        out.display()
    );
    println!(
        "  cargo ndk -t arm64-v8a -o {}/app/src/main/jniLibs/ build --release -p <your-app-crate>",
        out.display()
    );
    println!("then:");
    println!("  cd {} && ./gradlew assembleDebug", out.display());
    println!("  # -> app/build/outputs/apk/debug/app-debug.apk, transfer it however you like");
    Ok(())
}

/// `<os>_<arch>` for the machine running `gen`, matching the naming
/// `dl.google.com/android/cli/latest/<host>/android` uses. Only a default -- `--host`
/// overrides it, which is what you want when generating for someone else's machine.
fn default_host() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "windows",
        _ => "linux",
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x86_64",
    };
    format!("{os}_{arch}")
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    fs::write(path, contents).map_err(|e| format!("writing {}: {e}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

fn download(url: &str, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    let bytes = reqwest::blocking::get(url)
        .map_err(|e| format!("fetching {url}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("fetching {url}: {e}"))?
        .bytes()
        .map_err(|e| format!("reading {url}: {e}"))?;
    let mut f = fs::File::create(dest).map_err(|e| format!("creating {}: {e}", dest.display()))?;
    f.write_all(&bytes)
        .map_err(|e| format!("writing {}: {e}", dest.display()))?;
    #[cfg(unix)]
    if dest.file_name().and_then(|n| n.to_str()) == Some("gradlew") {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dest)
            .map_err(|e| format!("stat {}: {e}", dest.display()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dest, perms).map_err(|e| format!("chmod {}: {e}", dest.display()))?;
    }
    println!("downloaded {url} -> {}", dest.display());
    Ok(())
}
