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
    gradlew_path: PathBuf,
}

fn prepare_args(args: Vec<String>) -> Args {
    if let Some(package) = args.get(1) {
        if let Some(arch) = args.get(2) {
            if let Some(ndk_home) = args.get(3) {
                if let Some(sdk_home) = args.get(4) {
                    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                        .canonicalize()
                        .unwrap();
                    let this = root.join(env!("CARGO_PKG_NAME"));
                    let jni_output = this.join("jni_libs");
                    let app_source = this.join("app_src");
                    let gradlew_path = app_source.join("gradlew");
                    return Args {
                        package: OsString::from(package),
                        arch: OsString::from(arch),
                        ndk_home: Path::new(ndk_home).canonicalize().unwrap(),
                        sdk_home: Path::new(sdk_home).canonicalize().unwrap(),
                        working_directory: root,
                        jni_output,
                        app_source,
                        gradlew_path,
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
    .unwrap();
    // let build = std::fs::read_to_string(args.working_directory.join("template").join("build.gradle")).unwrap();
    let activity = std::fs::read_to_string(
        args.working_directory
            .join("template")
            .join("MainActivity.java"),
    )
    .unwrap();
    let activity_dest = args
        .app_source
        .join("app")
        .join("src")
        .join("main")
        .join("java")
        .join("")
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
    let process = std::process::Command::new(env!("CARGO"))
        .env("ANDROID_NDK_HOME", args.ndk_home.clone())
        .env("ANDROID_HOME", args.sdk_home.clone())
        .current_dir(args.working_directory)
        .arg("ndk")
        .args(["-t", args.arch])
        .args(["-o", args.jni_output])
        .args(["build", "--package", args.package])
        .status()
        .unwrap();
    if !process.success() {
        println!("error build_android");
        return;
    }
    let gradle_process = std::process::Command::new(args.gradlew_path)
        .env("ANDROID_NDK_HOME", args.ndk_home)
        .env("ANDROID_HOME", args.sdk_home)
        .current_dir(args.app_source)
        .args(["build", "--stacktrace"])
        .status()
        .unwrap();
    if !gradle_process.success() {
        println!("error gradle build")
    }
    // cp apk to apk_destination
}
