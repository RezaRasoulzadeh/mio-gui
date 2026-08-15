// icon.rs

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    Direction, DirectionSetting, ImageAlignment, ImageDraw, ImageFit, ImageLayout,
    LogicalConstraints, LogicalPoint, PixelFormat, PixelImage, SemanticColorToken, SemanticRole,
    Semantics,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IconError {
    UnsupportedPixelFormat(PixelFormat),
}

impl Display for IconError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPixelFormat(format) => {
                write!(
                    formatter,
                    "icons require Alpha8 pixel data, received {format:?}"
                )
            }
        }
    }
}

impl Error for IconError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Icon {
    pub source: PixelImage,
    pub fit: ImageFit,
    pub alignment: ImageAlignment,
    pub direction: DirectionSetting,
    pub mirror_in_rtl: bool,
    pub color: SemanticColorToken,
    alternative_text: Option<String>,
}

impl Icon {
    pub fn new(source: PixelImage) -> Result<Self, IconError> {
        if source.format() != PixelFormat::Alpha8 {
            return Err(IconError::UnsupportedPixelFormat(source.format()));
        }
        Ok(Self {
            source,
            fit: ImageFit::Contain,
            alignment: ImageAlignment::default(),
            direction: DirectionSetting::Inherit,
            mirror_in_rtl: false,
            color: SemanticColorToken::Text,
            alternative_text: None,
        })
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
    ) -> IconLayout {
        let mut image = crate::Image::new(self.source.clone());
        image.fit = self.fit;
        image.alignment = self.alignment;
        image.direction = self.direction;
        image.mirror_in_rtl = self.mirror_in_rtl;
        IconLayout(image.layout(inherited_direction, constraints))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconLayout(ImageLayout);

impl IconLayout {
    pub const fn size(self) -> crate::LogicalSize {
        self.0.size
    }

    pub const fn content_bounds(self) -> crate::LogicalRect {
        self.0.content_bounds
    }

    pub const fn direction(self) -> Direction {
        self.0.direction
    }

    pub const fn mirrored(self) -> bool {
        self.0.mirrored
    }

    pub fn draw(self, image: PixelImage, origin: LogicalPoint, tint: [f32; 4]) -> ImageDraw {
        let mut draw = self.0.draw(image, origin);
        draw.tint = Some(tint);
        draw
    }
}

#[cfg(test)]
mod tests {
    use super::{Icon, IconError};
    use crate::{
        Direction, ImageFit, LogicalConstraints, LogicalPoint, LogicalSize, PixelFormat,
        PixelImage, SemanticColorToken,
    };

    fn alpha_mask() -> PixelImage {
        PixelImage::new(4, 2, PixelFormat::Alpha8, vec![255_u8; 8]).unwrap()
    }

    #[test]
    fn icons_require_alpha_masks() {
        let rgba = PixelImage::new(1, 1, PixelFormat::Rgba8, vec![0_u8; 4]).unwrap();
        assert_eq!(
            Icon::new(rgba),
            Err(IconError::UnsupportedPixelFormat(PixelFormat::Rgba8))
        );
    }

    #[test]
    fn icons_default_to_semantic_text_tint_and_decorative_semantics() {
        let icon = Icon::new(alpha_mask()).unwrap();
        assert_eq!(icon.color, SemanticColorToken::Text);
        assert!(icon.semantics().state.hidden);
        assert_eq!(
            icon.with_alternative_text("Search")
                .semantics()
                .name
                .as_deref(),
            Some("Search")
        );
    }

    #[test]
    fn icon_layout_reuses_image_fitting_mirroring_and_clipping_with_tint() {
        let mut icon = Icon::new(alpha_mask()).unwrap();
        icon.fit = ImageFit::Cover;
        icon.mirror_in_rtl = true;
        let layout = icon.layout(
            Direction::Rtl,
            LogicalConstraints::tight(LogicalSize::new(20.0, 10.0)),
        );
        let draw = layout.draw(icon.source.clone(), LogicalPoint::new(5.0, 7.0), [0.5; 4]);
        assert_eq!(layout.content_bounds().size, LogicalSize::new(20.0, 10.0));
        assert!(layout.mirrored());
        assert_eq!(draw.clip.origin, LogicalPoint::new(5.0, 7.0));
        assert_eq!(draw.tint, Some([0.5; 4]));
    }
}
