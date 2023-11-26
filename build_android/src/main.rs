use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::Deserialize;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = prepare_from_file(args);
    build_template(&args);
    build_android(args);
}

struct Args {
    package: OsString,
    arch: OsString,
    ndk_home: PathBuf,
    sdk_home: PathBuf,
    working_directory: PathBuf,
    jni_output: PathBuf,
    app_source: PathBuf,
    min_sdk: String,
    target_sdk: String,
    compile_sdk: String,
    android_application_version: String,
    gradle_distribution_url: String,
    ndk_version: String,
    androidx_version: String,
    androidx_constraintlayout_version: String,
    androidx_games_activity_version: String,
    androidx_fragment_version: String,
    oboe_version: String,
}

#[derive(Deserialize)]
struct InputArgs {
    package: String,
    arch: String,
    ndk_home: String,
    sdk_home: String,
    min_sdk: u32,
    target_sdk: u32,
    compile_sdk: u32,
    android_application_version: String,
    gradle_distribution_url: String,
    ndk_version: String,
    androidx_version: String,
    androidx_constraintlayout_version: String,
    androidx_games_activity_version: String,
    androidx_fragment_version: String,
    oboe_version: String,
}

fn prepare_from_file(args: Vec<String>) -> Args {
    if let Some(filename) = args.get(1) {
        let absolute_filename = Path::new(filename).canonicalize().unwrap();
        if let Ok(file) = std::fs::read_to_string(absolute_filename) {
            let input: InputArgs = toml::from_str(file.as_str()).unwrap();
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .canonicalize()
                .unwrap();
            let app_source = root.join("app_src");
            let jni_output = app_source
                .join("app")
                .join("src")
                .join("main")
                .join("jniLibs");
            return Args {
                package: OsString::from(input.package),
                arch: OsString::from(input.arch),
                ndk_home: Path::new(input.ndk_home.as_str()).canonicalize().unwrap(),
                sdk_home: Path::new(input.sdk_home.as_str()).canonicalize().unwrap(),
                working_directory: root,
                jni_output,
                app_source,
                min_sdk: input.min_sdk.to_string(),
                target_sdk: input.target_sdk.to_string(),
                compile_sdk: input.compile_sdk.to_string(),
                android_application_version: input.android_application_version,
                gradle_distribution_url: input.gradle_distribution_url,
                ndk_version: input.ndk_version,
                androidx_version: input.androidx_version,
                androidx_constraintlayout_version: input.androidx_constraintlayout_version,
                androidx_games_activity_version: input.androidx_games_activity_version,
                androidx_fragment_version: input.androidx_fragment_version,
                oboe_version: input.oboe_version,
            };
        }
    }
    panic!("could not prepare args");
}

fn build_template(args: &Args) {
    let manifest_template = args
        .working_directory
        .join("template")
        .join("AndroidManifest.xml");
    let manifest = std::fs::read_to_string(manifest_template)
        .unwrap()
        .replace("{{package-name}}", args.package.to_str().unwrap())
        .replace("{{package-label}}", "foliage");
    let build_template = args.working_directory.join("template").join("build.gradle");
    let build = std::fs::read_to_string(build_template)
        .unwrap()
        .replace("{{compile-sdk}}", args.compile_sdk.as_str())
        .replace("{{min-sdk}}", args.min_sdk.as_str())
        .replace("{{target-sdk}}", args.target_sdk.as_str())
        .replace("{{androidx-version}}", args.androidx_version.as_str())
        .replace(
            "{{androidx-constraintlayout-version}}",
            args.androidx_constraintlayout_version.as_str(),
        )
        .replace(
            "{{androidx-fragment-version}}",
            args.androidx_fragment_version.as_str(),
        )
        .replace("{{oboe-version}}", args.oboe_version.as_str())
        .replace(
            "{{androidx-games-activity-version}}",
            args.androidx_games_activity_version.as_str(),
        )
        .replace("{{ndk-version}}", args.ndk_version.as_str());
    let activity_template = args
        .working_directory
        .join("template")
        .join("MainActivity.java");
    let activity = std::fs::read_to_string(activity_template)
        .unwrap()
        .replace("{{package-name}}", args.package.to_str().unwrap());
    let activity_dest = args
        .app_source
        .join("app")
        .join("src")
        .join("main")
        .join("java")
        .join("co")
        .join("foliage")
        .join("app")
        .join("MainActivity.java");
    let build_dest = args.app_source.join("app").join("build.gradle");
    let manifest_dest = args
        .app_source
        .join("app")
        .join("src")
        .join("main")
        .join("AndroidManifest.xml");
    std::fs::write(activity_dest, activity).unwrap();
    std::fs::write(build_dest, build).unwrap();
    std::fs::write(manifest_dest, manifest).unwrap();
    let toplevel_template = args
        .working_directory
        .join("template")
        .join("toplevel.build.gradle");
    let toplevel = std::fs::read_to_string(toplevel_template).unwrap().replace(
        "{{android-application-version}}",
        args.android_application_version.as_str(),
    );
    let toplevel_dest = args.app_source.join("build.gradle");
    std::fs::write(toplevel_dest, toplevel).unwrap();
    let gradle_wrapper_template = args
        .working_directory
        .join("template")
        .join("gradle-wrapper.properties");
    let gradle_wrapper = std::fs::read_to_string(gradle_wrapper_template)
        .unwrap()
        .replace(
            "{{gradle-distribution-url}}",
            args.gradle_distribution_url.as_str(),
        );
    let gradle_wrapper_dest = args
        .app_source
        .join("gradle")
        .join("wrapper")
        .join("gradle-wrapper.properties");
    std::fs::write(gradle_wrapper_dest, gradle_wrapper).unwrap();
}

fn build_android(args: Args) {
    let install = std::process::Command::new(env!("CARGO"))
        .args(["install", "cargo-ndk"])
        .status()
        .unwrap();
    if !install.success() {
        println!("error installing cargo-ndk");
        return;
    }
    let process = std::process::Command::new(env!("CARGO"))
        .env("ANDROID_NDK_HOME", args.ndk_home.clone())
        .env("ANDROID_HOME", args.sdk_home.clone())
        .current_dir(args.working_directory)
        .arg("ndk")
        .args(["-t", args.arch.to_str().unwrap()])
        .args(["-o", args.jni_output.to_str().unwrap()])
        .args([
            "build",
            "--package",
            args.package.to_str().unwrap(),
            "--lib",
        ])
        .status()
        .unwrap();
    if !process.success() {
        println!("error build_android");
        return;
    }
    println!("{:?}", args.app_source);
    let _java_version = std::process::Command::new("java")
        .arg("-version")
        .status()
        .unwrap();
    let gradle_process = std::process::Command::new("./gradlew")
        .env("ANDROID_NDK_HOME", args.ndk_home)
        .env("ANDROID_HOME", args.sdk_home)
        .current_dir(args.app_source)
        .args(["build", "--stacktrace"])
        .status()
        .unwrap();
    if !gradle_process.success() {
        println!("error gradle build");
        return;
    }
    println!("copying .apk to {}", "");
    // cp apk to apk_destination
}
