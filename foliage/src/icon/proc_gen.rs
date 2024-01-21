

#[cfg(test)]
#[test]
fn svg_to_png_to_cov() {
    use std::path::Path;
    use crate::ginkgo::Ginkgo;
    const SIZE: i32 = 32;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("icon")
        .join("resources");
    for entry in std::fs::read_dir(root.join("svg")).unwrap().flatten() {
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
    for entry in std::fs::read_dir(root.join("png")).unwrap().flatten() {
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
#[cfg(test)]
#[test]
fn generate() {
    use crate::icon::bundled_cov::{ICON_NAMES};
    use crate::icon::renderer::{icon_scale_range};
    use std::path::Path;
    use crate::ginkgo::Ginkgo;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("icon")
        .join("resources");
    for name in ICON_NAMES {
        let dest = root.join(format!("{}.atl", name));
        let range = icon_scale_range();
        let mut data: Vec<Vec<u8>> = vec![];
        for i in range {
            let svg_title = format!("{}.svg", name);
            let tmp_input_dir = root.join("tmp").join(i.to_string());
            let tmp_input = tmp_input_dir.join(svg_title.clone());
            let tmp_dest = tmp_input_dir.join(format!("{}.png", name));
            std::process::Command::new("inkscape")
                .current_dir(tmp_input_dir.clone())
                .arg(svg_title)
                .arg("-o")
                .arg(tmp_dest.clone())
                .status()
                .ok()
                .unwrap();
            let cov_location = tmp_input_dir.join(format!("{}.cov", name));
            Ginkgo::png_to_cov(tmp_dest, cov_location.clone());
            let sized_data =
                rmp_serde::from_slice(std::fs::read(cov_location).unwrap().as_slice()).unwrap();
            data.push(sized_data);
        }
        let data = rmp_serde::to_vec(&data).unwrap();
        std::fs::write(dest, data).unwrap();
    }
}
#[cfg(test)]
#[test]
fn place() {
    use std::path::Path;
    use crate::icon::bundled_cov::{ICON_NAMES, ICON_RESOURCE_FILES};
    use crate::icon::Icon;
    use crate::icon::renderer::placements;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("icon")
        .join("resources");
    for (index, file) in ICON_RESOURCE_FILES.iter().enumerate() {
        let mut grouped = vec![0u8; (Icon::TEXTURE_DIMENSIONS * Icon::TEXTURE_DIMENSIONS) as usize];
        let datum = rmp_serde::from_slice::<Vec<Vec<u8>>>(file).unwrap();
        let name = ICON_NAMES[index];
        for (i, (scale, place)) in placements().iter().enumerate() {
            let data = datum.get(i).unwrap();
            let offset = place.position;
            let mut zero_index = 0;
            for y in 0..*scale {
                for x in 0..*scale {
                    let index =
                        offset.x as u32 + (offset.y as u32 + y) * Icon::TEXTURE_DIMENSIONS + x;
                    let d = *data.get(zero_index).unwrap();
                    *grouped.get_mut(index as usize).unwrap() = d;
                    zero_index += 1;
                }
            }
        }
        let grouped = rmp_serde::to_vec(&grouped).unwrap();
        std::fs::write(root.join(format!("{}.gatl", name)), grouped).unwrap();
    }
}
