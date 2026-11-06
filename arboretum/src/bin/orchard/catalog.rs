//! What the shop sells: sixteen goods in four aisles, each weighed to the byte.
//!
//! Plain data and the pure questions asked of it -- which goods a shelf shows, in what order, and
//! how a price or a weight is printed. Nothing here knows about a page.

use arboretum::genart::Style;

/// The four aisles.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Aisle {
    Palettes,
    Shaders,
    Type,
    Sound,
}

impl Aisle {
    pub const ALL: [Aisle; 4] = [Aisle::Palettes, Aisle::Shaders, Aisle::Type, Aisle::Sound];

    /// What the aisle is called on its tab.
    pub fn name(self) -> &'static str {
        match self {
            Aisle::Palettes => "palettes",
            Aisle::Shaders => "shaders",
            Aisle::Type => "type",
            Aisle::Sound => "sound",
        }
    }

    /// What one good from the aisle is called on its label.
    pub fn single(self) -> &'static str {
        match self {
            Aisle::Palettes => "palette",
            Aisle::Shaders => "shader",
            Aisle::Type => "specimen",
            Aisle::Sound => "sound pack",
        }
    }

    /// The files a good from the aisle comes as.
    pub fn formats(self) -> &'static [&'static str] {
        match self {
            Aisle::Palettes => &[".ase", ".json", ".css"],
            Aisle::Shaders => &[".wgsl", ".glsl"],
            Aisle::Type => &[".pdf", ".png"],
            Aisle::Sound => &[".wav", ".flac"],
        }
    }

    /// Where the aisle stands among the four.
    pub fn index(self) -> usize {
        self as usize
    }
}

/// A colour as OKLCH: lightness, chroma, hue.
pub type Lch = (f32, f32, f32);

/// The faces a specimen sheet can be printed in.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Face {
    Xanh,
    Major,
    Plex,
    JetBrains,
}

/// How a good's picture is made.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Look {
    /// The palette itself: its colours, darkest to lightest.
    Palette(&'static [Lch]),
    /// A shader running in `style`, along `ramp`, from `seed`.
    Shader {
        style: Style,
        ramp: &'static [Lch],
        seed: u64,
    },
    /// A specimen poster of `face`: the paper it is printed on and its ink.
    Specimen { face: Face, paper: Lch, ink: Lch },
    /// A waveform: the ground it is drawn on, its ink, and the seed of its shape.
    Sound {
        ground: Lch,
        ink: Lch,
        seed: u64,
        kind: Wave,
    },
}

/// The character of a sound pack's waveform.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Wave {
    /// Many small sharp hits.
    Rain,
    /// Long swells rising.
    Swell,
    /// Irregular, with a few events.
    Field,
    /// Low and even.
    Hiss,
}

/// One good.
#[derive(Clone, Debug, PartialEq)]
pub struct Good {
    /// Its number in the catalogue.
    pub no: u32,
    pub name: &'static str,
    pub aisle: Aisle,
    /// Its price for one seat of a personal licence, in cents.
    pub cents: u32,
    /// One line on what it is.
    pub blurb: &'static str,
    /// What is in it, as the label says.
    pub contents: &'static str,
    /// Who made what it is made of: a face's designers.
    pub credit: Option<&'static str>,
    /// How many files.
    pub files: u32,
    /// What it weighs, exactly.
    pub bytes: u64,
    /// Words a search finds it by, beside its name and aisle.
    pub tags: &'static [&'static str],
    pub look: Look,
    /// Where it stands when the shelf is in its featured order.
    pub featured: u32,
}

