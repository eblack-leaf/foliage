//! Finding the SDK, and saying what is missing.
//!
//! Android's toolchain is reached entirely through environment variables, and setting them by hand
//! in every shell is the step most easily forgotten -- `cargo ndk` without `ANDROID_NDK_HOME` and
//! Gradle without `ANDROID_HOME` both fail, at different points, with different messages.
//!
//! So they are resolved here and set on the processes this tool starts, rather than expected from
//! the shell it was started in. Nothing is exported and nothing outlives the command.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::project::Project;

/// Where an SDK is kept when it belongs to one repo rather than to the machine.
///
/// Checked after the environment and before the user-wide location, so a repo carrying its own SDK
/// is found without being pointed at, and a machine-wide one still wins when it is asked for.
pub const LOCAL: &str = ".android-sdk";

/// The SDK and the NDK inside it, once both have been found.
pub struct Environment {
    /// What `ANDROID_HOME` names.
    pub sdk: PathBuf,
    /// One NDK inside it. Any reasonably recent one works -- `cargo ndk` does not care which -- so
    /// the newest installed is taken unless `ANDROID_NDK_HOME` says otherwise.
    pub ndk: PathBuf,
}

impl Environment {
    /// Finds the SDK and an NDK in it.
    ///
    /// In order: what the environment already says, an SDK kept beside the repo, then the location
    /// Android's own tools default to. The first that exists wins.
    pub fn resolve(root: &Path) -> Result<Self, String> {
        let sdk = ["ANDROID_HOME", "ANDROID_SDK_ROOT"]
            .iter()
            .filter_map(|name| std::env::var_os(name).map(PathBuf::from))
            .chain(std::iter::once(root.join(LOCAL)))
            .chain(home().map(|home| home.join("Android/Sdk")))
            .find(|path| path.is_dir())
            .ok_or_else(|| {
                format!(
                    "no Android SDK -- looked at $ANDROID_HOME, $ANDROID_SDK_ROOT, {} and \
                     ~/Android/Sdk.",
                    root.join(LOCAL).display()
                )
            })?;
        let ndk = match std::env::var_os("ANDROID_NDK_HOME") {
            Some(ndk) => PathBuf::from(ndk),
            None => newest_ndk(&sdk).ok_or_else(|| {
                format!(
                    "no NDK in {} -- install one with `android sdk install ndk/<version>`, or set \
                     $ANDROID_NDK_HOME.",
                    sdk.join("ndk").display()
                )
            })?,
        };
        if !ndk.is_dir() {
            return Err(format!("no NDK at {}", ndk.display()));
        }
        Ok(Self { sdk, ndk })
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

    match Environment::resolve(root) {
        Ok(environment) => {
            checks.have(&format!("SDK at {}", environment.sdk.display()));
            checks.have(&format!("NDK at {}", environment.ndk.display()));
            checks.check(
                environment
                    .sdk
                    .join("platforms")
                    .join(format!("android-{}", project.compile_sdk))
                    .is_dir(),
                &format!("platforms/android-{}", project.compile_sdk),
                "run `foliage-android setup`",
            );
            checks.check(
                environment
                    .sdk
                    .join("build-tools")
                    .read_dir()
                    .is_ok_and(|mut entries| entries.next().is_some()),
                "build-tools",
                "run `foliage-android setup`",
            );
            checks.check(
                environment.adb().is_file(),
                "platform-tools (adb)",
                "run `foliage-android setup`",
            );
        }
        Err(reason) => checks.check(false, "an Android SDK", &format!("{reason}\n          {INSTALL}")),
    }

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
     \x20         package into .android-sdk/ beside the repo, and touches nothing else.";

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

/// The highest-numbered NDK installed, by major version.
///
/// Compared as numbers rather than as strings, because `9` sorts after `27` as text and the two
/// have coexisted.
fn newest_ndk(sdk: &Path) -> Option<PathBuf> {
    let mut versions: Vec<_> = sdk
        .join("ndk")
        .read_dir()
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            let major: u32 = name.split('.').next()?.parse().ok()?;
            Some((major, entry.path()))
        })
        .collect();
    versions.sort_by_key(|(major, _)| *major);
    versions.pop().map(|(_, path)| path)
}

/// A program's file name on this platform.
pub fn executable(name: &str) -> String {
    match cfg!(windows) {
        true => format!("{name}.exe"),
        false => name.to_string(),
    }
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn cargo_home() -> Option<PathBuf> {
    std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| home().map(|home| home.join(".cargo")))
}
