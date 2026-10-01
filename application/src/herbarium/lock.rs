//! The lock: what stands under the key while the vault is shut.
//!
//! A passphrase field with the unlock press at its end, the key's width, where the sections will
//! stand once it is open; and a line over it for what the lock has to say -- here, what the
//! passphrase is, since this vault is a toy. The field is masked and emptied the moment what was
//! typed is handed up.
//!
//! The tips are held back while the lock stands: there is no section to open onto a vault that is
//! shut.

use foliage::{
    Boxed, Elevation, Font, Grove, Grow, Leaf, Location, Motion, Place, Pollen, Source, Stem,
    Timing, anchor, bottom, center_x, left, right, top,
};
use lichen::{Chip, Field, Press, STANDOFF, Say, Specimen, Voice, measure};

use super::vault::PASSPHRASE;
use crate::icons::Icons;

/// The room between the field and the press at its end.
const GAP: f32 = 8.0;
/// The cell the field's name is set in, in letters.
const CELL: f32 = 12.0;
/// How wide the lock is.
const WIDTH: f32 = 300.0;
/// How long the lock takes to go, and to come back.
const FADE_MS: u64 = 260;

/// The lock, grown, and whether it is standing.
pub(crate) struct Lock {
    stand: Leaf,
    field: Field,
    open: Chip,
    says: Say,
    shut: bool,
}

impl Lock {
    /// Grows the lock on `key`, standing, with focus in the field.
    pub(crate) fn grow(grove: &mut Grove, key: &Specimen, icons: &Icons, italic: Font) -> Self {
        let stand = grove.branch(
            key.root(),
            Stem::new()
                .at(place())
                .elevate(Elevation::up(6))
                .font_size(lichen::caption())
                .intangible()
                .focus_scope(),
        );
        let height = measure().height;
        let mut open = Chip::grow(
            grove,
            stand,
            Location::new().xs(
                right(100.pct()).width(Chip::width("unlock")),
                bottom(100.pct()).height(height.px()),
            ),
            Elevation::up(1),
            icons.unlock,
            "unlock",
        );
        open.arm(grove, Press::Inert);
        let field = Field::grow(
            grove,
            stand,
            Location::new().xs(
                left(0.px()).right(anchor().left() - GAP.px()),
                bottom(100.pct()).height(height.px()),
            ),
            CELL,
            "passphrase",
            false,
        );
        grove.anchor(field.leaf(), open.leaf());
        grove.masked(field.input(), true);
        let says = Say::grow(
            grove,
            stand,
            Location::new().xs(
                left(0.px()).right(100.pct()),
                top(0.px()).height(1.letters()),
            ),
            Some(italic),
        );
        field.focus(grove);
        let mut lock = Self {
            stand,
            field,
            open,
            says,
            shut: true,
        };
        lock.waiting(grove);
        lock
    }

    /// Carries the lock for a frame. Whether the passphrase was given and opened it.
    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) -> bool {
        if !self.shut {
            return false;
        }
        if self.field.frame(grove, pollen) {
            let typed = !self.field.value(grove).is_empty();
            self.open.arm(grove, Press::armed_if(typed));
        }
        let asked = self.open.pressed(pollen) || pollen.submitted(self.field.input());
        if !asked {
            return false;
        }
        let passphrase = self.field.value(grove);
        if passphrase.is_empty() {
            return false;
        }
        self.field.clear(grove);
        self.open.arm(grove, Press::Inert);
        if passphrase.trim() != PASSPHRASE {
            self.says.tell(
                grove,
                &format!("wrong passphrase · it is \"{PASSPHRASE}\""),
                Voice::Warn,
            );
            self.field.focus(grove);
            return false;
        }
        self.shut = false;
        self.says.tell(grove, "", Voice::Quiet);
        grove.disable(self.stand);
        grove.animate(
            self.stand,
            Motion::Opacity(0.0),
            Timing::ms(FADE_MS).ease(foliage::Ease::Accelerate),
        );
        true
    }

    /// The page came back up: focus goes back to the field, if the lock is standing.
    pub(crate) fn shown(&self, grove: &mut Grove) {
        if self.shut {
            self.field.focus(grove);
        }
    }

    /// The vault was locked: the lock comes back, emptied, with focus in it.
    pub(crate) fn locked(&mut self, grove: &mut Grove) {
        self.shut = true;
        self.field.clear(grove);
        self.open.arm(grove, Press::Inert);
        grove.enable(self.stand);
        grove.animate(
            self.stand,
            Motion::Opacity(1.0),
            Timing::ms(FADE_MS).ease(foliage::Ease::Decelerate),
        );
        self.field.focus(grove);
        self.waiting(grove);
    }

    fn waiting(&mut self, grove: &mut Grove) {
        self.says.tell(
            grove,
            &format!("shut · the passphrase is \"{PASSPHRASE}\""),
            Voice::Quiet,
        );
    }
}

/// Where the lock stands on the key: under it, centred, and wider than an upright key is, so the
/// field has room for what is typed.
fn place() -> Location {
    let tall = measure().height + GAP + 16.0;
    Location::new().xs(
        center_x(50.pct()).width(WIDTH.px()),
        top(100.pct() + STANDOFF.px()).height(tall.px()),
    )
}