const VERDIGRIS: [Lch; 6] = [
    (0.30, 0.045, 178.0),
    (0.43, 0.075, 172.0),
    (0.57, 0.090, 166.0),
    (0.71, 0.085, 160.0),
    (0.84, 0.050, 150.0),
    (0.74, 0.110, 82.0),
];
const NOCTURNE: [Lch; 6] = [
    (0.19, 0.030, 268.0),
    (0.29, 0.050, 264.0),
    (0.41, 0.060, 256.0),
    (0.57, 0.050, 242.0),
    (0.80, 0.140, 78.0),
    (0.92, 0.060, 92.0),
];
const PAPER_RUST: [Lch; 6] = [
    (0.24, 0.020, 55.0),
    (0.35, 0.080, 36.0),
    (0.49, 0.130, 38.0),
    (0.63, 0.120, 46.0),
    (0.85, 0.030, 80.0),
    (0.95, 0.018, 88.0),
];
const SIGNAL_PASTEL: [Lch; 6] = [
    (0.32, 0.030, 272.0),
    (0.80, 0.070, 312.0),
    (0.84, 0.060, 205.0),
    (0.85, 0.070, 32.0),
    (0.87, 0.075, 152.0),
    (0.90, 0.085, 96.0),
];

const CAUSTIC: [Lch; 4] = [
    (0.24, 0.05, 230.0),
    (0.46, 0.09, 215.0),
    (0.78, 0.09, 195.0),
    (0.97, 0.03, 180.0),
];
const AURORA: [Lch; 4] = [
    (0.17, 0.035, 268.0),
    (0.42, 0.11, 200.0),
    (0.78, 0.16, 155.0),
    (0.96, 0.07, 120.0),
];
const MOIRE: [Lch; 3] = [
    (0.21, 0.012, 45.0),
    (0.55, 0.010, 60.0),
    (0.95, 0.010, 85.0),
];
const LAVA: [Lch; 4] = [
    (0.28, 0.07, 28.0),
    (0.52, 0.18, 32.0),
    (0.76, 0.16, 58.0),
    (0.94, 0.09, 95.0),
];

