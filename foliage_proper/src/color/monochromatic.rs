use crate::color::Color;

pub trait Monochromatic {
    const PLUS_THREE: Color;
    const PLUS_TWO: Color;
    const PLUS_ONE: Color;
    const BASE: Color;
    const MINUS_ONE: Color;
    const MINUS_TWO: Color;
    const MINUS_THREE: Color;
}

pub struct Orange;

impl Monochromatic for Orange {
    const PLUS_THREE: Color = Color::rgb(1.0, 0.51, 0.302);
    const PLUS_TWO: Color = Color::rgb(1.0, 0.439, 0.20);
    const PLUS_ONE: Color = Color::rgb(1.0, 0.369, 0.102);
    const BASE: Color = Color::rgb(1.0, 0.298, 0.0);
    const MINUS_ONE: Color = Color::rgb(0.902, 0.267, 0.0);
    const MINUS_TWO: Color = Color::rgb(0.80, 0.239, 0.0);
    const MINUS_THREE: Color = Color::rgb(0.706, 0.208, 0.0);
}

pub struct AquaMarine;

impl Monochromatic for AquaMarine {
    const PLUS_THREE: Color = Color::rgb(0.627, 0.878, 0.792);
    const PLUS_TWO: Color = Color::rgb(0.549, 0.855, 0.749);
    const PLUS_ONE: Color = Color::rgb(0.475, 0.827, 0.71);
    const BASE: Color = Color::rgb(0.4, 0.804, 0.667);
    const MINUS_ONE: Color = Color::rgb(0.325, 0.78, 0.624);
    const MINUS_TWO: Color = Color::rgb(0.251, 0.753, 0.548);
    const MINUS_THREE: Color = Color::rgb(0.224, 0.682, 0.525);
}

pub struct FluorescentYellow;

impl Monochromatic for FluorescentYellow {
    const PLUS_THREE: Color = Color::rgb(0.859, 0.99, 0.302);
    const PLUS_TWO: Color = Color::rgb(0.839, 0.99, 0.2);
    const PLUS_ONE: Color = Color::rgb(0.82, 0.99, 0.102);
    const BASE: Color = Color::rgb(0.80, 0.95, 0.00);
    const MINUS_ONE: Color = Color::rgb(0.722, 0.902, 0.0);
    const MINUS_TWO: Color = Color::rgb(0.639, 0.80, 0.0);
    const MINUS_THREE: Color = Color::rgb(0.561, 0.702, 0.0);
}

pub struct StrongCyan;

impl Monochromatic for StrongCyan {
    const PLUS_THREE: Color = Color::rgb(0.00, 0.80, 0.99);
    const PLUS_TWO: Color = Color::rgb(0.00, 0.722, 0.902);
    const PLUS_ONE: Color = Color::rgb(0.00, 0.639, 0.804);
    const BASE: Color = Color::rgb(0.00, 0.561, 0.702);
    const MINUS_ONE: Color = Color::rgb(0.00, 0.482, 0.604);
    const MINUS_TWO: Color = Color::rgb(0.00, 0.40, 0.502);
    const MINUS_THREE: Color = Color::rgb(0.00, 0.322, 0.404);
}

pub struct Magenta;

impl Monochromatic for Magenta {
    const PLUS_THREE: Color = Color::rgb(0.9, 0.00, 0.793);
    const PLUS_TWO: Color = Color::rgb(0.805, 0.00, 0.710);
    const PLUS_ONE: Color = Color::rgb(0.710, 0.00, 0.630);
    const BASE: Color = Color::rgb(0.615, 0.00, 0.549);
    const MINUS_ONE: Color = Color::rgb(0.520, 0.00, 0.466);
    const MINUS_TWO: Color = Color::rgb(0.426, 0.00, 0.383);
    const MINUS_THREE: Color = Color::rgb(0.326, 0.00, 0.303);
}

pub struct Asparagus;

impl Monochromatic for Asparagus {
    const PLUS_THREE: Color = Color::rgb(0.682, 0.773, 0.608);
    const PLUS_TWO: Color = Color::rgb(0.631, 0.737, 0.545);
    const PLUS_ONE: Color = Color::rgb(0.58, 0.698, 0.482);
    const BASE: Color = Color::rgb(0.529, 0.663, 0.42);
    const MINUS_ONE: Color = Color::rgb(0.478, 0.62, 0.361);
    const MINUS_TWO: Color = Color::rgb(0.431, 0.557, 0.325);
    const MINUS_THREE: Color = Color::rgb(0.38, 0.494, 0.286);
}

pub struct MostlyDesaturatedDarkBlue;

impl Monochromatic for MostlyDesaturatedDarkBlue {
    const PLUS_THREE: Color = Color::rgb(0.42, 0.529, 0.663);
    const PLUS_TWO: Color = Color::rgb(0.361, 0.478, 0.62);
    const PLUS_ONE: Color = Color::rgb(0.322, 0.427, 0.557);
    const BASE: Color = Color::rgb(0.286, 0.38, 0.494);
    const MINUS_ONE: Color = Color::rgb(0.251, 0.333, 0.431);
    const MINUS_TWO: Color = Color::rgb(0.212, 0.282, 0.369);
    const MINUS_THREE: Color = Color::rgb(0.176, 0.235, 0.306);
}

pub struct DarkOliveGreen;

impl Monochromatic for DarkOliveGreen {
    const PLUS_THREE: Color = Color::rgb(0.498, 0.627, 0.275);
    const PLUS_TWO: Color = Color::rgb(0.443, 0.557, 0.247);
    const PLUS_ONE: Color = Color::rgb(0.388, 0.49, 0.216);
    const BASE: Color = Color::rgb(0.333, 0.42, 0.184);
    const MINUS_ONE: Color = Color::rgb(0.278, 0.349, 0.153);
    const MINUS_TWO: Color = Color::rgb(0.224, 0.282, 0.122);
    const MINUS_THREE: Color = Color::rgb(0.169, 0.212, 0.1);
}

pub struct Greyscale;

impl Monochromatic for Greyscale {
    const PLUS_THREE: Color = Color::rgb(0.45, 0.45, 0.45);
    const PLUS_TWO: Color = Color::rgb(0.40, 0.40, 0.40);
    const PLUS_ONE: Color = Color::rgb(0.35, 0.35, 0.35);
    const BASE: Color = Color::rgb(0.30, 0.30, 0.30);
    const MINUS_ONE: Color = Color::rgb(0.25, 0.25, 0.25);
    const MINUS_TWO: Color = Color::rgb(0.20, 0.20, 0.20);
    const MINUS_THREE: Color = Color::rgb(0.15, 0.15, 0.15);
}
