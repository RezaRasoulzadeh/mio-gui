// lib.rs
mod app;
mod glyph_atlas;
mod raster;
mod renderer;
mod text;

pub use app::run;
pub use renderer::{RenderError, Renderer, RendererInitError};
pub use text::{
    DEFAULT_FONT_FAMILY, GlyphAtlasStats, GlyphImageContent, GlyphRasterDescriptor,
    RasterizedGlyph, ShapedGlyph, ShapedLine, TextCacheStats, TextSlant, TextStyle, TextSystem,
};
