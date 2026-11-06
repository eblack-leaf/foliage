//! A picture arriving the way a diffusion model's preview does now: out of the dark as a dim, soft
//! wash of its own colours, which brightens and sharpens -- all of it at once, or a patch at a time.

use super::noise::white;
use super::seed::Seed;

/// A finished picture, and the soft, dim readings of it that it is shown resolving through.
///
/// A step is a blend, never a render: the picture is painted once and blurred at a handful of
/// widths once, and every step after that is arithmetic over bytes that already exist. There is no
/// static anywhere in it. What makes it read as diffusion rather than as a fade is the order things
/// arrive in -- the broad masses and colours first, dim, then the light, then the detail -- and,
/// [a patch at a time](Denoise::patches), the picture being further along in some places than in
/// others.
pub struct Denoise {
    width: u32,
    height: u32,
    /// The picture at each softness, softest first; the last is the picture itself.
    levels: Vec<Vec<u8>>,
    /// What decides which patches come in first.
    key: u32,
}

/// How soft each reading is, as a blur radius in texels of a picture 256 wide, softest first. The
/// last is no blur: the picture.
const SOFTNESS: [f32; 5] = [28.0, 12.0, 5.0, 2.0, 0.0];
/// How bright the first reading is, of the picture's own light: a wash in the dark.
const FIRST_LIGHT: f32 = 0.16;
/// How far along a reading is at full brightness; after that it only sharpens.
const LIT: f32 = 0.72;
/// How much of the way a patch takes to come in, once it starts.
const ARRIVE: f32 = 0.34;

impl Denoise {
    /// The readings `picture` resolves through, its patches ordered by `seed`. `picture` is RGBA,
    /// `width` × `height`.
    ///
    /// # Panics
    ///
    /// If `picture` is smaller than its size says.
    pub fn new(picture: Vec<u8>, width: u32, height: u32, seed: Seed) -> Self {
        assert!(
            picture.len() >= (width * height * 4) as usize,
            "a {width}x{height} picture is {} bytes",
            width * height * 4
        );
        let scale = width as f32 / 256.0;
        let levels = SOFTNESS
            .iter()
            .map(|&radius| blur(&picture, width, height, (radius * scale).round() as u32))
            .collect();
        Self {
            width,
            height,
            levels,
            key: seed.key(),
        }
    }

    /// The picture's size.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// How far from finished the picture is at `step` of `of`: `1.0` at the first, `0.0` at the
    /// last.
    pub fn sigma(step: u32, of: u32) -> f32 {
        let along = (step as f32 / of.max(1) as f32).clamp(0.0, 1.0);
        (1.0 - along).powf(1.6)
    }

    /// What is on screen at `step` of `of`, the whole picture equally far along. The first step is
    /// a dim, soft wash; the last is the picture exactly.
    pub fn frame(&self, step: u32, of: u32) -> Vec<u8> {
        let along = (step as f32 / of.max(1) as f32).clamp(0.0, 1.0);
        let mut out = vec![255u8; (self.width * self.height * 4) as usize];
        for at in (0..out.len()).step_by(4) {
            self.texel(at, along, &mut out);
        }
        out
    }

