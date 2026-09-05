//! Image -- pixels, fitted into a box.

use bevy_ecs::component::Component;
use tracing::info;

use crate::coordinate::{Area, Section};
use crate::elm::{Chlorophyll, Pigment};
use crate::op::Bud;
use crate::place::{Boxed, Caller, Placement, Places};
use crate::rounding::Corners;
use crate::seed::Buds;

/// A registered picture: one bitmap, at whatever size it was decoded.
///
/// Handed out by [`Foliage::image`](crate::Foliage::image) at boot or by
/// [`Grove::image`](crate::Grove::image) at any frame after it, and named on an element with
/// [`Image::new`]. Opaque: there is nothing to be done with one but draw it.
///
/// A name is valid the moment it is handed out, whether or not its pixels have arrived: an element
/// drawing a plate with nothing behind it yet occupies its box and draws nothing, and appears when
/// the pixels do. That is what lets a name be taken now and filled from a fetch that has not
/// finished, without an app holding an "is it loaded" flag of its own.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Plate(pub(crate) u32);

/// How a picture's own pixels are fitted into the box its placement resolved to.
///
/// Three answers, because the two ratios disagreeing has three reasonable readings and no default
/// that is right for all of them.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Fit {
    /// Scaled to sit inside the box, keeping its own ratio, centred, leaving the rest of the box
    /// empty. Every pixel of the picture is shown.
    #[default]
    Aspect,
    /// Scaled to fill the box, keeping its own ratio, centred, with the overflowing axis cropped.
    /// Every pixel of the box is covered.
    Crop,
    /// Stretched to the box exactly, distorting the picture where the ratios disagree.
    Stretch,
}

/// A picture, drawn into its box.
///
/// ```no_run
/// # use foliage::{Boxed, Fit, Image, Location, Plate, Rounding, Source, left, top};
/// # fn f(avatar: Plate) {
/// Image::new(avatar)
///     .fit(Fit::Crop)
///     .rounding(Rounding::Full)
///     .at(Location::new().xs(
///         left(0.px()).width(40.px()),
///         top(0.px()).height(40.px()),
///     ));
/// # }
/// ```
///
/// Its corners round exactly as a [`Panel`](crate::Panel)'s do, through the same field, so a
/// full-bleed picture sits flush inside a rounded card rather than nearly flush. It has no
/// [`color`](crate::Grow::color): a picture carries its own, and a fill would be a second opinion
/// about what it is. Its opacity, and the opacity of everything it is grown under, still apply.
#[derive(Clone, Debug)]
pub struct Image {
    pub(crate) placement: Placement,
    pub(crate) plate: Plate,
    pub(crate) fit: Fit,
    pub(crate) rounding: Corners,
}

impl Image {
    /// A picture drawing the registered `plate`, fitted inside its box with square corners.
    pub fn new(plate: Plate) -> Self {
        Self {
            placement: Placement::default(),
            plate,
            fit: Fit::default(),
            rounding: Corners::default(),
        }
    }

    /// How its pixels are fitted into its box. Undeclared, it is [`Fit::Aspect`].
    pub fn fit(mut self, fit: Fit) -> Self {
        self.fit = fit;
        self
    }

    /// How its corners are rounded, per corner or all at once. Undeclared, they are square.
    pub fn rounding(mut self, rounding: impl Into<Corners>) -> Self {
        self.rounding = rounding.into();
        self
    }
}

impl Places for Image {
    fn placement(&mut self) -> &mut Placement {
        &mut self.placement
    }
}

impl Boxed for Image {}

impl Buds for Image {
    fn bud(self, at: Caller) -> Bud {
        Bud {
            chlorophyll: Chlorophyll::Image,
            pigment: Some(Pigment::Image(ImagePigment {
                plate: self.plate,
                fit: self.fit,
                rounding: self.rounding,
            })),
            placement: self.placement,
            at,
            ..Bud::bare()
        }
    }
}

/// What the image renderer was told.
///
/// Grown alongside [`Chlorophyll::Image`] and by nothing else, so an element carries both or
/// neither.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub(crate) struct ImagePigment {
    pub(crate) plate: Plate,
    pub(crate) fit: Fit,
    pub(crate) rounding: Corners,
}

