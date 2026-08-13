// glyph_atlas.rs
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AtlasInsert {
    Existing(AtlasRegion),
    Inserted(AtlasRegion),
    ResetAndInserted(AtlasRegion),
    TooLarge,
}

pub(crate) struct GlyphAtlas<K> {
    width: u32,
    height: u32,
    padding: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    generation: u64,
    entries: HashMap<K, AtlasRegion>,
}

impl<K> GlyphAtlas<K>
where
    K: Clone + Eq + Hash,
{
    pub fn new(width: u32, height: u32, padding: u32) -> Self {
        Self {
            width,
            height,
            padding,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            generation: 0,
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, width: u32, height: u32) -> AtlasInsert {
        if let Some(region) = self.entries.get(&key) {
            return AtlasInsert::Existing(*region);
        }
        let Some(padded_width) = width.checked_add(self.padding.saturating_mul(2)) else {
            return AtlasInsert::TooLarge;
        };
        let Some(padded_height) = height.checked_add(self.padding.saturating_mul(2)) else {
            return AtlasInsert::TooLarge;
        };
        if width == 0 || height == 0 || padded_width > self.width || padded_height > self.height {
            return AtlasInsert::TooLarge;
        }

        if let Some(region) = self.place(key.clone(), width, height, padded_width, padded_height) {
            return AtlasInsert::Inserted(region);
        }

        self.reset();
        let region = self
            .place(key, width, height, padded_width, padded_height)
            .expect("validated glyph must fit an empty atlas");
        AtlasInsert::ResetAndInserted(region)
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    fn place(
        &mut self,
        key: K,
        width: u32,
        height: u32,
        padded_width: u32,
        padded_height: u32,
    ) -> Option<AtlasRegion> {
        if self.cursor_x + padded_width > self.width {
            self.cursor_x = 0;
            self.cursor_y = self.cursor_y.checked_add(self.row_height)?;
            self.row_height = 0;
        }
        if self.cursor_y + padded_height > self.height {
            return None;
        }

        let region = AtlasRegion {
            x: self.cursor_x + self.padding,
            y: self.cursor_y + self.padding,
            width,
            height,
            generation: self.generation,
        };
        self.cursor_x += padded_width;
        self.row_height = self.row_height.max(padded_height);
        self.entries.insert(key, region);
        Some(region)
    }

    fn reset(&mut self) {
        self.entries.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_height = 0;
        self.generation = self.generation.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{AtlasInsert, AtlasRegion, GlyphAtlas};

    #[test]
    fn packs_padded_glyphs_without_overlap() {
        let mut atlas = GlyphAtlas::new(16, 16, 1);

        assert_eq!(
            atlas.insert(1, 4, 4),
            AtlasInsert::Inserted(AtlasRegion {
                x: 1,
                y: 1,
                width: 4,
                height: 4,
                generation: 0,
            })
        );
        assert_eq!(
            atlas.insert(2, 4, 4),
            AtlasInsert::Inserted(AtlasRegion {
                x: 7,
                y: 1,
                width: 4,
                height: 4,
                generation: 0,
            })
        );
    }

    #[test]
    fn returns_existing_region_without_consuming_space() {
        let mut atlas = GlyphAtlas::new(16, 16, 1);
        let inserted = atlas.insert("alef", 4, 4);

        let AtlasInsert::Inserted(region) = inserted else {
            panic!("first insertion must allocate a region");
        };
        assert_eq!(atlas.insert("alef", 4, 4), AtlasInsert::Existing(region));
        assert_eq!(atlas.len(), 1);
    }

    #[test]
    fn resets_as_one_generation_when_full() {
        let mut atlas = GlyphAtlas::new(8, 8, 1);
        assert!(matches!(atlas.insert(1, 6, 6), AtlasInsert::Inserted(_)));

        let AtlasInsert::ResetAndInserted(region) = atlas.insert(2, 6, 6) else {
            panic!("full atlas must begin a new generation");
        };
        assert_eq!(region.generation, 1);
        assert_eq!(atlas.generation(), 1);
        assert_eq!(atlas.len(), 1);
        assert!(matches!(
            atlas.insert(1, 6, 6),
            AtlasInsert::ResetAndInserted(_)
        ));
    }

    #[test]
    fn rejects_empty_oversized_and_overflowing_glyphs() {
        let mut atlas = GlyphAtlas::new(8, 8, 1);

        assert_eq!(atlas.insert(1, 0, 2), AtlasInsert::TooLarge);
        assert_eq!(atlas.insert(2, 7, 2), AtlasInsert::TooLarge);
        assert_eq!(atlas.insert(3, u32::MAX, 2), AtlasInsert::TooLarge);
        assert_eq!(atlas.generation(), 0);
        assert_eq!(atlas.len(), 0);
    }

    #[test]
    fn starts_a_new_row_at_the_tallest_padded_height() {
        let mut atlas = GlyphAtlas::new(12, 20, 1);
        atlas.insert(1, 4, 3);
        atlas.insert(2, 4, 6);

        let AtlasInsert::Inserted(region) = atlas.insert(3, 4, 2) else {
            panic!("third glyph must fit on a new row");
        };
        assert_eq!((region.x, region.y), (1, 9));
    }
}
