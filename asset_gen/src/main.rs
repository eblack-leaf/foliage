use foliage::icon::FeatherIcon;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let asset_dir = args.get(1).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap();
    let actual_asset_dir = root.join(asset_dir);
    let output_file = args.get(2).unwrap();
    let mut output = vec![];
    output.push("#[foliage::assets()]".to_string());
    output.push("#[derive(Resource, Clone)]".to_string());
    output.push("pub(crate) struct AssetGen {".to_string());
    for dir in walkdir::WalkDir::new(actual_asset_dir.to_path_buf()).min_depth(1) {
        if let Some(d) = dir.ok() {
            let stem = d.path().file_stem().unwrap();
            if d.path().is_dir() {
                continue;
            }
            let name = d.path().file_name().unwrap();
            let extension = d.path().extension().unwrap();
            let mut relative = vec![];
            let mut path = d.path().to_path_buf();
            while let Some(parent) = path.parent() {
                if parent.to_str().unwrap() != actual_asset_dir.to_str().unwrap() {
                    let last = parent.file_name().unwrap().to_os_string();
                    relative.push(last);
                    path.pop();
                } else {
                    break;
                }
            }
            relative.reverse();
            let p = if relative.is_empty() {
                d.file_name().to_str().unwrap().to_string()
            } else {
                let mut p = PathBuf::from(relative.get(0).unwrap());
                for a in relative[1..].iter() {
                    p = p.join(a);
                }
                p.join(name).to_str().unwrap().to_string()
            };
            if extension == "icon" {
                let opt: FeatherIcon = stem.to_str().unwrap().into();
                output.push(format!(
                    "\t#[icon(path = \"{}\", opt = FeatherIcon::{})]\n\t_id: AssetKey,",
                    p, opt
                ));
            } else {
                output.push(format!(
                    "\t#[bytes(path = \"{}\", group = generated)]\n\t_id: AssetKey,",
                    p
                ))
            }
        }
    }
    output.push("}".to_string());
    let o_file = root.join(output_file);
    std::fs::write(
        o_file,
        output.iter().map(|o| o.clone() + "\n").collect::<String>(),
    )
    .unwrap();
}