/// One picture, as extraction states it.
///
/// The section here is already the box the pixels are drawn into rather than the box the element
/// resolved to -- [`Fit::Aspect`] shrinks it to the picture's own ratio -- because fitting is a
/// question about the picture's dimensions, and those are the engine's to know. What is left to the
/// backend is which texture to bind and what part of it to sample.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct ImageInstance {
    pub(crate) section: Section,
    /// What part of the picture is shown, as a fraction of it: origin then size. The whole of it
    /// except under [`Fit::Crop`], which is the one fit that leaves pixels out.
    pub(crate) crop: [f32; 4],
    pub(crate) radii: [f32; 4],
    /// The resolved opacity of the element and everything above it. Carried rather than folded into
    /// a colour, because a picture has no colour of its own for it to be folded into.
    pub(crate) opacity: f32,
    pub(crate) plate: Plate,
}

/// Every registered picture's pixels.
///
/// Held on the engine rather than in the backend for the same reason a font's bytes are: a picture
/// may be registered before there is a device to upload it to. The backend takes what is here the
/// first time it draws one.
#[derive(Default)]
pub(crate) struct Plates {
    pictures: Vec<Option<Picture>>,
}

/// One registered picture.
pub(crate) struct Picture {
    /// The pixels, `size.width` by `size.height` texels of RGBA, row-major.
    pub(crate) pixels: Vec<u8>,
    pub(crate) size: Area,
}

impl Plates {
    /// Takes a name for a picture whose pixels have not arrived, so an element can name it now.
    pub(crate) fn name(&mut self) -> Plate {
        self.pictures.push(None);
        Plate(self.pictures.len() as u32 - 1)
    }

    /// Fills a name with pixels, reporting whether it is one this registry handed out.
    ///
    /// Replaces whatever was there, which is what makes a picture swappable: an app that re-fetches
    /// at a higher resolution writes the same name again and every element drawing it follows.
    pub(crate) fn load(&mut self, plate: Plate, pixels: &[u8], size: Area) -> bool {
        let texels = (size.width.max(0.0) as usize) * (size.height.max(0.0) as usize) * 4;
        assert!(
            texels > 0 && pixels.len() >= texels,
            "a {}x{} picture is {texels} bytes of RGBA, and {} were given",
            size.width,
            size.height,
            pixels.len(),
        );
        let Some(slot) = self.pictures.get_mut(plate.0 as usize) else {
            return false;
        };
        *slot = Some(Picture {
            pixels: pixels.to_vec(),
            size,
        });
        info!(plate = plate.0, width = size.width, height = size.height, "image loaded");
        true
    }

    /// Fills a name from encoded bytes, reporting why it could not be where it could not.
    ///
    /// The refusing counterpart to [`load`](Plates::load), for pixels that arrived from a path or a
    /// URL rather than from something the program stated.
    pub(crate) fn decoded(&mut self, plate: Plate, bytes: &[u8]) -> Result<(), String> {
        let (pixels, size) = decode(bytes)?;
        let Some(slot) = self.pictures.get_mut(plate.0 as usize) else {
            return Err("no such plate".to_string());
        };
        *slot = Some(Picture { pixels, size });
        info!(plate = plate.0, width = size.width, height = size.height, "image loaded");
        Ok(())
    }

    /// The picture `plate` names, or `None` while its pixels have yet to arrive.
    pub(crate) fn picture(&self, plate: Plate) -> Option<&Picture> {
        self.pictures.get(plate.0 as usize)?.as_ref()
    }

    /// How large `plate` is, or `None` while its pixels have yet to arrive.
    pub(crate) fn size(&self, plate: Plate) -> Option<Area> {
        Some(self.picture(plate)?.size)
    }
}

/// PNG or JPEG bytes as RGBA and the size they turned out to be.
///
/// The format is read from the bytes rather than declared, because a name and a path are both things
/// that can be wrong about what a file holds, and the file cannot be. A picture states no size
/// coming this way for the same reason: it has one, and asking an app to repeat it is asking for a
/// second answer that can disagree.
pub(crate) fn decode(bytes: &[u8]) -> Result<(Vec<u8>, Area), String> {
    let decoded = image::load_from_memory(bytes)
        .map_err(|failed| format!("the picture could not be decoded: {failed}"))?
        .to_rgba8();
    let size = Area::new(decoded.width() as f32, decoded.height() as f32);
    Ok((decoded.into_raw(), size))
}