    /// What is on screen at `step` of `of` when the picture comes in a region at a time: each
    /// `patch`-texel region has its own moment, so that partway there some of the picture is
    /// finished, some is still a dim wash and the rest is on its way between. Regions near each
    /// other tend to come together, and the moments are blended between their middles, so there are
    /// no edges between them -- the picture is simply further along in some places than in others.
    ///
    /// The first step is [`frame`](Self::frame)'s first exactly, and the last the picture exactly.
    pub fn patches(&self, step: u32, of: u32, patch: u32) -> Vec<u8> {
        let along = (step as f32 / of.max(1) as f32).clamp(0.0, 1.0);
        let last = self.levels.len() - 1;
        if along >= 1.0 {
            return self.levels[last].clone();
        }
        let patch = patch.max(1);
        let columns = self.width.div_ceil(patch);
        let rows = self.height.div_ceil(patch);
        // How far along each patch is: 0 is the first wash, 1 the picture.
        let progress: Vec<f32> = (0..rows)
            .flat_map(|row| {
                (0..columns).map(move |column| {
                    let start = Self::moment(column, row, self.key) * (1.0 - ARRIVE);
                    smoothstep(start, start + ARRIVE, along)
                })
            })
            .collect();
        let moment = |column: i32, row: i32| {
            let column = column.clamp(0, columns as i32 - 1) as u32;
            let row = row.clamp(0, rows as i32 - 1) as u32;
            progress[(row * columns + column) as usize]
        };
        let mut out = vec![255u8; (self.width * self.height * 4) as usize];
        for y in 0..self.height {
            // Where this texel falls between the middles of the regions round it.
            let fy = (y as f32 + 0.5) / patch as f32 - 0.5;
            let (row, ty) = (fy.floor() as i32, fy - fy.floor());
            for x in 0..self.width {
                let fx = (x as f32 + 0.5) / patch as f32 - 0.5;
                let (column, tx) = (fx.floor() as i32, fx - fx.floor());
                let top = moment(column, row) * (1.0 - tx) + moment(column + 1, row) * tx;
                let bottom = moment(column, row + 1) * (1.0 - tx) + moment(column + 1, row + 1) * tx;
                let here = top * (1.0 - ty) + bottom * ty;
                self.texel(((y * self.width + x) * 4) as usize, here, &mut out);
            }
        }
        out
    }

    /// Writes the texel at byte `at` as it reads `here` of the way from the first wash to the
    /// picture: softer and dimmer the earlier it is.
    fn texel(&self, at: usize, here: f32, out: &mut [u8]) {
        let last = self.levels.len() - 1;
        if here >= 1.0 {
            out[at..at + 3].copy_from_slice(&self.levels[last][at..at + 3]);
            return;
        }
        let along = here * last as f32;
        let level = along.floor() as usize;
        let between = along - level as f32;
        let (soft, sharp) = (&self.levels[level], &self.levels[(level + 1).min(last)]);
        let light = FIRST_LIGHT + (1.0 - FIRST_LIGHT) * smoothstep(0.0, LIT, here);
        for channel in 0..3 {
            let a = soft[at + channel] as f32;
            let b = sharp[at + channel] as f32;
            out[at + channel] = ((a + (b - a) * between) * light) as u8;
        }
    }

    /// When patch `column`, `row` starts to come in, `0.0` to `1.0`: mostly the moment of the
    /// cluster it is in, partly its own.
    fn moment(column: u32, row: u32, key: u32) -> f32 {
        let cluster = white((column / 2) as i32, (row / 2) as i32, key ^ 0x7a7c_51d3);
        let own = white(column as i32, row as i32, key ^ 0x0c1a_9e37);
        0.62 * cluster + 0.38 * own
    }
}

