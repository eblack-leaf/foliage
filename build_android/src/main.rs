use std::ffi::OsString;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = prepare_args(args);
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
}

fn prepare_args(args: Vec<String>) -> Args {
    if let Some(package) = args.get(1) {
        if let Some(arch) = args.get(2) {
            if let Some(ndk_home) = args.get(3) {
                if let Some(sdk_home) = args.get(4) {
                    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                        .canonicalize()
                        .unwrap();
                    let app_source = root.join("app_src");
                    let jni_output = app_source
                        .join("app")
                        .join("src")
                        .join("main")
                        .join("jniLibs");
                    let ndk_home = Path::new(ndk_home).canonicalize().unwrap();
                    let sdk_home = Path::new(sdk_home).canonicalize().unwrap();
                    return Args {
                        package: OsString::from(package),
                        arch: OsString::from(arch),
                        ndk_home,
                        sdk_home,
                        working_directory: root,
                        jni_output,
                        app_source,
                    };
                } else {
                    panic!("could not prepare args: check sdk_home");
                }
            } else {
                panic!("could not prepare args: check ndk_home");
            }
        } else {
            panic!("could not prepare args: check arch");
        }
    } else {
        panic!("could not prepare args: check package");
    }
}

fn build_template(args: &Args) {
    let manifest = std::fs::read_to_string(
        args.working_directory
            .join("template")
            .join("AndroidManifest.xml"),
    )
        .unwrap()
        .replace("{{package-name}}", args.package.to_str().unwrap())
        .replace("{{package-label}}", "foliage");
    // let build = std::fs::read_to_string(args.working_directory.join("template").join("build.gradle")).unwrap();
    let activity = std::fs::read_to_string(
        args.working_directory
            .join("template")
            .join("MainActivity.java"),
    )
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
    // let build_dest = args.app_source.join("app").join("src").join("build.gradle");
    let manifest_dest = args
        .app_source
        .join("app")
        .join("src")
        .join("main")
        .join("AndroidManifest.xml");
    std::fs::write(activity_dest, activity).unwrap();
    // std::fs::write(build_dest, build).unwrap();
    std::fs::write(manifest_dest, manifest).unwrap();
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
    // cp apk to apk_destination
}