/// Every good, in catalogue order.
pub static GOODS: [Good; 16] = [
    Good {
        no: 1,
        name: "Verdigris",
        aisle: Aisle::Palettes,
        cents: 900,
        blurb: "brass gone green: six patinas, from new metal to an old roof",
        contents: "6 swatches",
        credit: None,
        files: 3,
        bytes: 3_917,
        tags: &["green", "brass", "metal", "patina", "colour"],
        look: Look::Palette(&VERDIGRIS),
        featured: 5,
    },
    Good {
        no: 2,
        name: "Nocturne",
        aisle: Aisle::Palettes,
        cents: 1100,
        blurb: "a city after midnight: slate, sodium light and one lit window",
        contents: "6 swatches",
        credit: None,
        files: 3,
        bytes: 3_884,
        tags: &["night", "blue", "city", "dark", "colour"],
        look: Look::Palette(&NOCTURNE),
        featured: 9,
    },
    Good {
        no: 3,
        name: "Paper & Rust",
        aisle: Aisle::Palettes,
        cents: 900,
        blurb: "newsprint, iron oxide and the pencil that marked it up",
        contents: "6 swatches",
        credit: None,
        files: 3,
        bytes: 3_902,
        tags: &["rust", "paper", "red", "warm", "colour"],
        look: Look::Palette(&PAPER_RUST),
        featured: 1,
    },
    Good {
        no: 4,
        name: "Signal Pastel",
        aisle: Aisle::Palettes,
        cents: 1200,
        blurb: "the colours of hazard tape, left out in the sun for a summer",
        contents: "6 swatches",
        credit: None,
        files: 3,
        bytes: 3_951,
        tags: &["pastel", "soft", "pink", "mint", "colour"],
        look: Look::Palette(&SIGNAL_PASTEL),
        featured: 13,
    },
    Good {
        no: 5,
        name: "Caustic",
        aisle: Aisle::Shaders,
        cents: 1800,
        blurb: "light through moving water: the bright web on a pool's floor",
        contents: "1 fragment shader, 4 presets",
        credit: None,
        files: 6,
        bytes: 12_804,
        tags: &["water", "pool", "light", "blue", "animated"],
        look: Look::Shader {
            style: Style::Caustic,
            ramp: &CAUSTIC,
            seed: 0xca05_7c00,
        },
        featured: 4,
    },
    Good {
        no: 6,
        name: "Aurora",
        aisle: Aisle::Shaders,
        cents: 2200,
        blurb: "curtains of light over a dark field, their hems wavering",
        contents: "1 fragment shader, 3 presets",
        credit: None,
        files: 5,
        bytes: 14_233,
        tags: &["sky", "night", "green", "light", "animated"],
        look: Look::Shader {
            style: Style::Aurora,
            ramp: &AURORA,
            seed: 0xa070_7a00,
        },
        featured: 0,
    },
    Good {
        no: 7,
        name: "Moiré No. 2",
        aisle: Aisle::Shaders,
        cents: 1600,
        blurb: "two sets of rings, beating against each other in ink",
        contents: "1 fragment shader, 2 presets",
        credit: None,
        files: 4,
        bytes: 9_118,
        tags: &["moire", "rings", "ink", "optical", "animated"],
        look: Look::Shader {
            style: Style::Moire,
            ramp: &MOIRE,
            seed: 0x0010_e002,
        },
        featured: 12,
    },
    Good {
        no: 8,
        name: "Lava",
        aisle: Aisle::Shaders,
        cents: 1800,
        blurb: "blobs that merge and part, very slowly, for ever",
        contents: "1 fragment shader, 3 presets",
        credit: None,
        files: 5,
        bytes: 11_472,
        tags: &["lamp", "warm", "orange", "blobs", "animated"],
        look: Look::Shader {
            style: Style::Lava,
            ramp: &LAVA,
            seed: 0x01a7_a000,
        },
        featured: 8,
    },
    Good {
        no: 9,
        name: "Xanh Mono",
        aisle: Aisle::Type,
        cents: 1500,
        blurb: "a mono with a calligrapher's wrist. this shop is set in it",
        contents: "1 poster, A2, roman and italic",
        credit: Some("yellow type foundry"),
        files: 2,
        bytes: 6_118_402,
        tags: &["font", "serif", "italic", "poster", "ofl"],
        look: Look::Specimen {
            face: Face::Xanh,
            paper: (0.97, 0.012, 88.0),
            ink: (0.21, 0.012, 45.0),
        },
        featured: 2,
    },
    Good {
        no: 10,
        name: "Major Mono",
        aisle: Aisle::Type,
        cents: 1500,
        blurb: "capitals and lower case, cut to one geometric width",
        contents: "1 poster, A2",
        credit: Some("emre parlak"),
        files: 2,
        bytes: 5_407_311,
        tags: &["font", "display", "geometric", "poster", "ofl"],
        look: Look::Specimen {
            face: Face::Major,
            paper: (0.23, 0.012, 45.0),
            ink: (0.95, 0.012, 85.0),
        },
        featured: 7,
    },
    Good {
        no: 11,
        name: "IBM Plex Mono",
        aisle: Aisle::Type,
        cents: 1500,
        blurb: "an engineer's mono, drawn for a company that sold machines",
        contents: "1 poster, A2",
        credit: Some("ibm, with bold monday"),
        files: 2,
        bytes: 5_932_047,
        tags: &["font", "sans", "code", "poster", "ofl"],
        look: Look::Specimen {
            face: Face::Plex,
            paper: (0.50, 0.15, 262.0),
            ink: (0.97, 0.012, 88.0),
        },
        featured: 11,
    },
    Good {
        no: 12,
        name: "JetBrains Mono",
        aisle: Aisle::Type,
        cents: 1500,
        blurb: "made for reading code all day. it sets this shop's small print",
        contents: "1 poster, A2",
        credit: Some("jetbrains"),
        files: 2,
        bytes: 5_288_930,
        tags: &["font", "code", "sans", "poster", "ofl"],
        look: Look::Specimen {
            face: Face::JetBrains,
            paper: (0.86, 0.06, 95.0),
            ink: (0.21, 0.012, 45.0),
        },
        featured: 15,
    },
    Good {
        no: 13,
        name: "Rain on Tin",
        aisle: Aisle::Sound,
        cents: 1900,
        blurb: "a tin roof in a passing shower, close-miked, 24-bit",
        contents: "12 loops, 0:24 each",
        credit: None,
        files: 24,
        bytes: 31_402_118,
        tags: &["rain", "weather", "roof", "loops", "ambient"],
        look: Look::Sound {
            ground: (0.36, 0.030, 240.0),
            ink: (0.88, 0.030, 220.0),
            seed: 0x7a10,
            kind: Wave::Rain,
        },
        featured: 3,
    },
    Good {
        no: 14,
        name: "Modular Dawn",
        aisle: Aisle::Sound,
        cents: 2400,
        blurb: "patch cables, slow envelopes and a sequencer waking up",
        contents: "16 loops, 0:32 each",
        credit: None,
        files: 32,
        bytes: 48_219_604,
        tags: &["synth", "modular", "drone", "loops", "electronic"],
        look: Look::Sound {
            ground: (0.86, 0.070, 70.0),
            ink: (0.30, 0.060, 40.0),
            seed: 0xda57,
            kind: Wave::Swell,
        },
        featured: 6,
    },
    Good {
        no: 15,
        name: "Fernwood Field Recording",
        aisle: Aisle::Sound,
        cents: 2100,
        blurb: "a morning in the woods behind the studio: wind, wrens, a gate",
        contents: "8 takes, 1:40 each",
        credit: None,
        files: 16,
        bytes: 41_877_250,
        tags: &["birds", "woods", "wind", "nature", "ambient"],
        look: Look::Sound {
            ground: (0.90, 0.030, 140.0),
            ink: (0.33, 0.060, 160.0),
            seed: 0xf1e1,
            kind: Wave::Field,
        },
        featured: 14,
    },
    Good {
        no: 16,
        name: "Tape Hiss",
        aisle: Aisle::Sound,
        cents: 700,
        blurb: "the sound of nothing, recorded lovingly onto cassette",
        contents: "4 loops, 1:00 each",
        credit: None,
        files: 8,
        bytes: 18_930_022,
        tags: &["tape", "noise", "lofi", "cassette", "texture"],
        look: Look::Sound {
            ground: (0.22, 0.012, 45.0),
            ink: (0.80, 0.020, 80.0),
            seed: 0x7a9e,
            kind: Wave::Hiss,
        },
        featured: 10,
    },
];

