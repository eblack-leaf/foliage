//! The three ways an element can be off, and the products they resolve to.
//!
//! They are genuinely different, and stating them together is the point:
//!
//! | | Draws | In the box stack | Receives input |
//! |---|---|---|---|
//! | `visible(false)` | no | no | no |
//! | `disable` | yes | **yes** | no -- swallows presses, passes scrolling on |
//! | `opacity(0.0)` | nothing to see | no | no |
//!
//! A disabled element still draws and still blocks, which is what makes it different from
//! decoration and what makes disabling a page enough on its own when a drawer opens over it. A
//! fully transparent one is not there at all, which closes the case where an element faded out went
//! on taking presses.
//!
//! Each is declared on one element and **inherited as a product** over the whole ancestry, computed
//! in [`inherit`](crate::rowan) every frame. Nothing has a cascade to write and nothing has one to
//! get wrong: an element grown under a disabled trunk is disabled on its first frame, and enabling
//! the trunk leaves anything disabled in its own right disabled.

/// Whether the app has hidden the element.
///
/// App intent, and only that. Being scrolled out of view is not a kind of hidden -- culling is a
/// decision extraction makes from the clip rect and is never recorded here, so content scrolled
/// past still counts toward its region's extent and can be scrolled back to.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Visible(pub(crate) bool);

impl Default for Visible {
    fn default() -> Self {
        Self(true)
    }
}

/// How opaque the element is, in `0.0..=1.0`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Opacity(pub(crate) f32);

impl Opacity {
    pub(crate) fn new(opacity: f32) -> Self {
        Self(opacity.clamp(0.0, 1.0))
    }
}

impl Default for Opacity {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Whether the element was disabled in its own right, as against by an ancestor.
///
/// Held separately from the product for exactly that reason: re-enabling an ancestor recomputes the
/// product over the whole ancestry, and an element that turned itself off stays off.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Disabled(pub(crate) bool);

/// What the three resolved to over an element's whole ancestry.
///
/// R7's one output, and what the box stack and extraction read. Nothing reads the declarations
/// directly, so there is no path by which an element could act on its own value while ignoring the
/// subtree it sits in. Held in resolution's own columns rather than on the element.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Inherited {
    pub(crate) visible: bool,
    /// The whole product, over every ancestor: how faint it looks, and what decides whether it is
    /// there at all.
    pub(crate) opacity: f32,
    pub(crate) disabled: bool,
    /// The picture it is drawn into, where something it is grown under -- or it -- is drawn
    /// [`together`](crate::Place::together) and fading: one more than the slot of the element
    /// that opened it, or zero.
    pub(crate) layer: u32,
    /// How opaque it is drawn into that picture: the product from the element that opened it down,
    /// that element's own and everything above it left out, since the picture is faded by those
    /// once, whole. The same as [`opacity`](Self::opacity) where there is no picture.
    pub(crate) drawn: f32,
}

impl Inherited {
    /// This element's own declarations resolved against what its trunk resolved to. `together`
    /// is the picture it would open, where it is declared [`together`](crate::Place::together).
    pub(crate) fn under(
        trunk: Inherited,
        visible: bool,
        opacity: f32,
        disabled: bool,
        together: Option<u32>,
    ) -> Self {
        let product = trunk.opacity * opacity;
        let (layer, drawn) = match (trunk.layer, together) {
            // Already in a picture: drawn into it, faded by everything between.
            (layer, _) if layer != 0 => (layer, trunk.drawn * opacity),
            // Opens one, but only while there is a fade to apply to it whole. At full opacity
            // there is nothing to show through, and at none there is nothing to show: in neither
            // is anything drawn apart, so a card faded out and left standing holds no picture.
            (_, Some(layer)) if product > 0.0 && product < 1.0 => (layer, 1.0),
            _ => (0, product),
        };
        Self {
            visible: trunk.visible && visible,
            opacity: product,
            disabled: trunk.disabled || disabled,
            layer,
            drawn,
        }
    }

    /// Whether the element opens the picture it is drawn into, as against being drawn into one
    /// something above it opened. `slot` is its own.
    pub(crate) fn opens(&self, slot: usize) -> bool {
        self.layer != 0 && self.layer == slot as u32 + 1
    }

    /// Whether the element is in the box stack at all.
    ///
    /// Hidden or fully transparent is not there, wherever the fade was declared -- on the element,
    /// on the root of the part it belongs to, or on anything above -- so fading something out is
    /// enough to be rid of it, and no part's inner structure decides whether a fade leaves an
    /// invisible control behind. A press target that shows nothing is said by what it draws
    /// rather than by fading it: a stem, or a clear color, at full opacity. Disabled is there: it
    /// blocks, which is the whole difference between a disabled control and decoration.
    pub(crate) fn present(&self) -> bool {
        self.visible && self.opacity > 0.0
    }
}

impl Default for Inherited {
    fn default() -> Self {
        Self {
            visible: true,
            opacity: 1.0,
            disabled: false,
            layer: 0,
            drawn: 1.0,
        }
    }
}
