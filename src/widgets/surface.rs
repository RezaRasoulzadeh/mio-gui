// surface.rs

use crate::{
    LogicalConstraints, LogicalPoint, LogicalSize, RectDraw, ResolvedTheme, SemanticColorToken,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Surface {
    pub size: LogicalSize,
    pub color: SemanticColorToken,
    pub radius: f32,
}

impl Surface {
    pub const fn new(size: LogicalSize) -> Self {
        Self {
            size,
            color: SemanticColorToken::Surface,
            radius: 0.0,
        }
    }
    pub fn layout(self, constraints: LogicalConstraints) -> LogicalSize {
        constraints.constrain(self.size)
    }
    pub fn draw(self, origin: LogicalPoint, size: LogicalSize, theme: &ResolvedTheme) -> RectDraw {
        RectDraw {
            position: [origin.x, origin.y],
            size: [size.width, size.height],
            radii: [self.radius.max(0.0); 4],
            color: theme.colors.resolve(self.color).to_array(),
            border_width: 0.0,
            border_color: [0.0; 4],
        }
    }
}
