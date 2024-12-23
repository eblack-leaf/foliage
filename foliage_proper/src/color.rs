use crate::Component;

#[derive(Component, Copy, Clone, PartialEq)]
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
            Self::OneHundred
        } else if value >= 200 {
            Self::OneHundred
        } else if value >= 100 {
            Self::OneHundred
        } else {
            Self::Fifty
        }
    }
}
impl Color {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            value: bevy_color::Srgba::new(red, green, blue, alpha),
        }
    }
    pub fn r(&self) -> f32 {
        self.value.red
    }
    pub fn g(&self) -> f32 {
        self.value.green
    }
    pub fn b(&self) -> f32 {
        self.value.blue
    }
    pub fn a(&self) -> f32 {
        self.value.alpha
    }
    pub fn set_red(&mut self, red: f32) {
        self.value.red = red;
    }
    pub fn set_green(&mut self, green: f32) {
        self.value.green = green;
    }
    pub fn set_blue(&mut self, blue: f32) {
        self.value.blue = blue;
    }
    pub fn set_alpha(&mut self, alpha: f32) {
        self.value.alpha = alpha;
    }
    pub fn gray<L: Into<Luminance>>(luminance: L) -> Self {
        Self {
            value: match luminance.into() {
                Luminance::Fifty => bevy_color::palettes::tailwind::GRAY_50,
                Luminance::OneHundred => bevy_color::palettes::tailwind::GRAY_100,
                Luminance::TwoHundred => bevy_color::palettes::tailwind::GRAY_200,
                Luminance::ThreeHundred => bevy_color::palettes::tailwind::GRAY_300,
                Luminance::FourHundred => bevy_color::palettes::tailwind::GRAY_400,
                Luminance::FiveHundred => bevy_color::palettes::tailwind::GRAY_500,
                Luminance::SixHundred => bevy_color::palettes::tailwind::GRAY_600,
                Luminance::SevenHundred => bevy_color::palettes::tailwind::GRAY_700,
                Luminance::EightHundred => bevy_color::palettes::tailwind::GRAY_800,
                Luminance::NineHundred => bevy_color::palettes::tailwind::GRAY_900,
                Luminance::NineHundredFifty => bevy_color::palettes::tailwind::GRAY_950,
            },
        }
    }
    pub fn orange<L: Into<Luminance>>(luminance: L) -> Self {
        Self {
            value: match luminance.into() {
                Luminance::Fifty => bevy_color::palettes::tailwind::ORANGE_50,
                Luminance::OneHundred => bevy_color::palettes::tailwind::ORANGE_100,
                Luminance::TwoHundred => bevy_color::palettes::tailwind::ORANGE_200,
                Luminance::ThreeHundred => bevy_color::palettes::tailwind::ORANGE_300,
                Luminance::FourHundred => bevy_color::palettes::tailwind::ORANGE_400,
                Luminance::FiveHundred => bevy_color::palettes::tailwind::ORANGE_500,
                Luminance::SixHundred => bevy_color::palettes::tailwind::ORANGE_600,
                Luminance::SevenHundred => bevy_color::palettes::tailwind::ORANGE_700,
                Luminance::EightHundred => bevy_color::palettes::tailwind::ORANGE_800,
                Luminance::NineHundred => bevy_color::palettes::tailwind::ORANGE_900,
                Luminance::NineHundredFifty => bevy_color::palettes::tailwind::ORANGE_950,
            },
        }
    }
    pub fn green<L: Into<Luminance>>(luminance: L) -> Self {
        Self {
            value: match luminance.into() {
                Luminance::Fifty => bevy_color::palettes::tailwind::GREEN_50,
                Luminance::OneHundred => bevy_color::palettes::tailwind::GREEN_100,
                Luminance::TwoHundred => bevy_color::palettes::tailwind::GREEN_200,
                Luminance::ThreeHundred => bevy_color::palettes::tailwind::GREEN_300,
                Luminance::FourHundred => bevy_color::palettes::tailwind::GREEN_400,
                Luminance::FiveHundred => bevy_color::palettes::tailwind::GREEN_500,
                Luminance::SixHundred => bevy_color::palettes::tailwind::GREEN_600,
                Luminance::SevenHundred => bevy_color::palettes::tailwind::GREEN_700,
                Luminance::EightHundred => bevy_color::palettes::tailwind::GREEN_800,
                Luminance::NineHundred => bevy_color::palettes::tailwind::GREEN_900,
                Luminance::NineHundredFifty => bevy_color::palettes::tailwind::GREEN_950,
            },
        }
    }
}
