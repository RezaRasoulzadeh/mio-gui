// divider.rs

use crate::{LogicalConstraints, LogicalPoint, LogicalSize, RectDraw, SemanticColorToken};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DividerAxis {
    Horizontal,
    Vertical,
}

#[cfg(test)]
mod tests {
    use super::{Divider, DividerAxis};
    use crate::{LogicalConstraints, LogicalSize};

    #[test]
    fn dividers_apply_axis_thickness_and_constraints() {
        let mut divider = Divider::horizontal();
        divider.thickness = 2.0;
        assert_eq!(
            divider.layout(LogicalConstraints::unconstrained()),
            LogicalSize::new(0.0, 2.0)
        );
        assert_eq!(Divider::vertical().axis, DividerAxis::Vertical);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Divider {
    pub axis: DividerAxis,
    pub thickness: f32,
    pub color: SemanticColorToken,
}

impl Divider {
    pub const fn horizontal() -> Self {
        Self {
            axis: DividerAxis::Horizontal,
            thickness: 1.0,
            color: SemanticColorToken::Border,
        }
    }
    pub const fn vertical() -> Self {
        Self {
            axis: DividerAxis::Vertical,
            thickness: 1.0,
            color: SemanticColorToken::Border,
        }
    }
    pub fn layout(self, constraints: LogicalConstraints) -> LogicalSize {
        let thickness = if self.thickness.is_finite() {
            self.thickness.max(0.0)
        } else {
            0.0
        };
        let preferred = match self.axis {
            DividerAxis::Horizontal => LogicalSize::new(0.0, thickness),
            DividerAxis::Vertical => LogicalSize::new(thickness, 0.0),
        };
        constraints.constrain(preferred)
    }
    pub fn draw(self, origin: LogicalPoint, size: LogicalSize, color: [f32; 4]) -> RectDraw {
        RectDraw {
            position: [origin.x, origin.y],
            size: [size.width, size.height],
            radii: [0.0; 4],
            color,
            border_width: 0.0,
            border_color: [0.0; 4],
        }
    }
}
