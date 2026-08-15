// scroll_view.rs

use crate::{Direction, LogicalConstraints, LogicalPoint, LogicalRect, LogicalSize};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ScrollAxis {
    Inline,
    Block,
    #[default]
    Both,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollOffset {
    pub inline: f32,
    pub block: f32,
}

impl ScrollOffset {
    pub fn new(inline: f32, block: f32) -> Self {
        Self {
            inline: valid(inline),
            block: valid(block),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollView {
    pub axis: ScrollAxis,
    pub offset: ScrollOffset,
    pub viewport: Option<LogicalSize>,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self {
            axis: ScrollAxis::Both,
            offset: ScrollOffset::default(),
            viewport: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollLayout {
    pub viewport: LogicalSize,
    pub content_bounds: LogicalRect,
    pub clip: LogicalRect,
    pub offset: ScrollOffset,
    pub maximum_offset: ScrollOffset,
}

impl ScrollView {
    pub fn layout(
        self,
        direction: Direction,
        content: LogicalSize,
        constraints: LogicalConstraints,
    ) -> ScrollLayout {
        let viewport = constraints.constrain(self.viewport.unwrap_or(content));
        let maximum_offset = ScrollOffset {
            inline: (content.width - viewport.width).max(0.0),
            block: (content.height - viewport.height).max(0.0),
        };
        let offset = ScrollOffset {
            inline: if self.axis == ScrollAxis::Block {
                0.0
            } else {
                valid(self.offset.inline).min(maximum_offset.inline)
            },
            block: if self.axis == ScrollAxis::Inline {
                0.0
            } else {
                valid(self.offset.block).min(maximum_offset.block)
            },
        };
        let x = match direction {
            Direction::Ltr => -offset.inline,
            Direction::Rtl => viewport.width - content.width + offset.inline,
        };
        ScrollLayout {
            viewport,
            content_bounds: LogicalRect::new(LogicalPoint::new(x, -offset.block), content),
            clip: LogicalRect::new(LogicalPoint::default(), viewport),
            offset,
            maximum_offset,
        }
    }
}

fn valid(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{ScrollAxis, ScrollOffset, ScrollView};
    use crate::{Direction, LogicalConstraints, LogicalPoint, LogicalSize};

    #[test]
    fn logical_inline_start_uses_opposite_content_edges_in_ltr_and_rtl() {
        let constraints = LogicalConstraints::tight(LogicalSize::new(40.0, 20.0));
        let content = LogicalSize::new(100.0, 20.0);
        let ltr = ScrollView::default().layout(Direction::Ltr, content, constraints);
        let rtl = ScrollView::default().layout(Direction::Rtl, content, constraints);
        assert_eq!(ltr.content_bounds.origin, LogicalPoint::new(0.0, 0.0));
        assert_eq!(rtl.content_bounds.origin, LogicalPoint::new(-60.0, 0.0));
        assert_eq!(ltr.maximum_offset.inline, 60.0);
    }

    #[test]
    fn offsets_clamp_and_disabled_axis_stays_at_logical_start() {
        let view = ScrollView {
            axis: ScrollAxis::Block,
            offset: ScrollOffset::new(30.0, 90.0),
            viewport: None,
        };
        let result = view.layout(
            Direction::Ltr,
            LogicalSize::new(100.0, 80.0),
            LogicalConstraints::tight(LogicalSize::new(40.0, 20.0)),
        );
        assert_eq!(result.offset, ScrollOffset::new(0.0, 60.0));
        assert_eq!(result.content_bounds.origin, LogicalPoint::new(0.0, -60.0));
        assert_eq!(result.clip.size, LogicalSize::new(40.0, 20.0));
    }
}
