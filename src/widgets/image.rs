// image.rs

use crate::{
    Direction, DirectionSetting, HorizontalAlignment, ImageDraw, InlineAlignment,
    LogicalConstraints, LogicalPoint, LogicalRect, LogicalSize, PixelImage, SemanticRole,
    Semantics,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ImageFit {
    #[default]
    Contain,
    Cover,
    Fill,
    None,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum BlockAlignment {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImageAlignment {
    pub inline: InlineAlignment,
    pub block: BlockAlignment,
}

impl Default for ImageAlignment {
    fn default() -> Self {
        Self {
            inline: InlineAlignment::Center,
            block: BlockAlignment::Center,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub source: PixelImage,
    pub fit: ImageFit,
    pub alignment: ImageAlignment,
    pub direction: DirectionSetting,
    pub mirror_in_rtl: bool,
    alternative_text: Option<String>,
}

impl Image {
    pub fn new(source: PixelImage) -> Self {
        Self {
            source,
            fit: ImageFit::Contain,
            alignment: ImageAlignment::default(),
            direction: DirectionSetting::Inherit,
            mirror_in_rtl: false,
            alternative_text: None,
        }
    }

    pub fn with_alternative_text(mut self, text: impl Into<String>) -> Self {
        self.set_alternative_text(Some(text.into()));
        self
    }

    pub fn alternative_text(&self) -> Option<&str> {
        self.alternative_text.as_deref()
    }

    pub fn set_alternative_text(&mut self, text: Option<String>) {
        self.alternative_text = text.filter(|text| !text.trim().is_empty());
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Image);
        if let Some(text) = &self.alternative_text {
            semantics.set_name(text.clone());
        } else {
            semantics.state.hidden = true;
        }
        semantics
    }

    pub fn layout(
        &self,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> ImageLayout {
        let direction = self.direction.resolve(inherited_direction);
        let intrinsic = LogicalSize::new(self.source.width() as f32, self.source.height() as f32);
        let size = constraints.constrain(intrinsic);
        let content_size = fitted_size(intrinsic, size, self.fit);
        let remaining_inline = size.width - content_size.width;
        let remaining_block = size.height - content_size.height;
        let inline_offset = match self.alignment.inline.resolve(direction) {
            HorizontalAlignment::Left => 0.0,
            HorizontalAlignment::Center => remaining_inline * 0.5,
            HorizontalAlignment::Right => remaining_inline,
        };
        let block_offset = match self.alignment.block {
            BlockAlignment::Start => 0.0,
            BlockAlignment::Center => remaining_block * 0.5,
            BlockAlignment::End => remaining_block,
        };
        ImageLayout {
            size,
            content_bounds: LogicalRect::new(
                LogicalPoint::new(inline_offset, block_offset),
                content_size,
            ),
            direction,
            mirrored: self.mirror_in_rtl && direction == Direction::Rtl,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageLayout {
    pub size: LogicalSize,
    pub content_bounds: LogicalRect,
    pub direction: Direction,
    pub mirrored: bool,
}

impl ImageLayout {
    pub fn draw(&self, image: PixelImage, origin: LogicalPoint) -> ImageDraw {
        ImageDraw {
            image,
            bounds: LogicalRect::new(
                LogicalPoint::new(
                    origin.x + self.content_bounds.origin.x,
                    origin.y + self.content_bounds.origin.y,
                ),
                self.content_bounds.size,
            ),
            clip: LogicalRect::new(origin, self.size),
            mirror_horizontal: self.mirrored,
            tint: None,
        }
    }
}

fn fitted_size(intrinsic: LogicalSize, available: LogicalSize, fit: ImageFit) -> LogicalSize {
    if intrinsic.is_empty() || available.is_empty() {
        return LogicalSize::default();
    }
    match fit {
        ImageFit::Fill => available,
        ImageFit::None => intrinsic,
        ImageFit::Contain | ImageFit::Cover => {
            let inline_scale = available.width / intrinsic.width;
            let block_scale = available.height / intrinsic.height;
            let scale = if fit == ImageFit::Contain {
                inline_scale.min(block_scale)
            } else {
                inline_scale.max(block_scale)
            };
            LogicalSize::new(intrinsic.width * scale, intrinsic.height * scale)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MaskShape {
    #[default]
    Circle,
    Rounded,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mask {
    image: Image,
    pub shape: MaskShape,
}

impl Mask {
    pub fn new(source: PixelImage, shape: MaskShape) -> Self {
        let width = source.width();
        let height = source.height();
        let mut data = Vec::with_capacity(width as usize * height as usize * 4);
        for y in 0..height {
            for x in 0..width {
                let pixel = y as usize * width as usize + x as usize;
                let (red, green, blue, alpha) = match source.format() {
                    crate::PixelFormat::Rgba8 => {
                        let offset = pixel * 4;
                        (
                            source.data()[offset],
                            source.data()[offset + 1],
                            source.data()[offset + 2],
                            source.data()[offset + 3],
                        )
                    }
                    crate::PixelFormat::Alpha8 => (255, 255, 255, source.data()[pixel]),
                };
                let nx = (x as f32 + 0.5) / width as f32 * 2.0 - 1.0;
                let ny = (y as f32 + 0.5) / height as f32 * 2.0 - 1.0;
                let visible = match shape {
                    MaskShape::Circle => nx * nx + ny * ny <= 1.0,
                    MaskShape::Rounded => {
                        let corner_x = (nx.abs() - 0.72).max(0.0);
                        let corner_y = (ny.abs() - 0.72).max(0.0);
                        corner_x * corner_x + corner_y * corner_y <= 0.2_f32.powi(2)
                    }
                };
                data.extend([red, green, blue, if visible { alpha } else { 0 }]);
            }
        }
        let source = PixelImage::new(width, height, crate::PixelFormat::Rgba8, data).unwrap();
        Self {
            image: Image::new(source),
            shape,
        }
    }

    pub fn with_alternative_text(mut self, text: impl Into<String>) -> Self {
        self.image = self.image.with_alternative_text(text);
        self
    }

    pub fn semantics(&self) -> Semantics {
        self.image.semantics()
    }

    pub fn layout(
        &self,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> ImageLayout {
        self.image.layout(inherited_direction, constraints)
    }

    pub fn draw(&self, layout: &ImageLayout, origin: LogicalPoint) -> ImageDraw {
        layout.draw(self.image.source.clone(), origin)
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockAlignment, Image, ImageAlignment, ImageFit, Mask, MaskShape};
    use crate::{
        Direction, LogicalConstraints, LogicalPoint, LogicalSize, PixelFormat, PixelImage,
    };

    fn image() -> PixelImage {
        PixelImage::new(4, 2, PixelFormat::Rgba8, vec![255_u8; 32]).unwrap()
    }

    #[test]
    fn contain_and_cover_preserve_intrinsic_aspect_ratio() {
        let mut widget = Image::new(image());
        let constraints = LogicalConstraints::tight(LogicalSize::new(100.0, 100.0));
        let contain = widget.layout(Direction::Ltr, constraints);
        widget.fit = ImageFit::Cover;
        let cover = widget.layout(Direction::Ltr, constraints);
        assert_eq!(contain.content_bounds.size, LogicalSize::new(100.0, 50.0));
        assert_eq!(contain.content_bounds.origin, LogicalPoint::new(0.0, 25.0));
        assert_eq!(cover.content_bounds.size, LogicalSize::new(200.0, 100.0));
        assert_eq!(cover.content_bounds.origin, LogicalPoint::new(-50.0, 0.0));
    }

    #[test]
    fn fill_and_none_have_explicit_rules() {
        let mut widget = Image::new(image());
        let constraints = LogicalConstraints::tight(LogicalSize::new(80.0, 60.0));
        widget.fit = ImageFit::Fill;
        assert_eq!(
            widget
                .layout(Direction::Ltr, constraints)
                .content_bounds
                .size,
            LogicalSize::new(80.0, 60.0)
        );
        widget.fit = ImageFit::None;
        assert_eq!(
            widget
                .layout(Direction::Ltr, constraints)
                .content_bounds
                .size,
            LogicalSize::new(4.0, 2.0)
        );
    }

    #[test]
    fn masks_clear_shape_corners_and_preserve_accessible_names() {
        let source =
            PixelImage::new(8, 8, crate::PixelFormat::Rgba8, vec![255; 8 * 8 * 4]).unwrap();
        for shape in [MaskShape::Circle, MaskShape::Rounded] {
            let mask = Mask::new(source.clone(), shape).with_alternative_text("Profile");
            assert_eq!(mask.image.source.format(), crate::PixelFormat::Rgba8);
            assert_eq!(mask.image.source.data()[3], 0);
            assert_eq!(mask.semantics().name.as_deref(), Some("Profile"));
        }
    }

    #[test]
    fn logical_inline_alignment_mirrors_but_block_alignment_does_not() {
        let mut widget = Image::new(image());
        widget.fit = ImageFit::None;
        widget.alignment = ImageAlignment {
            inline: crate::InlineAlignment::Start,
            block: BlockAlignment::End,
        };
        let constraints = LogicalConstraints::tight(LogicalSize::new(20.0, 10.0));
        let ltr = widget.layout(Direction::Ltr, constraints);
        let rtl = widget.layout(Direction::Rtl, constraints);
        assert_eq!(ltr.content_bounds.origin, LogicalPoint::new(0.0, 8.0));
        assert_eq!(rtl.content_bounds.origin, LogicalPoint::new(16.0, 8.0));
    }

    #[test]
    fn mirroring_is_opt_in_and_only_applies_in_rtl() {
        let mut widget = Image::new(image());
        let constraints = LogicalConstraints::unconstrained();
        assert!(!widget.layout(Direction::Rtl, constraints).mirrored);
        widget.mirror_in_rtl = true;
        assert!(!widget.layout(Direction::Ltr, constraints).mirrored);
        assert!(widget.layout(Direction::Rtl, constraints).mirrored);
    }

    #[test]
    fn alternative_text_controls_semantics() {
        let decorative = Image::new(image());
        let informative = Image::new(image()).with_alternative_text("Profile photograph");
        assert_eq!(decorative.semantics().name, None);
        assert!(decorative.semantics().state.hidden);
        assert_eq!(
            informative.semantics().name.as_deref(),
            Some("Profile photograph")
        );
        assert!(!informative.semantics().state.hidden);
    }

    #[test]
    fn draw_preserves_clip_mirror_and_source() {
        let mut widget = Image::new(image());
        widget.mirror_in_rtl = true;
        let layout = widget.layout(
            Direction::Rtl,
            LogicalConstraints::tight(LogicalSize::new(20.0, 10.0)),
        );
        let draw = layout.draw(widget.source.clone(), LogicalPoint::new(5.0, 7.0));
        assert_eq!(draw.image, widget.source);
        assert_eq!(draw.clip.origin, LogicalPoint::new(5.0, 7.0));
        assert!(draw.mirror_horizontal);
    }
}
