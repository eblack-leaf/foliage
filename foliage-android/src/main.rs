//! The Android side of a foliage app: scaffolded once, then built and run from here.
//!
//! Android is the one platform a Rust program cannot reach on its own. The process belongs to a
//! Java activity, `androidx.games:games-activity` is an AAR that only Gradle can link, and the
//! toolchain that compiles for it is reached through environment variables. None of that is
//! avoidable, so this generates it and then drives it.
//!
//! ```text
//! foliage-android init --app-id io.github.you.yourapp
//! foliage-android setup
//! foliage-android run
//! ```
//!
//! [`init`] settles every choice once and records it in `foliage-android.toml`; every command after
//! that reads it and needs no arguments. What the generated project does *not* hold is the SDK --
//! [`setup`] installs one, into the directory that file names -- the only place any command looks.
//! No environment variable is read, so a build does not depend on how its shell was prepared; the
//! variables `cargo ndk` and Gradle need are written onto those processes instead.
//!
//! [`init`]: Cli::Init
//! [`setup`]: Cli::Setup

mod environment;
mod project;
mod templates;

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;

use environment::Environment;
use project::Project;

/// The Gradle wrapper's upstream, which is the same example the templates come from. A wrapper jar
/// is a binary, not something to hand-transcribe.
const WRAPPER: &str =
    "https://raw.githubusercontent.com/rust-mobile/android-activity/main/examples/agdk-mainloop";

/// The commands.
#[derive(Parser)]
#[command(name = "foliage-android", version, about, long_about = None)]
enum Cli {
    /// Write the Gradle project and the entry crate, and record the choices in
    /// `foliage-android.toml`.
    ///
    /// Only `--app-id` is needed, and only the first time: a rerun reads the file back and takes
    /// every value from it, so this is also how the project is regenerated after editing it.
    Init {
        /// Reverse-domain application id, e.g. `io.github.yourname.yourapp`.
        ///
        /// Required the first time and permanent afterwards: it is the identity a device installs
        /// under, and an update has to carry the id it shipped with.
        #[arg(long)]
        app_id: Option<String>,
        /// The crate holding the entry function. Defaults to the package in the current directory.
        #[arg(long)]
        app_crate: Option<String>,
        /// The label under the launcher icon. Defaults to `--app-crate`.
        #[arg(long)]
        app_name: Option<String>,
        /// The function in `--app-crate` taking a `Foliage`, called from `android_main`.
        #[arg(long, default_value = "run")]
        app_entry: String,
        /// Rewrite the entry crate even though one is already there.
        ///
        /// Off by default. The Gradle project is generated output and safe to replace; the entry
        /// crate is a crate in your workspace and may hold edits worth keeping.
        #[arg(long)]
        overwrite_entry_crate: bool,
        /// Do not fetch the Gradle wrapper -- for a machine with no network, or a wrapper already
        /// set up elsewhere.
        #[arg(long)]
        skip_wrapper: bool,
    },
    /// Install the toolchain into the `sdk` directory `foliage-android.toml` names.
    ///
    /// Everything Android needs and nothing outside that directory, so removing it is one `rm -rf`
    /// and no part of the machine is left changed. The two exceptions are a rustup target and
    /// `cargo-ndk`, which are toolchain components shared with every Rust project and have no
    /// per-repo equivalent.
    ///
    /// A JDK is the one thing this cannot install: it is the package manager's, and every route to
    /// one needs a privilege this should not take. `doctor` says so if it is missing.
    Setup {
        /// The host to download the `android` CLI for, as `<os>_<arch>` -- `linux_x86_64`,
        /// `darwin_arm64`, `darwin_x86_64`, `windows_x86_64`. Defaults to this machine.
        #[arg(long)]
        host: Option<String>,
    },
    /// Report what the toolchain is missing, and the command that installs each piece.
    Doctor,
    /// Compile every configured ABI and assemble the APK.
    Build {
        /// Build the Rust in release and assemble the release APK.
        ///
        /// Unsigned unless `<project>/keystore.properties` is present, which no device will
        /// install. See the generated README's "Release signing".
        #[arg(long)]
        release: bool,
        /// Only this ABI, rather than every one in `foliage-android.toml`.
        #[arg(long)]
        abi: Option<String>,
    },
    /// Build, install onto a connected device, and launch.
    Run {
        #[arg(long)]
        release: bool,
        #[arg(long)]
        abi: Option<String>,
        /// The device serial, as `adb devices` lists it. Only needed when more than one is attached.
        #[arg(long)]
        device: Option<String>,
        /// Follow the log after launching, filtered to this app.
        #[arg(long)]
        logcat: bool,
    },
}

