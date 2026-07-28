use crate::anim::interpolation::Interpolations;
use crate::{Animate, Attachment, Component, Foliage};
use bevy_color::Alpha;
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
}
