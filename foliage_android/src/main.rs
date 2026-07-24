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
        /// `androidx.games:games-activity` version.
        #[arg(long, default_value = "4.4.0")]
        games_activity_version: String,
        /// Android Gradle Plugin version.
        #[arg(long, default_value = "9.1.0")]
        agp_version: String,
        #[arg(long, default_value = "9.3.1")]
        gradle_version: String,
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
        agp_version,
        gradle_version,
        skip_wrapper,
    } = Cli::parse();
    let app_name = app_name.unwrap_or_else(|| lib_name.clone());
    let package_path = app_id.replace('.', "/");

    write(&out.join("build.gradle"), &templates::root_build_gradle(&agp_version))?;
    write(&out.join("settings.gradle"), &templates::settings_gradle())?;
    write(&out.join("gradle.properties"), &templates::gradle_properties())?;
    write(
        &out.join("gradle/wrapper/gradle-wrapper.properties"),
        &templates::gradle_wrapper_properties(&gradle_version),
    )?;
    write(
        &out.join("app/build.gradle"),
        &templates::app_build_gradle(&app_id, min_sdk, compile_sdk, target_sdk, &games_activity_version),
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
        &out.join(format!("app/src/main/java/{package_path}/MainActivity.java")),
        &templates::main_activity_java(&app_id, &lib_name),
    )?;

    if skip_wrapper {
        println!(
            "skipped wrapper download -- run `gradle wrapper --gradle-version {gradle_version}` \
             inside {} yourself before building",
            out.display()
        );
    } else {
        download(&format!("{WRAPPER_SOURCE}/gradlew"), &out.join("gradlew"))?;
        download(&format!("{WRAPPER_SOURCE}/gradlew.bat"), &out.join("gradlew.bat"))?;
        download(
            &format!("{WRAPPER_SOURCE}/gradle/wrapper/gradle-wrapper.jar"),
            &out.join("gradle/wrapper/gradle-wrapper.jar"),
        )?;
    }

    println!();
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
    println!(
        "  # -> app/build/outputs/apk/debug/app-debug.apk, transfer it however you like"
    );
    Ok(())
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
        fs::set_permissions(dest, perms)
            .map_err(|e| format!("chmod {}: {e}", dest.display()))?;
    }
    println!("downloaded {url} -> {}", dest.display());
    Ok(())
}