fn main() -> Result<(), String> {
    let root = root()?;
    match Cli::parse() {
        Cli::Init {
            app_id,
            app_crate,
            app_name,
            app_entry,
            overwrite_entry_crate,
            skip_wrapper,
        } => init(
            &root,
            app_id,
            app_crate,
            app_name,
            &app_entry,
            overwrite_entry_crate,
            skip_wrapper,
        ),
        Cli::Setup { host } => setup(&root, &Project::load(&root)?, host.as_deref()),
        Cli::Doctor => environment::doctor(&root, &Project::load(&root)?),
        Cli::Build { release, abi } => {
            build(&root, &Project::load(&root)?, release, abi.as_deref()).map(|apk| {
                println!("\n{}", apk.display());
            })
        }
        Cli::Run {
            release,
            abi,
            device,
            logcat,
        } => run(
            &root,
            &Project::load(&root)?,
            release,
            abi.as_deref(),
            device.as_deref(),
            logcat,
        ),
    }
}

/// Writes the project, and the file every later command reads.
fn init(
    root: &Path,
    app_id: Option<String>,
    app_crate: Option<String>,
    app_name: Option<String>,
    app_entry: &str,
    overwrite_entry_crate: bool,
    skip_wrapper: bool,
) -> Result<(), String> {
    // A rerun is a regeneration: everything not passed comes from what `init` decided last time,
    // so no argument has to be repeated to change one of them.
    let existing = Project::load(root).ok();
    let app_crate = app_crate
        .or_else(|| existing.as_ref().map(|project| project.app_crate.clone()))
        .or_else(|| package_name(&std::env::current_dir().ok()?.join("Cargo.toml")))
        .ok_or(
            "no --app-crate, and no [package] in ./Cargo.toml to take one from -- run this from \
             your app's crate, or pass --app-crate <name>.",
        )?;
    let app_id = app_id
        .or_else(|| existing.as_ref().map(|project| project.app_id.clone()))
        .ok_or(
            "--app-id is required the first time. It is the identity a device installs under and \
             cannot change afterwards, so there is no default worth guessing -- something like \
             `io.github.yourname.yourapp`.",
        )?;
    let entry_crate = existing
        .as_ref()
        .map(|project| project.entry_crate.clone())
        .unwrap_or_else(|| format!("{app_crate}-android"));
    // Where the app crate actually is, which is what the generated project is put beside. The
    // directory this was run from when that is the crate, so a layout cargo allows but does not
    // require -- a crate somewhere other than a directory named after it -- still resolves.
    let here = std::env::current_dir().map_err(|e| format!("no working directory: {e}"))?;
    let app_dir = match package_name(&here.join("Cargo.toml")).as_deref() == Some(&app_crate) {
        true => here,
        false => crate_dir(root, &app_crate).ok_or_else(|| {
            format!(
                "no crate `{app_crate}` in {} -- is it a workspace member?",
                root.display()
            )
        })?,
    };
    let project = Project {
        app_name: app_name
            .or_else(|| existing.as_ref().map(|project| project.app_name.clone()))
            .unwrap_or_else(|| app_crate.clone()),
        app_id,
        app_crate,
        entry_crate,
        // Beside the crate it belongs to, rather than at the root. It is that app's Android
        // project, and a repo with two apps in it would otherwise have them collide.
        sdk: existing
            .as_ref()
            .map_or_else(|| project::SDK.to_string(), |p| p.sdk.clone()),
        project: existing.as_ref().map_or_else(
            || match relative(root, &app_dir) {
                Some(app) => format!("{app}/{}", project::PROJECT),
                None => project::PROJECT.to_string(),
            },
            |p| p.project.clone(),
        ),
        // A phone and this machine's emulator, which are the two things anyone builds for first.
        // Both live in `jniLibs/` at once and Android prefers a native ABI to a translated one.
        abis: existing.as_ref().map_or_else(
            || {
                let mut abis = vec!["arm64-v8a".to_string()];
                let host = environment::host_abi().to_string();
                if !abis.contains(&host) {
                    abis.push(host);
                }
                abis
            },
            |p| p.abis.clone(),
        ),
        min_sdk: existing.as_ref().map_or(project::MIN_SDK, |p| p.min_sdk),
        compile_sdk: existing
            .as_ref()
            .map_or(project::COMPILE_SDK, |p| p.compile_sdk),
        target_sdk: existing
            .as_ref()
            .map_or(project::TARGET_SDK, |p| p.target_sdk),
        games_activity: existing.as_ref().map_or_else(
            || project::GAMES_ACTIVITY.to_string(),
            |p| p.games_activity.clone(),
        ),
        agp: existing
            .as_ref()
            .map_or_else(|| project::AGP.to_string(), |p| p.agp.clone()),
        gradle: existing
            .as_ref()
            .map_or_else(|| project::GRADLE.to_string(), |p| p.gradle.clone()),
        ndk: existing
            .as_ref()
            .map_or_else(|| project::NDK.to_string(), |p| p.ndk.clone()),
    };
    for abi in &project.abis {
        environment::rust_target(abi)?;
    }
    project.save(root)?;

    let out = project.project(root);
    let lib_name = project.lib_name();
    write(
        &out.join("build.gradle"),
        &templates::root_build_gradle(&project.agp),
    )?;
    write(&out.join("settings.gradle"), &templates::settings_gradle())?;
    write(
        &out.join("gradle.properties"),
        &templates::gradle_properties(),
    )?;
    write(
        &out.join("gradle/wrapper/gradle-wrapper.properties"),
        &templates::gradle_wrapper_properties(&project.gradle),
    )?;
    write(
        &out.join("app/build.gradle"),
        &templates::app_build_gradle(
            &project.app_id,
            project.min_sdk,
            project.compile_sdk,
            project.target_sdk,
            &project.games_activity,
        ),
    )?;
    write(
        &out.join("app/src/main/AndroidManifest.xml"),
        &templates::android_manifest(&project.app_name, &lib_name),
    )?;
    write(
        &out.join("app/src/main/res/values/themes.xml"),
        &templates::themes_xml(),
    )?;
    write(
        &out.join(format!(
            "app/src/main/java/{}/MainActivity.java",
            project.app_id.replace('.', "/")
        )),
        &templates::main_activity_java(&project.app_id, &lib_name),
    )?;
    write(
        &out.join("README.md"),
        &templates::readme(
            &project.app_name,
            &lib_name,
            &project.project,
            project.compile_sdk,
        ),
    )?;
    ignore(root, &project)?;

    let entry = root.join(&project.entry_crate);
    let manifest = entry.join("Cargo.toml");
    if manifest.exists() && !overwrite_entry_crate {
        println!(
            "kept {} -- it already exists. Pass --overwrite-entry-crate to replace it.",
            manifest.display()
        );
    } else {
        write(
            &manifest,
            &templates::entry_crate_manifest(
                &project.entry_crate,
                &project.app_crate,
                &relative(&entry, &app_dir).unwrap_or_else(|| app_dir.display().to_string()),
                &dependency(&app_dir, &entry, "foliage")?,
            ),
        )?;
        write(
            &entry.join("src/lib.rs"),
            &templates::entry_crate_lib(&project.app_crate, app_entry),
        )?;
    }
    member(root, &project.entry_crate)?;

    if skip_wrapper {
        println!(
            "skipped the wrapper -- run `gradle wrapper --gradle-version {}` in {} before building",
            project.gradle,
            out.display()
        );
    } else {
        for (name, path) in [
            ("gradlew", "gradlew"),
            ("gradlew.bat", "gradlew.bat"),
            (
                "gradle/wrapper/gradle-wrapper.jar",
                "gradle/wrapper/gradle-wrapper.jar",
            ),
        ] {
            download(&format!("{WRAPPER}/{name}"), &out.join(path))?;
        }
    }

    println!("\nnext: `foliage-android doctor`, then `foliage-android run`.");
    Ok(())
}

