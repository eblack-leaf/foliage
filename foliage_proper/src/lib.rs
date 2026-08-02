// TODO: `Display`/`Debug` across the public types. `Section` and `Coordinates` have
// hand-written ones already (delegating Debug to the compact Display rather than the
// derived nested-struct form); the rest are uncovered. Ergonomics for consumers, not
// correctness, and purely additive -- so it can land any time after 1.0.
mod alignment;
mod anim;
mod ash;
mod asset;
mod attachment;
mod author;
mod boundary;
mod clipboard;
mod color;
mod coordinate;
mod disable;
mod enable;
mod foliage;
mod ginkgo;
mod grid;
mod icon;
mod image;
mod interaction;
mod line;
mod node;
mod opacity;
mod ops;
mod panel;
mod photosynthesis;
mod platform;
mod polygon;
mod polyline;
mod remove;
mod text;
mod text_input;
mod texture;
mod time;
mod tree;
mod virtual_keyboard;
mod visibility;
mod web_ext;
mod willow;
pub use crate::coordinate::{
    CoordinateContext, CoordinateUnit, Coordinates, Logical, Numerical, Physical,
    area::{Area, CReprArea},
    points::Points,
    position::{CReprPosition, Position},
    section::{CReprSection, LayoutSection, Section},
};
pub use anim::{
    Animate, Animation, Repeat,
    ease::{ControlPoints, Ease, Easement},
    interpolation::{Interpolation, Interpolations},
};
pub(crate) use ash::differential::Differential;
pub use asset::{Asset, AssetKey, AssetLoader, AssetSource, OnRetrieval};
pub(crate) use attachment::Attachment;
pub use author::Sprout;
pub(crate) use author::{Author, LeafSprout};
/// The ECS is an implementation detail from here on: nothing an app touches is a bevy type,
/// and nothing it writes needs one in scope. `pub(crate)` rather than deleted because the
/// engine's own modules are written against these names, and `foliage_macros` emits
/// `crate::bevy_ecs::...` paths for foliage's own internal event types.
pub(crate) use bevy_ecs::{self, prelude::*};
pub use boundary::bloom::Bloom;
pub use boundary::canopy::{Canopy, Sample, Sap};
pub use boundary::leaf::{Leaf, Presence};
pub use boundary::op::{Motion, Spec, Timing};
pub use boundary::sprig::Sprig;
pub use boundary::tween::{Channel, Tween};
pub use boundary::verbs::Grows;
/// bevy 0.17+ renamed the observer parameter `Trigger` to `On`. Every observer inside foliage
/// is written against the `Trigger<E>` spelling, so keep it as the canonical internal name.
pub(crate) type Trigger<'w, 't, E, B = ()> = bevy_ecs::observer::On<'w, 't, E, B>;
pub use alignment::{HorizontalAlignment, VerticalAlignment};
pub use ash::clip::ClipToViewport;
pub use clipboard::Clipboard;
pub use color::{CReprColor, Color, Luminance};
pub use coordinate::elevation::{Elevation, ResolvedElevation};
pub use disable::Disable;
pub use enable::Enable;
pub use foliage::Foliage;
pub use grid::{
    Adjust, Anchor, AnchorDeps, AnchorDescriptor, ConfigurationDescriptor, GridExt, Justify,
    LocationValue, ValueDescriptor,
};
pub use grid::{
    AspectRatio, Grid, Layout, Location, ScrollMomentum, ScrollProgress, ScrollTo, Short, View,
    anchor, text_content, view::OverscrollPropagation,
};
pub use icon::{Icon, IconId, IconMemory, IconSprout, IconValue};
pub use image::{Image, ImageMetrics, ImageSprout, ImageView};
pub use interaction::CurrentInteraction;
pub use interaction::{Disengaged, Dragged, Engaged, Focused, Unfocused};
pub use interaction::{
    FocusBehavior, InputSequence, Interaction, InteractionMethod, InteractionPhase,
    InteractionPropagation, Key, Modifiers, OnClick, PhysicalInputSequence, PhysicalKey,
    listener::InteractionListener, listener::InteractionShape, listener::InteractionState,
};
pub use line::{Line, LineSprout, MIN_LINE_WEIGHT};
pub use node::Bare;
pub(crate) use node::{Children, Node, Parent};
pub use opacity::Opacity;
pub use ops::Named;
pub use ops::{Keyring, Resolve, Resolved};
pub use panel::{Outline, Panel, PanelSprout, Rounding, Side};
#[cfg(target_os = "android")]
pub use platform::AndroidApp;
pub use platform::AndroidConnection;
pub use polygon::{Polygon, PolygonSprout};
pub use polyline::{
    DashPattern, Polyline, PolylineDrawProgress, PolylineDroppedPoints, PolylinePoints,
    PolylineSprout, PolylineStyle,
};
pub use text::GlyphOffset;
pub use text::monospaced::FontId;
pub use text::{
    FontSize, GlyphColors, Text, TextContentHeight, TextContentWidth, TextSprout, TextValue,
};
pub use text_input::action::{InputAction, TextInputAction};
pub use text_input::{
    HintColor, HintText, InsertText, LineConstraint, TextChanged, TextInput, TextInputSprout,
    TextInputStyle, keybindings::KeyBindings,
};
pub use time::{Moment, OnEnd, Time, TimeDelta, TimeMarker, Timer};
pub(crate) use tree::{AsTree, TargetedEvent, Tree};
pub use visibility::{InheritedVisibility, ResolvedVisibility, Visibility};
pub use web_ext::{Extensions, HrefLink};
