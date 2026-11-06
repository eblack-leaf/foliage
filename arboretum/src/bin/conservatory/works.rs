//! What is on the walls: twelve works in four rooms, and the visitors' wall.
//!
//! All of it is fiction. *ilex* is not a model and nothing was trained on anything; each work is a
//! style, a seed and a ramp, and grows the same picture every time it is shown.

use arboretum::genart::{Ramp, Seed, Style};

use crate::theme::ramp;

/// Where a work's picture comes from.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Art {
    /// Grown from its style, seed and ramp, in front of the visitor.
    Grown,
    /// A real picture, a PNG or JPEG in `assets/works/` under this name: read from the site on
    /// the web and from the crate's directory natively, and grown after all if it is not there.
    /// The drop-in slot, which none of the works fills as shipped.
    #[allow(dead_code)]
    File(&'static str),
}

/// One room of the exhibition.
pub struct Room {
    pub numeral: &'static str,
    pub name: &'static str,
}

pub const ROOMS: [Room; 4] = [
    Room {
        numeral: "i",
        name: "germination",
    },
    Room {
        numeral: "ii",
        name: "canopy",
    },
    Room {
        numeral: "iii",
        name: "understory",
    },
    Room {
        numeral: "iv",
        name: "decay",
    },
];

/// One work on the wall.
pub struct Work {
    pub no: u32,
    pub title: &'static str,
    pub room: usize,
    pub style: Style,
    pub seed: u64,
    /// The ramp it is coloured along: OKLCH stops, dark to light.
    pub ramp: &'static [(f32, f32, f32)],
    pub year: u32,
    pub prompt: &'static str,
    pub steps: u32,
    pub cfg: f32,
    pub sampler: &'static str,
    pub edition: (u32, u32),
    /// What a print costs, in euros.
    pub price: u32,
    pub art: Art,
}

impl Work {
    /// The ramp, built.
    pub fn ramp(&self) -> Ramp {
        ramp(self.ramp)
    }

    /// The seed, as a seed.
    pub fn seed(&self) -> Seed {
        Seed::new(self.seed)
    }

    /// The seed as it is printed on the placard.
    pub fn seed_label(&self) -> String {
        format!("0x{:04x}", self.seed & 0xffff)
    }
}

// Every ramp rises out of the dark -- a work is a print with a black ground -- through one pigment
// to a pale end, so the twelve come out of the same dark wall.
const SEEDLING: &[(f32, f32, f32)] = &[
    (0.15, 0.01, 90.0),
    (0.38, 0.07, 95.0),
    (0.70, 0.13, 95.0),
    (0.94, 0.06, 100.0),
];
const AMBER: &[(f32, f32, f32)] = &[
    (0.13, 0.01, 60.0),
    (0.40, 0.10, 50.0),
    (0.74, 0.13, 70.0),
    (0.96, 0.04, 90.0),
];
const SPROUT: &[(f32, f32, f32)] = &[
    (0.16, 0.01, 140.0),
    (0.42, 0.08, 135.0),
    (0.72, 0.12, 120.0),
    (0.93, 0.06, 105.0),
];
const CHLOROPHYLL: &[(f32, f32, f32)] = &[
    (0.14, 0.01, 165.0),
    (0.36, 0.07, 160.0),
    (0.62, 0.11, 150.0),
    (0.88, 0.07, 130.0),
];
const CATHEDRAL: &[(f32, f32, f32)] = &[
    (0.14, 0.02, 230.0),
    (0.38, 0.07, 215.0),
    (0.70, 0.09, 195.0),
    (0.95, 0.03, 180.0),
];
const NIGHTSHIFT: &[(f32, f32, f32)] = &[
    (0.10, 0.01, 260.0),
    (0.34, 0.08, 250.0),
    (0.68, 0.10, 200.0),
    (0.94, 0.05, 170.0),
];
const MYCELIUM: &[(f32, f32, f32)] = &[
    (0.13, 0.01, 70.0),
    (0.40, 0.04, 75.0),
    (0.74, 0.05, 85.0),
    (0.95, 0.02, 95.0),
];
const SPORE: &[(f32, f32, f32)] = &[
    (0.12, 0.01, 250.0),
    (0.36, 0.06, 200.0),
    (0.70, 0.10, 165.0),
    (0.94, 0.05, 130.0),
];
const DEW: &[(f32, f32, f32)] = &[
    (0.15, 0.005, 240.0),
    (0.45, 0.02, 235.0),
    (0.78, 0.02, 225.0),
    (0.97, 0.005, 210.0),
];
const RUST: &[(f32, f32, f32)] = &[
    (0.14, 0.02, 35.0),
    (0.40, 0.11, 35.0),
    (0.66, 0.15, 50.0),
    (0.90, 0.08, 80.0),
];
const COMPOST: &[(f32, f32, f32)] = &[
    (0.15, 0.01, 70.0),
    (0.40, 0.06, 75.0),
    (0.66, 0.09, 85.0),
    (0.90, 0.05, 95.0),
];
const PRESSED: &[(f32, f32, f32)] = &[
    (0.16, 0.005, 80.0),
    (0.42, 0.02, 80.0),
    (0.72, 0.03, 85.0),
    (0.94, 0.01, 90.0),
];

