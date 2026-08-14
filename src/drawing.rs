// drawing.rs

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::{LogicalRect, TextStyle};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PixelFormat {
    Rgba8,
    Alpha8,
}

impl PixelFormat {
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgba8 => 4,
            Self::Alpha8 => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelImageError {
    ZeroDimension,
    DimensionOverflow,
    InvalidDataLength { expected: usize, actual: usize },
}

impl Display for PixelImageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroDimension => formatter.write_str("pixel image dimensions must be non-zero"),
            Self::DimensionOverflow => formatter.write_str("pixel image dimensions overflow"),
            Self::InvalidDataLength { expected, actual } => {
                write!(
                    formatter,
                    "expected {expected} pixel bytes, received {actual}"
                )
            }
        }
    }
}

impl Error for PixelImageError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PixelImage {
    width: u32,
    height: u32,
    format: PixelFormat,
    data: Arc<[u8]>,
}

impl PixelImage {
    pub fn new(
        width: u32,
        height: u32,
        format: PixelFormat,
        data: impl Into<Arc<[u8]>>,
    ) -> Result<Self, PixelImageError> {
        if width == 0 || height == 0 {
            return Err(PixelImageError::ZeroDimension);
        }
        let expected = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(format.bytes_per_pixel()))
            .ok_or(PixelImageError::DimensionOverflow)?;
        let data = data.into();
        if data.len() != expected {
            return Err(PixelImageError::InvalidDataLength {
                expected,
                actual: data.len(),
            });
        }
        Ok(Self {
            width,
            height,
            format,
            data,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }
    pub const fn height(&self) -> u32 {
        self.height
    }
    pub const fn format(&self) -> PixelFormat {
        self.format
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextDraw {
    pub text: String,
    pub style: TextStyle,
    pub baseline: [f32; 2],
    pub align: TextAlign,
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectDraw {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub radii: [f32; 4],
    pub color: [f32; 4],
    pub border_width: f32,
    pub border_color: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageDraw {
    pub image: PixelImage,
    pub bounds: LogicalRect,
    pub clip: LogicalRect,
    pub mirror_horizontal: bool,
    pub tint: Option<[f32; 4]>,
}

#[cfg(test)]
mod tests {
    use super::{PixelFormat, PixelImage, PixelImageError};

    #[test]
    fn pixel_images_validate_dimensions_format_and_data_length() {
        assert_eq!(
            PixelImage::new(0, 1, PixelFormat::Rgba8, Vec::<u8>::new()),
            Err(PixelImageError::ZeroDimension)
        );
        assert_eq!(
            PixelImage::new(2, 2, PixelFormat::Rgba8, vec![0_u8; 15]),
            Err(PixelImageError::InvalidDataLength {
                expected: 16,
                actual: 15,
            })
        );
        assert_eq!(
            PixelImage::new(2, 2, PixelFormat::Alpha8, vec![0_u8; 4])
                .unwrap()
                .format(),
            PixelFormat::Alpha8
        );
    }

    #[test]
    fn cloned_pixel_images_share_immutable_bytes_and_identity() {
        let image = PixelImage::new(1, 1, PixelFormat::Rgba8, vec![1, 2, 3, 4]).unwrap();
        let clone = image.clone();

        assert_eq!(image, clone);
        assert_eq!(image.data().as_ptr(), clone.data().as_ptr());
        assert_eq!(image.data(), &[1, 2, 3, 4]);
    }
}
