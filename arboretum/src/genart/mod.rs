//! Pictures made from seeds.
//!
//! Everything here is arithmetic over bytes: no engine, no tree, nothing that needs a frame. A
//! picture is RGBA, row-major, ready for `Grow::load`. Which is also why all of it is tested
//! directly -- a picture that came out flat, or differently on a second run, is a failing test
//! rather than something to notice on screen.
//!
//! | Module | What it makes |
//! |---|---|
//! | [`Seed`] | the numbers, from a seed that can be printed and grown again |
//! | [`noise`] | smooth randomness: gradient noise and its fractal sums |
//! | [`Ramp`] | colours along a line, read through OKLab |
//! | [`Painter`] | a picture in one of eight [`Style`]s, painted a bounded step at a time |
//! | [`Denoise`] | a picture resolving out of static, as a diffusion model's does |
//! | [`sprite`] | light, shadow and gradients, for a renderer that draws none of them |

mod denoise;
pub mod noise;
mod paint;
mod ramp;
mod seed;
pub mod sprite;

pub use denoise::Denoise;
pub use paint::{Painter, Params, Style, paint};
pub use ramp::{Lut, Ramp, bytes};
pub use seed::Seed;
