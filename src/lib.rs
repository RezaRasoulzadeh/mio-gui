// lib.rs
mod app;
mod clipboard;
mod digit_format;
mod geometry;
mod glyph_atlas;
mod layout;
mod linear_layout;
mod raster;
mod renderer;
mod text;
mod text_edit;
mod widget_tree;

#[cfg(test)]
pub(crate) static GPU_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub use app::run;
pub use clipboard::{ClipboardError, SystemClipboard, TextClipboard};
pub use digit_format::DecimalDigitSet;
pub use geometry::{
    ClipRegion, ClipStack, Constraints, Edges, Logical, LogicalConstraints, LogicalEdges,
    LogicalPoint, LogicalRect, LogicalSize, LogicalTransform, Overflow, Physical, PhysicalEdges,
    PhysicalPixelRect, PhysicalPoint, PhysicalRect, PhysicalSize, PhysicalTransform, PixelSnap,
    Point, Rect, ScaleFactor, Size, Transform,
};
pub use layout::{Direction, DirectionSetting, FlowEdges, HorizontalAlignment, InlineAlignment};
pub use linear_layout::{Axis, CrossAlignment, LayoutChild, LinearLayout, LinearLayoutResult};
pub use renderer::{
    GlyphAtlasPlacement, GlyphQuad, RenderError, Renderer, RendererInitError, TextAlign, TextDraw,
};
pub use text::{
    BUNDLED_FALLBACK_FAMILIES, DEFAULT_FONT_FAMILY, DigitPolicy, GlyphAtlasKey, GlyphAtlasStats,
    GlyphImageContent, GlyphRasterDescriptor, RasterizedGlyph, ShapedGlyph, ShapedLine,
    TextCacheStats, TextCaret, TextCaretRect, TextDirection, TextHit, TextSelectionRect, TextSlant,
    TextStyle, TextSystem,
};
pub use text_edit::TextEditState;
pub use widget_tree::{WidgetId, WidgetNode, WidgetTree, WidgetTreeError};
