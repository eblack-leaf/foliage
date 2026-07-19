#![allow(unused)]
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
    area::{Area, CReprArea},
    points::Points,
    position::{CReprPosition, Position},
    section::{CReprSection, Section},
    CoordinateContext, CoordinateUnit, Coordinates, Logical, Numerical, Physical,
};
pub use anim::{
    ease::{ControlPoints, Ease, Easement},
    interpolation::{Interpolation, Interpolations},
    Animate, Animation,
};
pub(crate) use ash::differential::Differential;
pub use asset::{asset_retrieval, AssetKey, AssetRetrieval, AssetSource, LoadAsset};
pub use attachment::Attachment;
pub use author::{LeafSprout, Sprout, With};
pub use bevy_ecs::{self, prelude::*};
/// bevy 0.17+ renamed the observer parameter `Trigger` to `On`. Every observer in foliage and
/// its consumers is written against the `Trigger<E>` spelling, so keep it as the canonical
/// name here; `On` is also available (via the prelude re-export above) for new code.
pub type Trigger<'w, 't, E, B = ()> = bevy_ecs::observer::On<'w, 't, E, B>;
pub use clipboard::Clipboard;
pub use color::{CReprColor, Color, Luminance};
pub use composite::text_input::action::{InputAction, TextInputAction};
pub use composite::{
    button::{Button, ButtonSprout, ButtonStyle, Engagement},
    carousel::{Carousel, CarouselConfig, CarouselPages, CarouselSprout, CarouselStyle},
    checkbox::{Checkbox, CheckboxSprout, CheckboxState, CheckboxStyle, Checked},
    dropdown::{
        Dropdown, DropdownConfig, DropdownOptions, DropdownSprout, DropdownStyle, Expanded,
        Selected, SelectionChanged,
    },
    list::{List, ListItems, ListLayout, ListSprout},
    modal::{CloseModal, Closed, Modal, ModalSprout, ModalStyle},
    pagination::{PageChanged, Pagination, PaginationMode, PaginationSprout, PaginationStyle},
    popover::{
        Popover, PopoverClosed, PopoverExpanded, PopoverOpened, PopoverPlacement, PopoverSprout,
        PopoverStyle,
    },
    radio_group::{RadioChanged, RadioGroup, RadioGroupSprout, RadioOptions, RadioSelected, RadioStyle},
    segmented_control::{
        SegmentChanged, SegmentedControl, SegmentedControlSprout, SegmentedOptions,
        SegmentedSelected, SegmentedStyle,
    },
    slider::{ProgressChanged, Slider, SliderBehavior, SliderSprout, SliderStyle},
    tabs::{Tabs, TabsPages, TabsSprout, TabsStyle},
    text_input::{
        keybindings::KeyBindings, HintColor, HintText, InsertText, LineConstraint, TextChanged,
        TextInput, TextInputSprout, TextInputStyle,
    },
    toggle::{Toggle, ToggleSprout, ToggleState, ToggleStyle, Toggled},
    Root,
};
pub use composite::{IconValue, IndexedSlotFn, PageCount, PageIndex, Progress, SlotFn, TextValue};
pub use coordinate::elevation::{Elevation, ResolvedElevation};
pub use disable::Disable;
pub use enable::Enable;
pub use foliage::Foliage;
pub use grid::{
    anchor, text_content, view::OverscrollPropagation, AspectRatio, Grid, Layout, Location, View,
};
pub use grid::{Anchor, AnchorDeps, AnchorDescriptor, GridExt, Justify, LocationValue};
pub use icon::{Icon, IconId, IconMemory, IconRenderSizes, IconSprout};
pub use image::{Image, ImageMetrics, ImageSprout, ImageView};
pub use interaction::CurrentInteraction;
pub use interaction::{
    listener::InteractionListener, listener::InteractionShape, listener::InteractionState,
    FocusBehavior, InputSequence, Interaction, InteractionPhase, InteractionPropagation, Key,
    Modifiers, OnClick, PhysicalInputSequence, PhysicalKey,
};
pub use interaction::{Disengaged, Dragged, Engaged, Focused, Unfocused};
pub use leaf::{Branch, Leaf, Stem};
pub use line::{Line, LineSprout};
pub use opacity::Opacity;
pub use ops::Named;
pub use ops::{Keyring, Update, Write};
pub use panel::{Outline, Panel, PanelSprout, Rounding, Side};
#[cfg(target_os = "android")]
pub use platform::AndroidApp;
pub use platform::AndroidConnection;
pub use polygon::{Polygon, PolygonSprout};
pub use alignment::{HorizontalAlignment, VerticalAlignment};
pub use text::GlyphOffset;
pub use text::{TextContentHeight, TextContentWidth, FontSize, GlyphColors, Text, TextSprout};
pub use time::{Moment, OnEnd, Time, TimeDelta, TimeMarker, Timer};
pub use tree::{EcsExtension, Graft, IntoTargets, Refire, Sequence, TargetedEvent, Tree};
pub use visibility::{InheritedVisibility, ResolvedVisibility, Visibility};
pub use web_ext::{Extensions, HrefLink};