const ILEX: &str = "euler a";

pub const WORKS: [Work; 12] = [
    Work {
        no: 1,
        title: "first light in the seed vault",
        room: 0,
        style: Style::Bloom,
        seed: 0x3f2a_19c4,
        ramp: SEEDLING,
        year: 2025,
        prompt: "first light in the seed vault, a single shoot, herbarium plate, soft focus",
        steps: 35,
        cfg: 6.5,
        sampler: ILEX,
        edition: (3, 7),
        price: 420,
        art: Art::Grown,
    },
    Work {
        no: 2,
        title: "radicle",
        room: 0,
        style: Style::Flow,
        seed: 0x91d0_7e21,
        ramp: AMBER,
        year: 2025,
        prompt: "the first root finding its way down, ink on wet paper, long exposure",
        steps: 40,
        cfg: 7.0,
        sampler: ILEX,
        edition: (1, 5),
        price: 560,
        art: Art::Grown,
    },
    Work {
        no: 3,
        title: "cotyledon choir",
        room: 0,
        style: Style::Lattice,
        seed: 0x0b17_c3d9,
        ramp: SPROUT,
        year: 2025,
        prompt: "a choir of seed leaves in stained glass, morning, cathedral window",
        steps: 30,
        cfg: 8.0,
        sampler: ILEX,
        edition: (4, 9),
        price: 380,
        art: Art::Grown,
    },
    Work {
        no: 4,
        title: "moss liturgy",
        room: 1,
        style: Style::Bloom,
        seed: 0x5ee0_4a12,
        ramp: CHLOROPHYLL,
        year: 2025,
        prompt: "moss growing over a hymn book, macro, deep green, quiet",
        steps: 35,
        cfg: 6.0,
        sampler: ILEX,
        edition: (2, 7),
        price: 480,
        art: Art::Grown,
    },
    Work {
        no: 5,
        title: "a fern unfurling in a cathedral of glass",
        room: 1,
        style: Style::Caustic,
        seed: 0xc0ff_ee55,
        ramp: CATHEDRAL,
        year: 2026,
        prompt: "a fern unfurling in a cathedral of glass, caustics on the floor, noon",
        steps: 50,
        cfg: 7.5,
        sampler: "dpm++ 2m",
        edition: (1, 3),
        price: 920,
        art: Art::Grown,
    },
    Work {
        no: 6,
        title: "photosynthesis at 3am",
        room: 1,
        style: Style::Orbit,
        seed: 0x7a11_0300,
        ramp: NIGHTSHIFT,
        year: 2026,
        prompt: "photosynthesis at 3am, a leaf lit by a phone screen, long exposure, orbit",
        steps: 45,
        cfg: 9.0,
        sampler: "dpm++ 2m",
        edition: (5, 9),
        price: 640,
        art: Art::Grown,
    },
    Work {
        no: 7,
        title: "mycelial radio",
        room: 2,
        style: Style::Flow,
        seed: 0x4d1e_5ea7,
        ramp: MYCELIUM,
        year: 2026,
        prompt: "mycelium carrying a radio signal under the forest floor, cross-section",
        steps: 40,
        cfg: 6.5,
        sampler: ILEX,
        edition: (2, 5),
        price: 590,
        art: Art::Grown,
    },
    Work {
        no: 8,
        title: "the fungus remembers",
        room: 2,
        style: Style::Aurora,
        seed: 0x2bad_f00d,
        ramp: SPORE,
        year: 2026,
        prompt: "spores lit from below, an aurora inside a mushroom gill, archival",
        steps: 35,
        cfg: 7.0,
        sampler: ILEX,
        edition: (6, 9),
        price: 450,
        art: Art::Grown,
    },
    Work {
        no: 9,
        title: "dew on a spider's index",
        room: 2,
        style: Style::Moire,
        seed: 0x1de0_ca7a,
        ramp: DEW,
        year: 2026,
        prompt: "dew on a spider's web read as a card index, interference, silver",
        steps: 30,
        cfg: 8.5,
        sampler: "ddim",
        edition: (3, 5),
        price: 510,
        art: Art::Grown,
    },
    Work {
        no: 10,
        title: "rust bloom",
        room: 3,
        style: Style::Lava,
        seed: 0x0ac1_d0ff,
        ramp: RUST,
        year: 2026,
        prompt: "a flower made of rust on a ship's hull, heat, slow",
        steps: 45,
        cfg: 7.0,
        sampler: "dpm++ 2m",
        edition: (1, 7),
        price: 700,
        art: Art::Grown,
    },
    Work {
        no: 11,
        title: "compost hymn",
        room: 3,
        style: Style::Lattice,
        seed: 0xdec0_de11,
        ramp: COMPOST,
        year: 2026,
        prompt: "the inside of a compost heap as a stained glass rose window",
        steps: 35,
        cfg: 6.0,
        sampler: ILEX,
        edition: (2, 9),
        price: 390,
        art: Art::Grown,
    },
    Work {
        no: 12,
        title: "the last herbarium",
        room: 3,
        style: Style::Bloom,
        seed: 0xfade_0012,
        ramp: PRESSED,
        year: 2026,
        prompt: "the last page of a herbarium, a pressed flower fading into the paper",
        steps: 60,
        cfg: 5.5,
        sampler: "ddim",
        edition: (1, 1),
        price: 1800,
        art: Art::Grown,
    },
];