fn smoothstep(from: f32, to: f32, at: f32) -> f32 {
    let t = ((at - from) / (to - from).max(f32::EPSILON)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `picture` blurred `radius` texels each way: three box passes across and down, which is near
/// enough a gaussian to read as soft focus rather than as squares.
fn blur(picture: &[u8], width: u32, height: u32, radius: u32) -> Vec<u8> {
    let length = (width * height * 4) as usize;
    let mut out = picture[..length].to_vec();
    if radius == 0 || width == 0 || height == 0 {
        return out;
    }
    let mut scratch = vec![255u8; length];
    for _ in 0..3 {
        pass(&out, &mut scratch, width, height, radius, true);
        pass(&scratch, &mut out, width, height, radius, false);
    }
    out
}

/// One box pass of `radius`, across the rows or down the columns, edges held.
fn pass(from: &[u8], to: &mut [u8], width: u32, height: u32, radius: u32, across: bool) {
    let (lines, along) = if across { (height, width) } else { (width, height) };
    let reach = radius as i32;
    let span = 2 * radius + 1;
    for line in 0..lines {
        let at = |i: i32| -> usize {
            let i = i.clamp(0, along as i32 - 1) as u32;
            let (x, y) = if across { (i, line) } else { (line, i) };
            ((y * width + x) * 4) as usize
        };
        let mut sum = [0u32; 3];
        for i in -reach..=reach {
            let p = at(i);
            for channel in 0..3 {
                sum[channel] += from[p + channel] as u32;
            }
        }
        for i in 0..along as i32 {
            let p = at(i);
            for channel in 0..3 {
                to[p + channel] = (sum[channel] / span) as u8;
            }
            to[p + 3] = 255;
            let (enter, leave) = (at(i + reach + 1), at(i - reach));
            for channel in 0..3 {
                sum[channel] += from[enter + channel] as u32;
                sum[channel] -= from[leave + channel] as u32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picture(width: u32, height: u32) -> Vec<u8> {
        (0..width * height)
            .flat_map(|n| [(n % 251) as u8, (n * 7 % 253) as u8, (n * 13 % 241) as u8, 255])
            .collect()
    }

    #[test]
    fn the_last_step_is_the_picture_exactly() {
        let source = picture(40, 30);
        let denoise = Denoise::new(source.clone(), 40, 30, Seed::new(5));
        assert_eq!(denoise.frame(24, 24), source);
    }

    #[test]
    fn the_first_step_is_a_dim_wash() {
        let source = picture(40, 30);
        let denoise = Denoise::new(source, 40, 30, Seed::new(5));
        let first = denoise.frame(0, 24);
        assert_eq!(first.len(), 40 * 30 * 4);
        let brightest = first.chunks(4).flat_map(|texel| &texel[..3]).max().copied().unwrap_or(0);
        assert!(brightest as f32 <= 255.0 * FIRST_LIGHT + 1.0, "{brightest} is not dim");
    }

    #[test]
    fn a_blur_keeps_the_picture_s_size_and_softens_it() {
        let source = picture(48, 48);
        let soft = blur(&source, 48, 48, 6);
        assert_eq!(soft.len(), source.len());
        let rough = |p: &[u8]| -> u32 {
            p.chunks(4).collect::<Vec<_>>().windows(2).map(|w| w[0][0].abs_diff(w[1][0]) as u32).sum()
        };
        assert!(rough(&soft) < rough(&source) / 4);
    }

    #[test]
    fn patches_run_from_the_wash_to_the_picture() {
        let source = picture(40, 30);
        let denoise = Denoise::new(source.clone(), 40, 30, Seed::new(9));
        assert_eq!(denoise.patches(0, 12, 8), denoise.frame(0, 12));
        assert_eq!(denoise.patches(12, 12, 8), source);
    }

    #[test]
    fn partway_some_of_it_is_done_and_some_is_not() {
        let source = picture(64, 64);
        let denoise = Denoise::new(source.clone(), 64, 64, Seed::new(3));
        let first = denoise.frame(0, 12);
        let half = denoise.patches(6, 12, 8);
        let (mut done, mut waiting) = (0, 0);
        for at in (0..half.len()).step_by(4) {
            if half[at..at + 3] == source[at..at + 3] {
                done += 1;
            } else if half[at..at + 3] == first[at..at + 3] {
                waiting += 1;
            }
        }
        assert!(done > 0, "none of it has come in halfway through");
        assert!(waiting > 0, "all of it has started halfway through");
    }

    #[test]
    fn what_is_left_only_ever_falls() {
        let mut last = f32::MAX;
        for step in 0..=24 {
            let sigma = Denoise::sigma(step, 24);
            assert!(sigma <= last);
            last = sigma;
        }
        assert_eq!(last, 0.0);
    }
}
