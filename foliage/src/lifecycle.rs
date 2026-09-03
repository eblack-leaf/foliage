//! The three ways an element can be off, and the products they resolve to.
//!
//! They are genuinely different, and stating them together is the point:
//!
//! | | Draws | In the box stack | Receives input |
//! |---|---|---|---|
//! | `visible(false)` | no | no | no |
//! | `disable` | yes | **yes** | no -- and swallows |
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

use bevy_ecs::component::Component;

/// Whether the app has hidden the element.
///
/// App intent, and only that. Being scrolled out of view is not a kind of hidden -- culling is a
/// decision extraction makes from the clip rect and is never recorded here, so content scrolled
/// past still counts toward its region's extent and can be scrolled back to.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub(crate) struct Visible(pub(crate) bool);

impl Default for Visible {
    fn default() -> Self {
        Self(true)
    }
}

/// How opaque the element is, in `0.0..=1.0`.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
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
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Disabled(pub(crate) bool);

/// What the three resolved to over an element's whole ancestry.
///
/// R7's one output, and what the box stack and extraction read. Nothing reads the declarations
/// directly, so there is no path by which an element could act on its own value while ignoring the
/// subtree it sits in.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub(crate) struct Inherited {
    pub(crate) visible: bool,
    pub(crate) opacity: f32,
    pub(crate) disabled: bool,
}

impl Inherited {
    /// This element's own declarations resolved against what its trunk resolved to.
    pub(crate) fn under(trunk: Inherited, visible: Visible, opacity: Opacity, disabled: Disabled) -> Self {
        Self {
            visible: trunk.visible && visible.0,
            opacity: trunk.opacity * opacity.0,
            disabled: trunk.disabled || disabled.0,
        }
    }

    /// Whether the element is in the box stack at all.
    ///
    /// Hidden or fully transparent is not there. Disabled is: it blocks, which is the whole
    /// difference between a disabled control and decoration.
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
        }
    }
}
