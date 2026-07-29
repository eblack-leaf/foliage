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
mod clipboard;
mod color;
mod composite;
mod coordinate;
mod disable;
mod enable;
mod foliage;
mod ginkgo;
mod grid;
mod icon;
mod image;
mod interaction;
mod leaf;
mod line;
mod opacity;
mod ops;
mod panel;
mod photosynthesis;
mod platform;
mod polygon;
mod remove;
mod text;
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
    section::{CReprSection, Section},
};
pub use anim::{
    Animate, Animation, Repeat,
    ease::{ControlPoints, Ease, Easement},
    interpolation::{Interpolation, Interpolations},
};
pub(crate) use ash::differential::Differential;
pub use asset::{Asset, AssetKey, AssetLoader, AssetSource, OnRetrieval};
pub use attachment::Attachment;
pub use author::{LeafSprout, Sprout, With};
pub use bevy_ecs::{self, prelude::*};
/// bevy 0.17+ renamed the observer parameter `Trigger` to `On`. Every observer in foliage and
/// its consumers is written against the `Trigger<E>` spelling, so keep it as the canonical
/// name here; `On` is also available (via the prelude re-export above) for new code.
pub type Trigger<'w, 't, E, B = ()> = bevy_ecs::observer::On<'w, 't, E, B>;
pub use alignment::{HorizontalAlignment, VerticalAlignment};
pub use ash::clip::ClipToViewport;
pub use clipboard::Clipboard;
pub use color::{CReprColor, Color, Luminance};
pub use composite::text_input::action::{InputAction, TextInputAction};
pub use composite::{
    IconValue, IndexedSlotFn, PageChanged, PageCount, PageIndex, Progress, SlotFn, TextValue,
};
pub use composite::{
    button::{Button, ButtonSprout, ButtonStyle, Engagement},
    card::{Card, CardSprout, CardStyle},
    router::{RouteFn, Router, RouterHandle, RouterRoutes, RouterSprout},
    slider::{ProgressChanged, Slider, SliderBehavior, SliderSprout, SliderStyle},
    text_input::{
        HintColor, HintText, InsertText, LineConstraint, TextChanged, TextInput, TextInputSprout,
        TextInputStyle, keybindings::KeyBindings,
    },
};
#[cfg(feature = "composite-extras")]
pub use composite::{
    carousel::{Carousel, CarouselConfig, CarouselPages, CarouselSprout, CarouselStyle},
    checkbox::{Checkbox, CheckboxSprout, CheckboxState, CheckboxStyle, Checked},
    dropdown::{
        Dropdown, DropdownConfig, DropdownOptions, DropdownSprout, DropdownStyle, Expanded,
        Selected, SelectionChanged,
    },
    list::{List, ListItems, ListLayout, ListSprout},
    pagination::{Pagination, PaginationMode, PaginationSprout, PaginationStyle},
    polyline::{
        DashPattern, Polyline, PolylineDrawProgress, PolylineDroppedPoints, PolylinePoints,
        PolylineSprout, PolylineStyle,
    },
    popover::{
        Popover, PopoverClosed, PopoverExpanded, PopoverOpened, PopoverPlacement, PopoverSprout,
        PopoverStyle,
    },
    radio_group::{
        RadioChanged, RadioGroup, RadioGroupSprout, RadioOptions, RadioSelected, RadioStyle,
    },
    segmented_control::{
        SegmentChanged, SegmentedControl, SegmentedControlSprout, SegmentedOptions,
        SegmentedSelected, SegmentedStyle,
    },
    tabs::{Tabs, TabsPages, TabsSprout, TabsStyle},
    toggle::{Toggle, ToggleSprout, ToggleState, ToggleStyle, Toggled},
};
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
pub use icon::{Icon, IconId, IconMemory, IconSprout};
pub use image::{Image, ImageMetrics, ImageSprout, ImageView};
pub use interaction::CurrentInteraction;
pub use interaction::{Disengaged, Dragged, Engaged, Focused, Unfocused};
pub use interaction::{
    FocusBehavior, InputSequence, Interaction, InteractionMethod, InteractionPhase,
    InteractionPropagation, Key, Modifiers, OnClick, PhysicalInputSequence, PhysicalKey,
    listener::InteractionListener, listener::InteractionShape, listener::InteractionState,
};
pub use leaf::{Branch, Leaf, SpawnedAt, Stem};
pub use line::{Line, LineSprout};
pub use opacity::Opacity;
pub use ops::Named;
pub use ops::{Keyring, Resolve, Resolved};
pub use panel::{Outline, Panel, PanelSprout, Rounding, Side};
#[cfg(target_os = "android")]
pub use platform::AndroidApp;
pub use platform::AndroidConnection;
pub use polygon::{Polygon, PolygonSprout};
pub use text::GlyphOffset;
pub use text::monospaced::FontId;
pub use text::{FontSize, GlyphColors, Text, TextContentHeight, TextContentWidth, TextSprout};
pub use time::{Moment, OnEnd, Time, TimeDelta, TimeMarker, Timer};
pub use tree::{EcsExtension, Graft, IntoTargets, Refire, Sequence, TargetedEvent, Tree};
pub use visibility::{InheritedVisibility, ResolvedVisibility, Visibility};
pub use web_ext::{Extensions, HrefLink};
