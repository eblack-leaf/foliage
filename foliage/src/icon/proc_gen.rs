#[test]
fn load() {
    use crate::ginkgo::Ginkgo;
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("icon")
        .join("resources");
    for size in ["24", "48", "96"] {
        for entry in std::fs::read_dir(root.join("svg").join(size)).unwrap().flatten() {
            let dest = entry
                .file_name()
                .as_os_str()
                .to_str()
                .unwrap()
                .replace(".svg", ".png");
            std::process::Command::new("inkscape")
                .current_dir(root.join("svg"))
                .arg(entry.file_name().as_os_str())
                .arg("-o")
                .arg(root.join("png").join(size).join(dest))
                .status()
                .ok()
                .unwrap();
        }
        for entry in std::fs::read_dir(root.join("png").join(size)).unwrap().flatten() {
            let dest = entry
                .file_name()
                .as_os_str()
                .to_str()
                .unwrap()
                .replace(".png", ".cov");
            Ginkgo::png_to_cov(
                entry.path(),
                root.join("cov").join(size).join(dest),
            )
        }
    }
}