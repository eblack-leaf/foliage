#[test]
fn textures() {
    use crate::coordinate::section::Section;
    use crate::coordinate::NumericalContext;
    use crate::rectangle::Rectangle;
    use nalgebra::DMatrix;
    use std::f64::consts::PI;
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("rectangle")
        .join("texture_resources");
    let mut filled = vec![];
    let mut ring = vec![];
    let ring_depth = 5;
    let within_ring = |x, y, max_x, max_y, ring_depth| {
        if x <= ring_depth {
            return true;
        }
        if x >= max_x - ring_depth {
            return true;
        }
        if y <= ring_depth {
            return true;
        }
        if y >= max_y - ring_depth {
            return true;
        }
        false
    };
    for y in 0..Rectangle::TEXTURE_DIMENSIONS {
        for x in 0..Rectangle::TEXTURE_DIMENSIONS {
            filled.push(255u8);
            if within_ring(
                x,
                y,
                Rectangle::TEXTURE_DIMENSIONS,
                Rectangle::TEXTURE_DIMENSIONS,
                ring_depth,
            ) {
                ring.push(255u8);
            } else {
                ring.push(0u8);
            }
        }
    }
    let size = Rectangle::TEXTURE_DIMENSIONS;
    const PRECISION: u32 = 1000;
    {
        let tex = ring.clone();
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
        std::fs::write(root.join("rectangle-ring.prog"), data_string).unwrap();
    }
    {
        let mut filled_data = vec![0f32; (size * size) as usize];
        for unit in 0..PRECISION {
            for y in 0..size {
                for x in 0..size {
                    if x > unit {
                        let index = x + size as u32 * y;
                        *filled_data.get_mut(index as usize).unwrap() += 1f32;
                    }
                }
            }
        }
        let data = filled_data
            .drain(..)
            .map(|p| {
                let normalized = p / size as f32;
                let scaled = normalized * 255f32;
                scaled as u8
            })
            .collect::<Vec<u8>>();
        let data_string = serde_json::to_string(&data).unwrap();
        std::fs::write(root.join("rectangle.prog"), data_string).unwrap();
    }
    {
        let filled = serde_json::to_string(&filled).unwrap();
        let ring = serde_json::to_string(&ring).unwrap();
        std::fs::write(root.join("rectangle-texture.cov"), filled).unwrap();
        std::fs::write(root.join("rectangle-ring-texture.cov"), ring).unwrap();
    }
}