/// Installs the toolchain into `.android-sdk/`.
///
/// Each step is skipped when what it installs is already there, so this is the command to run again
/// after changing `compile-sdk` or `ndk` -- it fills in what changed and leaves the rest.
fn setup(root: &Path, project: &Project, host: Option<&str>) -> Result<(), String> {
    let sdk = Environment::sdk(root, project);
    let cli = sdk.join("bin").join(environment::executable("android"));
    let host = host.map_or_else(environment::host, str::to_string);

    if cli.is_file() {
        println!("have {}", cli.display());
    } else {
        println!("== the android CLI");
        download(
            &format!("https://dl.google.com/android/cli/latest/{host}/android"),
            &cli,
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = fs::metadata(&cli)
                .map_err(|e| format!("reading {}: {e}", cli.display()))?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&cli, permissions)
                .map_err(|e| format!("setting {} executable: {e}", cli.display()))?;
        }
    }

    println!("\n== the CLI's own update");
    // Reported and carried on with rather than stopping the run. The update fetches a newer
    // launcher and nothing here needs one -- the version just downloaded installs packages
    // perfectly well -- and a CLI that is genuinely broken fails loudly at the first install
    // rather than being taken on trust.
    if let Err(reason) = update(&cli, &sdk, root) {
        println!("could not update the CLI ({reason}) -- carrying on with the version on disk");
    }

    println!("\n== SDK packages");
    let platform = format!("platforms/android-{}", project.compile_sdk);
    let ndk = format!("ndk/{}", project.ndk);
    let wanted = [
        ("platform-tools", sdk.join("platform-tools")),
        (
            platform.as_str(),
            sdk.join("platforms")
                .join(format!("android-{}", project.compile_sdk)),
        ),
        (ndk.as_str(), sdk.join("ndk").join(&project.ndk)),
    ];
    for (package, at) in &wanted {
        match at.is_dir() {
            true => println!("have {package}"),
            false => install(&cli, &sdk, root, package)?,
        }
    }

    println!("\n== rust targets");
    let installed = environment::rustup_targets();
    for abi in &project.abis {
        let target = environment::rust_target(abi)?;
        match installed.iter().any(|have| have == target) {
            true => println!("have {target}"),
            false => run_in(
                {
                    let mut command = Command::new("rustup");
                    command.args(["target", "add", target]);
                    command
                },
                root,
            )?,
        }
    }
    if environment::have_cargo_ndk() {
        println!("have cargo-ndk");
    } else {
        println!("\n== cargo-ndk");
        run_in(
            {
                let mut command = Command::new("cargo");
                command.args(["install", "cargo-ndk", "--locked"]);
                command
            },
            root,
        )?;
    }

    println!("\n== what that leaves");
    environment::doctor(root, project)
}

