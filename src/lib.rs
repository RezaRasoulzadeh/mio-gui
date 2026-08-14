// lib.rs
mod accessibility;
mod accesskit_adapter;
mod app;
mod clipboard;
mod digit_format;
mod drawing;
mod event;
mod focus;
mod frame;
mod geometry;
mod glyph_atlas;
mod interaction;
mod keyboard;
mod layout;
mod linear_layout;
mod preferences;
mod raster;
mod renderer;
mod style;
mod text;
mod text_edit;
mod theme;
mod update;
mod widget_tree;
mod widgets;
mod winit_keyboard;
mod winit_preferences;

#[cfg(test)]
pub(crate) static GPU_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub use accessibility::{
    SemanticAction, SemanticActionRequest, SemanticNode, SemanticRole, SemanticSnapshot,
    SemanticState, Semantics,
};
pub use accesskit_adapter::PlatformAccessibility;
pub use app::run;
pub use clipboard::{ClipboardError, SystemClipboard, TextClipboard};
pub use digit_format::DecimalDigitSet;
pub use drawing::{
    ImageDraw, PixelFormat, PixelImage, PixelImageError, RectDraw, TextAlign, TextDraw,
};
pub use event::{
    EventControl, EventDelivery, EventDispatch, EventPhase, PointerCapture, PointerEvent,
    PointerId, PointerPhase,
};
pub use focus::{
    ArrowKey, EffectiveFocusPolicy, FocusIndicator, FocusIndicatorStyle, FocusManager, FocusPolicy,
    FocusSnapshot, FocusTraversal,
};
pub use frame::{FrameDamage, FrameNode, FrameSnapshot, WidgetGeometry};
pub use geometry::{
    ClipRegion, ClipStack, Constraints, Edges, Logical, LogicalConstraints, LogicalEdges,
    LogicalPoint, LogicalRect, LogicalSize, LogicalTransform, Overflow, Physical, PhysicalEdges,
    PhysicalPixelRect, PhysicalPoint, PhysicalRect, PhysicalSize, PhysicalTransform, PixelSnap,
    Point, Rect, ScaleFactor, Size, Transform,
};
pub use interaction::{
    InteractionEvent, InteractionInput, InteractionState, PointerKind, TargetedInteraction,
};
pub use keyboard::{
    Key, KeyModifiers, KeyState, KeyboardEvent, apply_focus_navigation, dispatch_keyboard_event,
    semantic_action_for_key,
};
pub use layout::{Direction, DirectionSetting, FlowEdges, HorizontalAlignment, InlineAlignment};
pub use linear_layout::{Axis, CrossAlignment, LayoutChild, LinearLayout, LinearLayoutResult};
pub use preferences::{ContrastPreference, MotionPreference, UserPreferences};
pub use renderer::{GlyphAtlasPlacement, GlyphQuad, RenderError, Renderer, RendererInitError};
pub use style::{
    AdornmentPlacement, ComponentAppearance, ComponentMetrics, ComponentSize, ComponentState,
    ComponentStyle, PhysicalAdornmentPlacement, ResolvedComponentStyle, VisualVariant,
};
pub use text::{
    BUNDLED_FALLBACK_FAMILIES, DEFAULT_FONT_FAMILY, DigitPolicy, GlyphAtlasKey, GlyphAtlasStats,
    GlyphImageContent, GlyphRasterDescriptor, RasterizedGlyph, ShapedGlyph, ShapedLine,
    TextCacheStats, TextCaret, TextCaretRect, TextDirection, TextHit, TextSelectionRect, TextSlant,
    TextStyle, TextSystem,
};
pub use text_edit::TextEditState;
pub use theme::{
    BorderTokens, ColorScheme, ContrastThemePairs, ElevationTokens, LinearColor,
    MINIMUM_TEXT_CONTRAST, MINIMUM_UI_CONTRAST, MotionTokens, RadiusTokens, ResolvedTheme,
    SemanticColorToken, SemanticColors, ShadowToken, SpacingTokens, ThemeController,
    ThemeDefinition, ThemeMode, ThemePair, TypographyToken,
};
pub use update::{
    DispatchReport, Invalidation, RedrawRequest, UpdateQueue, UpdateRuntime, WidgetMessage,
};
pub use widget_tree::{WidgetId, WidgetNode, WidgetTree, WidgetTreeError};
pub use widgets::{
    BlockAlignment, Image, ImageAlignment, ImageFit, ImageLayout, Text, TextLayout, TextLayoutLine,
    TextWrap, Widget, WidgetFrame, WidgetPlacement,
};
pub use winit_keyboard::keyboard_event_from_winit;
pub use winit_preferences::{apply_winit_theme, color_scheme_from_winit};
