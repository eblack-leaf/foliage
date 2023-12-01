

#[cfg(test)]
#[test]
fn svg_to_png_to_cov() {
    use crate::ginkgo::Ginkgo;
    use std::path::Path;
    use crate::icon::Icon;
    const SIZE: i32 = Icon::TEXTURE_DIMENSIONS as i32;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("icon")
        .join("resources");
    for entry in std::fs::read_dir(root.join("svg")).unwrap() {
        if let Some(entry) = entry.ok() {
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
                .arg(root.join("png").join(dest))
                .status()
                .ok()
                .unwrap();
        }
    }
    for entry in std::fs::read_dir(root.join("png")).unwrap() {
        if let Some(entry) = entry.ok() {
            let dest = entry
                .file_name()
                .as_os_str()
                .to_str()
                .unwrap()
                .replace(".png", ".cov");
            Ginkgo::png_to_cov(
                entry.path(),
                root.join("cov").join(SIZE.to_string()).join(dest),
            )
        }
    }
}