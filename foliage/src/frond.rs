//! Frond -- one leaf, divided.
//!
//! Almost everything foliage grows is a single element. A few are several: the app names one
//! [`Leaf`] and the parts under it are grown, placed and hidden from the seed that named them, so
//! what one is made of is never a surface an app has to keep in step with. A frond is a leaf divided
//! into leaflets and is still one leaf, which is the whole of the idea.
//!
//! # The whole of what the frame knows about them
//!
//! Three questions, and the last two are asked of a *kind* rather than of an element, because both
//! are about every one of them at once:
//!
//! | | When | What it is for |
//! |---|---|---|
//! | [`Sprouts`] | the drain that grew it | grow my leaflets |
//! | [`Fronds::gestured`] | after dispatch | what did this frame's gestures mean to me |
//! | [`Fronds::settled`] | the end of the drain | put my leaflets back in step |
//!
//! [`FRONDS`] is the list, and it is the only place a kind is named. Adding one is a seed, an impl
//! and a line here -- [`fern`](crate::fern) walks the list and names none of them, so no pass of the
//! engine grows a branch for whatever one of them happens to want.
//!
//! # Why these two and no others
//!
//! Neither is a general hook for arbitrary work. `gestured` runs where reported gestures are still
//! standing and before the app's frame, so a frond reads interaction on the same terms an app does
//! and whatever the app queues afterwards still has the last word. `settled` runs where focus and
//! every queued write are final, so leaflets are put in step with state that cannot change again
//! this frame.
//!
//! Both **queue or write like anything else**. Neither reaches into resolution, and nothing about a
//! frond is an input to a pass that would otherwise not know it existed.

use crate::grove::Grove;
use crate::leaf::Leaf;
use crate::text_input;

/// What a frond grows underneath itself, carried by the [`Bud`](crate::op::Bud) from the call that
/// described it to the drain that grows it.
///
/// The whole of the drain's knowledge of divided elements: it grows the leaf the app named, hands
/// the rest to this, and knows nothing about what any of them are.
///
/// Called in the same drain step that grew the leaf rather than as more queued ops, because the
/// leaflets are not the app's to order against anything: a frond is one thing to plant, and the
/// frame that planted it is the frame the whole of it is live in.
pub(crate) trait Sprouts: Send + Sync + 'static {
    fn sprout(self: Box<Self>, grove: &mut Grove, leaf: Leaf);
}

/// One kind of divided leaf, and what the frame asks of every one of that kind.
///
/// Stateless: an implementor is a name for a kind and finds its own leaves in the tree. Both methods
/// are handed the whole [`Grove`] and are expected to do nothing with it an app could not.
pub(crate) trait Fronds: Sync {
    /// What this frame's reported gestures mean to leaves of this kind.
    ///
    /// A tap and a drag are ordinary gestures; what they *mean* is the reader's. Interaction reports
    /// where each landed and knows nothing about who is listening for it.
    fn gestured(&self, grove: &mut Grove);

    /// Puts every leaf of this kind back in step, once nothing can still change what they read.
    fn settled(&self, grove: &mut Grove);
}

/// Every kind of leaf that is divided.
///
/// The one place a kind is named. A second one is a line here and nothing else.
pub(crate) const FRONDS: &[&dyn Fronds] = &[&text_input::Field];

/// Asks every kind what this frame's gestures meant to it.
pub(crate) fn gestured(grove: &mut Grove) {
    for frond in FRONDS {
        frond.gestured(grove);
    }
}

/// Puts every kind's leaflets back in step.
pub(crate) fn settled(grove: &mut Grove) {
    for frond in FRONDS {
        frond.settled(grove);
    }
}