/// How many times a package download is attempted before giving up.
///
/// The NDK is some two gigabytes over one connection, which is long enough that an ordinary dropped
/// connection is a normal outcome rather than an exceptional one.
const ATTEMPTS: u32 = 3;

/// Installs one package, retrying a dropped download.
///
/// One command per package, rather than one naming all of them. `android sdk install` stages what
/// it downloads and unpacks at the end, so a batch that fails on its last and largest member throws
/// away the three that already arrived -- and a rerun starts over. Separately, each finishes and is
/// on disk before the next begins, which is also what lets [`setup`] skip what is already there.
fn install(cli: &Path, sdk: &Path, root: &Path, package: &str) -> Result<(), String> {
    let mut attempt = 1;
    loop {
        let mut command = Command::new(cli);
        command.args(["sdk", "install", package]);
        command.env("ANDROID_HOME", sdk);
        command.env("ANDROID_SDK_ROOT", sdk);
        match run_in(command, root) {
            Ok(()) => return Ok(()),
            Err(reason) if attempt < ATTEMPTS => {
                println!(
                    "{package} failed ({reason}) -- retrying, attempt {} of {ATTEMPTS}",
                    attempt + 1
                );
                attempt += 1;
            }
            Err(reason) => {
                return Err(format!(
                    "{package}: {reason}\ngave up after {ATTEMPTS} attempts. Whatever installed \
                     before this is kept -- rerun `foliage-android setup` to resume."
                ));
            }
        }
    }
}

