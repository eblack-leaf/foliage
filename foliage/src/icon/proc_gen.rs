use std::collections::HashMap;

#[test]
fn load() {
    use crate::ginkgo::Ginkgo;
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("icon")
        .join("resources");
    let sizes = ["24", "48", "96"];
    for size in sizes {
        for entry in std::fs::read_dir(root.join("svg").join(size))
            .unwrap()
            .flatten()
        {
            let dest = entry
                .file_name()
                .as_os_str()
                .to_str()
                .unwrap()
                .replace(".svg", ".png");
            std::process::Command::new("inkscape")
                .current_dir(root.join("svg").join(size))
                .arg(entry.file_name().as_os_str())
                .arg("-o")
                .arg(root.join("png").join(size).join(dest))
                .status()
                .ok()
                .unwrap();
        }
        for entry in std::fs::read_dir(root.join("png").join(size))
            .unwrap()
            .flatten()
        {
            let dest = entry
                .file_name()
                .as_os_str()
                .to_str()
                .unwrap()
                .replace(".png", ".cov");
            Ginkgo::png_to_cov(entry.path(), root.join("cov").join(size).join(dest))
        }
    }
    let mut twenty_fours = HashMap::new();
    let mut forty_eights = HashMap::new();
    let mut ninety_six = HashMap::new();
    for size in sizes {
        for entry in std::fs::read_dir(root.join("cov").join(size))
            .unwrap()
            .flatten()
        {
            let dest = entry
                .file_name()
                .as_os_str()
                .to_str()
                .unwrap()
                .replace(".cov", ".icon");
            let bytes = std::fs::read(entry.path()).unwrap();
            if size == "24" {
                twenty_fours.insert(dest.clone(), bytes);
            } else if size == "48" {
                forty_eights.insert(dest.clone(), bytes);
            } else {
                ninety_six.insert(dest.clone(), bytes);
            }
        }
    }
    for (dest, bytes) in twenty_fours.drain() {
        let mut aggregate = bytes;
        aggregate.extend(forty_eights.get(&dest).unwrap().clone());
        aggregate.extend(ninety_six.get(&dest).unwrap().clone());
        std::fs::write(root.join("icon").join(dest), aggregate).unwrap();
    }
}