/// A visitor's piece on the wall, as it was left.
pub struct Visitor {
    pub handle: u32,
    pub ago: &'static str,
    pub prompt: &'static str,
    pub style: Style,
    pub seed: u64,
}

pub const VISITORS: [Visitor; 8] = [
    Visitor {
        handle: 412,
        ago: "2m ago",
        prompt: "tide pool in a teacup",
        style: Style::Caustic,
        seed: 0x7e1d_e000,
    },
    Visitor {
        handle: 398,
        ago: "9m ago",
        prompt: "neon moss on a night bus",
        style: Style::Flow,
        seed: 0x0b05_0398,
    },
    Visitor {
        handle: 377,
        ago: "21m ago",
        prompt: "ember fern",
        style: Style::Bloom,
        seed: 0xe3be_7f00,
    },
    Visitor {
        handle: 360,
        ago: "40m ago",
        prompt: "glass orchard at dusk",
        style: Style::Lattice,
        seed: 0x9a55_0360,
    },
    Visitor {
        handle: 341,
        ago: "1h ago",
        prompt: "honey dripping upward",
        style: Style::Lava,
        seed: 0x4011_e700,
    },
    Visitor {
        handle: 322,
        ago: "1h ago",
        prompt: "rain on a greenhouse roof",
        style: Style::Moire,
        seed: 0x2a10_0322,
    },
    Visitor {
        handle: 301,
        ago: "2h ago",
        prompt: "aurora over the seed bank",
        style: Style::Aurora,
        seed: 0x5eed_ba4c,
    },
    Visitor {
        handle: 288,
        ago: "3h ago",
        prompt: "copper vines, orbiting",
        style: Style::Orbit,
        seed: 0xc0bb_e400,
    },
];

#[cfg(test)]
mod tests {
    use super::{ROOMS, WORKS};

    #[test]
    fn every_room_holds_three_works_in_order() {
        for (room, _) in ROOMS.iter().enumerate() {
            assert_eq!(WORKS.iter().filter(|work| work.room == room).count(), 3);
        }
        for (n, work) in WORKS.iter().enumerate() {
            assert_eq!(work.no as usize, n + 1);
        }
        assert!(WORKS.windows(2).all(|pair| pair[0].room <= pair[1].room));
    }

    #[test]
    fn no_two_works_share_a_seed() {
        for (n, a) in WORKS.iter().enumerate() {
            for b in &WORKS[n + 1..] {
                assert_ne!(a.seed, b.seed, "{} and {}", a.title, b.title);
            }
        }
    }
}
