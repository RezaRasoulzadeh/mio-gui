// text.rs
use cosmic_text::{
    Attrs, Buffer, CacheKey, CacheKeyFlags, Fallback, Family, FontSystem, Metrics,
    PlatformFallback, Shaping, Style, SwashCache, SwashContent, Weight,
};
use std::collections::HashMap;
use std::ops::Range;
use unicode_script::Script;
use unicode_segmentation::UnicodeSegmentation;

use crate::glyph_atlas::{AtlasInsert, GlyphAtlas};

pub const DEFAULT_FONT_FAMILY: &str = "Vazirmatn";
pub const BUNDLED_FALLBACK_FAMILIES: [&str; 3] =
    ["Noto Sans", "Noto Sans Devanagari", "Noto Sans Thai"];

const BUNDLED_FONTS: [&[u8]; 7] = [
    include_bytes!("../assets/fonts/vazirmatn/Vazirmatn-Regular.ttf"),
    include_bytes!("../assets/fonts/vazirmatn/Vazirmatn-Medium.ttf"),
    include_bytes!("../assets/fonts/vazirmatn/Vazirmatn-SemiBold.ttf"),
    include_bytes!("../assets/fonts/vazirmatn/Vazirmatn-Bold.ttf"),
    include_bytes!("../assets/fonts/noto-sans/NotoSans-Variable.ttf"),
    include_bytes!("../assets/fonts/noto-sans-devanagari/NotoSansDevanagari-Variable.ttf"),
    include_bytes!("../assets/fonts/noto-sans-thai/NotoSansThai-Variable.ttf"),
];
const BUNDLED_FONT_FAMILIES: [&str; 4] = [
    DEFAULT_FONT_FAMILY,
    BUNDLED_FALLBACK_FAMILIES[0],
    BUNDLED_FALLBACK_FAMILIES[1],
    BUNDLED_FALLBACK_FAMILIES[2],
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

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextDirection {
    #[default]
    Auto,
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DigitPolicy {
    #[default]
    Preserve,
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
    direction: TextDirection,
    digit_policy: DigitPolicy,
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
            direction: style.direction,
            digit_policy: style.digit_policy,
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
    pub direction: TextDirection,
    pub digit_policy: DigitPolicy,
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
            direction: TextDirection::Auto,
            digit_policy: DigitPolicy::Preserve,
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
    pub baseline: f32,
    pub glyphs: Vec<ShapedGlyph>,
    grapheme_boundaries: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextHit {
    pub byte_index: usize,
    pub trailing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextCaret {
    pub byte_index: usize,
    pub x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextSelectionRect {
    pub x: f32,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextCaretRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ShapedLine {
    pub fn has_missing_glyphs(&self) -> bool {
        self.glyphs.iter().any(|glyph| glyph.glyph_id == 0)
    }

    pub fn missing_glyph_ranges(&self) -> Vec<Range<usize>> {
        let mut ranges = self
            .glyphs
            .iter()
            .filter(|glyph| glyph.glyph_id == 0)
            .map(|glyph| glyph.start..glyph.end)
            .collect::<Vec<_>>();
        ranges.sort_by_key(|range| (range.start, range.end));

        let mut merged: Vec<Range<usize>> = Vec::new();
        for range in ranges {
            if let Some(previous) = merged.last_mut() {
                if range.start <= previous.end {
                    previous.end = previous.end.max(range.end);
                    continue;
                }
            }
            merged.push(range);
        }
        merged
    }

    pub fn hit_test(&self, x: f32) -> TextHit {
        let Some(glyph) = self.glyphs.iter().min_by(|left, right| {
            distance_to_glyph(x, left)
                .total_cmp(&distance_to_glyph(x, right))
                .then_with(|| left.x.total_cmp(&right.x))
        }) else {
            return TextHit {
                byte_index: 0,
                trailing: false,
            };
        };
        let visual_trailing = x >= glyph.x + glyph.width * 0.5;
        let logical_trailing = visual_trailing != glyph.rtl;
        TextHit {
            byte_index: self.snap_to_grapheme(
                if logical_trailing {
                    glyph.end
                } else {
                    glyph.start
                },
                logical_trailing,
            ),
            trailing: logical_trailing,
        }
    }

    pub fn caret_positions(&self, byte_index: usize) -> Vec<f32> {
        let mut positions = Vec::new();
        for glyph in &self.glyphs {
            if glyph.start == byte_index {
                positions.push(if glyph.rtl {
                    glyph.x + glyph.width
                } else {
                    glyph.x
                });
            }
            if glyph.end == byte_index {
                positions.push(if glyph.rtl {
                    glyph.x
                } else {
                    glyph.x + glyph.width
                });
            }
        }
        positions.sort_by(f32::total_cmp);
        positions.dedup_by(|left, right| (*left - *right).abs() < 0.001);
        positions
    }

    pub fn logical_positions(&self) -> Vec<usize> {
        self.grapheme_boundaries.clone()
    }

    pub fn move_logical(&self, byte_index: usize, forward: bool) -> Option<usize> {
        let positions = self.logical_positions();
        let current = positions.binary_search(&byte_index).ok()?;
        if forward {
            positions.get(current + 1).copied()
        } else {
            current
                .checked_sub(1)
                .and_then(|previous| positions.get(previous).copied())
        }
    }

    pub fn visual_carets(&self) -> Vec<TextCaret> {
        let mut carets = self
            .logical_positions()
            .into_iter()
            .flat_map(|byte_index| {
                self.caret_positions(byte_index)
                    .into_iter()
                    .map(move |x| TextCaret { byte_index, x })
            })
            .collect::<Vec<_>>();
        carets.sort_by(|left, right| {
            left.x
                .total_cmp(&right.x)
                .then_with(|| left.byte_index.cmp(&right.byte_index))
        });
        carets.dedup_by(|left, right| {
            left.byte_index == right.byte_index && (left.x - right.x).abs() < 0.001
        });
        carets
    }

    pub fn move_visual(&self, caret: TextCaret, right: bool) -> Option<TextCaret> {
        let carets = self.visual_carets();
        let current = carets.iter().position(|candidate| {
            candidate.byte_index == caret.byte_index && (candidate.x - caret.x).abs() < 0.001
        })?;
        if right {
            carets.get(current + 1).copied()
        } else {
            current
                .checked_sub(1)
                .and_then(|previous| carets.get(previous).copied())
        }
    }

    pub fn selection_rects(&self, range: Range<usize>) -> Vec<TextSelectionRect> {
        if range.start >= range.end {
            return Vec::new();
        }
        let range =
            self.snap_to_grapheme(range.start, false)..self.snap_to_grapheme(range.end, true);
        let mut rects = self
            .glyphs
            .iter()
            .filter(|glyph| glyph.end > range.start && glyph.start < range.end)
            .map(|glyph| TextSelectionRect {
                x: glyph.x,
                width: glyph.width,
            })
            .collect::<Vec<_>>();
        rects.sort_by(|left, right| left.x.total_cmp(&right.x));
        let mut merged: Vec<TextSelectionRect> = Vec::new();
        for rect in rects {
            if let Some(previous) = merged.last_mut() {
                let previous_end = previous.x + previous.width;
                if rect.x <= previous_end + 0.001 {
                    previous.width = previous.width.max(rect.x + rect.width - previous.x);
                    continue;
                }
            }
            merged.push(rect);
        }
        merged
    }

    pub fn caret_rect(&self, caret: TextCaret, thickness: f32) -> TextCaretRect {
        let width = thickness.max(0.0);
        TextCaretRect {
            x: caret.x - width * 0.5,
            y: 0.0,
            width,
            height: self.line_height,
        }
    }

    pub fn grapheme_boundaries(&self) -> &[usize] {
        &self.grapheme_boundaries
    }

    fn snap_to_grapheme(&self, byte_index: usize, trailing: bool) -> usize {
        match self.grapheme_boundaries.binary_search(&byte_index) {
            Ok(index) => self.grapheme_boundaries[index],
            Err(index) if trailing => self
                .grapheme_boundaries
                .get(index)
                .copied()
                .unwrap_or_else(|| *self.grapheme_boundaries.last().unwrap_or(&0)),
            Err(index) => index
                .checked_sub(1)
                .and_then(|previous| self.grapheme_boundaries.get(previous).copied())
                .unwrap_or(0),
        }
    }
}

fn distance_to_glyph(x: f32, glyph: &ShapedGlyph) -> f32 {
    if x < glyph.x {
        glyph.x - x
    } else if x > glyph.x + glyph.width {
        x - glyph.x - glyph.width
    } else {
        0.0
    }
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

struct FrameworkFallback {
    platform: PlatformFallback,
}

impl Fallback for FrameworkFallback {
    fn common_fallback(&self) -> &[&'static str] {
        &[
            "Noto Sans",
            "Noto Color Emoji",
            "Segoe UI Emoji",
            "Apple Color Emoji",
        ]
    }

    fn forbidden_fallback(&self) -> &[&'static str] {
        self.platform.forbidden_fallback()
    }

    fn script_fallback(&self, script: Script, locale: &str) -> &[&'static str] {
        match script {
            Script::Devanagari => &["Noto Sans Devanagari"],
            Script::Thai => &["Noto Sans Thai"],
            _ => self.platform.script_fallback(script, locale),
        }
    }
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    pub fn new() -> Self {
        let (locale, mut font_database) = FontSystem::new().into_locale_and_db();
        let replaced_faces = font_database
            .faces()
            .filter(|face| {
                face.families
                    .iter()
                    .any(|(family, _)| BUNDLED_FONT_FAMILIES.contains(&family.as_str()))
            })
            .map(|face| face.id)
            .collect::<Vec<_>>();
        for face_id in replaced_faces {
            font_database.remove_face(face_id);
        }
        for font in BUNDLED_FONTS {
            font_database.load_font_data(font.to_vec());
        }
        font_database.set_sans_serif_family(DEFAULT_FONT_FAMILY);
        let font_system = FontSystem::new_with_locale_and_db_and_fallback(
            locale,
            font_database,
            FrameworkFallback {
                platform: PlatformFallback,
            },
        );

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

    pub fn glyph_font_family(&self, glyph: &ShapedGlyph) -> Option<String> {
        self.font_system
            .db()
            .face(glyph.raster.font_id)
            .and_then(|face| face.families.first())
            .map(|(family, _)| family.clone())
    }

    pub fn resolve_font_family(&self, requested: Option<&str>) -> String {
        requested
            .filter(|family| self.has_font_family(family))
            .unwrap_or(DEFAULT_FONT_FAMILY)
            .to_owned()
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
        let mut resolved_style = style.clone();
        resolved_style.family = Some(self.resolve_font_family(style.family.as_deref()));
        let cache_key = ShapedLineCacheKey::new(text, &resolved_style);
        if let Some(line) = self.shaped_line_cache.get(&cache_key) {
            self.cache_hits += 1;
            return line.clone();
        }
        self.cache_misses += 1;
        let line = self.shape_uncached(text, &resolved_style);
        if self.shaped_line_cache.len() >= SHAPED_LINE_CACHE_CAPACITY {
            self.shaped_line_cache.clear();
        }
        self.shaped_line_cache.insert(cache_key, line.clone());
        line
    }

    fn shape_uncached(&mut self, text: &str, style: &TextStyle) -> ShapedLine {
        let (shaped_text, source_offset) = match style.direction {
            TextDirection::Auto => (text.to_owned(), 0),
            TextDirection::Ltr => (format!("\u{200e}{text}"), '\u{200e}'.len_utf8()),
            TextDirection::Rtl => (format!("\u{200f}{text}"), '\u{200f}'.len_utf8()),
        };
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
            buffer.set_text(&shaped_text, &attrs, Shaping::Advanced, None);
            buffer.shape_until_scroll(true);
        }
        let Some(run) = buffer.layout_runs().next() else {
            return ShapedLine {
                rtl: false,
                width: 0.0,
                line_height: metrics.line_height,
                baseline: 0.0,
                glyphs: Vec::new(),
                grapheme_boundaries: vec![0],
            };
        };
        let mut glyphs = run
            .glyphs
            .iter()
            .filter(|glyph| glyph.end > source_offset)
            .map(|glyph| ShapedGlyph {
                start: glyph.start.saturating_sub(source_offset),
                end: glyph.end.saturating_sub(source_offset),
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
            .collect::<Vec<_>>();
        let line_origin = glyphs
            .iter()
            .map(|glyph| glyph.x)
            .reduce(f32::min)
            .unwrap_or(0.0);
        for glyph in &mut glyphs {
            glyph.x -= line_origin;
            glyph.raster.x -= line_origin;
        }

        ShapedLine {
            rtl: run.rtl,
            width: run.line_w,
            line_height: run.line_height,
            baseline: run.line_y - run.line_top,
            glyphs,
            grapheme_boundaries: text
                .grapheme_indices(true)
                .map(|(index, _)| index)
                .chain(std::iter::once(text.len()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DigitPolicy, TextDirection, TextSlant, TextStyle, TextSystem};

    fn text_mask_digest(text_system: &mut TextSystem, text: &str) -> (u32, u32, u64) {
        let line = text_system.shape_line(text, 18.0, 26.0);
        let mut images = Vec::new();
        for glyph in &line.glyphs {
            if let Some(image) = text_system.rasterize_glyph(glyph, 1.0) {
                images.push((glyph.x + image.left as f32, -(image.top as f32), image));
            }
        }
        let min_x = images
            .iter()
            .map(|(x, _, _)| x.floor() as i32)
            .min()
            .unwrap_or(0);
        let min_y = images
            .iter()
            .map(|(_, y, _)| y.floor() as i32)
            .min()
            .unwrap_or(0);
        let max_x = images
            .iter()
            .map(|(x, _, image)| (x + image.width as f32).ceil() as i32)
            .max()
            .unwrap_or(0);
        let max_y = images
            .iter()
            .map(|(_, y, image)| (y + image.height as f32).ceil() as i32)
            .max()
            .unwrap_or(0);
        let width = (max_x - min_x).max(0) as u32;
        let height = (max_y - min_y).max(0) as u32;
        let mut pixels = vec![0_u8; (width * height) as usize];
        for (x, y, image) in images {
            let stride = match image.content {
                super::GlyphImageContent::Mask => 1,
                super::GlyphImageContent::Color => 4,
                super::GlyphImageContent::SubpixelMask => 3,
            };
            for source_y in 0..image.height {
                for source_x in 0..image.width {
                    let source = ((source_y * image.width + source_x) as usize) * stride;
                    let alpha = match image.content {
                        super::GlyphImageContent::Mask => image.data[source],
                        super::GlyphImageContent::Color => image.data[source + 3],
                        super::GlyphImageContent::SubpixelMask => {
                            *image.data[source..source + 3].iter().max().unwrap()
                        }
                    };
                    let destination_x = x.floor() as i32 - min_x + source_x as i32;
                    let destination_y = y.floor() as i32 - min_y + source_y as i32;
                    let destination =
                        (destination_y as u32 * width + destination_x as u32) as usize;
                    let existing = u16::from(pixels[destination]);
                    pixels[destination] =
                        (existing + u16::from(alpha) * (255 - existing) / 255) as u8;
                }
            }
        }
        let mut digest = 0xcbf29ce484222325_u64;
        for byte in width
            .to_le_bytes()
            .into_iter()
            .chain(height.to_le_bytes())
            .chain(pixels)
        {
            digest ^= u64::from(byte);
            digest = digest.wrapping_mul(0x100000001b3);
        }
        (width, height, digest)
    }

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
        assert!(line.glyphs.iter().all(|glyph| glyph.x >= 0.0));
        assert!(
            line.glyphs
                .iter()
                .all(|glyph| glyph.x + glyph.width <= line.width + 0.001)
        );
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
    fn shapes_arabic_contextual_forms() {
        let mut text_system = TextSystem::new();
        let isolated = text_system.shape_line("ب", 24.0, 32.0);
        let joined = text_system.shape_line("ببب", 24.0, 32.0);

        assert!(joined.rtl);
        assert_eq!(joined.glyphs.len(), 3);
        assert!(joined.glyphs.iter().all(|glyph| glyph.glyph_id != 0));
        assert!(
            joined
                .glyphs
                .iter()
                .any(|glyph| glyph.glyph_id != isolated.glyphs[0].glyph_id)
        );
    }

    #[test]
    fn resolves_mixed_arabic_english_numbers_and_punctuation() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("الإصدار (Mio-GUI 2.0)، جاهز؟", 24.0, 32.0);

        assert!(line.rtl);
        assert!(line.glyphs.iter().any(|glyph| glyph.rtl));
        assert!(line.glyphs.iter().any(|glyph| !glyph.rtl));
        assert!(line.glyphs.iter().all(|glyph| glyph.glyph_id != 0));
        assert!(
            line.glyphs
                .iter()
                .all(|glyph| glyph.x + glyph.width <= line.width + 0.001)
        );
    }

    #[test]
    fn preserves_arabic_combining_mark_clusters() {
        let mut text_system = TextSystem::new();
        let text = "مُحَمَّد";
        let line = text_system.shape_line(text, 24.0, 32.0);

        assert!(line.rtl);
        assert!(line.glyphs.iter().all(|glyph| glyph.glyph_id != 0));
        for glyph in &line.glyphs {
            assert!(text.is_char_boundary(glyph.start));
            assert!(text.is_char_boundary(glyph.end));
            assert!(glyph.start <= glyph.end);
            assert!(glyph.end <= text.len());
        }
        assert!(
            line.glyphs
                .windows(2)
                .any(|glyphs| glyphs[0].start == glyphs[1].start)
        );
    }

    #[test]
    fn supports_explicit_ltr_rtl_and_automatic_base_direction() {
        let mut text_system = TextSystem::new();
        let text = "(Mio-GUI) رابط";
        let automatic = text_system.shape_line_with_style(text, &TextStyle::default());
        let ltr = text_system.shape_line_with_style(
            text,
            &TextStyle {
                direction: TextDirection::Ltr,
                ..TextStyle::default()
            },
        );
        let rtl = text_system.shape_line_with_style(
            text,
            &TextStyle {
                direction: TextDirection::Rtl,
                ..TextStyle::default()
            },
        );

        assert!(!automatic.rtl);
        assert!(!ltr.rtl);
        assert!(rtl.rtl);
        for line in [&automatic, &ltr, &rtl] {
            assert!(line.glyphs.iter().all(|glyph| glyph.end <= text.len()));
            assert!(line.glyphs.iter().all(|glyph| glyph.glyph_id != 0));
        }
    }

    #[test]
    fn forced_direction_preserves_original_byte_offsets() {
        let mut text_system = TextSystem::new();
        let text = "سلام Mio";
        for direction in [TextDirection::Ltr, TextDirection::Rtl] {
            let line = text_system.shape_line_with_style(
                text,
                &TextStyle {
                    direction,
                    ..TextStyle::default()
                },
            );
            for glyph in line.glyphs {
                assert!(text.is_char_boundary(glyph.start));
                assert!(text.is_char_boundary(glyph.end));
                assert!(glyph.end <= text.len());
            }
        }
    }

    #[test]
    fn mirrors_paired_punctuation_without_changing_source_offsets() {
        let mut text_system = TextSystem::new();
        let ltr_open = text_system.shape_line_with_style(
            "(",
            &TextStyle {
                direction: TextDirection::Ltr,
                ..TextStyle::default()
            },
        );
        let ltr_close = text_system.shape_line_with_style(
            ")",
            &TextStyle {
                direction: TextDirection::Ltr,
                ..TextStyle::default()
            },
        );
        let rtl_open = text_system.shape_line_with_style(
            "(",
            &TextStyle {
                direction: TextDirection::Rtl,
                ..TextStyle::default()
            },
        );

        assert_ne!(ltr_open.glyphs[0].glyph_id, ltr_close.glyphs[0].glyph_id);
        assert_eq!(rtl_open.glyphs[0].glyph_id, ltr_close.glyphs[0].glyph_id);
        assert_eq!((rtl_open.glyphs[0].start, rtl_open.glyphs[0].end), (0, 1));
    }

    #[test]
    fn preserves_all_digit_sets_by_default() {
        let mut text_system = TextSystem::new();
        let text = "Latin 012 Arabic ٠١٢ Persian ۰۱۲";
        let line = text_system.shape_line_with_style(
            text,
            &TextStyle {
                digit_policy: DigitPolicy::Preserve,
                ..TextStyle::default()
            },
        );

        assert!(line.glyphs.iter().all(|glyph| glyph.end <= text.len()));
        for digit in ["0", "٠", "۰"] {
            let start = text.find(digit).unwrap();
            let end = start + digit.len();
            assert!(
                line.glyphs
                    .iter()
                    .any(|glyph| glyph.start == start && glyph.end == end)
            );
        }
    }

    #[test]
    fn hit_tests_ltr_glyph_halves_to_source_boundaries() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("Mio", 20.0, 28.0);
        let glyph = line
            .glyphs
            .iter()
            .min_by(|a, b| a.x.total_cmp(&b.x))
            .unwrap();

        let leading = line.hit_test(glyph.x + glyph.width * 0.25);
        let trailing = line.hit_test(glyph.x + glyph.width * 0.75);

        assert_eq!(leading.byte_index, glyph.start);
        assert!(!leading.trailing);
        assert_eq!(trailing.byte_index, glyph.end);
        assert!(trailing.trailing);
    }

    #[test]
    fn hit_tests_rtl_glyph_halves_in_visual_order() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("سلام", 20.0, 28.0);
        let glyph = line
            .glyphs
            .iter()
            .min_by(|a, b| a.x.total_cmp(&b.x))
            .unwrap();

        let visual_left = line.hit_test(glyph.x + glyph.width * 0.25);
        let visual_right = line.hit_test(glyph.x + glyph.width * 0.75);

        assert_eq!(visual_left.byte_index, glyph.end);
        assert!(visual_left.trailing);
        assert_eq!(visual_right.byte_index, glyph.start);
        assert!(!visual_right.trailing);
    }

    #[test]
    fn hit_testing_combining_marks_returns_cluster_boundaries() {
        let mut text_system = TextSystem::new();
        let text = "مُحَمَّد";
        let line = text_system.shape_line(text, 20.0, 28.0);

        for glyph in &line.glyphs {
            for x in [glyph.x + glyph.width * 0.25, glyph.x + glyph.width * 0.75] {
                let hit = line.hit_test(x);
                assert!(text.is_char_boundary(hit.byte_index));
                assert!(line.glyphs.iter().any(|candidate| {
                    candidate.start == hit.byte_index || candidate.end == hit.byte_index
                }));
            }
        }
    }

    #[test]
    fn hit_testing_empty_line_returns_start() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("", 20.0, 28.0);

        assert_eq!(
            line.hit_test(100.0),
            super::TextHit {
                byte_index: 0,
                trailing: false,
            }
        );
    }

    #[test]
    fn maps_ltr_source_boundaries_to_visual_carets() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("Mio", 20.0, 28.0);
        let glyph = line
            .glyphs
            .iter()
            .min_by(|a, b| a.x.total_cmp(&b.x))
            .unwrap();

        assert_eq!(line.caret_positions(glyph.start), vec![glyph.x]);
        assert!(
            line.caret_positions(glyph.end)
                .iter()
                .any(|position| (*position - glyph.x - glyph.width).abs() < 0.001)
        );
    }

    #[test]
    fn maps_rtl_source_boundaries_to_reversed_visual_carets() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("سلام", 20.0, 28.0);
        let glyph = &line.glyphs[0];

        assert!(
            line.caret_positions(glyph.start)
                .iter()
                .any(|position| (*position - glyph.x - glyph.width).abs() < 0.001)
        );
        assert!(
            line.caret_positions(glyph.end)
                .iter()
                .any(|position| (*position - glyph.x).abs() < 0.001)
        );
    }

    #[test]
    fn exposes_both_visual_carets_at_mixed_direction_boundaries() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("نسخه Mio", 20.0, 28.0);
        let boundary = "نسخه ".len();
        let positions = line.caret_positions(boundary);

        assert!(positions.len() >= 2);
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn rejects_non_cluster_caret_positions() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("سلام", 20.0, 28.0);

        assert!(line.caret_positions(usize::MAX).is_empty());
    }

    #[test]
    fn moves_logically_by_source_cluster_order() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("سلام Mio", 20.0, 28.0);
        let positions = line.logical_positions();

        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        for pair in positions.windows(2) {
            assert_eq!(line.move_logical(pair[0], true), Some(pair[1]));
            assert_eq!(line.move_logical(pair[1], false), Some(pair[0]));
        }
        assert_eq!(line.move_logical(positions[0], false), None);
        assert_eq!(line.move_logical(*positions.last().unwrap(), true), None);
    }

    #[test]
    fn moves_visually_by_screen_order_across_bidi_runs() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("نسخه Mio 2", 20.0, 28.0);
        let carets = line.visual_carets();

        assert!(carets.windows(2).all(|pair| pair[0].x <= pair[1].x));
        for pair in carets.windows(2) {
            assert_eq!(line.move_visual(pair[0], true), Some(pair[1]));
            assert_eq!(line.move_visual(pair[1], false), Some(pair[0]));
        }
        assert_eq!(line.move_visual(carets[0], false), None);
        assert_eq!(line.move_visual(*carets.last().unwrap(), true), None);
    }

    #[test]
    fn visual_and_logical_rtl_movement_are_distinct() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("سلام", 20.0, 28.0);
        let logical = line.logical_positions();
        let visual = line.visual_carets();

        assert_eq!(logical.first().copied(), Some(0));
        assert_eq!(
            visual.first().map(|caret| caret.byte_index),
            logical.last().copied()
        );
        assert_eq!(visual.last().map(|caret| caret.byte_index), Some(0));
    }

    #[test]
    fn merges_contiguous_ltr_selection_into_one_visual_rect() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("Mio GUI", 20.0, 28.0);
        let rects = line.selection_rects(0.."Mio".len());

        assert_eq!(rects.len(), 1);
        assert!(rects[0].width > 0.0);
    }

    #[test]
    fn maps_rtl_logical_selection_to_visual_geometry() {
        let mut text_system = TextSystem::new();
        let text = "سلام دنیا";
        let line = text_system.shape_line(text, 20.0, 28.0);
        let rects = line.selection_rects(0.."سلام".len());

        assert!(!rects.is_empty());
        assert!(rects.iter().all(|rect| rect.x >= 0.0 && rect.width > 0.0));
        assert!(rects.windows(2).all(|pair| pair[0].x < pair[1].x));
    }

    #[test]
    fn keeps_separated_bidi_selection_runs_as_separate_rects() {
        let mut text_system = TextSystem::new();
        let text = "الف ABC باء";
        let line = text_system.shape_line(text, 20.0, 28.0);
        let latin_start = text.find("ABC").unwrap();
        let rects = line.selection_rects(latin_start + 1..text.len());

        assert!(rects.len() >= 2);
        assert!(
            rects
                .windows(2)
                .all(|pair| pair[0].x + pair[0].width < pair[1].x)
        );
    }

    #[test]
    fn empty_or_out_of_range_selection_has_no_geometry() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("متن", 20.0, 28.0);

        assert!(line.selection_rects(0..0).is_empty());
        assert!(line.selection_rects(usize::MAX - 1..usize::MAX).is_empty());
    }

    #[test]
    fn logical_movement_skips_entire_combining_sequences() {
        let mut text_system = TextSystem::new();
        let text = "مُحَ";
        let line = text_system.shape_line(text, 20.0, 28.0);

        assert_eq!(line.grapheme_boundaries(), &[0, "مُ".len(), text.len()]);
        assert_eq!(line.move_logical(0, true), Some("مُ".len()));
        assert_ne!(line.move_logical(0, true), Some("م".len()));
    }

    #[test]
    fn logical_movement_skips_entire_emoji_zwj_sequences() {
        let mut text_system = TextSystem::new();
        let emoji = "👩‍💻";
        let text = format!("{emoji}A");
        let line = text_system.shape_line(&text, 20.0, 28.0);

        assert_eq!(line.grapheme_boundaries(), &[0, emoji.len(), text.len()]);
        assert_eq!(line.move_logical(0, true), Some(emoji.len()));
    }

    #[test]
    fn selection_expands_partial_combining_sequence_to_grapheme() {
        let mut text_system = TextSystem::new();
        let text = "مُح";
        let line = text_system.shape_line(text, 20.0, 28.0);
        let whole = line.selection_rects(0.."مُ".len());
        let combining_mark_only = line.selection_rects("م".len().."مُ".len());

        assert_eq!(combining_mark_only, whole);
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
                direction: TextDirection::Auto,
                digit_policy: DigitPolicy::Preserve,
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
    fn reports_missing_glyphs_by_original_source_range() {
        let mut text_system = TextSystem::new();
        let unsupported = '\u{10ffff}';
        let text = format!("Mio {unsupported} متن");
        let line = text_system.shape_line(&text, 20.0, 28.0);
        let start = text.find(unsupported).unwrap();

        assert!(line.has_missing_glyphs());
        assert_eq!(
            line.missing_glyph_ranges(),
            vec![start..start + unsupported.len_utf8()]
        );
        assert_eq!(&text[line.missing_glyph_ranges()[0].clone()], "\u{10ffff}");
    }

    #[test]
    fn bundled_persian_and_latin_have_no_missing_glyphs() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("رابط Mio-GUI 2026", 20.0, 28.0);

        assert!(!line.has_missing_glyphs());
        assert!(line.missing_glyph_ranges().is_empty());
    }

    #[test]
    fn bundled_cross_script_fallbacks_are_selected_deterministically() {
        let mut text_system = TextSystem::new();
        let fixtures = [
            ("γειά", "Noto Sans"),
            ("नमस्ते", "Noto Sans Devanagari"),
            ("สวัสดี", "Noto Sans Thai"),
        ];

        for (text, expected_family) in fixtures {
            assert!(text_system.has_font_family(expected_family));
            let line = text_system.shape_line(text, 20.0, 28.0);
            assert!(!line.has_missing_glyphs(), "missing glyph in {text:?}");
            assert!(!line.glyphs.is_empty());
            assert!(line.glyphs.iter().all(|glyph| {
                text_system.glyph_font_family(glyph).as_deref() == Some(expected_family)
            }));
        }
    }

    #[test]
    fn deterministically_resolves_missing_family_to_vazirmatn() {
        let mut text_system = TextSystem::new();
        let text = "رابط Mio-GUI";
        let default = text_system.shape_line_with_style(text, &TextStyle::default());
        let missing = text_system.shape_line_with_style(
            text,
            &TextStyle {
                family: Some("Mio GUI Font That Does Not Exist".to_owned()),
                ..TextStyle::default()
            },
        );

        assert_eq!(
            text_system.resolve_font_family(None),
            super::DEFAULT_FONT_FAMILY
        );
        assert_eq!(
            text_system.resolve_font_family(Some("Mio GUI Font That Does Not Exist")),
            super::DEFAULT_FONT_FAMILY
        );
        assert_eq!(default, missing);
        assert_eq!(text_system.shaped_line_cache_stats().entries, 1);
        assert_eq!(text_system.shaped_line_cache_stats().hits, 1);
    }

    #[test]
    fn preserves_an_explicit_loaded_family() {
        let text_system = TextSystem::new();

        assert_eq!(
            text_system.resolve_font_family(Some(super::DEFAULT_FONT_FAMILY)),
            super::DEFAULT_FONT_FAMILY
        );
    }

    #[test]
    fn bundled_font_text_masks_match_goldens() {
        let mut text_system = TextSystem::new();
        let fixtures = [
            ("سلام دنیا", (65, 19, 7985671594921089175)),
            ("مرحبا بالعالم", (90, 19, 3834719249868747265)),
            ("Mio-GUI 2026", (112, 14, 17634512585228747464)),
            ("نسخه Mio-GUI 2", (129, 14, 10389384962691154384)),
        ];

        for (text, expected) in fixtures {
            let actual = text_mask_digest(&mut text_system, text);
            assert_eq!(actual, expected, "text={text:?}");
        }
    }

    #[test]
    fn bundled_font_line_metrics_match_goldens() {
        let mut text_system = TextSystem::new();
        let fixtures = [
            ("سلام دنیا", (1115890176, 1104150528, 1099638784)),
            ("مرحبا بالعالم", (1119066240, 1104150528, 1099638784)),
            ("Mio-GUI 2026", (1122091392, 1104150528, 1099638784)),
            ("نسخه Mio-GUI 2", (1124158400, 1104150528, 1099638784)),
        ];

        for (text, expected) in fixtures {
            let line = text_system.shape_line(text, 18.0, 26.0);
            let actual = (
                line.width.to_bits(),
                line.line_height.to_bits(),
                line.baseline.to_bits(),
            );
            assert_eq!(actual, expected, "text={text:?}");
        }
    }

    #[test]
    fn caret_rectangle_uses_complete_line_metrics() {
        let mut text_system = TextSystem::new();
        let line = text_system.shape_line("سلام Mio", 20.0, 28.0);
        let caret = line.visual_carets()[0];
        let rect = line.caret_rect(caret, 2.0);

        assert_eq!(rect.x, caret.x - 1.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 2.0);
        assert_eq!(rect.height, 28.0);
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