/// Runs `android update`, reading what it said rather than what it returned.
///
/// The CLI prints its own failures and exits 0 regardless, so the status is not an answer here. A
/// download that dies halfway is reported on the output and nowhere else.
fn update(cli: &Path, sdk: &Path, root: &Path) -> Result<(), String> {
    let output = Command::new(cli)
        .arg("update")
        .env("ANDROID_HOME", sdk)
        .env("ANDROID_SDK_ROOT", sdk)
        .current_dir(root)
        .output()
        .map_err(|e| format!("running {}: {e}", cli.display()))?;
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    print!("{said}");
    match output.status.success() && !said.to_lowercase().contains("failed") {
        true => Ok(()),
        false => Err("it reported a failure".to_string()),
    }
}

/// Compiles every ABI and assembles the APK, and answers with where it landed.
fn build(
    root: &Path,
    project: &Project,
    release: bool,
    abi: Option<&str>,
) -> Result<PathBuf, String> {
    let environment = Environment::resolve(root, project).map_err(doctor_first)?;
    let abis: Vec<String> = match abi {
        Some(abi) => {
            environment::rust_target(abi)?;
            vec![abi.to_string()]
        }
        None => project.abis.clone(),
    };
    let jni_libs = project.jni_libs(root);
    for abi in &abis {
        println!("\n== {abi}");
        let mut command = Command::new("cargo");
        command.arg("ndk");
        command.args(["-t", abi]);
        // Absolute, and run from the repo root. `-o` is relative to the working directory, and
        // getting that wrong writes a mirrored path nothing reads -- while cargo still reports
        // success and Gradle still packages whatever was in `jniLibs/` beforehand.
        command.args(["-o".as_ref(), jni_libs.as_os_str()]);
        command.arg("build");
        if release {
            command.arg("--release");
        }
        command.args(["-p", &project.entry_crate]);
        environment.apply(&mut command);
        run_in(command, root)?;
        // `jniLibs/` is an input to Gradle rather than an output it tracks, so an absent library
        // here is a successful build of the previous one. Checked before anything is assembled.
        let library = jni_libs
            .join(abi)
            .join(format!("lib{}.so", project.lib_name()));
        if !library.is_file() {
            return Err(format!(
                "{} was not written.\ncargo ndk reported success, so this is a naming \
                 disagreement: the entry crate `{}` should emit `lib{}.so`.",
                library.display(),
                project.entry_crate,
                project.lib_name()
            ));
        }
        println!("   {} ({} bytes)", library.display(), size(&library));
    }

    let out = project.project(root);
    let task = match release {
        true => "assembleRelease",
        false => "assembleDebug",
    };
    println!("\n== {task}");
    let mut command = Command::new(out.join(environment::executable(match cfg!(windows) {
        true => "gradlew.bat",
        false => "gradlew",
    })));
    command.arg(task);
    environment.apply(&mut command);
    run_in(command, &out)?;

    let variant = match release {
        true => "release",
        false => "debug",
    };
    let apk = out.join(format!("app/build/outputs/apk/{variant}/app-{variant}.apk"));
    match apk.is_file() {
        true => Ok(apk),
        false => Err(format!("{} was not assembled", apk.display())),
    }
}