/// The good the front of the shop shows.
pub const FEATURED: usize = 5;

/// The orders a shelf can be put in.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sort {
    Featured,
    PriceUp,
    PriceDown,
}

impl Sort {
    pub const ALL: [Sort; 3] = [Sort::Featured, Sort::PriceUp, Sort::PriceDown];
    /// The names the sort pick offers, in the order of [`Sort::ALL`].
    pub const NAMES: &'static [&'static str] = &["featured", "price ↑", "price ↓"];
}

/// Whether `good` is found by `query`: every word of it the start of a word in the good's name,
/// aisle, blurb or tags, case and accents set aside. An empty query finds everything.
pub fn matches(good: &Good, query: &str) -> bool {
    let haystack = format!(
        "{} {} {} {} {}",
        good.name,
        good.aisle.name(),
        good.aisle.single(),
        good.blurb,
        good.tags.join(" ")
    )
    .to_lowercase();
    let haystack = fold(&haystack);
    let words: Vec<&str> = haystack
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    // Each word asked for begins a word of the good's: "rain" finds rain, not JetBrains.
    fold(&query.to_lowercase())
        .split_whitespace()
        .all(|asked| words.iter().any(|word| word.starts_with(asked)))
}

/// Accents set aside, so "moire" finds "Moiré".
fn fold(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'é' | 'è' | 'ê' => 'e',
            'á' | 'à' => 'a',
            other => other,
        })
        .collect()
}

