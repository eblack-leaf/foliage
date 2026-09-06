//! The SDK, and saying what is missing.
//!
//! Android's toolchain is normally reached through `ANDROID_HOME` and `ANDROID_NDK_HOME`, which
//! means a build depends on how the shell that started it was prepared. Nothing here reads either.
//! The SDK is the path `foliage-android.toml` names and the NDK is the version it names inside that
//! -- so the answer is the same from any shell, and changing it is an edit rather than an export.
//!
//! The variables are still *written*, onto the processes this tool starts, because `cargo ndk` and
//! Gradle read them and there is no other way to tell them. That is the opposite direction: the
//! environment is constructed here rather than consulted. Nothing is exported and nothing outlives
//! the command.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::project::Project;

/// The SDK and the NDK inside it.
pub struct Environment {
    /// What the children are told `ANDROID_HOME` is.
    pub sdk: PathBuf,
    /// The NDK named by `foliage-android.toml`, inside the SDK.
    pub ndk: PathBuf,
}

impl Environment {
    /// The SDK the project names, and the NDK in it.
    ///
    /// One path, from one file. There is no search and no fallback: a missing SDK is reported as
    /// the directory that is not there, rather than as a list of places that were tried.
    pub fn resolve(root: &Path, project: &Project) -> Result<Self, String> {
        let sdk = Self::sdk(root, project);
        if !sdk.is_dir() {
            return Err(format!(
                "no Android SDK at {} (`sdk` in {}).",
                sdk.display(),
                crate::project::FILE
            ));
        }
        let ndk = sdk.join("ndk").join(&project.ndk);
        if !ndk.is_dir() {
            return Err(format!(
                "no NDK at {} (`ndk` in {}).",
                ndk.display(),
                crate::project::FILE
            ));
        }
        Ok(Self { sdk, ndk })
    }

    /// Where the SDK is, whether or not anything is there yet.
    ///
    /// What `setup` installs into and what every other command builds against, so the two are the
    /// same path by construction. Relative to the repo root unless the project names an absolute
    /// one, which is how an SDK shared between repos is spelled.
    pub fn sdk(root: &Path, project: &Project) -> PathBuf {
        root.join(&project.sdk)
    }

    /// Puts the SDK on a command, the way a shell that had been set up would have.
    ///
    /// `PATH` gains `platform-tools` so `adb` resolves, and the SDK's own `bin` so `android` does.
    pub fn apply(&self, command: &mut Command) {
        command.env("ANDROID_HOME", &self.sdk);
        command.env("ANDROID_SDK_ROOT", &self.sdk);
        command.env("ANDROID_NDK_HOME", &self.ndk);
        let mut paths = vec![self.sdk.join("bin"), self.sdk.join("platform-tools")];
        if let Some(existing) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&existing));
        }
        if let Ok(path) = std::env::join_paths(paths) {
            command.env("PATH", path);
        }
    }

    /// `adb`, which lives in the SDK rather than on anyone's `PATH` by default.
    pub fn adb(&self) -> PathBuf {
        self.sdk.join("platform-tools").join(executable("adb"))
    }
}

/// Reports what is installed and what is not, with the command that fixes each.
///
/// Every check is one thing that stops a build, and each is reported rather than the first one
/// aborting -- a fresh machine is missing several, and finding them one command at a time is the
/// experience this exists to replace.
pub fn doctor(root: &Path, project: &Project) -> Result<(), String> {
    let mut checks = Checks::default();

    checks.check(
        runs("java", &["-version"]),
        "a JDK",
        "install one: `sudo apt install default-jdk`, or https://adoptium.net",
    );

    // Each package is asked about on its own rather than behind a resolved [`Environment`]. They are
    // installed by one command and fail separately -- a dropped NDK download leaves a perfectly good
    // SDK -- so a single answer for all of them would report three installed packages as missing.
    let sdk = Environment::sdk(root, project);
    if !sdk.is_dir() {
        checks.check(false, &format!("no SDK at {}", sdk.display()), INSTALL);
        return checks.finish();
    }
    checks.have(&format!("SDK at {}", sdk.display()));
    checks.check(
        sdk.join("platforms")
            .join(format!("android-{}", project.compile_sdk))
            .is_dir(),
        &format!("platforms/android-{}", project.compile_sdk),
        "run `foliage-android setup`",
    );
    checks.check(
        sdk.join("platform-tools").join(executable("adb")).is_file(),
        "platform-tools (adb)",
        "run `foliage-android setup`",
    );
    checks.check(
        sdk.join("ndk").join(&project.ndk).is_dir(),
        &format!("ndk/{}", project.ndk),
        "run `foliage-android setup`",
    );

    let targets = rustup_targets();
    for abi in &project.abis {
        let target = rust_target(abi)?;
        checks.check(
            targets.iter().any(|installed| installed == target),
            &format!("rustup target {target} (for {abi})"),
            "run `foliage-android setup`",
        );
    }
    checks.check(have_cargo_ndk(), "cargo-ndk", "run `foliage-android setup`");

    // The one mismatch that cannot be found by building: it aborts at launch, before any Rust runs,
    // with a `RegisterNatives` failure naming a Java method rather than a version.
    match vendored_games_activity(root) {
        Ok(Some(vendored)) => checks.check(
            vendored == project.games_activity,
            &format!(
                "games-activity {} matches the compiled GameActivity",
                project.games_activity
            ),
            &format!(
                "the compiled GameActivity is {vendored}. Set `games-activity = \"{vendored}\"` in \
                 foliage-android.toml and rerun `init`.\n          Prefer updating the lock over \
                 pinning an older AAR: a GameActivity years behind the running platform can \
                 deliver no touch input at all, silently."
            ),
        ),
        Ok(None) => checks.skip("games-activity", "no Cargo.lock yet -- build once"),
        Err(reason) => checks.skip("games-activity", &reason),
    }

    checks.finish()
}