/// Builds, installs and launches.
fn run(
    root: &Path,
    project: &Project,
    release: bool,
    abi: Option<&str>,
    device: Option<&str>,
    logcat: bool,
) -> Result<(), String> {
    let apk = build(root, project, release, abi)?;
    let environment = Environment::resolve(root, project).map_err(doctor_first)?;
    let adb = environment.adb();
    if !adb.is_file() {
        return Err(format!(
            "no adb at {} -- install it with `android sdk install platform-tools`, or transfer \
             {} to the device yourself.",
            adb.display(),
            apk.display()
        ));
    }
    let adb = |args: &[&str]| -> Result<(), String> {
        let mut command = Command::new(&adb);
        if let Some(device) = device {
            command.args(["-s", device]);
        }
        command.args(args);
        environment.apply(&mut command);
        run_in(command, root)
    };
    println!("\n== install");
    // Replacing rather than installing fresh, so a rerun does not need the app uninstalled first.
    adb(&["install", "-r", &apk.to_string_lossy()])?;
    println!("\n== launch");
    // The log is cleared first so what follows is this launch and not the last one's.
    adb(&["logcat", "-c"]).ok();
    adb(&[
        "shell",
        "am",
        "start",
        "-n",
        &format!("{}/.MainActivity", project.app_id),
    ])?;
    if logcat {
        println!("\n== logcat (ctrl-c to stop)");
        adb(&["logcat", "-v", "brief"])?;
    }
    Ok(())
}

/// The repo root: whichever ancestor holds `foliage-android.toml`, else the workspace, else here.
///
/// Taken from the directory rather than from an argument so that every command works from anywhere
/// inside the repo, and so that the paths handed to `cargo ndk` and Gradle are the same ones
/// whichever directory they were typed in.
fn root() -> Result<PathBuf, String> {
    let here = std::env::current_dir().map_err(|e| format!("no working directory: {e}"))?;
    for directory in here.ancestors() {
        if directory.join(project::FILE).is_file() {
            return Ok(directory.to_path_buf());
        }
    }
    for directory in here.ancestors() {
        let manifest = directory.join("Cargo.toml");
        if fs::read_to_string(&manifest).is_ok_and(|text| text.contains("[workspace]")) {
            return Ok(directory.to_path_buf());
        }
    }
    Ok(here)
}

/// How the entry crate should depend on `foliage`, taken from how the app already does.
///
/// Not a choice this tool has any business making. The app crate takes a [`Foliage`] and therefore
/// already depends on the engine -- as a git revision, a published version, a tag, a local path --
/// and the entry crate has to name the *same* one or the two halves of the program link against
/// different engines. Copying the app's own line is the only answer that is right by construction.
///
/// A relative path is the one part that cannot be copied verbatim: it is written relative to the
/// crate that declares it, and the entry crate sits somewhere else. So a `path` is resolved against
/// the app crate and rewritten against the entry crate, and everything else is carried across
/// untouched.
///
/// [`Foliage`]: https://docs.rs/foliage
fn dependency(app: &Path, entry: &Path, name: &str) -> Result<String, String> {
    let manifest = app.join("Cargo.toml");
    let spec = dependency_spec(&manifest, name).ok_or_else(|| {
        format!(
            "{} does not depend on `{name}`.\nthe entry crate has to name the same engine the app \
             does, and there is nothing here to copy -- add `{name}` to the app crate first.",
            manifest.display()
        )
    })?;
    let Some(path) = value(&spec, "path") else {
        return Ok(spec);
    };
    let absolute = normalize(&app.join(&path));
    let rewritten = relative(entry, &absolute).unwrap_or_else(|| absolute.display().to_string());
    Ok(spec.replace(&format!("\"{path}\""), &format!("\"{rewritten}\"")))
}

