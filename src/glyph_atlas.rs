// glyph_atlas.rs
use std::collections::HashMap;
use std::hash::Hash;

use crate::text::{GlyphAtlasKey, GlyphImageContent, RasterizedGlyph};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub generation: u64,
}

pub(crate) struct GpuGlyphAtlas {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    allocator: GlyphAtlas<GlyphAtlasKey>,
    padding: u32,
}

impl GpuGlyphAtlas {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, padding: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph_atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            allocator: GlyphAtlas::new(width, height, padding),
            padding,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn upload(
        &mut self,
        queue: &wgpu::Queue,
        key: GlyphAtlasKey,
        glyph: &RasterizedGlyph,
    ) -> Option<AtlasInsert> {
        let pixels = padded_rgba(glyph, self.padding)?;
        let inserted = self.allocator.insert(key, glyph.width, glyph.height);
        let region = match inserted {
            AtlasInsert::Inserted(region) | AtlasInsert::ResetAndInserted(region) => region,
            AtlasInsert::Existing(_) | AtlasInsert::TooLarge => return Some(inserted),
        };
        let upload_width = glyph.width + self.padding * 2;
        let upload_height = glyph.height + self.padding * 2;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.x - self.padding,
                    y: region.y - self.padding,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(upload_width * 4),
                rows_per_image: Some(upload_height),
            },
            wgpu::Extent3d {
                width: upload_width,
                height: upload_height,
                depth_or_array_layers: 1,
            },
        );
        Some(inserted)
    }
}

fn padded_rgba(glyph: &RasterizedGlyph, padding: u32) -> Option<Vec<u8>> {
    let pixel_count = glyph.width.checked_mul(glyph.height)? as usize;
    let source_stride = match glyph.content {
        GlyphImageContent::Mask => 1,
        GlyphImageContent::Color => 4,
        GlyphImageContent::SubpixelMask => 3,
    };
    if glyph.data.len() != pixel_count.checked_mul(source_stride)? {
        return None;
    }
    let output_width = glyph.width.checked_add(padding.checked_mul(2)?)? as usize;
    let output_height = glyph.height.checked_add(padding.checked_mul(2)?)? as usize;
    let mut output = vec![0; output_width.checked_mul(output_height)?.checked_mul(4)?];
    for pixel_index in 0..pixel_count {
        let source = pixel_index * source_stride;
        let source_x = pixel_index % glyph.width as usize;
        let source_y = pixel_index / glyph.width as usize;
        let destination =
            ((source_y + padding as usize) * output_width + source_x + padding as usize) * 4;
        match glyph.content {
            GlyphImageContent::Mask => {
                output[destination..destination + 4].copy_from_slice(&[
                    255,
                    255,
                    255,
                    glyph.data[source],
                ]);
            }
            GlyphImageContent::Color => {
                output[destination..destination + 4]
                    .copy_from_slice(&glyph.data[source..source + 4]);
            }
            GlyphImageContent::SubpixelMask => {
                let channels: [u8; 3] = glyph.data[source..source + 3].try_into().unwrap();
                output[destination..destination + 4].copy_from_slice(&[
                    channels[0],
                    channels[1],
                    channels[2],
                    channels.into_iter().max().unwrap(),
                ]);
            }
        }
    }
    Some(output)
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
    use super::{AtlasInsert, AtlasRegion, GlyphAtlas, GpuGlyphAtlas, padded_rgba};
    use crate::{GlyphImageContent, RasterizedGlyph, TextSystem};

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

    #[test]
    fn converts_mask_to_padded_white_rgba() {
        let glyph = RasterizedGlyph {
            left: 0,
            top: 0,
            width: 2,
            height: 1,
            content: GlyphImageContent::Mask,
            data: vec![64, 192],
        };

        let pixels = padded_rgba(&glyph, 1).unwrap();
        assert_eq!(pixels.len(), 4 * 3 * 4);
        assert_eq!(&pixels[20..28], &[255, 255, 255, 64, 255, 255, 255, 192]);
        assert!(pixels[..16].iter().all(|channel| *channel == 0));
        assert!(pixels[32..].iter().all(|channel| *channel == 0));
    }

    #[test]
    fn rejects_malformed_glyph_pixel_data() {
        let glyph = RasterizedGlyph {
            left: 0,
            top: 0,
            width: 2,
            height: 2,
            content: GlyphImageContent::Color,
            data: vec![0; 15],
        };

        assert!(padded_rgba(&glyph, 1).is_none());
    }

    #[test]
    fn gpu_atlas_rebuilds_one_complete_generation_after_eviction() {
        let _guard = crate::GPU_TEST_LOCK.lock().unwrap();
        pollster::block_on(async {
            let instance = wgpu::Instance::default();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                })
                .await
                .unwrap();
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor::default())
                .await
                .unwrap();
            let mut text_system = TextSystem::new();
            let line = text_system.shape_line("ABC", 16.0, 24.0);
            let keys = line
                .glyphs
                .iter()
                .map(|glyph| glyph.raster.atlas_key(1.0))
                .collect::<Vec<_>>();
            let image = RasterizedGlyph {
                left: 0,
                top: 0,
                width: 6,
                height: 6,
                content: GlyphImageContent::Mask,
                data: vec![255; 36],
            };
            let mut atlas = GpuGlyphAtlas::new(&device, 16, 8, 1);

            assert!(matches!(
                atlas.upload(&queue, keys[0], &image),
                Some(AtlasInsert::Inserted(AtlasRegion { generation: 0, .. }))
            ));
            assert!(matches!(
                atlas.upload(&queue, keys[1], &image),
                Some(AtlasInsert::Inserted(AtlasRegion { generation: 0, .. }))
            ));
            assert!(matches!(
                atlas.upload(&queue, keys[2], &image),
                Some(AtlasInsert::ResetAndInserted(AtlasRegion {
                    generation: 1,
                    ..
                }))
            ));
            assert!(matches!(
                atlas.upload(&queue, keys[0], &image),
                Some(AtlasInsert::Inserted(AtlasRegion { generation: 1, .. }))
            ));
            assert!(matches!(
                atlas.upload(&queue, keys[2], &image),
                Some(AtlasInsert::Existing(AtlasRegion { generation: 1, .. }))
            ));

            let submission = queue.submit([]);
            device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(submission),
                    timeout: None,
                })
                .unwrap();
        });
    }
}
