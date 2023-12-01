#[test]
fn progress_map() {
    use crate::circle::Circle;
    use crate::coordinate::section::Section;
    use crate::coordinate::NumericalContext;
    use crate::ginkgo::Ginkgo;
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("circle")
        .join("texture_resources");
    const RING_FILENAMES: [(&str, f32); Circle::MIPS as usize] = [
        ("circle-ring-1536.png", 1536f32),
        ("circle-ring-768.png", 768f32),
        ("circle-ring-384.png", 384.0),
        ("circle-ring-192.png", 192.0),
        ("circle-ring-96.png", 96.0),
        ("circle-ring-48.png", 48.0),
        ("circle-ring-24.png", 24.0),
        ("circle-ring-12.png", 12.0),
    ];
    const PRECISION: u32 = 1000;
    use nalgebra::DMatrix;
    use std::f64::consts::PI;
    for (filename, size) in RING_FILENAMES {
        let tex = Ginkgo::png_to_r8unorm_d2(root.join(filename));
        let section = Section::<NumericalContext>::new((0, 0), (size, size));
        let center = section.center();
        let interval = 2f64 * PI / PRECISION as f64;
        let mut data = vec![0f32; (size * size) as usize];
        for unit in 0..PRECISION {
            let current = interval * unit as f64;
            for y in 0..size as u32 {
                for x in 0..size as u32 {
                    let index = x + size as u32 * y;
                    let px = *tex.get(index as usize).unwrap();
                    let opaque = px > 0u8;
                    let (ax, ay) = (x as f64 - center.x as f64, center.y as f64 - y as f64);
                    let mut angle = f64::atan(ay / ax);
                    if ay.is_sign_positive() && angle.is_sign_negative() {
                        angle += PI;
                    }
                    if ay.is_sign_negative() && angle.is_sign_positive() {
                        angle += PI;
                    }
                    if ay.is_sign_negative() && angle.is_sign_negative() {
                        angle += 2f64 * PI;
                    }
                    if angle > current && opaque {
                        *data.get_mut(index as usize).unwrap() += 1f32;
                    }
                }
            }
        }
        let data = data
            .drain(..)
            .map(|p| {
                let normalized = p / PRECISION as f32;
                let scaled = normalized * 255f32;
                scaled as u8
            })
            .collect::<Vec<u8>>();
        let matrix = DMatrix::from_vec(size as usize, size as usize, data);
        let matrix = matrix.transpose();
        let data_string = serde_json::to_string(&matrix.data.as_vec()).unwrap();
        std::fs::write(
            root.join(format!("circle-ring-{}.prog", size as i32)),
            data_string,
        )
        .unwrap();
    }
    const FILL_FILENAMES: [(&str, f32); Circle::MIPS as usize] = [
        ("circle-1536.png", 1536.0),
        ("circle-768.png", 768.0),
        ("circle-384.png", 384.0),
        ("circle-192.png", 192.0),
        ("circle-96.png", 96.0),
        ("circle-48.png", 48.0),
        ("circle-24.png", 24.0),
        ("circle-12.png", 12.0),
    ];
    for (filename, size) in FILL_FILENAMES {
        let tex = Ginkgo::png_to_r8unorm_d2(root.join(filename));
        let section = Section::<NumericalContext>::new((0, 0), (size, size));
        let center = section.center();
        let interval = 2f64 * PI / PRECISION as f64;
        let mut data = vec![0f32; (size * size) as usize];
        for unit in 0..PRECISION {
            let current = interval * unit as f64;
            for y in 0..size as u32 {
                for x in 0..size as u32 {
                    let index = x + size as u32 * y;
                    let px = *tex.get(index as usize).unwrap();
                    let opaque = px > 0u8;
                    let (ax, ay) = (x as f64 - center.x as f64, center.y as f64 - y as f64);
                    let mut angle = f64::atan(ay / ax);
                    if ay.is_sign_positive() && angle.is_sign_negative() {
                        angle += PI;
                    }
                    if ay.is_sign_negative() && angle.is_sign_positive() {
                        angle += PI;
                    }
                    if ay.is_sign_negative() && angle.is_sign_negative() {
                        angle += 2f64 * PI;
                    }
                    if angle > current && opaque {
                        *data.get_mut(index as usize).unwrap() += 1f32;
                    }
                }
            }
        }
        let data = data
            .drain(..)
            .map(|p| {
                let normalized = p / PRECISION as f32;
                let scaled = normalized * 255f32;
                scaled as u8
            })
            .collect::<Vec<u8>>();
        let matrix = DMatrix::from_vec(size as usize, size as usize, data);
        let matrix = matrix.transpose();
        let data_string = serde_json::to_string(&matrix.data.as_vec()).unwrap();
        std::fs::write(
            root.join(format!("circle-{}.prog", size as i32)),
            data_string,
        )
        .unwrap();
    }
}

#[test]
fn coverage_maps() {
    use crate::circle::Circle;
    use crate::ginkgo::Ginkgo;
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("circle")
        .join("texture_resources");
    for mip in Circle::MIPS_TARGETS {
        Ginkgo::png_to_cov(
            root.join(format!("circle-ring-{}.png", mip)),
            root.join(format!("circle-ring-texture-{}.cov", mip)),
        );
        Ginkgo::png_to_cov(
            root.join(format!("circle-{}.png", mip)),
            root.join(format!("circle-texture-{}.cov", mip)),
        );
    }
}