/// Which goods a shelf shows -- of `aisle`, or every aisle, found by `query` -- in `sort`'s order,
/// as indices into [`GOODS`].
pub fn shelf(aisle: Option<Aisle>, query: &str, sort: Sort) -> Vec<usize> {
    let mut found: Vec<usize> = (0..GOODS.len())
        .filter(|&n| aisle.is_none_or(|aisle| GOODS[n].aisle == aisle))
        .filter(|&n| matches(&GOODS[n], query))
        .collect();
    match sort {
        Sort::Featured => found.sort_by_key(|&n| GOODS[n].featured),
        // Ties are broken by the catalogue's own order, so the shelf never shuffles on its own.
        Sort::PriceUp => found.sort_by_key(|&n| (GOODS[n].cents, GOODS[n].no)),
        Sort::PriceDown => found.sort_by_key(|&n| (std::cmp::Reverse(GOODS[n].cents), GOODS[n].no)),
    }
    found
}

/// The goods to show beside `good`: the rest of its aisle, in catalogue order.
pub fn related(good: usize) -> Vec<usize> {
    (0..GOODS.len())
        .filter(|&n| n != good && GOODS[n].aisle == GOODS[good].aisle)
        .collect()
}

/// A price as a label prints it: whole euros bare, anything else to the cent.
pub fn price(cents: u32) -> String {
    match cents % 100 {
        0 => format!("€{}", cents / 100),
        _ => money(cents),
    }
}

/// A sum of money, always to the cent.
pub fn money(cents: u32) -> String {
    format!("€{}.{:02}", cents / 100, cents % 100)
}

/// A weight, as a label prints it: bytes, kilobytes or megabytes, to a tenth.
pub fn weight(bytes: u64) -> String {
    match bytes {
        0..1_000 => format!("{bytes} B"),
        1_000..1_000_000 => format!("{:.1} kB", bytes as f64 / 1e3),
        _ => format!("{:.1} MB", bytes as f64 / 1e6),
    }
}

