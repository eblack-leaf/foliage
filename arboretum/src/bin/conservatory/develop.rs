//! A picture grown into a plate in front of the visitor: painted a bounded step a frame, then
//! developed out of the dark a step at a time and a patch at a time -- a dim, soft wash of its
//! colours brightening and sharpening -- as a diffusion model's preview resolves.

use foliage::{Area, Grove, Grow, Plate};

use arboretum::genart::{Denoise, Painter, Seed};

/// How many pictures a develop shows on its way out of the dark to the picture, whatever the
/// work's own step count: each is a [`load`](Grow::load), which costs a repaint.
pub const SHOWN: u32 = 14;

/// How many patches a plate is cut into across, as it comes in.
pub const PATCHES: u32 = 8;

/// What a develop did this frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// The picture is painted; developing starts.
    Painted,
    /// A step was shown: the model's step it stands for, of how many.
    Step(u32, u32),
    /// The last step: the picture itself is on the plate.
    Developed,
}

enum Stage {
    Painting(Box<Painter>),
    Developing,
    Done,
}

/// One picture on its way into one plate.
pub struct Develop {
    plate: Plate,
    size: (u32, u32),
    stage: Stage,
    denoise: Option<Denoise>,
    picture: Option<Vec<u8>>,
    seed: Seed,
    /// The model's step count, which is what the readout counts in.
    steps: u32,
    shown: u32,
    step_ms: f32,
    clock: f32,
}

impl Develop {
    /// Starts painting `painter`'s picture for `plate`, to be developed over about `ms` once it is
    /// painted and counted in `steps` of the model's. The plate stands dark meanwhile.
    pub fn new(grove: &mut Grove, plate: Plate, painter: Painter, steps: u32, ms: u64, seed: Seed) -> Self {
        let size = painter.size();
        grove.load(plate, blank(size), area(size));
        Self {
            plate,
            size,
            stage: Stage::Painting(Box::new(painter)),
            denoise: None,
            picture: None,
            seed,
            steps,
            shown: 0,
            step_ms: ms as f32 / SHOWN as f32,
            clock: 0.0,
        }
    }

    /// A picture already painted, put on `plate` at once.
    pub fn ready(grove: &mut Grove, plate: Plate, picture: Vec<u8>, size: (u32, u32), steps: u32, seed: Seed) -> Self {
        grove.load(plate, picture.clone(), area(size));
        Self {
            plate,
            size,
            stage: Stage::Done,
            denoise: None,
            picture: Some(picture),
            seed,
            steps,
            shown: SHOWN,
            step_ms: 90.0,
            clock: 0.0,
        }
    }

    /// The finished picture, once painted.
    pub fn picture(&self) -> Option<&[u8]> {
        self.picture.as_deref()
    }

    /// Whether the picture is on the plate and nothing is left to do.
    pub fn done(&self) -> bool {
        matches!(self.stage, Stage::Done)
    }

    /// Whether it is still being painted, before any developing.
    pub fn painting(&self) -> bool {
        matches!(self.stage, Stage::Painting(_))
    }

    /// How far the painting has got, before developing.
    pub fn progress(&self) -> f32 {
        match &self.stage {
            Stage::Painting(painter) => painter.progress(),
            _ => 1.0,
        }
    }

    /// Develops the picture again out of the dark, over about `ms`. Nothing before it is painted.
    pub fn replay(&mut self, grove: &mut Grove, ms: u64) {
        let Some(picture) = &self.picture else {
            return;
        };
        if self.denoise.is_none() {
            self.denoise = Some(Denoise::new(picture.clone(), self.size.0, self.size.1, self.seed));
        }
        self.step_ms = ms as f32 / SHOWN as f32;
        self.shown = 0;
        self.clock = 0.0;
        self.stage = Stage::Developing;
        if let Some(denoise) = &self.denoise {
            grove.load(self.plate, denoise.patches(0, SHOWN, self.patch()), area(self.size));
        }
        grove.again();
    }

    /// Paints a step or shows one, as the clock allows.
    pub fn frame(&mut self, grove: &mut Grove) -> Option<Event> {
        match &mut self.stage {
            Stage::Done => None,
            Stage::Painting(painter) => {
                grove.again();
                if !painter.step() {
                    return None;
                }
                let Stage::Painting(painter) = std::mem::replace(&mut self.stage, Stage::Developing)
                else {
                    unreachable!("matched as painting above");
                };
                let picture = painter.into_pixels();
                self.denoise = Some(Denoise::new(picture.clone(), self.size.0, self.size.1, self.seed));
                self.picture = Some(picture);
                self.shown = 0;
                self.clock = 0.0;
                Some(Event::Painted)
            }
            Stage::Developing => {
                grove.again();
                self.clock += grove.frame_time().as_secs_f32() * 1000.0;
                if self.clock < self.step_ms {
                    return None;
                }
                self.clock = (self.clock - self.step_ms).min(self.step_ms);
                self.shown += 1;
                let patch = self.patch();
                let denoise = self.denoise.as_ref()?;
                grove.load(self.plate, denoise.patches(self.shown, SHOWN, patch), area(self.size));
                if self.shown >= SHOWN {
                    self.stage = Stage::Done;
                    return Some(Event::Developed);
                }
                Some(Event::Step(self.model_step(), self.steps))
            }
        }
    }

    /// The model's step the plate stands at.
    pub fn model_step(&self) -> u32 {
        (self.shown * self.steps).div_ceil(SHOWN)
    }

    /// How large a patch of the plate is, in texels.
    fn patch(&self) -> u32 {
        (self.size.0 / PATCHES).max(1)
    }
}

/// A plate's size as an area.
pub fn area((w, h): (u32, u32)) -> Area {
    Area::new(w as f32, h as f32)
}

/// What a plate shows before there is anything to develop: the dark it is waiting in, lifted a
/// little off the wall toward its middle, as an unlit canvas catches what light there is. Smooth,
/// with nothing in it.
pub fn blank((w, h): (u32, u32)) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / w.max(1) as f32 - 0.5, y as f32 / h.max(1) as f32 - 0.42);
            let lift = (-(u * u + v * v) / 0.18).exp();
            let grey = 19.0 + 9.0 * lift;
            bytes.extend_from_slice(&[grey as u8, grey as u8, (grey * 1.05) as u8, 255]);
        }
    }
    bytes
}
