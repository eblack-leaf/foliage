use crate::anim::interpolation::Interpolations;
use crate::{Animate, Attachment, Component, Foliage};
use bevy_color::Alpha;
use crate::palette;
use bevy_color::palettes::tailwind;

/// An sRGBA color, and the color channel every primitive reads.
///
/// Usually built from the Tailwind palette rather than raw channels --
/// `Color::slate(500)`, `Color::amber(400)` -- so a UI draws from one consistent ramp.
/// See [`Luminance`] for how the number is read. [`Color::new`] takes raw 0..1 channels
/// when a palette entry won't do.
///
/// Animatable: tweening a `Color` interpolates each channel independently.
#[derive(Component, Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub value: bevy_color::Srgba,
}
impl Default for Color {
    fn default() -> Self {
        Self {
            value: bevy_color::Srgba::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}
impl Animate for Color {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new()
            .with(start.r(), end.r())
            .with(start.g(), end.g())
            .with(start.b(), end.b())
            .with(start.a(), start.a())
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(r) = interpolations.read(0) {
            self.set_red(r);
        }
        if let Some(g) = interpolations.read(1) {
            self.set_green(g);
        }
        if let Some(b) = interpolations.read(2) {
            self.set_blue(b);
        }
        if let Some(a) = interpolations.read(3) {
            self.set_alpha(a);
        }
    }
}
/// [`Color`] as four contiguous floats, laid out for upload to the GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CReprColor {
    pub value: [f32; 4],
}
impl From<Color> for CReprColor {
    fn from(color: Color) -> Self {
        Self {
            value: [color.r(), color.g(), color.b(), color.a()],
        }
    }
}
impl Attachment for Color {
    fn attach(foliage: &mut Foliage) {
        foliage.enable_animation::<Self>();
    }
}
impl Default for CReprColor {
    fn default() -> Self {
        Color::default().into()
    }
}
impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        wgpu::Color {
            r: color.value.red as f64,
            g: color.value.green as f64,
            b: color.value.blue as f64,
            a: color.value.alpha as f64,
        }
    }
}
/// A step on the Tailwind lightness ramp, from `Fifty` (lightest) to `NineHundredFifty`
/// (darkest).
///
/// Written as a plain number at the call site -- `Color::blue(600)` -- via the `From<i32>`
/// conversion, which rounds *down* to the nearest step, so any number in 600..700 gives
/// `SixHundred` and anything under 100 gives `Fifty`.
#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub enum Luminance {
    Fifty,
    OneHundred,
    TwoHundred,
    ThreeHundred,
    FourHundred,
    FiveHundred,
    SixHundred,
    SevenHundred,
    EightHundred,
    NineHundred,
    NineHundredFifty,
}
impl From<i32> for Luminance {
    fn from(value: i32) -> Self {
        if value >= 950 {
            Self::NineHundredFifty
        } else if value >= 900 {
            Self::NineHundred
        } else if value >= 800 {
            Self::EightHundred
        } else if value >= 700 {
            Self::SevenHundred
        } else if value >= 600 {
            Self::SixHundred
        } else if value >= 500 {
            Self::FiveHundred
        } else if value >= 400 {
            Self::FourHundred
        } else if value >= 300 {
            Self::ThreeHundred
        } else if value >= 200 {
            Self::TwoHundred
        } else if value >= 100 {
            Self::OneHundred
        } else {
            Self::Fifty
        }
    }
}
/// Defines one palette constructor: `Color::$name(luminance)` picking the matching step
/// off that hue's own ramp. One per Tailwind hue, generated below.
macro_rules! color_fn {
    ($name:ident: $c50:expr, $c100:expr, $c200:expr, $c300:expr, $c400:expr, $c500:expr, $c600:expr, $c700:expr, $c800:expr, $c900:expr, $c950:expr) => {
        pub fn $name<L: Into<Luminance>>(l: L) -> Self {
            Self {
                value: match l.into() {
                    Luminance::Fifty => $c50,
                    Luminance::OneHundred => $c100,
                    Luminance::TwoHundred => $c200,
                    Luminance::ThreeHundred => $c300,
                    Luminance::FourHundred => $c400,
                    Luminance::FiveHundred => $c500,
                    Luminance::SixHundred => $c600,
                    Luminance::SevenHundred => $c700,
                    Luminance::EightHundred => $c800,
                    Luminance::NineHundred => $c900,
                    Luminance::NineHundredFifty => $c950,
                },
            }
        }
    };
}
impl Color {
    /// Raw sRGBA channels, each 0..1. Prefer a palette constructor
    /// (`Color::slate(500)`) where one fits.
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            value: bevy_color::Srgba::new(red, green, blue, alpha),
        }
    }
    /// Red channel, 0..1.
    pub fn r(&self) -> f32 {
        self.value.red
    }
    /// Green channel, 0..1.
    pub fn g(&self) -> f32 {
        self.value.green
    }
    /// Blue channel, 0..1.
    pub fn b(&self) -> f32 {
        self.value.blue
    }
    /// Alpha channel, 0..1.
    pub fn a(&self) -> f32 {
        self.value.alpha
    }
    /// Scales the existing alpha by `value` rather than replacing it, so this composes:
    /// applying it twice at `0.5` leaves a quarter. For animating a whole subtree's
    /// transparency, use [`Opacity`](crate::Opacity) instead -- this is a fixed property
    /// of one color.
    pub fn with_opacity(mut self, value: f32) -> Self {
        self.value = self.value.with_alpha(value * self.a());
        self
    }
    /// Sets the red channel, 0..1.
    pub fn set_red(&mut self, red: f32) {
        self.value.red = red;
    }
    /// Sets the green channel, 0..1.
    pub fn set_green(&mut self, green: f32) {
        self.value.green = green;
    }
    /// Sets the blue channel, 0..1.
    pub fn set_blue(&mut self, blue: f32) {
        self.value.blue = blue;
    }
    /// Sets the alpha channel, 0..1.
    pub fn set_alpha(&mut self, alpha: f32) {
        self.value.alpha = alpha;
    }
    /// This color in its GPU-upload layout.
    pub fn c_repr(&self) -> CReprColor {
        CReprColor::from(*self)
    }
    color_fn!(red: tailwind::RED_50, tailwind::RED_100, tailwind::RED_200, tailwind::RED_300, tailwind::RED_400, tailwind::RED_500, tailwind::RED_600, tailwind::RED_700, tailwind::RED_800, tailwind::RED_900, tailwind::RED_950);
    color_fn!(amber: tailwind::AMBER_50, tailwind::AMBER_100, tailwind::AMBER_200, tailwind::AMBER_300, tailwind::AMBER_400, tailwind::AMBER_500, tailwind::AMBER_600, tailwind::AMBER_700, tailwind::AMBER_800, tailwind::AMBER_900, tailwind::AMBER_950);
    color_fn!(orange: tailwind::ORANGE_50, tailwind::ORANGE_100, tailwind::ORANGE_200, tailwind::ORANGE_300, tailwind::ORANGE_400, tailwind::ORANGE_500, tailwind::ORANGE_600, tailwind::ORANGE_700, tailwind::ORANGE_800, tailwind::ORANGE_900, tailwind::ORANGE_950);
    color_fn!(yellow: tailwind::YELLOW_50, tailwind::YELLOW_100, tailwind::YELLOW_200, tailwind::YELLOW_300, tailwind::YELLOW_400, tailwind::YELLOW_500, tailwind::YELLOW_600, tailwind::YELLOW_700, tailwind::YELLOW_800, tailwind::YELLOW_900, tailwind::YELLOW_950);
    color_fn!(lime: tailwind::LIME_50, tailwind::LIME_100, tailwind::LIME_200, tailwind::LIME_300, tailwind::LIME_400, tailwind::LIME_500, tailwind::LIME_600, tailwind::LIME_700, tailwind::LIME_800, tailwind::LIME_900, tailwind::LIME_950);
    color_fn!(green: tailwind::GREEN_50, tailwind::GREEN_100, tailwind::GREEN_200, tailwind::GREEN_300, tailwind::GREEN_400, tailwind::GREEN_500, tailwind::GREEN_600, tailwind::GREEN_700, tailwind::GREEN_800, tailwind::GREEN_900, tailwind::GREEN_950);
    color_fn!(emerald: tailwind::EMERALD_50, tailwind::EMERALD_100, tailwind::EMERALD_200, tailwind::EMERALD_300, tailwind::EMERALD_400, tailwind::EMERALD_500, tailwind::EMERALD_600, tailwind::EMERALD_700, tailwind::EMERALD_800, tailwind::EMERALD_900, tailwind::EMERALD_950);
    color_fn!(teal: tailwind::TEAL_50, tailwind::TEAL_100, tailwind::TEAL_200, tailwind::TEAL_300, tailwind::TEAL_400, tailwind::TEAL_500, tailwind::TEAL_600, tailwind::TEAL_700, tailwind::TEAL_800, tailwind::TEAL_900, tailwind::TEAL_950);
    color_fn!(cyan: tailwind::CYAN_50, tailwind::CYAN_100, tailwind::CYAN_200, tailwind::CYAN_300, tailwind::CYAN_400, tailwind::CYAN_500, tailwind::CYAN_600, tailwind::CYAN_700, tailwind::CYAN_800, tailwind::CYAN_900, tailwind::CYAN_950);
    color_fn!(sky: tailwind::SKY_50, tailwind::SKY_100, tailwind::SKY_200, tailwind::SKY_300, tailwind::SKY_400, tailwind::SKY_500, tailwind::SKY_600, tailwind::SKY_700, tailwind::SKY_800, tailwind::SKY_900, tailwind::SKY_950);
    color_fn!(blue: tailwind::BLUE_50, tailwind::BLUE_100, tailwind::BLUE_200, tailwind::BLUE_300, tailwind::BLUE_400, tailwind::BLUE_500, tailwind::BLUE_600, tailwind::BLUE_700, tailwind::BLUE_800, tailwind::BLUE_900, tailwind::BLUE_950);
    color_fn!(indigo: tailwind::INDIGO_50, tailwind::INDIGO_100, tailwind::INDIGO_200, tailwind::INDIGO_300, tailwind::INDIGO_400, tailwind::INDIGO_500, tailwind::INDIGO_600, tailwind::INDIGO_700, tailwind::INDIGO_800, tailwind::INDIGO_900, tailwind::INDIGO_950);
    color_fn!(violet: tailwind::VIOLET_50, tailwind::VIOLET_100, tailwind::VIOLET_200, tailwind::VIOLET_300, tailwind::VIOLET_400, tailwind::VIOLET_500, tailwind::VIOLET_600, tailwind::VIOLET_700, tailwind::VIOLET_800, tailwind::VIOLET_900, tailwind::VIOLET_950);
    color_fn!(purple: tailwind::PURPLE_50, tailwind::PURPLE_100, tailwind::PURPLE_200, tailwind::PURPLE_300, tailwind::PURPLE_400, tailwind::PURPLE_500, tailwind::PURPLE_600, tailwind::PURPLE_700, tailwind::PURPLE_800, tailwind::PURPLE_900, tailwind::PURPLE_950);
    color_fn!(fuchsia: tailwind::FUCHSIA_50, tailwind::FUCHSIA_100, tailwind::FUCHSIA_200, tailwind::FUCHSIA_300, tailwind::FUCHSIA_400, tailwind::FUCHSIA_500, tailwind::FUCHSIA_600, tailwind::FUCHSIA_700, tailwind::FUCHSIA_800, tailwind::FUCHSIA_900, tailwind::FUCHSIA_950);
    color_fn!(pink: tailwind::PINK_50, tailwind::PINK_100, tailwind::PINK_200, tailwind::PINK_300, tailwind::PINK_400, tailwind::PINK_500, tailwind::PINK_600, tailwind::PINK_700, tailwind::PINK_800, tailwind::PINK_900, tailwind::PINK_950);
    color_fn!(rose: tailwind::ROSE_50, tailwind::ROSE_100, tailwind::ROSE_200, tailwind::ROSE_300, tailwind::ROSE_400, tailwind::ROSE_500, tailwind::ROSE_600, tailwind::ROSE_700, tailwind::ROSE_800, tailwind::ROSE_900, tailwind::ROSE_950);
    color_fn!(slate: tailwind::SLATE_50, tailwind::SLATE_100, tailwind::SLATE_200, tailwind::SLATE_300, tailwind::SLATE_400, tailwind::SLATE_500, tailwind::SLATE_600, tailwind::SLATE_700, tailwind::SLATE_800, tailwind::SLATE_900, tailwind::SLATE_950);
    color_fn!(gray: tailwind::GRAY_50, tailwind::GRAY_100, tailwind::GRAY_200, tailwind::GRAY_300, tailwind::GRAY_400, tailwind::GRAY_500, tailwind::GRAY_600, tailwind::GRAY_700, tailwind::GRAY_800, tailwind::GRAY_900, tailwind::GRAY_950);
    color_fn!(zinc: tailwind::ZINC_50, tailwind::ZINC_100, tailwind::ZINC_200, tailwind::ZINC_300, tailwind::ZINC_400, tailwind::ZINC_500, tailwind::ZINC_600, tailwind::ZINC_700, tailwind::ZINC_800, tailwind::ZINC_900, tailwind::ZINC_950);
    color_fn!(neutral: tailwind::NEUTRAL_50, tailwind::NEUTRAL_100, tailwind::NEUTRAL_200, tailwind::NEUTRAL_300, tailwind::NEUTRAL_400, tailwind::NEUTRAL_500, tailwind::NEUTRAL_600, tailwind::NEUTRAL_700, tailwind::NEUTRAL_800, tailwind::NEUTRAL_900, tailwind::NEUTRAL_950);
    color_fn!(stone: tailwind::STONE_50, tailwind::STONE_100, tailwind::STONE_200, tailwind::STONE_300, tailwind::STONE_400, tailwind::STONE_500, tailwind::STONE_600, tailwind::STONE_700, tailwind::STONE_800, tailwind::STONE_900, tailwind::STONE_950);
    // Families beyond the Tailwind set, on the same eleven-step scale. Each one's hue and
    // chroma are calibrated against a reference hex for its name -- see [`crate::palette`].
    color_fn!(sand: palette::SAND_50, palette::SAND_100, palette::SAND_200, palette::SAND_300, palette::SAND_400, palette::SAND_500, palette::SAND_600, palette::SAND_700, palette::SAND_800, palette::SAND_900, palette::SAND_950);
    color_fn!(khaki: palette::KHAKI_50, palette::KHAKI_100, palette::KHAKI_200, palette::KHAKI_300, palette::KHAKI_400, palette::KHAKI_500, palette::KHAKI_600, palette::KHAKI_700, palette::KHAKI_800, palette::KHAKI_900, palette::KHAKI_950);
    color_fn!(olive: palette::OLIVE_50, palette::OLIVE_100, palette::OLIVE_200, palette::OLIVE_300, palette::OLIVE_400, palette::OLIVE_500, palette::OLIVE_600, palette::OLIVE_700, palette::OLIVE_800, palette::OLIVE_900, palette::OLIVE_950);
    color_fn!(gold: palette::GOLD_50, palette::GOLD_100, palette::GOLD_200, palette::GOLD_300, palette::GOLD_400, palette::GOLD_500, palette::GOLD_600, palette::GOLD_700, palette::GOLD_800, palette::GOLD_900, palette::GOLD_950);
    color_fn!(bronze: palette::BRONZE_50, palette::BRONZE_100, palette::BRONZE_200, palette::BRONZE_300, palette::BRONZE_400, palette::BRONZE_500, palette::BRONZE_600, palette::BRONZE_700, palette::BRONZE_800, palette::BRONZE_900, palette::BRONZE_950);
    color_fn!(copper: palette::COPPER_50, palette::COPPER_100, palette::COPPER_200, palette::COPPER_300, palette::COPPER_400, palette::COPPER_500, palette::COPPER_600, palette::COPPER_700, palette::COPPER_800, palette::COPPER_900, palette::COPPER_950);
    color_fn!(clay: palette::CLAY_50, palette::CLAY_100, palette::CLAY_200, palette::CLAY_300, palette::CLAY_400, palette::CLAY_500, palette::CLAY_600, palette::CLAY_700, palette::CLAY_800, palette::CLAY_900, palette::CLAY_950);
    color_fn!(terracotta: palette::TERRACOTTA_50, palette::TERRACOTTA_100, palette::TERRACOTTA_200, palette::TERRACOTTA_300, palette::TERRACOTTA_400, palette::TERRACOTTA_500, palette::TERRACOTTA_600, palette::TERRACOTTA_700, palette::TERRACOTTA_800, palette::TERRACOTTA_900, palette::TERRACOTTA_950);
    color_fn!(brown: palette::BROWN_50, palette::BROWN_100, palette::BROWN_200, palette::BROWN_300, palette::BROWN_400, palette::BROWN_500, palette::BROWN_600, palette::BROWN_700, palette::BROWN_800, palette::BROWN_900, palette::BROWN_950);
    color_fn!(sepia: palette::SEPIA_50, palette::SEPIA_100, palette::SEPIA_200, palette::SEPIA_300, palette::SEPIA_400, palette::SEPIA_500, palette::SEPIA_600, palette::SEPIA_700, palette::SEPIA_800, palette::SEPIA_900, palette::SEPIA_950);
    color_fn!(taupe: palette::TAUPE_50, palette::TAUPE_100, palette::TAUPE_200, palette::TAUPE_300, palette::TAUPE_400, palette::TAUPE_500, palette::TAUPE_600, palette::TAUPE_700, palette::TAUPE_800, palette::TAUPE_900, palette::TAUPE_950);
    color_fn!(peach: palette::PEACH_50, palette::PEACH_100, palette::PEACH_200, palette::PEACH_300, palette::PEACH_400, palette::PEACH_500, palette::PEACH_600, palette::PEACH_700, palette::PEACH_800, palette::PEACH_900, palette::PEACH_950);
    color_fn!(coral: palette::CORAL_50, palette::CORAL_100, palette::CORAL_200, palette::CORAL_300, palette::CORAL_400, palette::CORAL_500, palette::CORAL_600, palette::CORAL_700, palette::CORAL_800, palette::CORAL_900, palette::CORAL_950);
    color_fn!(salmon: palette::SALMON_50, palette::SALMON_100, palette::SALMON_200, palette::SALMON_300, palette::SALMON_400, palette::SALMON_500, palette::SALMON_600, palette::SALMON_700, palette::SALMON_800, palette::SALMON_900, palette::SALMON_950);
    color_fn!(blush: palette::BLUSH_50, palette::BLUSH_100, palette::BLUSH_200, palette::BLUSH_300, palette::BLUSH_400, palette::BLUSH_500, palette::BLUSH_600, palette::BLUSH_700, palette::BLUSH_800, palette::BLUSH_900, palette::BLUSH_950);
    color_fn!(crimson: palette::CRIMSON_50, palette::CRIMSON_100, palette::CRIMSON_200, palette::CRIMSON_300, palette::CRIMSON_400, palette::CRIMSON_500, palette::CRIMSON_600, palette::CRIMSON_700, palette::CRIMSON_800, palette::CRIMSON_900, palette::CRIMSON_950);
    color_fn!(wine: palette::WINE_50, palette::WINE_100, palette::WINE_200, palette::WINE_300, palette::WINE_400, palette::WINE_500, palette::WINE_600, palette::WINE_700, palette::WINE_800, palette::WINE_900, palette::WINE_950);
    color_fn!(chartreuse: palette::CHARTREUSE_50, palette::CHARTREUSE_100, palette::CHARTREUSE_200, palette::CHARTREUSE_300, palette::CHARTREUSE_400, palette::CHARTREUSE_500, palette::CHARTREUSE_600, palette::CHARTREUSE_700, palette::CHARTREUSE_800, palette::CHARTREUSE_900, palette::CHARTREUSE_950);
    color_fn!(moss: palette::MOSS_50, palette::MOSS_100, palette::MOSS_200, palette::MOSS_300, palette::MOSS_400, palette::MOSS_500, palette::MOSS_600, palette::MOSS_700, palette::MOSS_800, palette::MOSS_900, palette::MOSS_950);
    color_fn!(sage: palette::SAGE_50, palette::SAGE_100, palette::SAGE_200, palette::SAGE_300, palette::SAGE_400, palette::SAGE_500, palette::SAGE_600, palette::SAGE_700, palette::SAGE_800, palette::SAGE_900, palette::SAGE_950);
    color_fn!(forest: palette::FOREST_50, palette::FOREST_100, palette::FOREST_200, palette::FOREST_300, palette::FOREST_400, palette::FOREST_500, palette::FOREST_600, palette::FOREST_700, palette::FOREST_800, palette::FOREST_900, palette::FOREST_950);
    color_fn!(jade: palette::JADE_50, palette::JADE_100, palette::JADE_200, palette::JADE_300, palette::JADE_400, palette::JADE_500, palette::JADE_600, palette::JADE_700, palette::JADE_800, palette::JADE_900, palette::JADE_950);
    color_fn!(mint: palette::MINT_50, palette::MINT_100, palette::MINT_200, palette::MINT_300, palette::MINT_400, palette::MINT_500, palette::MINT_600, palette::MINT_700, palette::MINT_800, palette::MINT_900, palette::MINT_950);
    color_fn!(seafoam: palette::SEAFOAM_50, palette::SEAFOAM_100, palette::SEAFOAM_200, palette::SEAFOAM_300, palette::SEAFOAM_400, palette::SEAFOAM_500, palette::SEAFOAM_600, palette::SEAFOAM_700, palette::SEAFOAM_800, palette::SEAFOAM_900, palette::SEAFOAM_950);
    color_fn!(aqua: palette::AQUA_50, palette::AQUA_100, palette::AQUA_200, palette::AQUA_300, palette::AQUA_400, palette::AQUA_500, palette::AQUA_600, palette::AQUA_700, palette::AQUA_800, palette::AQUA_900, palette::AQUA_950);
    color_fn!(mist: palette::MIST_50, palette::MIST_100, palette::MIST_200, palette::MIST_300, palette::MIST_400, palette::MIST_500, palette::MIST_600, palette::MIST_700, palette::MIST_800, palette::MIST_900, palette::MIST_950);
    color_fn!(steel: palette::STEEL_50, palette::STEEL_100, palette::STEEL_200, palette::STEEL_300, palette::STEEL_400, palette::STEEL_500, palette::STEEL_600, palette::STEEL_700, palette::STEEL_800, palette::STEEL_900, palette::STEEL_950);
    color_fn!(azure: palette::AZURE_50, palette::AZURE_100, palette::AZURE_200, palette::AZURE_300, palette::AZURE_400, palette::AZURE_500, palette::AZURE_600, palette::AZURE_700, palette::AZURE_800, palette::AZURE_900, palette::AZURE_950);
    color_fn!(periwinkle: palette::PERIWINKLE_50, palette::PERIWINKLE_100, palette::PERIWINKLE_200, palette::PERIWINKLE_300, palette::PERIWINKLE_400, palette::PERIWINKLE_500, palette::PERIWINKLE_600, palette::PERIWINKLE_700, palette::PERIWINKLE_800, palette::PERIWINKLE_900, palette::PERIWINKLE_950);
    color_fn!(lavender: palette::LAVENDER_50, palette::LAVENDER_100, palette::LAVENDER_200, palette::LAVENDER_300, palette::LAVENDER_400, palette::LAVENDER_500, palette::LAVENDER_600, palette::LAVENDER_700, palette::LAVENDER_800, palette::LAVENDER_900, palette::LAVENDER_950);
    color_fn!(lilac: palette::LILAC_50, palette::LILAC_100, palette::LILAC_200, palette::LILAC_300, palette::LILAC_400, palette::LILAC_500, palette::LILAC_600, palette::LILAC_700, palette::LILAC_800, palette::LILAC_900, palette::LILAC_950);
    color_fn!(plum: palette::PLUM_50, palette::PLUM_100, palette::PLUM_200, palette::PLUM_300, palette::PLUM_400, palette::PLUM_500, palette::PLUM_600, palette::PLUM_700, palette::PLUM_800, palette::PLUM_900, palette::PLUM_950);
    color_fn!(mauve: palette::MAUVE_50, palette::MAUVE_100, palette::MAUVE_200, palette::MAUVE_300, palette::MAUVE_400, palette::MAUVE_500, palette::MAUVE_600, palette::MAUVE_700, palette::MAUVE_800, palette::MAUVE_900, palette::MAUVE_950);
}