/// A weight to the byte, grouped in threes: what the contents panel prints.
pub fn exact(bytes: u64) -> String {
    let digits = bytes.to_string();
    let mut out = String::new();
    for (n, c) in digits.chars().enumerate() {
        if n > 0 && (digits.len() - n).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    format!("{out} bytes")
}

/// How long one of a sound pack's loops runs, in seconds, as its contents say: the `m:ss` in them.
/// Nothing for anything else.
pub fn seconds(good: &Good) -> u32 {
    good.contents
        .split([' ', ','])
        .find_map(|word| {
            let (m, s) = word.split_once(':')?;
            Some(m.parse::<u32>().ok()? * 60 + s.parse::<u32>().ok()?)
        })
        .unwrap_or(0)
}

/// The heights of a sound pack's waveform: `bars` of them in `0.08..=1.0`, the same every time.
pub fn waveform(seed: u64, kind: Wave, bars: usize, t: f32) -> Vec<f32> {
    let mut rng = arboretum::genart::Seed::new(seed);
    (0..bars)
        .map(|n| {
            let x = n as f32 / bars.max(1) as f32;
            let jitter = rng.unit();
            let wobble = (t * 2.3 + n as f32 * 0.7).sin() * 0.5 + 0.5;
            let base = match kind {
                Wave::Rain => 0.25 + 0.55 * jitter * jitter + 0.15 * wobble,
                Wave::Swell => {
                    let swell = (x * std::f32::consts::PI * 2.0 - 0.6 + t * 0.4).sin() * 0.5 + 0.5;
                    0.18 + 0.7 * swell * (0.75 + 0.25 * jitter)
                }
                Wave::Field => {
                    let event = if jitter > 0.86 { 0.7 } else { 0.0 };
                    0.2 + 0.25 * jitter + event * (0.6 + 0.4 * wobble)
                }
                Wave::Hiss => 0.22 + 0.12 * jitter + 0.06 * wobble,
            };
            base.clamp(0.08, 1.0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_good_is_numbered_in_order_and_featured_once() {
        for (n, good) in GOODS.iter().enumerate() {
            assert_eq!(good.no as usize, n + 1);
        }
        let mut featured: Vec<u32> = GOODS.iter().map(|g| g.featured).collect();
        featured.sort();
        assert_eq!(featured, (0..16).collect::<Vec<_>>());
        assert_eq!(
            GOODS[FEATURED].featured, 0,
            "the front shows the first featured good"
        );
    }

    #[test]
    fn four_aisles_of_four() {
        for aisle in Aisle::ALL {
            assert_eq!(
                GOODS.iter().filter(|g| g.aisle == aisle).count(),
                4,
                "{aisle:?}"
            );
        }
    }

    #[test]
    fn a_shelf_filters_by_aisle_and_words() {
        assert_eq!(shelf(None, "", Sort::Featured).len(), 16);
        let shaders = shelf(Some(Aisle::Shaders), "", Sort::Featured);
        assert_eq!(shaders.len(), 4);
        assert!(shaders.iter().all(|&n| GOODS[n].aisle == Aisle::Shaders));
        // A word from a tag, and from a name, in any case.
        let rain = shelf(None, "RAIN", Sort::Featured);
        assert_eq!(rain, vec![12]);
        assert_eq!(
            shelf(None, "moire", Sort::Featured),
            vec![6],
            "accents are set aside"
        );
        // Every word must be found.
        assert_eq!(shelf(None, "night blue", Sort::Featured), vec![1]);
        assert!(shelf(None, "plutonium", Sort::Featured).is_empty());
        assert!(shelf(Some(Aisle::Type), "rain", Sort::Featured).is_empty());
    }

    #[test]
    fn a_shelf_sorts_by_price_either_way_and_ties_keep_catalogue_order() {
        let up = shelf(None, "", Sort::PriceUp);
        assert!(
            up.windows(2)
                .all(|w| GOODS[w[0]].cents <= GOODS[w[1]].cents)
        );
        assert_eq!(GOODS[up[0]].name, "Tape Hiss");
        let down = shelf(None, "", Sort::PriceDown);
        assert_eq!(GOODS[down[0]].name, "Modular Dawn");
        // The four specimens cost the same, and stand in catalogue order both ways.
        let specimens: Vec<u32> = down
            .iter()
            .filter(|&&n| GOODS[n].aisle == Aisle::Type)
            .map(|&n| GOODS[n].no)
            .collect();
        assert_eq!(specimens, vec![9, 10, 11, 12]);
        let featured = shelf(None, "", Sort::Featured);
        assert_eq!(featured[0], FEATURED);
    }

    #[test]
    fn prices_and_weights_print_as_a_label_does() {
        assert_eq!(price(1500), "€15");
        assert_eq!(price(1275), "€12.75");
        assert_eq!(money(1500), "€15.00");
        assert_eq!(money(5), "€0.05");
        assert_eq!(weight(812), "812 B");
        assert_eq!(weight(3_917), "3.9 kB");
        assert_eq!(weight(31_402_118), "31.4 MB");
        assert_eq!(exact(31_402_118), "31,402,118 bytes");
        assert_eq!(exact(917), "917 bytes");
    }

    #[test]
    fn a_loop_runs_as_long_as_its_contents_say() {
        assert_eq!(seconds(&GOODS[12]), 24);
        assert_eq!(seconds(&GOODS[14]), 100);
        assert_eq!(seconds(&GOODS[0]), 0);
    }

    #[test]
    fn related_goods_share_the_aisle() {
        let related = related(5);
        assert_eq!(related.len(), 3);
        assert!(
            related
                .iter()
                .all(|&n| GOODS[n].aisle == Aisle::Shaders && n != 5)
        );
    }

    #[test]
    fn a_waveform_is_the_same_every_time_and_in_range() {
        for kind in [Wave::Rain, Wave::Swell, Wave::Field, Wave::Hiss] {
            let a = waveform(7, kind, 32, 0.0);
            assert_eq!(a, waveform(7, kind, 32, 0.0));
            assert_eq!(a.len(), 32);
            assert!(a.iter().all(|h| (0.08..=1.0).contains(h)));
        }
    }
}
