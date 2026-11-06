//! Smooth randomness: gradient noise, and the sums of it every style is built on.
//!
//! Hash-keyed rather than table-keyed, so a field needs no permutation table and any 32-bit key is
//! a different field.

/// An integer lattice point and a key, mixed to 32 well-spread bits.
#[inline]
pub fn hash(x: i32, y: i32, key: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ key.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    h
}

/// A lattice point's own number in `0.0..1.0`: white noise, for grain.
#[inline]
pub fn white(x: i32, y: i32, key: u32) -> f32 {
    (hash(x, y, key) >> 8) as f32 / (1u32 << 24) as f32
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The dot of a lattice point's gradient with the offset to it. Eight directions, which is enough
/// for nothing to read as a grid.
#[inline]
fn slope(h: u32, x: f32, y: f32) -> f32 {
    match h & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x * 1.414,
        5 => -x * 1.414,
        6 => y * 1.414,
        _ => -y * 1.414,
    }
}

/// Gradient noise at a point: smooth, zero on every lattice point, roughly within `-1.0..=1.0`.
pub fn gradient(x: f32, y: f32, key: u32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let (fx, fy) = (x - x0, y - y0);
    let (u, v) = (fade(fx), fade(fy));
    let n00 = slope(hash(ix, iy, key), fx, fy);
    let n10 = slope(hash(ix + 1, iy, key), fx - 1.0, fy);
    let n01 = slope(hash(ix, iy + 1, key), fx, fy - 1.0);
    let n11 = slope(hash(ix + 1, iy + 1, key), fx - 1.0, fy - 1.0);
    lerp(lerp(n00, n10, u), lerp(n01, n11, u), v) * 0.72
}

/// Fractal noise: `octaves` of [`gradient`], each twice the frequency and half the weight of the
/// last, normalised back into roughly `-1.0..=1.0`.
pub fn fbm(x: f32, y: f32, key: u32, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut weight = 0.5;
    let mut total = 0.0;
    let (mut x, mut y) = (x, y);
    for octave in 0..octaves {
        sum += weight * gradient(x, y, key.wrapping_add(octave.wrapping_mul(0x9e37)));
        total += weight;
        weight *= 0.5;
        // Turned a little at each octave as well as scaled, so the lattices never line up.
        let (rx, ry) = (x * 1.6 - y * 1.2, x * 1.2 + y * 1.6);
        x = rx + 17.3;
        y = ry + 5.1;
    }
    sum / total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_zero_on_the_lattice_and_bounded_between() {
        assert_eq!(gradient(3.0, -2.0, 9), 0.0);
        for i in 0..2_000 {
            let x = i as f32 * 0.137;
            let y = i as f32 * 0.091 - 40.0;
            assert!(gradient(x, y, 4).abs() <= 1.1);
            assert!(fbm(x, y, 4, 5).abs() <= 1.1);
        }
    }

    #[test]
    fn a_key_is_a_different_field() {
        let a: f32 = (0..64).map(|i| fbm(i as f32 * 0.3, 0.7, 1, 4)).sum();
        let b: f32 = (0..64).map(|i| fbm(i as f32 * 0.3, 0.7, 2, 4)).sum();
        assert_ne!(a, b);
    }

    #[test]
    fn white_noise_stays_in_range() {
        for i in -50..50 {
            let n = white(i, i * 3, 77);
            assert!((0.0..1.0).contains(&n));
        }
    }
}
