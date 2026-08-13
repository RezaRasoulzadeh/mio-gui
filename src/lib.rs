// lib.rs
mod app;
mod glyph_atlas;
mod raster;
mod renderer;
mod text;

pub use app::run;
pub use renderer::{GlyphAtlasPlacement, GlyphQuad, RenderError, Renderer, RendererInitError};
pub use text::{
    DEFAULT_FONT_FAMILY, GlyphAtlasKey, GlyphAtlasStats, GlyphImageContent, GlyphRasterDescriptor,
    RasterizedGlyph, ShapedGlyph, ShapedLine, TextCacheStats, TextSlant, TextStyle, TextSystem,
};
