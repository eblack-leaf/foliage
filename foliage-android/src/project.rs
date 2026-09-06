//! What `init` decided, written down so nothing has to be told again.
//!
//! Every value here is either an identity that must not drift between builds -- the application id
//! a device installs under, the library name the activity loads -- or a version that has to agree
//! with something else. Passing them per invocation is how they drift, so they are settled once and
//! read from `foliage-android.toml` by every command afterwards.
//!
//! The file is meant to be edited. It is read back by [`Project::load`] and rewritten in full by
//! [`Project::save`], so a changed SDK level survives a rerun of `init` and takes effect in the
//! generated project.

use std::fs;
use std::path::{Path, PathBuf};

/// What the file is called, at the root of the repo it describes.
pub const FILE: &str = "foliage-android.toml";

/// Everything the generated project and the builds that fill it are described by.
pub struct Project {
    /// Reverse-domain application id. The identity a device installs under, and permanent: two
    /// builds with different ids are two applications, and an update has to keep the id it shipped.
    pub app_id: String,
    /// The label shown under the launcher icon.
    pub app_name: String,
    /// The crate holding the entry function, which is the app itself.
    pub app_crate: String,
    /// The `cdylib` crate the activity loads, which is generated and holds only `android_main`.
    pub entry_crate: String,
    /// The generated Gradle project, relative to the repo root.
    pub project: String,
    /// The Android SDK, relative to the repo root.
    ///
    /// The only place one is looked for. No environment variable is read and no user-wide location
    /// is tried: what is written here is what `setup` installs into and what every build uses, so
    /// the two cannot disagree and neither depends on how a shell was prepared.
    ///
    /// An absolute path works, for an SDK shared between repos.
    pub sdk: String,
    /// Which ABIs are built. `arm64-v8a` covers essentially every phone made since 2017; an
    /// emulator wants the host's, which is what `init` adds when it recognises one.
    pub abis: Vec<String>,
    /// The device gate: anything below this refuses to install.
    pub min_sdk: u32,
    /// Which API the compiler sees. Needs a matching `platforms/android-<n>` installed.
    pub compile_sdk: u32,
    /// Which platform behaviours the app opts into on newer devices. Not an install gate.
    pub target_sdk: u32,
    /// `androidx.games:games-activity`, the Java half of the backend winit talks to.
    ///
    /// It has to match the GameActivity C++ compiled into the app, which comes from the
    /// `android-activity` that `foliage`'s winit resolved. A mismatch aborts at launch, before any
    /// Rust runs, so `doctor` checks it rather than leaving it to be discovered on a device.
    pub games_activity: String,
    /// Android Gradle Plugin version.
    pub agp: String,
    /// Gradle version, which the wrapper fetches on first build.
    pub gradle: String,
    /// The NDK `setup` installs and `doctor` expects.
    ///
    /// A known-good version rather than a requirement: `cargo ndk` works against any reasonably
    /// recent one. It is pinned only because `android sdk install` takes an exact version and there
    /// is no spelling for "the current one".
    pub ndk: String,
}

/// The `androidx.games:games-activity` release matching `android-activity 0.6`, which is what winit
/// 0.30 depends on.
pub const GAMES_ACTIVITY: &str = "4.4.0";
/// Android Gradle Plugin.
pub const AGP: &str = "9.1.0";
/// Gradle itself.
pub const GRADLE: &str = "9.3.1";
/// The device gate. Android 8, which is where the surface behaviour winit relies on settles.
pub const MIN_SDK: u32 = 26;
/// What the compiler sees, and what `platforms/android-<n>` has to be installed for.
pub const COMPILE_SDK: u32 = 35;
/// What the app opts into.
pub const TARGET_SDK: u32 = 35;
/// Where the generated Gradle project goes, beside the crate it belongs to.
pub const PROJECT: &str = "android";
/// A known-good NDK. Any reasonably recent one works; this is the one `setup` installs.
pub const NDK: &str = "27.3.13750724";
/// Where the SDK goes unless [`Project::sdk`] says otherwise.
///
/// Beside the repo, so it is contained and `rm -rf` undoes it. A default rather than a convention --
/// Android's own tooling has no notion of a per-repo SDK, which is the reason this is written into
/// the file rather than assumed.
pub const SDK: &str = ".android-sdk";

impl Project {
    /// The library the activity loads, as `System.loadLibrary` names it.
    ///
    /// Not stored, because it is not a choice: a `cdylib` crate emits `lib<name>.so` where `<name>`
    /// is the crate name with dashes turned into underscores, and the manifest has to name that
    /// same string. Deriving it is what keeps the two from disagreeing.
    pub fn lib_name(&self) -> String {
        self.entry_crate.replace('-', "_")
    }

    /// Where the generated Gradle project sits.
    pub fn project(&self, root: &Path) -> PathBuf {
        root.join(&self.project)
    }

