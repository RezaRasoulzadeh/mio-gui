// text.rs
use cosmic_text::{
    Attrs, Buffer, CacheKey, CacheKeyFlags, Family, FontSystem, Metrics, Shaping, Style,
    SwashCache, SwashContent, Weight,
};
use std::collections::HashMap;

use crate::glyph_atlas::{AtlasInsert, GlyphAtlas};

pub const DEFAULT_FONT_FAMILY: &str = "Vazirmatn";

const BUNDLED_FONTS: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/vazirmatn/Vazirmatn-Regular.ttf"),
    include_bytes!("../assets/fonts/vazirmatn/Vazirmatn-Medium.ttf"),
    include_bytes!("../assets/fonts/vazirmatn/Vazirmatn-SemiBold.ttf"),
    include_bytes!("../assets/fonts/vazirmatn/Vazirmatn-Bold.ttf"),
];
const SHAPED_LINE_CACHE_CAPACITY: usize = 1024;
const RASTERIZED_GLYPH_CACHE_CAPACITY: usize = 4096;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextSlant {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextCacheStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GlyphAtlasStats {
    pub entries: usize,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlyphImageContent {
    Mask,
    Color,
    SubpixelMask,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RasterizedGlyph {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub content: GlyphImageContent,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyphRasterDescriptor {
    font_id: cosmic_text::fontdb::ID,
    glyph_id: u16,
    font_size: f32,
    font_weight: Weight,
    x: f32,
    y: f32,
    x_offset: f32,
    y_offset: f32,
    flags: CacheKeyFlags,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GlyphAtlasKey(CacheKey);

impl GlyphRasterDescriptor {
    fn cache_key(&self, scale_factor: f32) -> CacheKey {
        let scale_factor = scale_factor.max(f32::EPSILON);
        let x_offset = self.font_size * self.x_offset;
        let y_offset = self.font_size * self.y_offset;
        CacheKey::new(
            self.font_id,
            self.glyph_id,
            self.font_size * scale_factor,
            (
                (self.x + x_offset) * scale_factor,
                ((self.y - y_offset) * scale_factor).trunc(),
            ),
            self.font_weight,
            self.flags,
        )
        .0
    }

    pub fn atlas_key(&self, scale_factor: f32) -> GlyphAtlasKey {
        GlyphAtlasKey(self.cache_key(scale_factor))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ShapedLineCacheKey {
    text: String,
    family: Option<String>,
    font_size: u32,
    line_height: u32,
    letter_spacing: u32,
    weight: u16,
    slant: TextSlant,
}

impl ShapedLineCacheKey {
    fn new(text: &str, style: &TextStyle) -> Self {
        Self {
            text: text.to_owned(),
            family: style.family.clone(),
            font_size: style.font_size.max(1.0).to_bits(),
            line_height: style.line_height.max(1.0).to_bits(),
            letter_spacing: style.letter_spacing.to_bits(),
            weight: style.weight.clamp(1, 1000),
            slant: style.slant,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub family: Option<String>,
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub weight: u16,
    pub slant: TextSlant,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            family: None,
            font_size: 16.0,
            line_height: 24.0,
            letter_spacing: 0.0,
            weight: 400,
            slant: TextSlant::Normal,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapedGlyph {
    pub start: usize,
    pub end: usize,
    pub glyph_id: u16,
    pub weight: u16,
    pub x: f32,
    pub width: f32,
    pub rtl: bool,
    pub raster: GlyphRasterDescriptor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapedLine {
    pub rtl: bool,
    pub width: f32,
    pub line_height: f32,
    pub glyphs: Vec<ShapedGlyph>,
}

pub struct TextSystem {
    font_system: FontSystem,
    shaped_line_cache: HashMap<ShapedLineCacheKey, ShapedLine>,
    cache_hits: u64,
    cache_misses: u64,
    swash_cache: SwashCache,
    rasterized_glyph_cache: HashMap<CacheKey, Option<RasterizedGlyph>>,
    raster_cache_hits: u64,
    raster_cache_misses: u64,
    glyph_atlas: GlyphAtlas<GlyphAtlasKey>,
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();
        let replaced_faces = font_system
            .db()
            .faces()
            .filter(|face| {
                face.families
                    .iter()
                    .any(|(family, _)| family == DEFAULT_FONT_FAMILY)
            })
            .map(|face| face.id)
            .collect::<Vec<_>>();
        for face_id in replaced_faces {
            font_system.db_mut().remove_face(face_id);
        }
        for font in BUNDLED_FONTS {
            font_system.db_mut().load_font_data(font.to_vec());
        }
        font_system
            .db_mut()
            .set_sans_serif_family(DEFAULT_FONT_FAMILY);

        Self {
            font_system,
            shaped_line_cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            swash_cache: SwashCache::new(),
            rasterized_glyph_cache: HashMap::new(),
            raster_cache_hits: 0,
            raster_cache_misses: 0,
            glyph_atlas: GlyphAtlas::new(2048, 2048, 1),
        }
    }

    pub fn shape_line(&mut self, text: &str, font_size: f32, line_height: f32) -> ShapedLine {
        self.shape_line_with_style(
            text,
            &TextStyle {
                font_size,
                line_height,
                ..TextStyle::default()
            },
        )
    }

    pub fn load_font_data(&mut self, data: Vec<u8>) -> usize {
        let before = self.font_system.db().faces().count();
        self.font_system.db_mut().load_font_data(data);
        let loaded = self.font_system.db().faces().count() - before;
        if loaded > 0 {
            self.clear_shaped_line_cache();
            self.rasterized_glyph_cache.clear();
        }
        loaded
    }

    pub fn font_count(&self) -> usize {
        self.font_system.db().faces().count()
    }

    pub fn has_font_family(&self, family: &str) -> bool {
        self.font_system
            .db()
            .faces()
            .any(|face| face.families.iter().any(|(name, _)| name == family))
    }

    pub fn shaped_line_cache_stats(&self) -> TextCacheStats {
        TextCacheStats {
            entries: self.shaped_line_cache.len(),
            hits: self.cache_hits,
            misses: self.cache_misses,
        }
    }

    pub fn clear_shaped_line_cache(&mut self) {
        self.shaped_line_cache.clear();
    }

    pub fn rasterized_glyph_cache_stats(&self) -> TextCacheStats {
        TextCacheStats {
            entries: self.rasterized_glyph_cache.len(),
            hits: self.raster_cache_hits,
            misses: self.raster_cache_misses,
        }
    }

    pub fn glyph_atlas_stats(&self) -> GlyphAtlasStats {
        GlyphAtlasStats {
            entries: self.glyph_atlas.len(),
            generation: self.glyph_atlas.generation(),
        }
    }

    pub fn rasterize_glyph(
        &mut self,
        glyph: &ShapedGlyph,
        scale_factor: f32,
    ) -> Option<RasterizedGlyph> {
        let cache_key = glyph.raster.cache_key(scale_factor);
        if let Some(image) = self.rasterized_glyph_cache.get(&cache_key) {
            self.raster_cache_hits += 1;
            let image = image.clone();
            self.register_glyph_in_atlas(GlyphAtlasKey(cache_key), image.as_ref());
            return image;
        }
        self.raster_cache_misses += 1;
        let image = self
            .swash_cache
            .get_image_uncached(&mut self.font_system, cache_key)
            .map(|image| RasterizedGlyph {
                left: image.placement.left,
                top: image.placement.top,
                width: image.placement.width,
                height: image.placement.height,
                content: match image.content {
                    SwashContent::Mask => GlyphImageContent::Mask,
                    SwashContent::Color => GlyphImageContent::Color,
                    SwashContent::SubpixelMask => GlyphImageContent::SubpixelMask,
                },
                data: image.data,
            });
        if self.rasterized_glyph_cache.len() >= RASTERIZED_GLYPH_CACHE_CAPACITY {
            self.rasterized_glyph_cache.clear();
        }
        self.rasterized_glyph_cache.insert(cache_key, image.clone());
        self.register_glyph_in_atlas(GlyphAtlasKey(cache_key), image.as_ref());
        image
    }

    fn register_glyph_in_atlas(
        &mut self,
        cache_key: GlyphAtlasKey,
        image: Option<&RasterizedGlyph>,
    ) {
        let Some(image) = image else {
            return;
        };
        match self
            .glyph_atlas
            .insert(cache_key, image.width, image.height)
        {
            AtlasInsert::Existing(_)
            | AtlasInsert::Inserted(_)
            | AtlasInsert::ResetAndInserted(_)
            | AtlasInsert::TooLarge => {}
        }
    }

    pub fn shape_line_with_style(&mut self, text: &str, style: &TextStyle) -> ShapedLine {
        let cache_key = ShapedLineCacheKey::new(text, style);
        if let Some(line) = self.shaped_line_cache.get(&cache_key) {
            self.cache_hits += 1;
            return line.clone();
        }
        self.cache_misses += 1;
        let line = self.shape_uncached(text, style);
        if self.shaped_line_cache.len() >= SHAPED_LINE_CACHE_CAPACITY {
            self.shaped_line_cache.clear();
        }
        self.shaped_line_cache.insert(cache_key, line.clone());
        line
    }

    fn shape_uncached(&mut self, text: &str, style: &TextStyle) -> ShapedLine {
        let metrics = Metrics::new(style.font_size.max(1.0), style.line_height.max(1.0));
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        let family = style
            .family
            .as_deref()
            .map(Family::Name)
            .unwrap_or(Family::SansSerif);
        let slant = match style.slant {
            TextSlant::Normal => Style::Normal,
            TextSlant::Italic => Style::Italic,
            TextSlant::Oblique => Style::Oblique,
        };
        let attrs = Attrs::new()
            .family(family)
            .weight(Weight(style.weight.clamp(1, 1000)))
            .style(slant)
            .letter_spacing(style.letter_spacing / metrics.font_size);
        {
            let mut buffer = buffer.borrow_with(&mut self.font_system);
            buffer.set_size(Some(4096.0), Some(metrics.line_height));
            buffer.set_text(text, &attrs, Shaping::Advanced, None);
            buffer.shape_until_scroll(true);
        }
        let Some(run) = buffer.layout_runs().next() else {
            return ShapedLine {
                rtl: false,
                width: 0.0,
                line_height: metrics.line_height,
                glyphs: Vec::new(),
            };
        };
        let glyphs = run
            .glyphs
            .iter()
            .map(|glyph| ShapedGlyph {
                start: glyph.start,
                end: glyph.end,
                glyph_id: glyph.glyph_id,
                weight: glyph.font_weight.0,
                x: glyph.x,
                width: glyph.w,
                rtl: glyph.level.is_rtl(),
                raster: GlyphRasterDescriptor {
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    font_size: glyph.font_size,
                    font_weight: glyph.font_weight,
                    x: glyph.x,
                    y: glyph.y,
                    x_offset: glyph.x_offset,
                    y_offset: glyph.y_offset,
                    flags: glyph.cache_key_flags,
                },
            })
            .collect();

        ShapedLine {
            rtl: run.rtl,
            width: run.line_w,
            line_height: run.line_height,
            glyphs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TextSlant, TextStyle, TextSystem};

    #[test]
    fn shapes_persian_as_rtl() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("سلام دنیا", 24.0, 32.0);

        assert!(line.rtl);
        assert!(line.width > 0.0);
        assert_eq!(line.line_height, 32.0);
        assert!(!line.glyphs.is_empty());
        assert!(line.glyphs.iter().all(|glyph| glyph.glyph_id != 0));
        assert!(line.glyphs.iter().all(|glyph| glyph.rtl));
    }

    #[test]
    fn resolves_mixed_persian_and_english_levels() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("نسخه Mio-GUI 2", 24.0, 32.0);

        assert!(line.rtl);
        assert!(line.glyphs.iter().any(|glyph| glyph.rtl));
        assert!(line.glyphs.iter().any(|glyph| !glyph.rtl));
    }

    #[test]
    fn repeated_shaping_is_stable() {
        let mut text_system = TextSystem::new();
        let first = text_system.shape_line("فروش 2026", 24.0, 32.0);
        let second = text_system.shape_line("فروش 2026", 24.0, 32.0);

        assert_eq!(first, second);
    }

    #[test]
    fn applies_metrics_weight_and_slant() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line_with_style(
            "رابط کاربری",
            &TextStyle {
                family: None,
                font_size: 20.0,
                line_height: 30.0,
                letter_spacing: 0.0,
                weight: 700,
                slant: TextSlant::Italic,
            },
        );

        assert_eq!(line.line_height, 30.0);
        assert!(line.glyphs.iter().all(|glyph| glyph.weight == 700));
    }

    #[test]
    fn applies_letter_spacing_to_each_glyph_advance() {
        let mut text_system = TextSystem::new();
        let normal = text_system.shape_line_with_style("Mio GUI", &TextStyle::default());
        let spaced = text_system.shape_line_with_style(
            "Mio GUI",
            &TextStyle {
                letter_spacing: 2.0,
                ..TextStyle::default()
            },
        );

        assert_eq!(normal.glyphs.len(), spaced.glyphs.len());
        for (normal, spaced) in normal.glyphs.iter().zip(&spaced.glyphs) {
            assert!(
                (spaced.width - normal.width - 2.0).abs() < 0.001,
                "normal={}, spaced={}",
                normal.width,
                spaced.width
            );
        }
    }

    #[test]
    fn discovers_system_fonts() {
        let text_system = TextSystem::new();

        assert!(text_system.font_count() > 0);
    }

    #[test]
    fn loads_bundled_vazirmatn_family() {
        let mut text_system = TextSystem::new();

        assert!(text_system.has_font_family(super::DEFAULT_FONT_FAMILY));
        for weight in [400, 500, 600, 700] {
            let line = text_system.shape_line_with_style(
                "رابط Mio-GUI",
                &TextStyle {
                    weight,
                    ..TextStyle::default()
                },
            );
            assert!(line.glyphs.iter().all(|glyph| glyph.weight == weight));
            assert!(line.glyphs.iter().all(|glyph| glyph.glyph_id != 0));
        }
    }

    #[test]
    fn caches_identical_shaped_lines_and_separates_styles() {
        let mut text_system = TextSystem::new();
        let regular = TextStyle::default();
        let bold = TextStyle {
            weight: 700,
            ..TextStyle::default()
        };

        let first = text_system.shape_line_with_style("سلام Mio", &regular);
        let second = text_system.shape_line_with_style("سلام Mio", &regular);
        text_system.shape_line_with_style("سلام Mio", &bold);

        assert_eq!(first, second);
        assert_eq!(
            text_system.shaped_line_cache_stats(),
            super::TextCacheStats {
                entries: 2,
                hits: 1,
                misses: 2,
            }
        );
    }

    #[test]
    fn clearing_cache_preserves_counters() {
        let mut text_system = TextSystem::new();
        text_system.shape_line("متن", 16.0, 24.0);
        text_system.clear_shaped_line_cache();

        assert_eq!(text_system.shaped_line_cache_stats().entries, 0);
        assert_eq!(text_system.shaped_line_cache_stats().misses, 1);
    }

    #[test]
    fn rasterizes_and_caches_persian_glyph_masks_at_each_scale() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("سلام", 24.0, 32.0);
        let glyph = &line.glyphs[0];

        let first = text_system.rasterize_glyph(glyph, 1.0).unwrap();
        let second = text_system.rasterize_glyph(glyph, 1.0).unwrap();
        let scaled = text_system.rasterize_glyph(glyph, 2.0).unwrap();

        assert_eq!(first, second);
        assert!(!first.data.is_empty());
        assert!(scaled.height >= first.height);
        assert_eq!(
            text_system.rasterized_glyph_cache_stats(),
            super::TextCacheStats {
                entries: 2,
                hits: 1,
                misses: 2,
            }
        );
        assert_eq!(text_system.glyph_atlas_stats().entries, 2);
    }
}