/// One dependency's spec, as it is written after the `=`.
///
/// Reads `[dependencies]` and its target-specific variants, since an engine an app only uses on one
/// platform is still the engine the entry crate has to match. The first match wins, which is the
/// plain `[dependencies]` entry wherever there is one.
fn dependency_spec(manifest: &Path, name: &str) -> Option<String> {
    let text = fs::read_to_string(manifest).ok()?;
    let mut inside = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line.ends_with("dependencies]");
            continue;
        }
        if !inside || line.starts_with('#') {
            continue;
        }
        if let Some(spec) = line
            .strip_prefix(name)
            .and_then(|rest| rest.trim().strip_prefix('='))
        {
            return Some(spec.trim().to_string());
        }
    }
    None
}

/// One `key = "value"` out of an inline table.
fn value(spec: &str, key: &str) -> Option<String> {
    let at = spec.find(key)?;
    let rest = spec[at + key.len()..].trim_start().strip_prefix('=')?;
    let rest = rest.trim_start().strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// A path with `.` and `..` resolved textually.
///
/// Not [`std::fs::canonicalize`]: this runs before the entry crate exists, and a path that cannot
/// be resolved because nothing is there yet is exactly the case that has to work.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            part => out.push(part),
        }
    }
    out
}

/// `to` as reached from `from`, in the `../` form a manifest writes.
fn relative(from: &Path, to: &Path) -> Option<String> {
    let from = normalize(from);
    let shared = from
        .components()
        .zip(to.components())
        .take_while(|(a, b)| a == b)
        .count();
    let up = from.components().count() - shared;
    let down: Vec<_> = to
        .components()
        .skip(shared)
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect();
    let path = std::iter::repeat_n("..".to_string(), up)
        .chain(down)
        .collect::<Vec<_>>()
        .join("/");
    match path.is_empty() {
        true => None,
        false => Some(path),
    }
}

/// Where a workspace member lives, by package name.
///
/// The conventional directory first, then every member in turn -- a crate is usually in a directory
/// named after it and is not required to be.
fn crate_dir(root: &Path, name: &str) -> Option<PathBuf> {
    let conventional = root.join(name);
    if package_name(&conventional.join("Cargo.toml")).as_deref() == Some(name) {
        return Some(conventional);
    }
    let text = fs::read_to_string(root.join("Cargo.toml")).ok()?;
    let members = text.find("members")?;
    let close = text[members..].find(']')? + members;
    text[members..close]
        .split(',')
        .map(|member| root.join(member.trim().trim_matches(|c| c == '"' || c == '[')))
        .find(|member| package_name(&member.join("Cargo.toml")).as_deref() == Some(name))
}

/// The `[package] name` in a manifest, if it has one.
///
/// Enough of a TOML reader for one key that is conventionally on its own line directly under
/// `[package]`. A workspace manifest has no `[package]`, which is the case this has to tell apart
/// and does by finding nothing.
fn package_name(manifest: &Path) -> Option<String> {
    let text = fs::read_to_string(manifest).ok()?;
    let mut lines = text.lines().skip_while(|line| line.trim() != "[package]");
    lines.next()?;
    lines
        .take_while(|line| !line.trim_start().starts_with('['))
        .find_map(|line| {
            let value = line.trim().strip_prefix("name")?.trim().strip_prefix('=')?;
            Some(value.trim().trim_matches('"').to_string())
        })
}

/// Adds the entry crate to the workspace, which is the step cargo gives no warning for.
///
/// A crate that is not a member is not built by `-p`, and the error names the package rather than
/// the omission. Nothing happens when there is no workspace, or when it is already a member, or
/// when `members` is not a list this can extend -- in which case it is reported for you to add.
fn member(root: &Path, entry_crate: &str) -> Result<(), String> {
    let manifest = root.join("Cargo.toml");
    let Ok(text) = fs::read_to_string(&manifest) else {
        return Ok(());
    };
    if !text.contains("[workspace]") || text.contains(&format!("\"{entry_crate}\"")) {
        return Ok(());
    }
    let Some(members) = text.find("members") else {
        return Ok(());
    };
    let Some(close) = text[members..].find(']').map(|end| members + end) else {
        println!(
            "note: add \"{entry_crate}\" to `members` in {} -- cargo will not build it otherwise.",
            manifest.display()
        );
        return Ok(());
    };
    let separator = match text[members..close].trim_end().ends_with('[') {
        true => "",
        false => ", ",
    };
    let mut updated = text.clone();
    updated.insert_str(close, &format!("{separator}\"{entry_crate}\""));
    fs::write(&manifest, updated).map_err(|e| format!("writing {}: {e}", manifest.display()))?;
    println!(
        "added \"{entry_crate}\" to members in {}",
        manifest.display()
    );
    Ok(())
}