    /// Where per-ABI libraries are packaged from.
    ///
    /// An *input* to Gradle rather than an output it tracks: whatever is sitting here is what gets
    /// packaged, so a build that failed or wrote elsewhere ships the previous library instead of
    /// failing. That is why every path this tool hands to `cargo ndk` is absolute.
    pub fn jni_libs(&self, root: &Path) -> PathBuf {
        self.project(root).join("app/src/main/jniLibs")
    }

    /// Reads the file at the root of `root`.
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = root.join(FILE);
        let text = fs::read_to_string(&path).map_err(|e| {
            format!(
                "reading {}: {e}\nrun `foliage-android init --app-id <id>` first",
                path.display()
            )
        })?;
        let fields = parse(&text, &path)?;
        let string = |key: &str| -> Result<String, String> {
            fields
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| format!("{}: no `{key}`", path.display()))
        };
        let number = |key: &str| -> Result<u32, String> {
            let value = string(key)?;
            value
                .parse()
                .map_err(|_| format!("{}: `{key}` is not a number: {value}", path.display()))
        };
        Ok(Self {
            app_id: string("app-id")?,
            app_name: string("app-name")?,
            app_crate: string("app-crate")?,
            entry_crate: string("entry-crate")?,
            project: string("project")?,
            sdk: string("sdk")?,
            abis: string("abis")?
                .split(',')
                .map(|abi| abi.trim().to_string())
                .filter(|abi| !abi.is_empty())
                .collect(),
            min_sdk: number("min-sdk")?,
            compile_sdk: number("compile-sdk")?,
            target_sdk: number("target-sdk")?,
            games_activity: string("games-activity")?,
            agp: string("agp")?,
            gradle: string("gradle")?,
            ndk: string("ndk")?,
        })
    }

    /// Writes the file, replacing whatever was there.
    pub fn save(&self, root: &Path) -> Result<(), String> {
        let abis = self
            .abis
            .iter()
            .map(|abi| format!("\"{abi}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let Self {
            app_id,
            app_name,
            app_crate,
            entry_crate,
            project,
            sdk,
            min_sdk,
            compile_sdk,
            target_sdk,
            games_activity,
            agp,
            gradle,
            ndk,
            ..
        } = self;
        let text = format!(
            "\
# Written by `foliage-android init` and read by every other command. Edit it and rerun `init` to
# regenerate the Gradle project against the change.

# The identity a device installs under. Permanent: an update has to keep the id it shipped.
app-id = \"{app_id}\"
# The label under the launcher icon.
app-name = \"{app_name}\"

# The crate holding the entry function, and the generated cdylib the activity loads.
app-crate = \"{app_crate}\"
entry-crate = \"{entry_crate}\"
# Where the generated Gradle project lives, relative to this file.
project = \"{project}\"
# The Android SDK. The only place one is looked for -- no environment variable is read, so `setup`
# and every build always mean the same directory. Absolute paths work, for an SDK shared elsewhere.
sdk = \"{sdk}\"

# Which ABIs `build` compiles. Each needs its rustup target -- `doctor` says which are missing.
abis = [{abis}]

# The device gate, what the compiler sees, and what the app opts into. Only compile-sdk needs a
# matching `platforms/android-<n>` installed.
min-sdk = {min_sdk}
compile-sdk = {compile_sdk}
target-sdk = {target_sdk}

# The Java half of the backend winit talks to. Has to match the GameActivity compiled into the app,
# which comes from the `android-activity` winit resolved -- `doctor` checks the two agree.
games-activity = \"{games_activity}\"
agp = \"{agp}\"
gradle = \"{gradle}\"

# The NDK `setup` installs. Known-good rather than required -- cargo-ndk works against any
# reasonably recent one -- but `android sdk install` takes an exact version, so it is named here.
# Build-tools are deliberately absent: the Android Gradle Plugin resolves and installs the version
# it needs itself, so naming one here would only pin something nothing reads.
ndk = \"{ndk}\"
"
        );
        let path = root.join(FILE);
        fs::write(&path, text).map_err(|e| format!("writing {}: {e}", path.display()))?;
        println!("wrote {}", path.display());
        Ok(())
    }
}

/// Every `key = value` in the file, with quotes and brackets taken off and comments dropped.
///
/// Deliberately not a TOML parser. The file has one flat table and this tool wrote it, so what a
/// line may look like is known -- and anything else is reported rather than skipped, because a
/// misread key here would silently build something other than what was asked for.
fn parse(text: &str, path: &Path) -> Result<Vec<(String, String)>, String> {
    let mut fields = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "{}:{}: not a `key = value` line: {line}",
                path.display(),
                number + 1
            ));
        };
        let value = value.trim();
        // A list becomes one comma-separated string. The only list here is the ABIs, and every
        // reader of it wants the names rather than the syntax they were written in.
        let value = match value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
            Some(list) => list.replace('"', ""),
            None => value.trim_matches('"').to_string(),
        };
        fields.push((key.trim().to_string(), value));
    }
    Ok(fields)
}
