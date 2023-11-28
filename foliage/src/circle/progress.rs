#[test]
fn progress_map() {
    use crate::circle::Circle;
    use crate::coordinate::section::Section;
    use crate::coordinate::NumericalContext;
    use crate::ginkgo::Ginkgo;
    const RING_FILENAMES: [(&'static str, f32); Circle::MIPS as usize] = [
        (
            "/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-1024.png",
            1024f32,
        ),
        (
            "/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-512.png",
            512f32,
        ),
        (
            "/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-256.png",
            256f32,
        ),
        (
            "/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-128.png",
            128f32,
        ),
        (
            "/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-64.png",
            64f32,
        ),
        (
            "/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-32.png",
            32f32,
        ),
    ];
    const PRECISION: u32 = 1000;
    use nalgebra::DMatrix;
    use std::f64::consts::PI;
    for (filename, size) in RING_FILENAMES {
        let tex = Ginkgo::png_to_r8unorm_d2(filename);
        let section = Section::<NumericalContext>::new((0, 0), (size, size));
        let center = section.center();
        let interval = 2f64 * PI / PRECISION as f64;
        let mut data = vec![0f32; (size * size) as usize];
        for unit in 0..PRECISION {
            let current = interval * unit as f64;
            for y in 0..size as u32 {
                for x in 0..size as u32 {
                    let index = x + size as u32 * y;
                    let px = tex.get(index as usize).unwrap().clone();
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
            format!("/home/salt/Desktop/dev/foliage/foliage/src/circle/texture_resources/circle-ring-{}.prog",
                    size as i32), data_string).unwrap();
    }
}