/// Points a missing toolchain at the command that reports the whole of what is missing.
///
/// Only for the commands that need it to be there. `doctor` is that report, and telling it to run
/// itself would say nothing.
fn doctor_first(reason: String) -> String {
    format!("{reason}\nrun `foliage-android doctor` for everything that is missing.")
}

/// Keeps generated output and the SDK out of the repository.
///
/// Neither is source. The Gradle project is written in full by `init` and the SDK is installed in
/// full by `setup`, so a fresh clone reaches the same state by running them -- which is a smaller
/// thing to carry than a Gradle project and a wrapper binary in the history, and one that cannot go
/// stale against the tool that generates it.
///
/// Entries are appended and only when absent, so a `.gitignore` with its own content keeps it and a
/// rerun adds nothing. Nothing happens where there is no `.gitignore` and no repository to want one.
fn ignore(root: &Path, project: &Project) -> Result<(), String> {
    let path = root.join(".gitignore");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    if !path.is_file() && !root.join(".git").exists() {
        return Ok(());
    }
    let named = |entry: &str| {
        existing.lines().any(|line| {
            line.trim().trim_start_matches('/').trim_end_matches('/') == entry.trim_matches('/')
        })
    };
    let wanted = [
        (
            project.project.as_str(),
            "The Gradle project, written in full by `foliage-android init`.",
        ),
        (
            project.sdk.as_str(),
            "The Android SDK, installed by `foliage-android setup`.",
        ),
    ];
    let addition: String = wanted
        .iter()
        .filter(|(entry, _)| !named(entry))
        .map(|(entry, why)| format!("# {why}\n/{entry}/\n"))
        .collect();
    if addition.is_empty() {
        return Ok(());
    }
    let separator = match existing.is_empty() || existing.ends_with("\n\n") {
        true => "",
        false => "\n",
    };
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("opening {}: {e}", path.display()))?;
    file.write_all(format!("{separator}{addition}").as_bytes())
        .map_err(|e| format!("writing {}: {e}", path.display()))?;
    println!("updated {}", path.display());
    Ok(())
}

/// Runs a command, reporting a missing one as the install that fixes it.
fn run_in(mut command: Command, cwd: &Path) -> Result<(), String> {
    let program = command.get_program().to_string_lossy().to_string();
    let status = command
        .current_dir(cwd)
        .status()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => match program.as_str() {
                "cargo" => "no cargo on PATH".to_string(),
                _ => format!("{program} not found"),
            },
            _ => format!("running {program}: {e}"),
        })?;
    match status.success() {
        true => Ok(()),
        false => Err(format!("{program} failed")),
    }
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    fs::write(path, contents).map_err(|e| format!("writing {}: {e}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

fn size(path: &Path) -> u64 {
    fs::metadata(path).map(|data| data.len()).unwrap_or(0)
}

fn download(url: &str, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    let bytes = reqwest::blocking::get(url)
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.bytes())
        .map_err(|e| format!("fetching {url}: {e}"))?;
    let mut file =
        fs::File::create(dest).map_err(|e| format!("creating {}: {e}", dest.display()))?;
    file.write_all(&bytes)
        .map_err(|e| format!("writing {}: {e}", dest.display()))?;
    #[cfg(unix)]
    if dest.file_name().is_some_and(|name| name == "gradlew") {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(dest)
            .map_err(|e| format!("reading {}: {e}", dest.display()))?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(dest, permissions)
            .map_err(|e| format!("setting {} executable: {e}", dest.display()))?;
    }
    println!("downloaded {}", dest.display());
    Ok(())
}