/// What was found, and what was not.
///
/// Every check reports and only the total decides the outcome: a fresh machine is missing several,
/// and finding them one aborted command at a time is the experience this replaces.
#[derive(Default)]
struct Checks {
    missing: u32,
}

impl Checks {
    fn check(&mut self, ok: bool, label: &str, fix: &str) {
        match ok {
            true => self.have(label),
            false => {
                self.missing += 1;
                println!("  MISSING {label}\n          {fix}");
            }
        }
    }

    fn have(&self, label: &str) {
        println!("  ok      {label}");
    }

    /// Something that could not be checked, which is not the same as something missing.
    fn skip(&self, label: &str, why: &str) {
        println!("  --      {label} unchecked ({why})");
    }

    fn finish(self) -> Result<(), String> {
        match self.missing {
            0 => {
                println!("\neverything needed is installed.");
                Ok(())
            }
            missing => Err(format!("{missing} missing -- see above.")),
        }
    }
}

/// What to do about a missing SDK, which is one command rather than a section of a README.
const INSTALL: &str = "run `foliage-android setup` -- it installs the `android` CLI and every\n\
     \x20         package into that directory, and touches nothing outside it.";

/// The rust target one Android ABI is built for.
pub fn rust_target(abi: &str) -> Result<&'static str, String> {
    match abi {
        "arm64-v8a" => Ok("aarch64-linux-android"),
        "armeabi-v7a" => Ok("armv7-linux-androideabi"),
        "x86_64" => Ok("x86_64-linux-android"),
        "x86" => Ok("i686-linux-android"),
        _ => Err(format!(
            "unknown abi `{abi}` -- one of arm64-v8a, armeabi-v7a, x86_64, x86"
        )),
    }
}

/// The ABI an emulator on this machine runs, since an emulator runs the host's architecture.
pub fn host_abi() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64-v8a",
        _ => "x86_64",
    }
}

/// The GameActivity version compiled into the app, read from the `android-activity` the lock
/// resolved.
///
/// The AAR Gradle links and this C++ register one JNI table between them, in one call -- so a
/// single changed signature fails `RegisterNatives` and aborts the process at launch. Reading it
/// here is the same check the header would give by hand, made before a device is involved.
///
/// `None` where there is no lock yet. An error where the version is known but its source has not
/// been fetched, which is not a fault worth failing over.
fn vendored_games_activity(root: &Path) -> Result<Option<String>, String> {
    let Ok(lock) = std::fs::read_to_string(root.join("Cargo.lock")) else {
        return Ok(None);
    };
    let mut version = None;
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "name = \"android-activity\"" {
            version = lines
                .next()
                .and_then(|next| next.trim().strip_prefix("version = \""))
                .and_then(|next| next.strip_suffix('"'))
                .map(str::to_string);
            break;
        }
    }
    let Some(version) = version else {
        return Ok(None);
    };
    let source = cargo_home()
        .ok_or("no CARGO_HOME")?
        .join("registry/src")
        .read_dir()
        .map_err(|e| format!("reading the registry: {e}"))?
        .filter_map(Result::ok)
        .map(|registry| registry.path().join(format!("android-activity-{version}")))
        .find(|path| path.is_dir())
        .ok_or_else(|| format!("android-activity {version} is not unpacked yet"))?;
    let header = source.join(
        "android-games-sdk/game-activity/prefab-src/modules/game-activity/include/game-activity/GameActivity.h",
    );
    let header = std::fs::read_to_string(&header)
        .map_err(|e| format!("reading {}: {e}", header.display()))?;
    let define = |name: &str| {
        header
            .lines()
            .find_map(|line| line.trim().strip_prefix(&format!("#define {name} "))?.trim().parse::<u32>().ok())
            .ok_or_else(|| format!("no {name} in GameActivity.h"))
    };
    Ok(Some(format!(
        "{}.{}.{}",
        define("GAMEACTIVITY_MAJOR_VERSION")?,
        define("GAMEACTIVITY_MINOR_VERSION")?,
        define("GAMEACTIVITY_BUGFIX_VERSION")?
    )))
}

/// Whether a program is installed, by running it and seeing whether it was found.
///
/// The exit status is not read: `java -version` and `cargo ndk --version` both answer, and a
/// non-zero status from one that ran still means it is there.
fn runs(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// `<os>_<arch>` for this machine, as `dl.google.com/android/cli/latest/<host>/android` names it.
pub fn host() -> String {
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

/// Whether `cargo ndk` is installed.
pub fn have_cargo_ndk() -> bool {
    runs("cargo", &["ndk", "--version"])
}

/// Every rust target installed, as `rustup` lists them.
pub fn rustup_targets() -> Vec<String> {
    let Ok(output) = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .collect()
}


/// A program's file name on this platform.
pub fn executable(name: &str) -> String {
    match cfg!(windows) {
        true => format!("{name}.exe"),
        false => name.to_string(),
    }
}


/// Where cargo unpacks what it downloads.
///
/// The one location this does read from the environment, because it is cargo's own and there is no
/// other way to ask. Only the GameActivity version check reads it, and that check reports itself as
/// unchecked rather than failing when nothing is found.
fn cargo_home() -> Option<PathBuf> {
    std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(|home| PathBuf::from(home).join(".cargo"))
        })
}
