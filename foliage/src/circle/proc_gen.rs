pub(crate) const TEXTURE_SIZE: u32 = 874;
pub(crate) const UPPER_BOUND: u32 = 200;
pub(crate) const LOWER_BOUND: u32 = 12;
pub(crate) const STEP: usize = 4;

#[test]
#[cfg(test)]
fn generate() {
    use crate::circle;
    use crate::color::Rgba;
    use crate::coordinate::area::Area;
    use crate::coordinate::section::Section;
    use crate::coordinate::CoordinateUnit;
    use crate::coordinate::NumericalContext;
    use std::f64::consts::PI;
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("circle")
        .join("texture_resources");
    const RING_RATIO: CoordinateUnit = 0.95;
    const INTERVALS: u32 = 100;
    let mut data = vec![Rgba::default(); (TEXTURE_SIZE * TEXTURE_SIZE) as usize];
    let placements = circle::new_placements();
    for (placement_index, diameter) in (LOWER_BOUND..=UPPER_BOUND).step_by(STEP).enumerate() {
        // make circle
        let size: Area<NumericalContext> = (diameter, diameter).into();
        let section = Section::<NumericalContext>::new((0, 0), size);
        let center = section.center();
        let radius = diameter as f32 / 2f32;
        let tolerance = (radius * 0.025).max(1.0f32).min(2f32);
        let radius = radius - tolerance * 2f32;
        let ring_radius = radius - (radius - radius * RING_RATIO).max(0.025f32).min(3f32);
        let radii_diff = radius - ring_radius;
        let radii_half_diff = radii_diff / 2f32;
        let placement = placements.get(placement_index).unwrap().1;
        let offset = placement.position;
        for i in 0..INTERVALS {
            let current = 2f64 * PI / INTERVALS as f64 * i as f64;
            for y in 0..diameter {
                for x in 0..diameter {
                    let index = offset.x as u32 + (offset.y as u32 + y) * TEXTURE_SIZE + x;
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
                    let distance = nalgebra::distance(
                        &nalgebra::Point2::new(center.x, center.y),
                        &nalgebra::Point2::new(x as CoordinateUnit, y as CoordinateUnit),
                    );
                    let fill_threshold = radius + tolerance;
                    if distance <= fill_threshold {
                        let additive = (1f32 + (radius - distance) / tolerance).min(1.0);
                        data.get_mut(index as usize).as_mut().unwrap().0 += additive;
                        let ring_fill_threshold = ring_radius - tolerance;
                        if distance >= ring_fill_threshold {
                            let ring_diff = ring_radius + radii_half_diff - distance;
                            let ring_additive = if ring_diff.is_sign_positive()
                                && ring_diff >= radii_half_diff - tolerance
                            {
                                (1f32 - (ring_radius - distance) / tolerance).min(1.0)
                            } else {
                                additive
                            };
                            data.get_mut(index as usize).as_mut().unwrap().1 += ring_additive;
                        }
                    }
                    if angle > current {
                        let inverse_x = y;
                        let inverse_y = x;
                        let index = offset.x as u32
                            + (offset.y as u32 + inverse_y) * TEXTURE_SIZE
                            + inverse_x;
                        data.get_mut(index as usize).unwrap().2 += 1f32;
                    }
                }
            }
        }
    }
    for rgb in data.iter_mut() {
        rgb.0 /= INTERVALS as f32;
        rgb.1 /= INTERVALS as f32;
        rgb.2 /= INTERVALS as f32;
        rgb.0 *= 255f32;
        rgb.1 *= 255f32;
        rgb.2 *= 255f32;
    }
    let content = data
        .drain(..)
        .map(|d| vec![d.0 as u8, d.1 as u8, d.2 as u8, d.3 as u8])
        .flatten()
        .collect::<Vec<u8>>();
    // save to file
    let serialized = rmp_serde::to_vec(&content).unwrap();
    std::fs::write(root.join("packed.dat"), serialized).unwrap();
}
